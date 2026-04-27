use metrics::{counter, histogram};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use px_core::{
    canonical_event_id, manifests::KALSHI_MANIFEST, sort_asks, sort_bids, Candlestick, Exchange,
    ExchangeInfo, ExchangeManifest, FetchMarketsParams, FetchOrdersParams, Fill, Market,
    MarketStatus, MarketStatusFilter, MarketTrade, MarketType, OpenPxError, Order, OrderSide,
    OrderStatus, Orderbook, OutcomeToken, Position, PriceHistoryInterval, PriceHistoryRequest,
    PriceLevel, RateLimiter, TradesRequest,
};

use crate::auth::KalshiAuth;
use crate::config::KalshiConfig;
use crate::error::KalshiError;
use crate::normalize::normalize_kalshi_trade_price;

/// Parse Kalshi dollar-string fields (e.g. "0.65") to f64.
fn parse_dollars(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<f64> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
}

/// Parse Kalshi fixed-point string fields (e.g. "15000.5") to f64.
fn parse_fp(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<f64> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
}

/// Parse a price value that may be a dollar string ("0.65") or cents integer (65).
/// Dollar strings are used as-is; integers are divided by 100.
fn parse_price_value(val: &Option<serde_json::Value>) -> Option<f64> {
    let v = val.as_ref()?;
    if let Some(s) = v.as_str() {
        s.parse::<f64>().ok()
    } else if let Some(i) = v.as_i64() {
        Some(i as f64 / 100.0)
    } else {
        v.as_f64()
    }
}

/// Parse a numeric value that may be a string ("1500.5") or number (1500).
/// No cent-to-dollar conversion — used for volume, open interest, etc.
fn parse_numeric_value(val: &Option<serde_json::Value>) -> Option<f64> {
    let v = val.as_ref()?;
    if let Some(s) = v.as_str() {
        s.parse::<f64>().ok()
    } else {
        v.as_f64()
    }
}

pub(crate) fn to_openpx(e: KalshiError) -> OpenPxError {
    OpenPxError::Exchange(e.into())
}

pub struct Kalshi {
    config: KalshiConfig,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    auth: Option<KalshiAuth>,
}

#[derive(Debug, serde::Deserialize)]
struct KalshiBatchCandlesticksResponse {
    markets: Vec<KalshiMarketCandlesticks>,
}

#[derive(Debug, serde::Deserialize)]
struct KalshiMarketCandlesticks {
    #[serde(alias = "marketTicker")]
    market_ticker: String,
    #[serde(default)]
    candlesticks: Vec<KalshiCandlestick>,
}

