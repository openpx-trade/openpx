use metrics::histogram;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use px_core::{
    manifests::KALSHI_MANIFEST, sort_asks, sort_bids, CreateOrderRequest, Event, Exchange,
    ExchangeInfo, ExchangeManifest, FetchMarketsParams, FetchOrdersParams, Fill, Market,
    MarketLineage, MarketStatus, MarketStatusFilter, MarketTrade, MarketType, NewOrder,
    OpenPxError, Order, OrderOutcome, OrderSide, OrderStatus, OrderType as UnifiedOrderType,
    Orderbook, Outcome, Position, PriceLevel, RateLimiter, Series, SettlementSource,
    TradesRequest,
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

pub(crate) fn to_openpx(e: KalshiError) -> OpenPxError {
    OpenPxError::Exchange(e.into())
}

/// Map the unified `OrderType` to Kalshi's `time_in_force` wire string.
/// Kalshi: `fill_or_kill` | `good_till_canceled` | `immediate_or_cancel`.
fn unified_to_kalshi_tif(t: UnifiedOrderType) -> Option<&'static str> {
    match t {
        UnifiedOrderType::Gtc => Some("good_till_canceled"),
        UnifiedOrderType::Fok => Some("fill_or_kill"),
        UnifiedOrderType::Ioc => Some("immediate_or_cancel"),
    }
}

fn parse_iso_datetime(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}

fn parse_kalshi_event(value: &serde_json::Value) -> Option<Event> {
    let obj = value.as_object()?;
    let ticker = obj
        .get("event_ticker")
        .and_then(|v| v.as_str())?
        .to_string();

    let title = obj
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let description = obj
        .get("sub_title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    let series_ticker = obj
        .get("series_ticker")
        .and_then(|v| v.as_str())
        .map(String::from);

    let mutually_exclusive = obj.get("mutually_exclusive").and_then(|v| v.as_bool());

    // Strike date doubles as the resolution / end timestamp.
    let end_ts = obj
        .get("strike_date")
        .and_then(|v| v.as_str())
        .and_then(parse_iso_datetime);

    let last_updated_ts = obj
        .get("last_updated_ts")
        .and_then(|v| v.as_str())
        .and_then(parse_iso_datetime);

    let market_tickers = obj
        .get("markets")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("ticker").and_then(|v| v.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Some(Event {
        ticker,
        numeric_id: None,
        title,
        description,
        category: None,
        series_ticker,
        status: None,
        market_tickers,
        start_ts: None,
        end_ts,
        volume: None,
        open_interest: None,
        mutually_exclusive,
        last_updated_ts,
    })
}

fn parse_kalshi_series(value: &serde_json::Value) -> Option<Series> {
    let obj = value.as_object()?;
    let ticker = obj.get("ticker").and_then(|v| v.as_str())?.to_string();

    let title = obj
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let category = obj
        .get("category")
        .and_then(|v| v.as_str())
        .map(String::from);

    let frequency = obj
        .get("frequency")
        .and_then(|v| v.as_str())
        .map(String::from);

    let tags = obj
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let settlement_sources = obj
        .get("settlement_sources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|s| SettlementSource {
                    name: s.get("name").and_then(|v| v.as_str()).map(String::from),
                    url: s.get("url").and_then(|v| v.as_str()).map(String::from),
                })
                .collect()
        })
        .unwrap_or_default();

    let fee_type = obj
        .get("fee_type")
        .and_then(|v| v.as_str())
        .map(String::from);

    // `volume_fp` arrives as a fixed-point string when include_volume=true.
    let volume = parse_fp(obj, "volume_fp");

    let last_updated_ts = obj
        .get("last_updated_ts")
        .and_then(|v| v.as_str())
        .and_then(parse_iso_datetime);

    Some(Series {
        ticker,
        numeric_id: None,
        title,
        category,
        frequency,
        tags,
        settlement_sources,
        fee_type,
        volume,
        last_updated_ts,
    })
}

pub struct Kalshi {
    config: KalshiConfig,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    auth: Option<KalshiAuth>,
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
        let ticker = obj.get("ticker").and_then(|v| v.as_str())?.to_string();