#[derive(Debug, serde::Deserialize)]
struct KalshiCandlestick {
    #[serde(alias = "endPeriodTs")]
    end_period_ts: i64,
    price: KalshiOhlc,
    #[serde(alias = "volume")]
    volume_fp: Option<serde_json::Value>,
    #[serde(alias = "openInterest", alias = "open_interest")]
    open_interest_fp: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
struct KalshiOhlc {
    #[serde(alias = "open")]
    open_dollars: Option<serde_json::Value>,
    #[serde(alias = "high")]
    high_dollars: Option<serde_json::Value>,
    #[serde(alias = "low")]
    low_dollars: Option<serde_json::Value>,
    #[serde(alias = "close")]
    close_dollars: Option<serde_json::Value>,
}

impl Kalshi {
    pub fn new(config: KalshiConfig) -> Result<Self, KalshiError> {
        let client = px_core::http::tuned_client_builder()
            .timeout(config.base.timeout)
            .build()?;

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
            config.base.rate_limit_per_second,
        )));

        // Initialize auth if credentials are provided
        let auth = if config.is_authenticated() {
            let auth = if let Some(ref path) = config.private_key_path {
                KalshiAuth::from_file(path)?
            } else if let Some(ref pem) = config.private_key_pem {
                KalshiAuth::from_pem(pem)?
            } else {
                return Err(KalshiError::AuthRequired);
            };
            Some(auth)
        } else {
            None
        };

        Ok(Self {
            config,
            client,
            rate_limiter,
            auth,
        })
    }

    pub fn with_default_config() -> Result<Self, KalshiError> {
        Self::new(KalshiConfig::default())
    }

    async fn rate_limit(&self) {
        self.rate_limiter.lock().await.wait().await;
    }

    /// Construct the full signing path from the API URL base path and endpoint.
    /// e.g., api_url "https://api.elections.kalshi.com/trade-api/v2" + path "/portfolio/balance"
    /// => signing path "/trade-api/v2/portfolio/balance"
    fn signing_path(&self, path: &str) -> String {
        let url = &self.config.api_url;
        if let Some(scheme_end) = url.find("://") {
            if let Some(path_start) = url[scheme_end + 3..].find('/') {
                let base = &url[scheme_end + 3 + path_start..];
                return format!("{}{}", base, path);
            }
        }
        path.to_string()
    }

    fn auth_headers(
        &self,
        builder: reqwest::RequestBuilder,
        method: &str,
        path: &str,
    ) -> Result<reqwest::RequestBuilder, KalshiError> {
        if let (Some(ref auth), Some(ref api_key_id)) = (&self.auth, &self.config.api_key_id) {
            let timestamp_ms = chrono::Utc::now().timestamp_millis();
            let signature = auth.sign(timestamp_ms, method, path);

            Ok(builder
                .header("KALSHI-ACCESS-KEY", api_key_id)
                .header("KALSHI-ACCESS-SIGNATURE", signature)
                .header("KALSHI-ACCESS-TIMESTAMP", timestamp_ms.to_string()))
        } else {
            Ok(builder)
        }
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&serde_json::Value>,
        operation: Option<&str>,
    ) -> Result<T, KalshiError> {
        self.rate_limit().await;

        let url = format!("{}{}", self.config.api_url, path);
        let sign_path = self.signing_path(path);
        let method_str = method.to_string();

        let mut req = self.client.request(method, &url);
        if let Some(body) = body {
            req = req.json(body);
        }
        let req = self.auth_headers(req, &method_str, &sign_path)?;

        let send_start = Instant::now();
        let response = req.send().await?;
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;

        if let Some(op) = operation {
            histogram!(
                "openpx.exchange.order_http_send_us",
                "exchange" => "kalshi",
                "operation" => op.to_string()
            )
            .record(send_us);
        } else {
            histogram!("openpx.exchange.http_send_us", "exchange" => "kalshi").record(send_us);
        }

        let status = response.status();
        if status == 429 {
            return Err(KalshiError::RateLimited);
        }

        let body_start = Instant::now();
        let body = response.text().await?;
        let body_us = body_start.elapsed().as_secs_f64() * 1_000_000.0;

        if let Some(op) = operation {
            histogram!(
                "openpx.exchange.order_http_body_us",
                "exchange" => "kalshi",
                "operation" => op.to_string()
            )
            .record(body_us);
        } else {
            histogram!("openpx.exchange.http_body_us", "exchange" => "kalshi").record(body_us);
        }

        if status == 401 || status == 403 {
            return Err(KalshiError::AuthFailed(body));
        }

        if status == 404 {
            // Extract the resource identifier from the path (strip leading /endpoint/).
            let id = path.rsplit('/').next().unwrap_or(path);
            return Err(KalshiError::MarketNotFound(id.to_string()));
        }

        if !status.is_success() {
            // Parse Kalshi structured error to extract human-readable message + code
            if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(err_obj) = err_json.get("error").and_then(|e| e.as_object()) {
                    let code = err_obj.get("code").and_then(|c| c.as_str()).unwrap_or("");
                    let message = err_obj
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    return Err(match code {
                        "insufficient_balance" => KalshiError::InsufficientBalance(message),
                        _ => KalshiError::OrderRejected(message),
                    });
                }
            }
            return Err(KalshiError::Api(body));
        }

        let parse_start = Instant::now();
        let parsed = serde_json::from_str(&body)
            .map_err(|e| KalshiError::Api(format!("parse error: {e}")))?;
        let parse_us = parse_start.elapsed().as_secs_f64() * 1_000_000.0;

        if let Some(op) = operation {
            histogram!(
                "openpx.exchange.order_json_parse_us",
                "exchange" => "kalshi",
                "operation" => op.to_string()
            )
            .record(parse_us);
        } else {
            histogram!("openpx.exchange.json_parse_us", "exchange" => "kalshi").record(parse_us);
        }

        Ok(parsed)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, KalshiError> {
        self.request(reqwest::Method::GET, path, None, None).await
    }

    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T, KalshiError> {
        let body = serde_json::to_value(body)
            .map_err(|e| KalshiError::Api(format!("serialize error: {e}")))?;
        self.request(reqwest::Method::POST, path, Some(&body), Some("post"))
            .await
    }

    async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, KalshiError> {
        self.request(reqwest::Method::DELETE, path, None, Some("delete"))
            .await
    }

    fn ensure_auth(&self) -> Result<(), KalshiError> {
        if !self.config.is_authenticated() {
            return Err(KalshiError::AuthRequired);
        }
        Ok(())
    }

    fn parse_market(&self, data: &serde_json::Value) -> Option<Market> {
        let obj = data.as_object()?;

        // Kalshi uses 'ticker' as market ID
        let id = obj.get("ticker").and_then(|v| v.as_str())?.to_string();

        let title = obj
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = obj
            .get("subtitle")
            .or_else(|| obj.get("rules_primary"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let rules = obj
            .get("rules_primary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Group/event ID
        let group_id = obj
            .get("event_ticker")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Canonical cross-exchange event ID
        let event_id = group_id
            .as_deref()
            .and_then(|gid| canonical_event_id("kalshi", gid));

        // Status — Kalshi market objects use a granular lifecycle:
        // active, closed, determined, finalized, initialized, inactive, disputed, amended
        let status = obj
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "active" => MarketStatus::Active,
                "determined" | "finalized" => MarketStatus::Resolved,
                _ => MarketStatus::Closed,
            })
            .unwrap_or(MarketStatus::Active);

        // accepting_orders
        let accepting_orders = obj
            .get("accepting_orders")
            .and_then(|v| v.as_bool())
            .unwrap_or(status == MarketStatus::Active);

        // Binary markets: Yes/No outcomes
        let outcomes = vec!["Yes".to_string(), "No".to_string()];

        // ── Prices (new dollar-string fields first, legacy int/100 fallback) ──

        let yes_ask = parse_dollars(obj, "yes_ask_dollars").or_else(|| {
            obj.get("yes_ask")
                .and_then(|v| v.as_f64())
                .map(|p| p / 100.0)
        });

        let yes_bid = parse_dollars(obj, "yes_bid_dollars").or_else(|| {
            obj.get("yes_bid")
                .and_then(|v| v.as_f64())
                .map(|p| p / 100.0)
        });

        let last_price = parse_dollars(obj, "last_price_dollars").or_else(|| {
            obj.get("last_price")
                .and_then(|v| v.as_f64())
                .map(|p| p / 100.0)
        });

        let yes_price = yes_ask.or(last_price).unwrap_or(0.0);
        let no_price = 1.0 - yes_price;

        let mut outcome_prices = HashMap::new();
        outcome_prices.insert("Yes".to_string(), yes_price);
        outcome_prices.insert("No".to_string(), no_price);

        // ── Volume / liquidity (fixed-point migration) ──

        let volume = parse_fp(obj, "volume_fp")
            .or_else(|| obj.get("volume").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);

        let open_interest = parse_fp(obj, "open_interest_fp")
            .or_else(|| obj.get("open_interest").and_then(|v| v.as_f64()));

        // ── Tick size: try price_level_structure, fallback to tick_size in cents ──

        let tick_size = obj
            .get("tick_size")
            .and_then(|v| v.as_f64())
            .filter(|&v| v > 0.0)
            .map(|cents| cents / 100.0)
            .or(Some(0.01));

        let price_level_structure = obj
            .get("price_level_structure")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // ── Time fields ──

        let close_time = obj
            .get("close_time")
            .or_else(|| obj.get("expiration_time"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let open_time = obj
            .get("open_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let created_at = obj
            .get("created_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // settlement_ts: may be RFC3339 string (new API) or unix timestamp (legacy)
        let settlement_time = obj.get("settlement_ts").and_then(|v| {
            v.as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .or_else(|| {
                    v.as_i64()
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                })
        });

        // ── Kalshi-specific fields ──

        let notional_value = parse_dollars(obj, "notional_value_dollars")
            .or_else(|| obj.get("notional_value").and_then(|v| v.as_f64()));

        let previous_price = parse_dollars(obj, "previous_price_dollars").or_else(|| {
            obj.get("previous_price")
                .and_then(|v| v.as_f64())
                .map(|p| p / 100.0)
        });

        let settlement_value = parse_dollars(obj, "settlement_value_dollars").or_else(|| {
            obj.get("settlement_value")
                .and_then(|v| v.as_f64())
                .map(|p| p / 100.0)
        });

        let can_close_early = obj.get("can_close_early").and_then(|v| v.as_bool());

        let result = obj
            .get("result")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Spread from bid/ask
        let spread = match (yes_bid, yes_ask) {
            (Some(b), Some(a)) => Some(a - b),
            _ => None,
        };

        let openpx_id = Market::make_openpx_id("kalshi", &id);

        let market_type = obj
            .get("market_type")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "scalar" => MarketType::Scalar,
                _ => MarketType::Binary,
            })
            .unwrap_or(MarketType::Binary);

        Some(Market {
            openpx_id,
            exchange: "kalshi".into(),
            id,
            group_id,
            event_id,
            title,
            description,
            rules,
            status,
            market_type,
            accepting_orders,
            outcomes,
            outcome_prices,
            volume,
            open_interest,
            last_trade_price: last_price,
            best_bid: yes_bid,
            best_ask: yes_ask,
            spread,
            tick_size,
            close_time,
            open_time,
            created_at,
            settlement_time,
            notional_value,
            price_level_structure,
            settlement_value,
            previous_price,
            can_close_early,
            result,
            ..Default::default()
        })
    }

    fn parse_order(&self, data: &serde_json::Value) -> Order {
        let obj = data.as_object();

        let id = obj
            .and_then(|o| o.get("order_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let market_id = obj
            .and_then(|o| o.get("ticker"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Kalshi uses 'side' for yes/no and 'action' for buy/sell
        let action = obj
            .and_then(|o| o.get("action"))
            .and_then(|v| v.as_str())
            .unwrap_or("buy");

        let side = if action.to_lowercase() == "buy" {
            OrderSide::Buy
        } else {
            OrderSide::Sell
        };

        let outcome = obj
            .and_then(|o| o.get("side"))
            .and_then(|v| v.as_str())
            .map(|s| if s == "yes" { "Yes" } else { "No" })
            .unwrap_or("Yes")
            .to_string();

        let status = obj
            .and_then(|o| o.get("status"))
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "resting" | "active" | "pending" => OrderStatus::Open,
                "executed" | "filled" => OrderStatus::Filled,
                "canceled" | "cancelled" => OrderStatus::Cancelled,
                "partial" => OrderStatus::PartiallyFilled,
                _ => OrderStatus::Open,
            })
            .unwrap_or(OrderStatus::Open);

        // Price: prefer dollar strings, fall back to cents integer / 100
        let price = obj
            .and_then(|o| {
                parse_dollars(o, "yes_price_dollars")
                    .or_else(|| parse_dollars(o, "no_price_dollars"))
                    .or_else(|| {
                        o.get("yes_price")
                            .or(o.get("no_price"))
                            .and_then(|v| v.as_f64())
                            .map(|p| p / 100.0)
                    })
            })
            .unwrap_or(0.0);

        // Size: prefer fp string, fall back to integer
        let size = obj
            .and_then(|o| {
                parse_fp(o, "remaining_count_fp")
                    .or_else(|| parse_fp(o, "initial_count_fp"))
                    .or_else(|| {
                        o.get("remaining_count")
                            .or(o.get("count"))
                            .and_then(|v| v.as_f64())
                    })
            })
            .unwrap_or(0.0);

        // Filled: prefer fp string, fall back to integer
        let filled = obj
            .and_then(|o| {
                parse_fp(o, "fill_count_fp")
                    .or_else(|| o.get("filled_count").and_then(|v| v.as_f64()))
            })
            .unwrap_or(0.0);

        let created_at = obj
            .and_then(|o| o.get("created_time"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let updated_at = obj
            .and_then(|o| o.get("last_update_time").or(o.get("updated_time")))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        Order {
            id,
            market_id,
            outcome,
            side,
            price,
            size,
            filled,
            status,
            created_at,
            updated_at,
        }
    }

    fn parse_position(&self, data: &serde_json::Value) -> Position {
        let obj = data.as_object();

        let market_id = obj
            .and_then(|o| o.get("ticker"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Kalshi positions: positive = Yes, negative = No
        // Prefer fp string, fall back to integer
        let yes_count = obj
            .and_then(|o| {
                parse_fp(o, "position_fp").or_else(|| o.get("position").and_then(|v| v.as_f64()))
            })
            .unwrap_or(0.0);

        let (outcome, size) = if yes_count >= 0.0 {
            ("Yes".to_string(), yes_count)
        } else {
            ("No".to_string(), -yes_count)
        };

        // Cost basis and current value: prefer dollar strings, fall back to cents / 100
        let total_traded = obj
            .and_then(|o| {
                parse_dollars(o, "total_traded_dollars").or_else(|| {
                    o.get("total_traded")
                        .and_then(|v| v.as_f64())
                        .map(|c| c / 100.0)
                })
            })
            .unwrap_or(0.0);

        let market_exposure = obj
            .and_then(|o| {
                parse_dollars(o, "market_exposure_dollars").or_else(|| {
                    o.get("market_exposure")
                        .and_then(|v| v.as_f64())
                        .map(|c| c / 100.0)
                })
            })
            .unwrap_or(0.0);

        let average_price = if size > 0.0 { total_traded / size } else { 0.0 };

        let current_price = if size > 0.0 {
            market_exposure / size
        } else {
            0.0
        };

        Position {
            market_id,
            outcome,
            size,
            average_price,
            current_price,
        }
    }

    fn parse_fill(&self, data: &serde_json::Value) -> Option<Fill> {
        let obj = data.as_object()?;

        let fill_id = obj.get("fill_id").and_then(|v| v.as_str())?.to_string();
        let order_id = obj
            .get("order_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let market_id = obj
            .get("ticker")
            .or_else(|| obj.get("market_ticker"))
            .and_then(|v| v.as_str())?
            .to_string();

        // Kalshi: "side" = yes/no (outcome), "action" = buy/sell (order side)
        let outcome = obj
            .get("side")
            .and_then(|v| v.as_str())
            .map(|s| if s == "yes" { "Yes" } else { "No" })
            .unwrap_or("Yes")
            .to_string();

        let side = match obj.get("action").and_then(|v| v.as_str()).unwrap_or("buy") {
            "sell" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        // Price in cents → normalized 0-1
        let price = obj
            .get("yes_price")
            .and_then(|v| v.as_f64())
            .or_else(|| obj.get("price").and_then(|v| v.as_f64()))
            .map(|p| p / 100.0)
            .unwrap_or(0.0);

        let size = obj
            .get("count_fp")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| obj.get("count").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);

        let is_taker = obj
            .get("is_taker")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let fee = obj
            .get("fee_cost")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let created_at = obj
            .get("created_time")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Some(Fill {
            fill_id,
            order_id,
            market_id,
            outcome,
            side,
            price,
            size,
            is_taker,
            fee,
            created_at,
        })
    }

    async fn fetch_orderbook_raw(
        &self,
        ticker: &str,
    ) -> Result<(Vec<PriceLevel>, Vec<PriceLevel>), KalshiError> {
        self.ensure_auth()?;

        #[derive(serde::Deserialize)]
        struct OrderbookResponse {
            #[serde(alias = "orderbook")]
            orderbook_fp: OrderbookData,
        }

        #[derive(serde::Deserialize)]
        struct OrderbookData {
            #[serde(alias = "yes")]
            yes_dollars: Option<Vec<[String; 2]>>,
            #[serde(alias = "no")]
            no_dollars: Option<Vec<[String; 2]>>,
        }

        let path = format!("/markets/{ticker}/orderbook");
        let resp: OrderbookResponse = self.get(&path).await?;

        let mut yes = Vec::new();
        if let Some(yes_levels) = resp.orderbook_fp.yes_dollars {
            for [price_str, size_str] in yes_levels {
                if let (Ok(price), Ok(size)) = (price_str.parse::<f64>(), size_str.parse::<f64>()) {
                    yes.push(PriceLevel::new(price, size));
                }
            }
        }

        let mut no = Vec::new();
        if let Some(no_levels) = resp.orderbook_fp.no_dollars {
            for [price_str, size_str] in no_levels {
                if let (Ok(price), Ok(size)) = (price_str.parse::<f64>(), size_str.parse::<f64>()) {
                    no.push(PriceLevel::new(price, size));
                }
            }
        }

        Ok((yes, no))
    }

    /// Fetch a Kalshi event and construct a multi-outcome Market from its sub-markets.
    async fn fetch_event_as_market(&self, event_ticker: &str) -> Result<Market, OpenPxError> {
        #[derive(serde::Deserialize)]
        struct EventResponse {
            event: serde_json::Value,
            markets: Vec<serde_json::Value>,
        }

        let path = format!("/events/{event_ticker}");
        let resp: EventResponse = self.get(&path).await.map_err(to_openpx)?;

        self.parse_event_as_market(&resp.event, &resp.markets)
            .ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(event_ticker.into()))
            })
    }

    /// Build a multi-outcome `Market` from a Kalshi event + its sub-markets.
    /// Each sub-market becomes one outcome; its ticker becomes the token_id.
    fn parse_event_as_market(
        &self,
        event: &serde_json::Value,
        markets: &[serde_json::Value],
    ) -> Option<Market> {
        let event_obj = event.as_object()?;
        let event_ticker = event_obj.get("event_ticker").and_then(|v| v.as_str())?;

        let title = event_obj
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = event_obj
            .get("subtitle")
            .or_else(|| event_obj.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut outcomes = Vec::with_capacity(markets.len());
        let mut outcome_prices = HashMap::new();
        let mut outcome_tokens = Vec::with_capacity(markets.len());
        let mut total_volume = 0.0f64;
        let mut total_open_interest = 0.0f64;
        let mut close_time: Option<chrono::DateTime<chrono::Utc>> = None;

        for m in markets {
            let obj = m.as_object()?;
            let ticker = obj.get("ticker").and_then(|v| v.as_str())?;

            // Prefer yes_sub_title ("Natalie Anderson") over title
            // ("Will Natalie Anderson win...?") for clean outcome labels.
            let outcome_name = obj
                .get("yes_sub_title")
                .or_else(|| obj.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or(ticker)
                .to_string();

            // Fixed-point migration: try dollar-string fields first
            let yes_price = parse_dollars(obj, "yes_ask_dollars")
                .or_else(|| parse_dollars(obj, "last_price_dollars"))
                .or_else(|| {
                    obj.get("yes_ask")
                        .or_else(|| obj.get("last_price"))
                        .and_then(|v| v.as_f64())
                        .map(|p| p / 100.0)
                })
                .unwrap_or(0.0);

            outcomes.push(outcome_name.clone());
            outcome_prices.insert(outcome_name.clone(), yes_price);
            outcome_tokens.push(OutcomeToken {
                outcome: outcome_name,
                token_id: ticker.to_string(),
            });

            total_volume += parse_fp(obj, "volume_fp")
                .or_else(|| obj.get("volume").and_then(|v| v.as_f64()))
                .unwrap_or(0.0);
            total_open_interest += parse_fp(obj, "open_interest_fp")
                .or_else(|| obj.get("open_interest").and_then(|v| v.as_f64()))
                .unwrap_or(0.0);

            // Use the latest close_time across sub-markets
            if let Some(ct) = obj
                .get("close_time")
                .or_else(|| obj.get("expiration_time"))
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
            {
                close_time = Some(match close_time {
                    Some(existing) if existing > ct => existing,
                    _ => ct,
                });
            }
        }

        let openpx_id = Market::make_openpx_id("kalshi", event_ticker);
        let event_id = canonical_event_id("kalshi", event_ticker);

        Some(Market {
            openpx_id,
            exchange: "kalshi".into(),
            id: event_ticker.to_string(),
            group_id: Some(event_ticker.to_string()),
            event_id,
            title,
            description,
            status: MarketStatus::Active,
            market_type: MarketType::Categorical,
            accepting_orders: true,
            outcomes,
            outcome_tokens,
            outcome_prices,
            volume: total_volume,
            open_interest: Some(total_open_interest),
            close_time,
            tick_size: Some(0.01), // event-level: default to 1 cent
            ..Default::default()
        })
    }
}

fn kalshi_time_in_force(params: &HashMap<String, String>) -> Result<Option<String>, OpenPxError> {
    let order_type = params
        .get("order_type")
        .map(|v| v.as_str())
        .unwrap_or("gtc");

    match order_type {
        "gtc" => Ok(None),
        "ioc" => Ok(Some("immediate_or_cancel".to_string())),
        "fok" => Ok(Some("fill_or_kill".to_string())),
        _ => Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
            format!("invalid order_type '{order_type}' (allowed: gtc, ioc, fok)"),
        ))),
    }
}

impl Exchange for Kalshi {
    fn id(&self) -> &'static str {
        "kalshi"
    }

    fn name(&self) -> &'static str {
        "Kalshi"
    }

    fn manifest(&self) -> &'static ExchangeManifest {
        &KALSHI_MANIFEST
    }

    async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError> {
        // ── event_id short-circuit: fetch a single event's nested markets ──
        if let Some(ref event_ticker) = params.event_id {
            #[derive(serde::Deserialize)]
            struct SingleEventResponse {
                #[allow(dead_code)]
                event: serde_json::Value,
                markets: Vec<serde_json::Value>,
            }

            let path = format!("/events/{event_ticker}");
            let resp: SingleEventResponse = self.get(&path).await.map_err(to_openpx)?;

            let filter = params.status.unwrap_or(MarketStatusFilter::Active);
            let requested_status = match filter {
                MarketStatusFilter::Active => Some(MarketStatus::Active),
                MarketStatusFilter::Closed => Some(MarketStatus::Closed),
                MarketStatusFilter::Resolved => Some(MarketStatus::Resolved),
                MarketStatusFilter::All => None,
            };

            let markets: Vec<Market> = resp
                .markets
                .iter()
                .filter_map(|raw| {
                    let market = self.parse_market(raw)?;
                    if let Some(ref req) = requested_status {
                        if market.status != *req {
                            return None;
                        }
                    }
                    Some(market)
                })
                .collect();

            return Ok((markets, None));
        }

        #[derive(serde::Deserialize)]
        struct MarketsResponse {
            markets: Vec<serde_json::Value>,
            cursor: Option<String>,
        }

        /// Compound cursor for live + historical markets. `None` on either side
        /// means that source is exhausted. Both `None` ⇒ terminate pagination.
        /// Breaking change from 0.2.2 — old cursor strings are discarded; callers
        /// must restart pagination after upgrading.
        #[derive(serde::Serialize, serde::Deserialize, Default)]
        struct CursorState {
            #[serde(skip_serializing_if = "Option::is_none")]
            r: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            h: Option<String>,
        }

        let filter = params.status.unwrap_or(MarketStatusFilter::Active);

        // Map unified filter → /markets `status` query value. `/historical/markets`
        // has no status filter — we include it only when callers may want
        // pre-cutoff settled markets (Resolved, All).
        let live_status: Option<&str> = match filter {
            MarketStatusFilter::Active => Some("open"),
            MarketStatusFilter::Closed => Some("closed"),
            MarketStatusFilter::Resolved => Some("settled"),
            MarketStatusFilter::All => None,
        };
        let include_historical = matches!(
            filter,
            MarketStatusFilter::Resolved | MarketStatusFilter::All
        );

        // Seed compound state from caller cursor (fresh-start on first call or
        // on any unrecognized cursor — no legacy-shape compat).
        let mut state: CursorState = match params.cursor.as_deref() {
            None => CursorState {
                r: Some(String::new()),
                h: if include_historical {
                    Some(String::new())
                } else {
                    None
                },
            },
            Some(s) => serde_json::from_str(s).unwrap_or(CursorState {
                r: Some(String::new()),
                h: if include_historical {
                    Some(String::new())
                } else {
                    None
                },
            }),
        };

        let limit = params.limit.unwrap_or(200).min(1000);

        let live_endpoint = state.r.as_ref().map(|cursor| {
            let mut ep = format!("/markets?limit={limit}");
            if let Some(s) = live_status {
                ep.push_str(&format!("&status={s}"));
            }
            if let Some(ref st) = params.series_id {
                ep.push_str(&format!("&series_ticker={st}"));
            }
            if !cursor.is_empty() {
                ep.push_str(&format!("&cursor={cursor}"));
            }
            ep
        });

        let historical_endpoint = state.h.as_ref().map(|cursor| {
            let mut ep = format!("/historical/markets?limit={limit}");
            if let Some(ref st) = params.series_id {
                ep.push_str(&format!("&series_ticker={st}"));
            }
            if !cursor.is_empty() {
                ep.push_str(&format!("&cursor={cursor}"));
            }
            ep
        });

        let live_fut = async {
            match live_endpoint {
                Some(ref ep) => self.get::<MarketsResponse>(ep).await.map(Some),
                None => Ok(None),
            }
        };
        let historical_fut = async {
            match historical_endpoint {
                Some(ref ep) => self.get::<MarketsResponse>(ep).await.map(Some),
                None => Ok(None),
            }
        };

        let (live_res, historical_res) = tokio::join!(live_fut, historical_fut);
        let live_resp = live_res.map_err(to_openpx)?;
        let historical_resp = historical_res.map_err(to_openpx)?;

        // Client-side status filter: `/markets?status=...` is server-authoritative,
        // but `/historical/markets` has no status filter and `All` hits both
        // endpoints without a server filter. Keep the post-filter as a safety net.
        let requested_status = match filter {
            MarketStatusFilter::Active => Some(MarketStatus::Active),
            MarketStatusFilter::Closed => Some(MarketStatus::Closed),
            MarketStatusFilter::Resolved => Some(MarketStatus::Resolved),
            MarketStatusFilter::All => None,
        };

        let mut all_markets = Vec::new();
        let map_start = Instant::now();

        let mut extract = |rows: &[serde_json::Value]| {
            for raw in rows {
                if let Some(market) = self.parse_market(raw) {
                    if let Some(ref req) = requested_status {
                        if market.status != *req {
                            continue;
                        }
                    }
                    all_markets.push(market);
                }
            }
        };

        // `cursor == Some("")` from Kalshi signals end-of-stream; collapse it to
        // `None` so we stop issuing requests.
        let next_or_none = |c: Option<String>| c.filter(|s| !s.is_empty());

        if let Some(resp) = live_resp {
            extract(&resp.markets);
            state.r = next_or_none(resp.cursor);
        } else {
            state.r = None;
        }

        if let Some(resp) = historical_resp {
            extract(&resp.markets);
            state.h = next_or_none(resp.cursor);
        } else {
            state.h = None;
        }

        let map_us = map_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.mapping_us",
            "exchange" => "kalshi",
            "operation" => "fetch_markets"
        )
        .record(map_us);

        let next_cursor = match (&state.r, &state.h) {
            (None, None) => None,
            _ => Some(serde_json::to_string(&state).unwrap_or_default()),
        };

        Ok((all_markets, next_cursor))
    }

    async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError> {
        #[derive(serde::Deserialize)]
        struct MarketResponse {
            market: serde_json::Value,
        }

        let path = format!("/markets/{market_id}");
        match self.get::<MarketResponse>(&path).await {
            Ok(resp) => self.parse_market(&resp.market).ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_id.into()))
            }),
            Err(KalshiError::MarketNotFound(_)) => {
                // Not a market ticker — try as event ticker
                self.fetch_event_as_market(market_id).await
            }
            Err(e) => Err(to_openpx(e)),
        }
    }

    async fn fetch_orderbook(
        &self,
        req: px_core::OrderbookRequest,
    ) -> Result<Orderbook, OpenPxError> {
        // For event tickers, token_id holds the resolved sub-market ticker.
        let ticker = req.token_id.as_deref().unwrap_or(&req.market_id);
        let (yes, no) = self.fetch_orderbook_raw(ticker).await.map_err(to_openpx)?;

        let outcome = req.outcome.as_deref().unwrap_or("Yes");
        // For multivariate event outcomes (e.g. "Natalie Anderson"), each sub-market
        // is binary yes/no on that contestant, so treat as Yes perspective.
        let is_no = outcome.eq_ignore_ascii_case("no");

        let (mut bids, mut asks): (Vec<PriceLevel>, Vec<PriceLevel>) = if is_no {
            let asks: Vec<PriceLevel> = yes
                .into_iter()
                .map(|level| PriceLevel::with_fixed(level.price.complement(), level.size))
                .collect();
            (no, asks)
        } else {
            let asks: Vec<PriceLevel> = no
                .into_iter()
                .map(|level| PriceLevel::with_fixed(level.price.complement(), level.size))
                .collect();
            (yes, asks)
        };

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        Ok(Orderbook {
            market_id: req.market_id.clone(),
            asset_id: req.market_id,
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
            hash: None,
        })
    }

    async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        let period_interval = match req.interval {
            PriceHistoryInterval::OneMinute => 1,
            PriceHistoryInterval::OneHour => 60,
            PriceHistoryInterval::OneDay => 1440,
            other => {
                return Err(OpenPxError::Exchange(px_core::ExchangeError::NotSupported(
                    format!("kalshi does not support {} candlesticks", other.as_str()),
                )))
            }
        };

        let interval_secs = period_interval as i64 * 60;
        let now = chrono::Utc::now().timestamp();
        let start_ts = req.start_ts.unwrap_or(now - 86400);
        let end_ts = req.end_ts.unwrap_or(now);

        // Batch endpoint (no series_ticker needed).
        // For event tickers, token_id contains the resolved sub-market ticker.
        let ticker = req.token_id.as_deref().unwrap_or(&req.market_id);
        let path = format!(
            "/markets/candlesticks?market_tickers={}&start_ts={}&end_ts={}&period_interval={}&include_latest_before_start=true",
            ticker, start_ts, end_ts, period_interval
        );

        let resp: KalshiBatchCandlesticksResponse = self.get(&path).await.map_err(to_openpx)?;

        let market_data = resp
            .markets
            .into_iter()
            .find(|m| m.market_ticker == ticker)
            .ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(
                    "no candlestick data returned".into(),
                ))
            })?;

        let mut candles: Vec<Candlestick> = market_data
            .candlesticks
            .into_iter()
            .filter_map(|c| {
                // Parse OHLC prices: dollar strings or cents integers (÷100)
                let open = parse_price_value(&c.price.open_dollars)?;

                // CRITICAL: end_period_ts is the END of the period.
                // Subtract interval to get period START (what lightweight-charts expects).
                let ts = c.end_period_ts - interval_secs;
                let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)?;

                Some(Candlestick {
                    timestamp,
                    open,
                    high: parse_price_value(&c.price.high_dollars).unwrap_or(open),
                    low: parse_price_value(&c.price.low_dollars).unwrap_or(open),
                    close: parse_price_value(&c.price.close_dollars).unwrap_or(open),
                    volume: parse_numeric_value(&c.volume_fp).unwrap_or(0.0),
                    open_interest: parse_numeric_value(&c.open_interest_fp),
                })
            })
            .collect();

        candles.sort_by_key(|c| c.timestamp);
        Ok(candles)
    }

    async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        #[derive(Debug, serde::Deserialize)]
        struct TradesResponse {
            cursor: Option<String>,
            #[serde(default)]
            trades: Vec<Trade>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Trade {
            trade_id: Option<String>,
            created_time: chrono::DateTime<chrono::Utc>,
            /// Kalshi returns this as the YES price (0.0-1.0), even when taker_side=no.
            price: Option<f64>,
            yes_price_dollars: Option<String>,
            no_price: Option<f64>,
            no_price_dollars: Option<String>,
            count: Option<f64>,
            count_fp: Option<String>,
            taker_side: Option<String>,
        }

        let wants_no = req
            .outcome
            .as_deref()
            .is_some_and(|o| o.eq_ignore_ascii_case("no"));

        let ticker = req
            .token_id
            .as_deref()
            .or(req.market_ref.as_deref())
            .unwrap_or(&req.market_id);
        let limit = req.limit.unwrap_or(200).clamp(1, 1000);
        let mut path = format!("/markets/trades?ticker={}&limit={}", ticker, limit);
        if let Some(start_ts) = req.start_ts {
            path.push_str(&format!("&min_ts={}", start_ts));
        }
        if let Some(end_ts) = req.end_ts {
            path.push_str(&format!("&max_ts={}", end_ts));
        }
        if let Some(cursor) = &req.cursor {
            path.push_str(&format!("&cursor={}", cursor));
        }

        let resp: TradesResponse = self.get(&path).await.map_err(to_openpx)?;
        let next_cursor = resp.cursor.filter(|c| !c.is_empty());

        let trades = resp
            .trades
            .into_iter()
            .filter_map(|t| {
                let price_yes = if let Some(price) = t.price {
                    normalize_kalshi_trade_price(price)
                } else {
                    t.yes_price_dollars
                        .as_deref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .and_then(normalize_kalshi_trade_price)
                }?;

                // Parse no_price from exchange data; do NOT derive as 1-yes_price.
                let price_no = if let Some(np) = t.no_price {
                    normalize_kalshi_trade_price(np)
                } else {
                    t.no_price_dollars
                        .as_deref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .and_then(normalize_kalshi_trade_price)
                };

                let size =
                    if let Some(size) = t.count_fp.as_deref().and_then(|s| s.parse::<f64>().ok()) {
                        size
                    } else {
                        t.count?
                    };

                if size <= 0.0 {
                    return None;
                }

                let price = if wants_no {
                    match price_no {
                        Some(p) if p > 0.0 && p < 1.0 => p,
                        _ => {
                            counter!(
                                "openpx.kalshi.trade_no_price_null",
                                "operation" => "fetch_trades"
                            )
                            .increment(1);
                            return None; // skip trade when no_price unavailable
                        }
                    }
                } else {
                    price_yes
                };
                if price <= 0.0 || price >= 1.0 {
                    // Defensive: avoid emitting invalid normalized prices.
                    return None;
                }

                let aggressor_side = t
                    .taker_side
                    .as_deref()
                    .and_then(|s| match s.to_ascii_lowercase().as_str() {
                        "yes" => Some(if wants_no { "sell" } else { "buy" }),
                        "no" => Some(if wants_no { "buy" } else { "sell" }),
                        _ => None,
                    })
                    .map(str::to_string);

                Some(MarketTrade {
                    id: t.trade_id,
                    price,
                    size,
                    side: None,
                    aggressor_side,
                    timestamp: t.created_time,
                    source_channel: Cow::Borrowed("kalshi_rest_trade"),
                    tx_hash: None,
                    outcome: t.taker_side.as_deref().and_then(|s| {
                        match s.to_ascii_lowercase().as_str() {
                            "yes" => Some("Yes".to_string()),
                            "no" => Some("No".to_string()),
                            _ => None,
                        }
                    }),
                    yes_price: Some(price_yes),
                    no_price: price_no,
                    taker_address: None,
                })
            })
            .collect();

        Ok((trades, next_cursor))
    }

    async fn create_order(
        &self,
        market_id: &str,
        outcome: &str,
        side: OrderSide,
        price: f64,
        size: f64,
        params: HashMap<String, String>,
    ) -> Result<Order, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        if price <= 0.0 || price >= 1.0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "Price must be between 0 and 1".into(),
            )));
        }

        // Convert outcome to Kalshi side (yes/no)
        let kalshi_side = outcome.to_lowercase();
        if kalshi_side != "yes" && kalshi_side != "no" {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "Outcome must be 'Yes' or 'No'".into(),
            )));
        }

        // Convert side to Kalshi action
        let action = match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };

        // Format price as dollar string (e.g. 0.65 → "0.65") and size as fp string
        let price_dollars = format!("{:.2}", price);
        let count_fp = format!("{:.2}", size);

        #[derive(serde::Serialize)]
        struct CreateOrderRequest {
            ticker: String,
            action: String,
            side: String,
            #[serde(rename = "type")]
            order_type: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            time_in_force: Option<String>,
            count_fp: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            yes_price_dollars: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            no_price_dollars: Option<String>,
        }

        let time_in_force = kalshi_time_in_force(&params)?;

        let (yes_price_dollars, no_price_dollars) = if kalshi_side == "yes" {
            (Some(price_dollars), None)
        } else {
            (None, Some(price_dollars))
        };

        let request = CreateOrderRequest {
            ticker: market_id.to_string(),
            action: action.to_string(),
            side: kalshi_side.clone(),
            order_type: "limit".to_string(),
            time_in_force,
            count_fp,
            yes_price_dollars,
            no_price_dollars,
        };

        #[derive(serde::Deserialize)]
        struct CreateOrderResponse {
            order: serde_json::Value,
        }

        let resp: CreateOrderResponse = self
            .post("/portfolio/orders", &request)
            .await
            .map_err(to_openpx)?;

        Ok(self.parse_order(&resp.order))
    }

    async fn cancel_order(
        &self,
        order_id: &str,
        _market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct CancelResponse {
            order: serde_json::Value,
        }

        let path = format!("/portfolio/orders/{order_id}");
        let resp: CancelResponse = self.delete(&path).await.map_err(to_openpx)?;

        Ok(self.parse_order(&resp.order))
    }

    async fn fetch_order(
        &self,
        order_id: &str,
        _market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct OrderResponse {
            order: serde_json::Value,
        }

        let path = format!("/portfolio/orders/{order_id}");
        let resp: OrderResponse = self.get(&path).await.map_err(to_openpx)?;

        Ok(self.parse_order(&resp.order))
    }

    async fn fetch_open_orders(
        &self,
        _params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct OrdersResponse {
            orders: Vec<serde_json::Value>,
        }

        let path = "/portfolio/orders?status=resting";
        let resp: OrdersResponse = self.get(path).await.map_err(to_openpx)?;

        Ok(resp.orders.iter().map(|o| self.parse_order(o)).collect())
    }

    async fn fetch_positions(&self, market_id: Option<&str>) -> Result<Vec<Position>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct PositionsResponse {
            market_positions: Vec<serde_json::Value>,
        }

        let mut path = "/portfolio/positions?count_filter=position".to_string();
        if let Some(ticker) = market_id {
            path.push_str(&format!("&ticker={ticker}"));
        }

        let resp: PositionsResponse = self.get(&path).await.map_err(to_openpx)?;

        let positions: Vec<Position> = resp
            .market_positions
            .iter()
            .map(|p| self.parse_position(p))
            .filter(|p| p.size > 0.0)
            .collect();

        tracing::info!(
            exchange = "kalshi",
            ticker = market_id.unwrap_or("all"),
            count = positions.len(),
            "fetched positions"
        );

        Ok(positions)
    }

    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct BalanceResponse {
            balance: i64, // In cents
            #[serde(default)]
            portfolio_value: Option<i64>,
        }

        let path = "/portfolio/balance";
        let resp: BalanceResponse = self.get(path).await.map_err(to_openpx)?;

        let mut result = HashMap::new();
        result.insert("USD".to_string(), resp.balance as f64 / 100.0);
        if let Some(pv) = resp.portfolio_value {
            result.insert("portfolio_value".to_string(), pv as f64 / 100.0);
        }
        Ok(result)
    }

    async fn fetch_fills(
        &self,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct FillsResponse {
            fills: Vec<serde_json::Value>,
            #[allow(dead_code)]
            cursor: Option<String>,
        }

        let limit = limit.unwrap_or(100).clamp(1, 200);
        let mut path = format!("/portfolio/fills?limit={limit}");
        if let Some(ticker) = market_id {
            path.push_str(&format!("&ticker={ticker}"));
        }

        let resp: FillsResponse = self.get(&path).await.map_err(to_openpx)?;

        let fills: Vec<Fill> = resp
            .fills
            .iter()
            .filter_map(|f| self.parse_fill(f))
            .collect();

        tracing::info!(
            exchange = "kalshi",
            ticker = market_id.unwrap_or("all"),
            count = fills.len(),
            "fetched fills"
        );

        Ok(fills)
    }

    async fn fetch_balance_raw(&self) -> Result<serde_json::Value, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        let path = "/portfolio/balance";
        self.get::<serde_json::Value>(path).await.map_err(to_openpx)
    }

    fn describe(&self) -> ExchangeInfo {
        let authed = self.config.is_authenticated();
        ExchangeInfo {
            id: self.id(),
            name: self.name(),
            has_fetch_markets: true,
            has_create_order: authed,
            has_cancel_order: authed,
            has_fetch_positions: true,
            has_fetch_balance: true,
            has_fetch_orderbook: true,
            has_fetch_price_history: true,
            has_fetch_trades: true,
            has_fetch_user_activity: false,
            has_fetch_fills: true,
            has_fetch_server_time: false,
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: self.auth.is_some() && !self.config.demo,
            has_fetch_orderbook_history: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::kalshi_time_in_force;
    use std::collections::HashMap;

    #[test]
    fn order_type_defaults_to_gtc() {
        let params = HashMap::new();
        assert_eq!(kalshi_time_in_force(&params).unwrap(), None);
    }

    #[test]
    fn order_type_gtc_maps_to_none() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "gtc".to_string());
        assert_eq!(kalshi_time_in_force(&params).unwrap(), None);
    }

    #[test]
    fn order_type_ioc_maps_to_immediate_or_cancel() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "ioc".to_string());
        assert_eq!(
            kalshi_time_in_force(&params).unwrap(),
            Some("immediate_or_cancel".to_string())
        );
    }

    #[test]
    fn order_type_fok_maps_to_fill_or_kill() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "fok".to_string());
        assert_eq!(
            kalshi_time_in_force(&params).unwrap(),
            Some("fill_or_kill".to_string())
        );
    }

    #[test]
    fn invalid_order_type_is_rejected() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "market".to_string());
        assert!(kalshi_time_in_force(&params).is_err());
    }
}