        // Composite title from the spec-canonical short titles. The `title`
        // field on Kalshi's Market schema is marked `deprecated: true` in the
        // OpenAPI spec; `yes_sub_title` / `no_sub_title` are the current
        // canonical short labels for each side of the binary contract.
        let title = {
            let yes = obj
                .get("yes_sub_title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let no = obj
                .get("no_sub_title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            format!("{yes} | {no}")
        };

        // Resolution rules: concat of rules_primary + rules_secondary.
        let rules = {
            let primary = obj.get("rules_primary").and_then(|v| v.as_str());
            let secondary = obj.get("rules_secondary").and_then(|v| v.as_str());
            match (primary, secondary) {
                (Some(p), Some(s)) if !p.is_empty() && !s.is_empty() => Some(format!("{p} | {s}")),
                (Some(p), _) if !p.is_empty() => Some(p.to_string()),
                (_, Some(s)) if !s.is_empty() => Some(s.to_string()),
                _ => None,
            }
        };

        // Native event ticker passed through verbatim.
        let event_ticker = obj
            .get("event_ticker")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

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

        // Binary markets: Yes/No outcomes — Kalshi has no per-outcome token id.
        let outcomes = vec![
            Outcome {
                label: "Yes".into(),
                price: Some(yes_price),
                token_id: None,
            },
            Outcome {
                label: "No".into(),
                price: Some(no_price),
                token_id: None,
            },
        ];

        // ── Volume (fixed-point migration) ──

        let volume = parse_fp(obj, "volume_fp")
            .or_else(|| obj.get("volume").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);

        let volume_24h = parse_fp(obj, "volume_24h_fp")
            .or_else(|| obj.get("volume_24h").and_then(|v| v.as_f64()));

        // ── Tick size: smallest step across the tiered `price_ranges` array.
        // The unified surface exposes a single number; callers placing orders
        // outside the finest tier still need to round to that tier's step,
        // but the smallest step is the right "what's the finest precision
        // anywhere on this market" answer. The legacy scalar `tick_size`
        // integer is deprecated upstream — we read `price_ranges` exclusively.
        let tick_size = obj
            .get("price_ranges")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter()
                    .filter_map(|r| {
                        r.as_object()?
                            .get("step")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                    })
                    .reduce(f64::min)
            });

        // ── Min order size: synthesized from fractional_trading_enabled. ──
        // No upstream field exists; fractional markets accept 0.01-contract
        // orders, otherwise the minimum is one whole contract.
        let fractional = obj
            .get("fractional_trading_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let min_order_size = Some(if fractional { 0.01 } else { 1.0 });

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

        let result = obj
            .get("result")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let openpx_id = Market::make_openpx_id("kalshi", &ticker);

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
            ticker,
            event_ticker,
            title,
            rules,
            status,
            market_type,
            outcomes,
            volume,
            volume_24h,
            last_trade_price: last_price,
            best_bid: yes_bid,
            best_ask: yes_ask,
            tick_size,
            min_order_size,
            close_time,
            open_time,
            created_at,
            settlement_time,
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

        let market_ticker = obj
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
            market_ticker,
            outcome,
            side,
            price,
            size,
            filled,
            fee: None,
            status,
            created_at,
            updated_at,
        }
    }

    fn parse_position(&self, data: &serde_json::Value) -> Position {
        let obj = data.as_object();

        let market_ticker = obj
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
            market_ticker,
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
        let market_ticker = obj
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
            market_ticker,
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

    /// Fetch a single market by ticker. Falls back to `fetch_event_as_market`
    /// when the ticker resolves to an event (multi-outcome categorical) instead.
    /// Used internally by Kalshi's own trait methods (orderbook lookup, fills, …)
    /// and by mapping/contract tests; not exposed on the unified `Exchange` trait —
    /// callers use `fetch_markets` with `market_tickers: vec![ticker]` instead.
    pub async fn fetch_market(&self, market_ticker: &str) -> Result<Market, OpenPxError> {
        #[derive(serde::Deserialize)]
        struct MarketResponse {
            market: serde_json::Value,
        }

        let path = format!("/markets/{market_ticker}");
        match self.get::<MarketResponse>(&path).await {
            Ok(resp) => self.parse_market(&resp.market).ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_ticker.into()))
            }),
            Err(KalshiError::MarketNotFound(_)) => {
                // Not a market ticker — try as event ticker
                self.fetch_event_as_market(market_ticker).await
            }
            Err(e) => Err(to_openpx(e)),
        }
    }

    /// Fetch a single Kalshi event by `event_ticker`. Used by
    /// `fetch_market_lineage`; not exposed on the unified `Exchange` trait.
    pub async fn fetch_event_by_ticker(&self, event_ticker: &str) -> Result<Event, OpenPxError> {
        #[derive(Debug, serde::Deserialize)]
        struct EventResp {
            event: serde_json::Value,
            #[serde(default)]
            markets: Option<Vec<serde_json::Value>>,
        }

        let path = format!("/events/{event_ticker}?with_nested_markets=true");
        let resp: EventResp = self.get(&path).await.map_err(to_openpx)?;

        let mut event = parse_kalshi_event(&resp.event).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "could not parse event: {event_ticker}"
            )))
        })?;

        // Top-level `markets` is sibling of `event` when `with_nested_markets=false`
        // — fold it in if present and the event itself didn't carry it.
        if event.market_tickers.is_empty() {
            if let Some(markets) = resp.markets {
                event.market_tickers = markets
                    .iter()
                    .filter_map(|m| m.get("ticker").and_then(|v| v.as_str()).map(String::from))
                    .collect();
            }
        }

        Ok(event)
    }

    /// Fetch a single Kalshi series by `series_ticker`. Used by
    /// `fetch_market_lineage`; not exposed on the unified `Exchange` trait.
    pub async fn fetch_series_by_ticker(&self, series_ticker: &str) -> Result<Series, OpenPxError> {
        #[derive(Debug, serde::Deserialize)]
        struct SeriesResp {
            series: serde_json::Value,
        }

        let path = format!("/series/{series_ticker}?include_volume=true");
        let resp: SeriesResp = self.get(&path).await.map_err(to_openpx)?;
        parse_kalshi_series(&resp.series).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "could not parse series: {series_ticker}"
            )))
        })
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

        let rules = event_obj
            .get("subtitle")
            .or_else(|| event_obj.get("description"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let mut outcomes: Vec<Outcome> = Vec::with_capacity(markets.len());
        let mut total_volume = 0.0f64;
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
                });

            outcomes.push(Outcome {
                label: outcome_name,
                price: yes_price,
                token_id: None,
            });

            total_volume += parse_fp(obj, "volume_fp")
                .or_else(|| obj.get("volume").and_then(|v| v.as_f64()))
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

        Some(Market {
            openpx_id,
            exchange: "kalshi".into(),
            ticker: event_ticker.to_string(),
            event_ticker: Some(event_ticker.to_string()),
            title,
            rules,
            status: MarketStatus::Active,
            market_type: MarketType::Categorical,
            outcomes,
            volume: total_volume,
            close_time,
            tick_size: Some(0.01),
            min_order_size: Some(1.0),
            ..Default::default()
        })
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
        #[derive(serde::Deserialize)]
        struct MarketsResponse {
            markets: Vec<serde_json::Value>,
            cursor: Option<String>,
        }

        let filter_status = |markets: Vec<Market>, filter: MarketStatusFilter| -> Vec<Market> {
            let requested_status = match filter {
                MarketStatusFilter::Active => Some(MarketStatus::Active),
                MarketStatusFilter::Closed => Some(MarketStatus::Closed),
                MarketStatusFilter::Resolved => Some(MarketStatus::Resolved),
                MarketStatusFilter::All => return markets,
            };
            markets
                .into_iter()
                .filter(|m| Some(m.status) == requested_status)
                .collect()
        };

        // ── market_tickers short-circuit: explicit market lookup, single round-trip ──
        if !params.market_tickers.is_empty() {
            let joined = params.market_tickers.join(",");
            let endpoint = format!("/markets?tickers={joined}");
            let resp: MarketsResponse = self.get(&endpoint).await.map_err(to_openpx)?;
            let parsed: Vec<Market> = resp
                .markets
                .iter()
                .filter_map(|raw| self.parse_market(raw))
                .collect();
            let filter = params.status.unwrap_or(MarketStatusFilter::Active);
            return Ok((filter_status(parsed, filter), None));
        }

        // ── event_ticker short-circuit: fetch a single event's nested markets ──
        if let Some(ref event_ticker) = params.event_ticker {
            #[derive(serde::Deserialize)]
            struct SingleEventResponse {
                #[allow(dead_code)]
                event: serde_json::Value,
                markets: Vec<serde_json::Value>,
            }

            let path = format!("/events/{event_ticker}");
            let resp: SingleEventResponse = self.get(&path).await.map_err(to_openpx)?;
            let parsed: Vec<Market> = resp
                .markets
                .iter()
                .filter_map(|raw| self.parse_market(raw))
                .collect();
            let filter = params.status.unwrap_or(MarketStatusFilter::Active);
            return Ok((filter_status(parsed, filter), None));
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
            if let Some(ref st) = params.series_ticker {
                ep.push_str(&format!("&series_ticker={st}"));
            }
            if !cursor.is_empty() {
                ep.push_str(&format!("&cursor={cursor}"));
            }
            ep
        });

        let historical_endpoint = state.h.as_ref().map(|cursor| {
            let mut ep = format!("/historical/markets?limit={limit}");
            if let Some(ref st) = params.series_ticker {
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

    async fn fetch_orderbook(&self, asset_id: &str) -> Result<Orderbook, OpenPxError> {
        let (yes, no) = self
            .fetch_orderbook_raw(asset_id)
            .await
            .map_err(to_openpx)?;

        // Yes-perspective: yes-side bids as bids, no-side bids complemented as asks.
        let mut bids = yes;
        let mut asks: Vec<PriceLevel> = no
            .into_iter()
            .map(|level| PriceLevel::with_fixed(level.price.complement(), level.size))
            .collect();

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        Ok(Orderbook {
            asset_id: asset_id.to_string(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
            hash: None,
        })
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
            price: Option<f64>,
            yes_price_dollars: Option<String>,
            no_price: Option<f64>,
            no_price_dollars: Option<String>,
            count: Option<f64>,
            count_fp: Option<String>,
            taker_side: Option<String>,
        }

        let limit = req.limit.unwrap_or(200).clamp(1, 1000);
        let mut path = format!("/markets/trades?ticker={}&limit={}", req.asset_id, limit);
        if let Some(start_ts) = req.start_ts {
            path.push_str(&format!("&min_ts={start_ts}"));
        }
        if let Some(end_ts) = req.end_ts {
            path.push_str(&format!("&max_ts={end_ts}"));
        }
        if let Some(cursor) = &req.cursor {
            path.push_str(&format!("&cursor={cursor}"));
        }

        let resp: TradesResponse = self.get(&path).await.map_err(to_openpx)?;
        let next_cursor = resp.cursor.filter(|c| !c.is_empty());
        let openpx_ts = chrono::Utc::now();

        let trades = resp
            .trades
            .into_iter()
            .filter_map(|t| {
                let id = t.trade_id?;

                let price_yes = if let Some(price) = t.price {
                    normalize_kalshi_trade_price(price)
                } else {
                    t.yes_price_dollars
                        .as_deref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .and_then(normalize_kalshi_trade_price)
                }?;

                let price_no = if let Some(np) = t.no_price {
                    normalize_kalshi_trade_price(np)
                } else {
                    t.no_price_dollars
                        .as_deref()
                        .and_then(|s| s.parse::<f64>().ok())
                        .and_then(normalize_kalshi_trade_price)
                };

                let size = t
                    .count_fp
                    .as_deref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .or(t.count)?;
                if size <= 0.0 {
                    return None;
                }

                let (aggressor_side, outcome) = match t.taker_side.as_deref() {
                    Some(s) if s.eq_ignore_ascii_case("yes") => {
                        (Some("buy".to_string()), Some("Yes".to_string()))
                    }
                    Some(s) if s.eq_ignore_ascii_case("no") => {
                        (Some("sell".to_string()), Some("No".to_string()))
                    }
                    _ => (None, None),
                };

                Some(MarketTrade {
                    id,
                    price: price_yes,
                    size,
                    aggressor_side,
                    exchange_ts: t.created_time,
                    openpx_ts,
                    outcome,
                    yes_price: Some(price_yes),
                    no_price: price_no,
                    taker_address: None,
                })
            })
            .collect();

        Ok((trades, next_cursor))
    }

    async fn fetch_market_lineage(
        &self,
        market_ticker: &str,
    ) -> Result<MarketLineage, OpenPxError> {
        let market = self.fetch_market(market_ticker).await?;
        let event = match market.event_ticker.as_deref() {
            Some(t) => self.fetch_event_by_ticker(t).await.ok(),
            None => None,
        };
        let series = match event.as_ref().and_then(|e| e.series_ticker.as_deref()) {
            Some(t) => self.fetch_series_by_ticker(t).await.ok(),
            None => None,
        };
        Ok(MarketLineage {
            market,
            event,
            series,
        })
    }

    async fn fetch_orderbooks_batch(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<Vec<Orderbook>, OpenPxError> {
        if asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        if asset_ids.len() > 100 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "fetch_orderbooks_batch: Kalshi limit is 100 asset_ids per request".into(),
            )));
        }

        #[derive(serde::Deserialize)]
        struct BatchResp {
            #[serde(default)]
            orderbooks: Vec<BatchEntry>,
        }
        #[derive(serde::Deserialize)]
        struct BatchEntry {
            ticker: String,
            #[serde(default, alias = "orderbook")]
            orderbook_fp: Option<OrderbookData>,
        }
        #[derive(serde::Deserialize)]
        struct OrderbookData {
            #[serde(default, alias = "yes")]
            yes_dollars: Option<Vec<[String; 2]>>,
            #[serde(default, alias = "no")]
            no_dollars: Option<Vec<[String; 2]>>,
        }

        let query = asset_ids
            .iter()
            .map(|t| format!("tickers={t}"))
            .collect::<Vec<_>>()
            .join("&");
        let path = format!("/markets/orderbooks?{query}");
        let resp: BatchResp = self.get(&path).await.map_err(to_openpx)?;

        let now = chrono::Utc::now();
        let books = resp
            .orderbooks
            .into_iter()
            .map(|entry| {
                let mut bids = Vec::new();
                let mut asks = Vec::new();
                if let Some(data) = entry.orderbook_fp {
                    if let Some(yes) = data.yes_dollars {
                        for [p, s] in yes {
                            if let (Ok(price), Ok(size)) = (p.parse::<f64>(), s.parse::<f64>()) {
                                if price > 0.0 && size > 0.0 {
                                    bids.push(PriceLevel::new(price, size));
                                }
                            }
                        }
                    }
                    if let Some(no) = data.no_dollars {
                        for [p, s] in no {
                            if let (Ok(price), Ok(size)) = (p.parse::<f64>(), s.parse::<f64>()) {
                                // NO bids at price X mean YES asks at 1-X.
                                let yes_ask = 1.0 - price;
                                if yes_ask > 0.0 && yes_ask < 1.0 && size > 0.0 {
                                    asks.push(PriceLevel::new(yes_ask, size));
                                }
                            }
                        }
                    }
                }
                sort_bids(&mut bids);
                sort_asks(&mut asks);
                Orderbook {
                    asset_id: entry.ticker,
                    bids,
                    asks,
                    last_update_id: None,
                    timestamp: Some(now),
                    hash: None,
                }
            })
            .collect();
        Ok(books)
    }

    async fn cancel_all_orders(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Order>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        // Kalshi has no single "cancel all" verb. Two-step flow:
        // (1) GET /portfolio/orders?status=resting[&ticker=…] — collect order IDs.
        //     V1 only — there is no V2 GET endpoint for listing orders.
        // (2) DELETE /portfolio/events/orders/batched (V2 batch cancel).
        #[derive(serde::Deserialize)]
        struct OrdersResp {
            #[serde(default)]
            orders: Vec<serde_json::Value>,
            cursor: Option<String>,
        }

        let mut all_ids: Vec<String> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut path = String::from("/portfolio/orders?status=resting&limit=1000");
            if let Some(t) = market_ticker {
                path.push_str(&format!("&ticker={t}"));
            }
            if let Some(c) = cursor.as_deref() {
                path.push_str(&format!("&cursor={c}"));
            }
            let resp: OrdersResp = self.get(&path).await.map_err(to_openpx)?;
            for o in &resp.orders {
                if let Some(id) = o.get("order_id").and_then(|v| v.as_str()) {
                    all_ids.push(id.to_string());
                }
            }
            match resp.cursor.filter(|c| !c.is_empty()) {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        if all_ids.is_empty() {
            return Ok(Vec::new());
        }

        let body = serde_json::json!({
            "orders": all_ids
                .iter()
                .map(|id| serde_json::json!({ "order_id": id }))
                .collect::<Vec<_>>(),
        });

        #[derive(serde::Deserialize)]
        struct BatchCancelResp {
            #[serde(default)]
            orders: Vec<BatchCancelEntry>,
        }
        #[derive(serde::Deserialize)]
        struct BatchCancelEntry {
            order_id: String,
            #[serde(default)]
            reduced_by: Option<String>,
            #[serde(default)]
            error: Option<serde_json::Value>,
        }
        let resp: BatchCancelResp = self
            .request(
                reqwest::Method::DELETE,
                "/portfolio/events/orders/batched",
                Some(&body),
                Some("cancel_all_orders"),
            )
            .await
            .map_err(to_openpx)?;

        let now = chrono::Utc::now();
        let cancelled = resp
            .orders
            .into_iter()
            .filter(|e| e.error.is_none())
            .map(|e| {
                let cancelled_size = e
                    .reduced_by
                    .as_deref()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                Order {
                    id: e.order_id,
                    market_ticker: market_ticker.unwrap_or("").to_string(),
                    outcome: String::new(),
                    side: OrderSide::Buy,
                    price: 0.0,
                    size: cancelled_size,
                    filled: 0.0,
                    fee: None,
                    status: OrderStatus::Cancelled,
                    created_at: now,
                    updated_at: Some(now),
                }
            })
            .collect();
        Ok(cancelled)
    }

    async fn create_orders_batch(&self, orders: Vec<NewOrder>) -> Result<Vec<Order>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;
        if orders.is_empty() {
            return Ok(Vec::new());
        }

        let body_orders: Vec<serde_json::Value> = orders
            .iter()
            .map(|o| {
                let action = match o.side {
                    OrderSide::Buy => "buy",
                    OrderSide::Sell => "sell",
                };
                let kalshi_side = if o.outcome.eq_ignore_ascii_case("no") {
                    "no"
                } else {
                    "yes"
                };
                let mut entry = serde_json::json!({
                    "ticker": o.market_ticker,
                    "side": kalshi_side,
                    "action": action,
                    // Kalshi accepts decimal-dollar fixed-point strings post the
                    // 2026-04-02 fixed-point migration; the legacy integer
                    // `yes_price`/`no_price` fields are deprecated.
                    if kalshi_side == "yes" { "yes_price_dollars" } else { "no_price_dollars" }: format!("{:.4}", o.price),
                    "count_fp": format!("{:.2}", o.size),
                    "type": "limit",
                });
                if let Some(tif) = unified_to_kalshi_tif(o.order_type) {
                    entry["time_in_force"] = serde_json::Value::String(tif.to_string());
                }
                if let Some(p) = o.post_only {
                    entry["post_only"] = serde_json::Value::Bool(p);
                }
                if let Some(r) = o.reduce_only {
                    entry["reduce_only"] = serde_json::Value::Bool(r);
                }
                if let Some(c) = &o.client_order_id {
                    entry["client_order_id"] = serde_json::Value::String(c.clone());
                }
                if let Some(ts) = o.expiration_ts {
                    // Kalshi expects expiration in milliseconds.
                    entry["expiration_ts"] = serde_json::Value::from(ts * 1000);
                }
                entry
            })
            .collect();

        let body = serde_json::json!({ "orders": body_orders });

        #[derive(serde::Deserialize)]
        struct BatchCreateResp {
            #[serde(default)]
            orders: Vec<BatchCreateEntry>,
        }
        #[derive(serde::Deserialize)]
        struct BatchCreateEntry {
            #[serde(default)]
            order: Option<serde_json::Value>,
            #[serde(default)]
            client_order_id: Option<String>,
            #[serde(default)]
            error: Option<serde_json::Value>,
        }

        let resp: BatchCreateResp = self
            .request(
                reqwest::Method::POST,
                "/portfolio/orders/batched",
                Some(&body),
                Some("create_orders_batch"),
            )
            .await
            .map_err(to_openpx)?;

        let parsed: Vec<Order> = resp
            .orders
            .into_iter()
            .filter_map(|e| {
                if let Some(err) = e.error {
                    tracing::warn!(
                        client_order_id = ?e.client_order_id,
                        error = %err,
                        "Kalshi batch order rejected"
                    );
                    return None;
                }
                let o = e.order?;
                let obj = o.as_object()?;
                Some(Order {
                    id: obj
                        .get("order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    market_ticker: obj
                        .get("ticker")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    outcome: obj
                        .get("side")
                        .and_then(|v| v.as_str())
                        .map(|s| {
                            if s.eq_ignore_ascii_case("no") {
                                "No".to_string()
                            } else {
                                "Yes".to_string()
                            }
                        })
                        .unwrap_or_default(),
                    side: match obj.get("action").and_then(|v| v.as_str()) {
                        Some("sell") => OrderSide::Sell,
                        _ => OrderSide::Buy,
                    },
                    price: parse_dollars(obj, "yes_price_dollars")
                        .or_else(|| parse_dollars(obj, "no_price_dollars"))
                        .unwrap_or(0.0),
                    size: parse_fp(obj, "initial_count_fp").unwrap_or(0.0),
                    filled: parse_fp(obj, "fill_count_fp").unwrap_or(0.0),
                    fee: None,
                    status: match obj.get("status").and_then(|v| v.as_str()) {
                        Some("resting") => OrderStatus::Open,
                        Some("canceled") => OrderStatus::Cancelled,
                        Some("executed") => OrderStatus::Filled,
                        _ => OrderStatus::Open,
                    },
                    created_at: obj
                        .get("created_time")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(chrono::Utc::now),
                    updated_at: None,
                })
            })
            .collect();
        Ok(parsed)
    }

    async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        if req.price <= 0.0 || req.price >= 1.0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "price must be in (0, 1)".into(),
            )));
        }

        // Kalshi V2 quotes a single book in YES-frame: `bid` buys YES, `ask`
        // sells YES. NO-side intents are mirrored: buying NO at p is sending
        // an `ask` at (1 - p), and selling NO at p is sending a `bid` at
        // (1 - p). The unified `Order` we return reports caller-frame fields,
        // so we also un-mirror `average_fill_price` on the way back.
        let outcome_label = match &req.outcome {
            OrderOutcome::Yes => "Yes",
            OrderOutcome::No => "No",
            _ => {
                return Err(OpenPxError::InvalidInput(
                    "Kalshi markets are binary; outcome must be Yes or No".into(),
                ))
            }
        };
        let (side_str, wire_price) = match (&req.outcome, req.side) {
            (OrderOutcome::Yes, OrderSide::Buy) => ("bid", req.price),
            (OrderOutcome::Yes, OrderSide::Sell) => ("ask", req.price),
            (OrderOutcome::No, OrderSide::Buy) => ("ask", 1.0 - req.price),
            (OrderOutcome::No, OrderSide::Sell) => ("bid", 1.0 - req.price),
            _ => unreachable!("guarded by outcome_label match"),
        };

        // V2 requires `client_order_id` and `self_trade_prevention_type`. The
        // unified surface does not expose either: the UUID is generated once
        // per call (idempotent across in-adapter retries), and STP is fixed
        // to `taker_at_cross` — the standard "incoming order takes if
        // marketable, otherwise rests" behavior.
        let body = serde_json::json!({
            "ticker": req.market_ticker,
            "client_order_id": uuid::Uuid::new_v4().to_string(),
            "side": side_str,
            "count": format!("{:.2}", req.size),
            "price": format!("{:.4}", wire_price),
            "time_in_force": match req.order_type {
                UnifiedOrderType::Gtc => "good_till_canceled",
                UnifiedOrderType::Ioc => "immediate_or_cancel",
                UnifiedOrderType::Fok => "fill_or_kill",
            },
            "self_trade_prevention_type": "taker_at_cross",
        });

        #[derive(serde::Deserialize)]
        struct V2Response {
            order_id: String,
            fill_count: String,
            remaining_count: String,
            #[serde(default)]
            average_fill_price: Option<String>,
            #[serde(default)]
            average_fee_paid: Option<String>,
        }

        let resp: V2Response = self
            .post("/portfolio/events/orders", &body)
            .await
            .map_err(to_openpx)?;

        let fill_count: f64 = resp.fill_count.parse().unwrap_or(0.0);
        let remaining_count: f64 = resp.remaining_count.parse().unwrap_or(0.0);

        let status = match (fill_count > 0.0, remaining_count > 0.0) {
            (true, true) => OrderStatus::PartiallyFilled,
            (true, false) => OrderStatus::Filled,
            (false, true) => OrderStatus::Open,
            (false, false) => OrderStatus::Cancelled,
        };

        let order_price = match resp.average_fill_price.as_deref().and_then(|s| s.parse().ok()) {
            Some(yes_price) => match req.outcome {
                OrderOutcome::Yes => yes_price,
                OrderOutcome::No => 1.0 - yes_price,
                _ => req.price,
            },
            None => req.price,
        };

        // V2 reports `average_fee_paid` as per-contract; multiply by fills
        // to get the dollar fee for this order.
        let fee = resp
            .average_fee_paid
            .as_deref()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|avg| avg * fill_count);

        Ok(Order {
            id: resp.order_id,
            market_ticker: req.market_ticker,
            outcome: outcome_label.to_string(),
            side: req.side,
            price: order_price,
            size: fill_count + remaining_count,
            filled: fill_count,
            fee,
            status,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }

    async fn cancel_order(
        &self,
        order_id: &str,
        _market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        // V2 response is `{ order_id, client_order_id?, reduced_by }` —
        // strictly thinner than V1's full Order object. Synthesize a sparse
        // unified `Order` describing what was cancelled; callers needing
        // pre-cancel state should `fetch_order` first.
        #[derive(serde::Deserialize)]
        struct V2Response {
            order_id: String,
            #[serde(default)]
            #[allow(dead_code)]
            client_order_id: Option<String>,
            reduced_by: String,
        }

        let path = format!("/portfolio/events/orders/{order_id}");
        let resp: V2Response = self.delete(&path).await.map_err(to_openpx)?;

        let cancelled_size: f64 = resp.reduced_by.parse().unwrap_or(0.0);
        let now = chrono::Utc::now();

        Ok(Order {
            id: resp.order_id,
            market_ticker: String::new(),
            outcome: String::new(),
            side: OrderSide::Buy,
            price: 0.0,
            size: cancelled_size,
            filled: 0.0,
            fee: None,
            status: OrderStatus::Cancelled,
            created_at: now,
            updated_at: Some(now),
        })
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

    async fn fetch_positions(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError> {
        self.ensure_auth().map_err(to_openpx)?;

        #[derive(serde::Deserialize)]
        struct PositionsResponse {
            market_positions: Vec<serde_json::Value>,
        }

        let mut path = "/portfolio/positions?count_filter=position".to_string();
        if let Some(ticker) = market_ticker {
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
            ticker = market_ticker.unwrap_or("all"),
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
        market_ticker: Option<&str>,
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
        if let Some(ticker) = market_ticker {
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
            ticker = market_ticker.unwrap_or("all"),
            count = fills.len(),
            "fetched fills"
        );

        Ok(fills)
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
            has_fetch_trades: true,
            has_fetch_fills: true,
            has_fetch_server_time: false,
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: self.auth.is_some() && !self.config.demo,
            has_fetch_market_lineage: true,
            has_fetch_orderbooks_batch: true,
            has_cancel_all_orders: self.config.is_authenticated(),
            has_create_orders_batch: self.config.is_authenticated(),
        }
    }
}

#[cfg(test)]
mod parse_event_series_tests {
    use super::{parse_kalshi_event, parse_kalshi_series};

    #[test]
    fn parse_kalshi_event_minimal_payload() {
        let v = serde_json::json!({
            "event_ticker": "KXPRES-2028",
            "series_ticker": "KXPRES",
            "title": "2028 US Presidential Election",
            "sub_title": "Winning candidate",
            "mutually_exclusive": true,
            "strike_date": "2028-11-07T00:00:00Z",
            "last_updated_ts": "2026-04-26T12:34:56Z",
        });
        let e = parse_kalshi_event(&v).expect("should parse");
        assert_eq!(e.ticker, "KXPRES-2028");
        assert!(e.numeric_id.is_none());
        assert_eq!(e.series_ticker.as_deref(), Some("KXPRES"));
        assert_eq!(e.title, "2028 US Presidential Election");
        assert_eq!(e.description.as_deref(), Some("Winning candidate"));
        assert_eq!(e.mutually_exclusive, Some(true));
        assert!(e.end_ts.is_some());
        assert!(e.last_updated_ts.is_some());
        assert!(e.market_tickers.is_empty());
    }

    #[test]
    fn parse_kalshi_event_with_nested_markets() {
        let v = serde_json::json!({
            "event_ticker": "EVT",
            "title": "T",
            "markets": [
                { "ticker": "M-A" },
                { "ticker": "M-B" },
                { "no_ticker_here": "skip" },
            ],
        });
        let e = parse_kalshi_event(&v).expect("parse");
        assert_eq!(e.market_tickers, vec!["M-A".to_string(), "M-B".to_string()]);
    }

    #[test]
    fn parse_kalshi_event_missing_required_returns_none() {
        let v = serde_json::json!({ "title": "no ticker" });
        assert!(parse_kalshi_event(&v).is_none());
    }

    #[test]
    fn parse_kalshi_series_with_volume() {
        let v = serde_json::json!({
            "ticker": "KXSPX",
            "title": "S&P 500 Daily",
            "category": "Economics",
            "frequency": "daily",
            "tags": ["finance", "indices"],
            "settlement_sources": [
                { "name": "S&P Dow Jones", "url": "https://example.com" },
            ],
            "fee_type": "quadratic",
            "volume_fp": "1234567.89",
            "last_updated_ts": "2026-04-27T00:00:00Z",
        });
        let s = parse_kalshi_series(&v).expect("parse");
        assert_eq!(s.ticker, "KXSPX");
        assert!(s.numeric_id.is_none());
        assert_eq!(s.frequency.as_deref(), Some("daily"));
        assert_eq!(s.tags, vec!["finance".to_string(), "indices".to_string()]);
        assert_eq!(s.settlement_sources.len(), 1);
        assert_eq!(s.fee_type.as_deref(), Some("quadratic"));
        assert_eq!(s.volume, Some(1_234_567.89));
    }

    #[test]
    fn parse_kalshi_series_without_volume_omits_field() {
        let v = serde_json::json!({ "ticker": "KX", "title": "x" });
        let s = parse_kalshi_series(&v).expect("parse");
        assert!(s.volume.is_none());
    }
}

