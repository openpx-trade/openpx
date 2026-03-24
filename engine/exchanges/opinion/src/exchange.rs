use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use std::borrow::Cow;

use px_core::{
    canonical_event_id, manifests::OPINION_MANIFEST, sort_asks, sort_bids, Candlestick, Exchange,
    ExchangeInfo, ExchangeManifest, FetchMarketsParams, FetchOrdersParams, FetchUserActivityParams,
    Fill, Market, MarketStatus, MarketStatusFilter, MarketTrade, MarketType, OpenPxError, Order,
    OrderSide, OrderStatus, Orderbook, OutcomeToken, Position, PriceHistoryInterval,
    PriceHistoryRequest, PriceLevel, PricePoint, RateLimiter, TradesRequest,
};

use crate::config::OpinionConfig;
use crate::error::OpinionError;

#[derive(Debug, serde::Deserialize)]
struct ApiResponse<T> {
    #[serde(alias = "errno", alias = "code")]
    code: i32,
    #[serde(alias = "errmsg", alias = "msg")]
    msg: Option<String>,
    result: Option<ApiResult<T>>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiResult<T> {
    data: Option<T>,
    list: Option<Vec<T>>,
}

/// Response structure for balance endpoint which returns data directly in `result`
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BalanceResponse {
    #[serde(alias = "errno", alias = "code")]
    code: i32,
    #[serde(alias = "errmsg", alias = "msg")]
    msg: Option<String>,
    result: Option<BalanceResult>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BalanceResult {
    wallet_address: Option<String>,
    multi_sign_address: Option<String>,
    chain_id: Option<String>,
    balances: Option<Vec<BalanceItem>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BalanceItem {
    quote_token: Option<String>,
    token_decimals: Option<u32>,
    total_balance: Option<String>,
    available_balance: Option<String>,
    frozen_balance: Option<String>,
}

pub struct Opinion {
    config: OpinionConfig,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

impl Opinion {
    pub fn new(config: OpinionConfig) -> Result<Self, OpinionError> {
        let client = reqwest::Client::builder()
            .http2_adaptive_window(true)
            .pool_max_idle_per_host(8)
            .http2_keep_alive_interval(std::time::Duration::from_secs(15))
            .timeout(config.base.timeout)
            .user_agent("openpx/1.0")
            .no_proxy()
            .build()?;

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
            config.base.rate_limit_per_second,
        )));

        Ok(Self {
            config,
            client,
            rate_limiter,
        })
    }

    pub fn with_default_config() -> Result<Self, OpinionError> {
        Self::new(OpinionConfig::default())
    }

    async fn rate_limit(&self) {
        self.rate_limiter.lock().await.wait().await;
    }

    fn auth_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut b = builder;
        if let Some(ref api_key) = self.config.api_key {
            b = b.header("apikey", api_key);
        }
        b
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<ApiResponse<T>, OpinionError> {
        self.rate_limit().await;

        let url = format!("{}{}", self.config.api_url, endpoint);
        tracing::debug!("Opinion GET: {}", url);
        let req = self.auth_headers(self.client.get(&url));
        let response = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Opinion request failed: {} - {:?}", url, e);
                return Err(OpinionError::Http(e));
            }
        };

        if response.status() == 429 {
            return Err(OpinionError::RateLimited);
        }

        if response.status() == 401 || response.status() == 403 {
            return Err(OpinionError::AuthRequired);
        }

        if !response.status().is_success() {
            let msg = response.text().await.unwrap_or_default();
            return Err(OpinionError::Api(msg));
        }

        response
            .json()
            .await
            .map_err(|e| OpinionError::Api(e.to_string()))
    }

    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &impl serde::Serialize,
    ) -> Result<ApiResponse<T>, OpinionError> {
        self.rate_limit().await;

        let url = format!("{}{}", self.config.api_url, endpoint);
        let req = self.auth_headers(self.client.post(&url)).json(body);
        let response = req.send().await?;

        if response.status() == 429 {
            return Err(OpinionError::RateLimited);
        }

        if response.status() == 401 || response.status() == 403 {
            return Err(OpinionError::AuthRequired);
        }

        if !response.status().is_success() {
            let msg = response.text().await.unwrap_or_default();
            return Err(OpinionError::Api(msg));
        }

        response
            .json()
            .await
            .map_err(|e| OpinionError::Api(e.to_string()))
    }

    fn ensure_auth(&self) -> Result<(), OpinionError> {
        if !self.config.is_authenticated() {
            return Err(OpinionError::AuthRequired);
        }
        Ok(())
    }

    fn parse_market(&self, data: serde_json::Value) -> Option<Market> {
        let obj = data.as_object()?;

        let id = obj
            .get("marketId")
            .or_else(|| obj.get("market_id"))
            .or_else(|| obj.get("topic_id"))
            .or_else(|| obj.get("id"))
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })?;

        let title = obj
            .get("marketTitle")
            .or_else(|| obj.get("market_title"))
            .or_else(|| obj.get("title"))
            .or_else(|| obj.get("question"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let yes_token = obj
            .get("yesTokenId")
            .or_else(|| obj.get("yes_token_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let no_token = obj
            .get("noTokenId")
            .or_else(|| obj.get("no_token_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let yes_label = obj
            .get("yesLabel")
            .or_else(|| obj.get("yes_label"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("Yes");
        let no_label = obj
            .get("noLabel")
            .or_else(|| obj.get("no_label"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("No");

        let mut outcomes = Vec::new();
        let mut outcome_tokens = Vec::new();
        let mut token_id_yes = None;
        let mut token_id_no = None;

        if let (Some(yt), Some(nt)) = (yes_token, no_token) {
            outcomes.push(yes_label.to_string());
            outcomes.push(no_label.to_string());
            outcome_tokens.push(OutcomeToken {
                outcome: yes_label.to_string(),
                token_id: yt.clone(),
            });
            outcome_tokens.push(OutcomeToken {
                outcome: no_label.to_string(),
                token_id: nt.clone(),
            });
            token_id_yes = Some(yt);
            token_id_no = Some(nt);
        } else if let Some(children) = obj
            .get("childMarkets")
            .or_else(|| obj.get("child_markets"))
            .and_then(|v| v.as_array())
        {
            for child in children {
                if let Some(child_title) = child
                    .get("marketTitle")
                    .or_else(|| child.get("market_title"))
                    .and_then(|t| t.as_str())
                {
                    outcomes.push(child_title.to_string());
                    if let Some(token) = child
                        .get("yesTokenId")
                        .or_else(|| child.get("yes_token_id"))
                        .and_then(|t| t.as_str())
                    {
                        outcome_tokens.push(OutcomeToken {
                            outcome: child_title.to_string(),
                            token_id: token.to_string(),
                        });
                    }
                }
            }
        }

        if outcomes.is_empty() {
            outcomes = vec!["Yes".into(), "No".into()];
        }

        let market_type = if outcomes.len() == 2 {
            MarketType::Binary
        } else {
            MarketType::Categorical
        };

        let volume = obj
            .get("volume")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let volume_24h = obj.get("volume24h").and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        });

        let volume_1wk = obj.get("volume7d").and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        });

        let liquidity_value = obj.get("liquidity").and_then(|v| v.as_f64());

        let open_interest = obj
            .get("openInterest")
            .or_else(|| obj.get("open_interest"))
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });

        let close_time = obj
            .get("cutoffAt")
            .or_else(|| obj.get("cutoff_at"))
            .or_else(|| obj.get("cutoff_time"))
            .and_then(|v| v.as_i64())
            .filter(|&t| t > 0)
            .and_then(|t| chrono::DateTime::from_timestamp(t, 0));

        let created_at = obj
            .get("createdAt")
            .or_else(|| obj.get("created_at"))
            .and_then(|v| v.as_i64())
            .filter(|&t| t > 0)
            .and_then(|t| chrono::DateTime::from_timestamp(t, 0));

        let settlement_time = obj
            .get("resolvedAt")
            .or_else(|| obj.get("resolved_at"))
            .and_then(|v| v.as_i64())
            .filter(|&t| t > 0)
            .and_then(|t| chrono::DateTime::from_timestamp(t, 0));

        let rules_str = obj.get("rules").and_then(|v| v.as_str()).map(String::from);

        let description = rules_str
            .clone()
            .or_else(|| {
                obj.get("description")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .unwrap_or_default();

        let denomination_token = obj
            .get("quoteToken")
            .or_else(|| obj.get("quote_token"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let condition_id = obj
            .get("conditionId")
            .or_else(|| obj.get("condition_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let question_id = obj
            .get("questionId")
            .or_else(|| obj.get("question_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let group_id = obj
            .get("groupId")
            .or_else(|| obj.get("group_id"))
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            });

        let event_id = group_id
            .as_deref()
            .and_then(|gid| canonical_event_id("opinion", gid));

        // Parse status from statusEnum (Activated, Resolved, Created, etc.)
        // or fall back to integer status field.
        let status = obj
            .get("statusEnum")
            .or_else(|| obj.get("status_enum"))
            .and_then(|v| v.as_str())
            .map(|s| match s.to_lowercase().as_str() {
                "activated" => MarketStatus::Active,
                "resolved" => MarketStatus::Resolved,
                _ => MarketStatus::Closed,
            })
            .or_else(|| {
                obj.get("status")
                    .and_then(|v| v.as_i64())
                    .map(|code| match code {
                        2 => MarketStatus::Active,
                        4 => MarketStatus::Resolved,
                        _ => MarketStatus::Closed,
                    })
            })
            .unwrap_or(MarketStatus::Active);

        let accepting_orders = status == MarketStatus::Active;

        Some(Market {
            openpx_id: Market::make_openpx_id("opinion", &id),
            exchange: "opinion".into(),
            id,
            group_id,
            event_id,
            title: title.clone(),
            question: Some(title),
            description,
            rules: rules_str,
            status,
            market_type,
            accepting_orders,
            outcomes,
            outcome_tokens,
            outcome_prices: HashMap::new(),
            token_id_yes,
            token_id_no,
            condition_id,
            question_id,
            volume,
            volume_24h,
            volume_1wk,
            liquidity: liquidity_value,
            open_interest,
            tick_size: Some(0.001),
            close_time,
            created_at,
            settlement_time,
            denomination_token,
            chain_id: Some(self.config.chain_id.to_string()),
            ..Default::default()
        })
    }

    fn parse_order(&self, data: &serde_json::Value) -> Order {
        let obj = data.as_object();

        let id = obj
            .and_then(|o| {
                o.get("orderId")
                    .or(o.get("order_id"))
                    .or(o.get("id"))
                    .or(o.get("orderID"))
            })
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })
            .unwrap_or_default();

        let market_id = obj
            .and_then(|o| {
                o.get("marketId")
                    .or(o.get("topic_id"))
                    .or(o.get("market_id"))
            })
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })
            .unwrap_or_default();

        let side = obj
            .and_then(|o| o.get("sideEnum").or(o.get("side_enum")).or(o.get("side")))
            .map(|v| {
                if let Some(s) = v.as_str() {
                    if s.to_lowercase() == "buy" {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    }
                } else if let Some(n) = v.as_i64() {
                    if n == 1 {
                        OrderSide::Buy
                    } else {
                        OrderSide::Sell
                    }
                } else {
                    OrderSide::Buy
                }
            })
            .unwrap_or(OrderSide::Buy);

        // Opinion API status codes: 1=Pending, 2=Filled, 3=Canceled, 4=Expired, 5=Failed
        let status = obj
            .and_then(|o| o.get("status"))
            .map(|v| {
                if let Some(n) = v.as_i64() {
                    match n {
                        1 => OrderStatus::Pending,
                        2 => OrderStatus::Filled,
                        3 => OrderStatus::Cancelled,
                        4 => OrderStatus::Cancelled, // Expired → Cancelled
                        5 => OrderStatus::Rejected,  // Failed → Rejected
                        _ => OrderStatus::Open,
                    }
                } else if let Some(s) = v.as_str() {
                    match s.to_lowercase().as_str() {
                        "filled" | "matched" => OrderStatus::Filled,
                        "cancelled" | "canceled" | "expired" => OrderStatus::Cancelled,
                        "partially_filled" => OrderStatus::PartiallyFilled,
                        "pending" => OrderStatus::Pending,
                        "failed" => OrderStatus::Rejected,
                        _ => OrderStatus::Open,
                    }
                } else {
                    OrderStatus::Open
                }
            })
            .unwrap_or(OrderStatus::Open);

        // Opinion API returns price as string (e.g., "0.65"); fall back to f64
        let price = obj
            .and_then(|o| o.get("price"))
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        let size = obj
            .and_then(|o| {
                o.get("orderShares")
                    .or(o.get("order_shares"))
                    .or(o.get("maker_amount"))
                    .or(o.get("size"))
            })
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        let filled = obj
            .and_then(|o| {
                o.get("filledShares")
                    .or(o.get("filled_shares"))
                    .or(o.get("matched_amount"))
                    .or(o.get("filled"))
            })
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        let outcome = obj
            .and_then(|o| o.get("outcome"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created_at = obj
            .and_then(|o| o.get("createdAt").or(o.get("created_at")))
            .and_then(|v| {
                v.as_i64()
                    .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
                    .or_else(|| {
                        v.as_str()
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    })
            })
            .unwrap_or_else(chrono::Utc::now);

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
            updated_at: None,
        }
    }

    fn parse_position(&self, data: &serde_json::Value) -> Position {
        let obj = data.as_object();

        let market_id = obj
            .and_then(|o| {
                o.get("marketId")
                    .or(o.get("topic_id"))
                    .or(o.get("market_id"))
            })
            .and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_i64().map(|n| n.to_string()))
            })
            .unwrap_or_default();

        let outcome = obj
            .and_then(|o| o.get("outcome").or(o.get("token_name")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let size = obj
            .and_then(|o| {
                o.get("sharesOwned")
                    .or(o.get("shares_owned"))
                    .or(o.get("size"))
                    .or(o.get("balance"))
            })
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        let average_price = obj
            .and_then(|o| {
                o.get("avgEntryPrice")
                    .or(o.get("avg_entry_price"))
                    .or(o.get("average_price"))
            })
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        let current_price = obj
            .and_then(|o| {
                o.get("currentPrice")
                    .or(o.get("current_price"))
                    .or(o.get("price"))
            })
            .and_then(|v| {
                v.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| v.as_f64())
            })
            .unwrap_or(0.0);

        Position {
            market_id,
            outcome,
            size,
            average_price,
            current_price,
        }
    }

    pub async fn get_orderbook(&self, token_id: &str) -> Result<Orderbook, OpinionError> {
        let endpoint = format!("/openapi/token/orderbook?token_id={}", token_id);
        let resp: ApiResponse<serde_json::Value> = self.get(&endpoint).await?;

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        if resp.code == 0 {
            if let Some(result) = resp.result {
                if let Some(data) = result.data {
                    if let Some(bid_arr) = data.get("bids").and_then(|v| v.as_array()) {
                        for item in bid_arr {
                            let price = item
                                .get("price")
                                .and_then(|p| {
                                    p.as_f64()
                                        .or_else(|| p.as_str().and_then(|s| s.parse().ok()))
                                })
                                .unwrap_or(0.0);
                            let size = item
                                .get("size")
                                .and_then(|s| {
                                    s.as_f64()
                                        .or_else(|| s.as_str().and_then(|s| s.parse().ok()))
                                })
                                .unwrap_or(0.0);
                            if price > 0.0 && size > 0.0 {
                                bids.push(PriceLevel::new(price, size));
                            }
                        }
                    }
                    if let Some(ask_arr) = data.get("asks").and_then(|v| v.as_array()) {
                        for item in ask_arr {
                            let price = item
                                .get("price")
                                .and_then(|p| {
                                    p.as_f64()
                                        .or_else(|| p.as_str().and_then(|s| s.parse().ok()))
                                })
                                .unwrap_or(0.0);
                            let size = item
                                .get("size")
                                .and_then(|s| {
                                    s.as_f64()
                                        .or_else(|| s.as_str().and_then(|s| s.parse().ok()))
                                })
                                .unwrap_or(0.0);
                            if price > 0.0 && size > 0.0 {
                                asks.push(PriceLevel::new(price, size));
                            }
                        }
                    }
                }
            }
        }

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        Ok(Orderbook {
            market_id: String::new(),
            asset_id: token_id.to_string(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
            hash: None,
        })
    }

    pub async fn fetch_price_history(
        &self,
        token_id: &str,
        interval: PriceHistoryInterval,
        start_at: Option<i64>,
        end_at: Option<i64>,
    ) -> Result<Vec<PricePoint>, OpinionError> {
        self.ensure_auth()?;

        let mut endpoint = format!(
            "/openapi/token/price-history?token_id={}&interval={}",
            token_id,
            interval.as_str()
        );

        if let Some(start) = start_at {
            endpoint.push_str(&format!("&start_at={start}"));
        }
        if let Some(end) = end_at {
            endpoint.push_str(&format!("&end_at={end}"));
        }

        let resp: ApiResponse<serde_json::Value> = self.get(&endpoint).await?;

        if resp.code != 0 {
            return Err(OpinionError::Api(
                resp.msg
                    .unwrap_or_else(|| "fetch price history failed".into()),
            ));
        }

        let history = resp
            .result
            .and_then(|r| {
                r.list
                    .or_else(|| r.data.and_then(|d| d.as_array().cloned()))
            })
            .unwrap_or_default();

        let mut points = Vec::with_capacity(history.len());
        for item in history {
            let t = item
                .get("timestamp")
                .or_else(|| item.get("t"))
                .and_then(|v| v.as_i64());
            let p = item.get("price").or_else(|| item.get("p")).and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });

            if let (Some(timestamp), Some(price)) = (t, p) {
                if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
                    points.push(PricePoint {
                        timestamp: dt,
                        price,
                        raw: item.clone(),
                    });
                }
            }
        }

        points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(points)
    }

    /// Internal: fetch a single page of markets. Not part of the public API.
    async fn fetch_markets_page(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError> {
        let limit = params.limit.unwrap_or(20).min(20);
        let offset = params
            .cursor
            .as_ref()
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);
        // Cursor is offset, convert to page number (1-indexed)
        let page = (offset / limit) + 1;

        let status_str = match params.status {
            Some(MarketStatusFilter::Active) | None => Some("activated"),
            Some(MarketStatusFilter::Closed) | Some(MarketStatusFilter::Resolved) => {
                Some("resolved")
            }
            Some(MarketStatusFilter::All) => None,
        };

        let mut endpoint = format!("/openapi/market?marketType=2&page={}&limit={}", page, limit);
        if let Some(s) = status_str {
            endpoint.push_str(&format!("&status={}", s));
        }

        let resp: ApiResponse<serde_json::Value> = self
            .get(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
                resp.msg.unwrap_or_else(|| "unknown error".into()),
            )));
        }

        let markets_list = resp.result.and_then(|r| r.list).unwrap_or_default();
        let api_count = markets_list.len();

        let filter = params.status.unwrap_or(MarketStatusFilter::Active);
        let markets: Vec<Market> = markets_list
            .into_iter()
            .filter_map(|v| self.parse_market(v))
            .filter(|m| {
                if filter == MarketStatusFilter::All {
                    return true;
                }
                let requested = match filter {
                    MarketStatusFilter::Active => MarketStatus::Active,
                    MarketStatusFilter::Closed => MarketStatus::Closed,
                    MarketStatusFilter::Resolved => MarketStatus::Resolved,
                    MarketStatusFilter::All => unreachable!(),
                };
                m.status == requested
            })
            .collect();

        // Base cursor on API response count, not post-filter count.
        // Client-side filtering can reduce the count below `limit` even when
        // more pages exist on the server.
        let next_cursor = if api_count == limit {
            Some((offset + api_count).to_string())
        } else {
            None
        };

        Ok((markets, next_cursor))
    }

    pub async fn fetch_positions_for_market(
        &self,
        market: &Market,
    ) -> Result<Vec<Position>, OpinionError> {
        self.ensure_auth()?;
        self.fetch_positions(Some(&market.id))
            .await
            .map_err(|e| OpinionError::Api(format!("{e}")))
    }

    pub async fn fetch_token_ids(&self, market_id: &str) -> Result<Vec<String>, OpinionError> {
        let market = self
            .fetch_market(market_id)
            .await
            .map_err(|e| OpinionError::Api(format!("{e}")))?;

        let token_ids = market.get_token_ids();

        if token_ids.is_empty() {
            return Err(OpinionError::Api(format!(
                "no token IDs found for market {market_id}"
            )));
        }

        Ok(token_ids)
    }

    /// Auto-paginate all trades for a wallet address.
    /// Endpoint: GET /openapi/trade/user/{address}?page={page}&limit=20
    async fn fetch_user_trades_all(
        &self,
        address: &str,
    ) -> Result<Vec<serde_json::Value>, OpinionError> {
        const PAGE_SIZE: usize = 20;
        let mut all = Vec::new();
        let mut page: usize = 1;

        loop {
            let endpoint = format!("/openapi/trade/user/{address}?page={page}&limit={PAGE_SIZE}");
            let resp: ApiResponse<serde_json::Value> = self.get(&endpoint).await?;

            if resp.code != 0 {
                return Err(OpinionError::Api(
                    resp.msg
                        .unwrap_or_else(|| "fetch user trades failed".into()),
                ));
            }

            let items = resp.result.and_then(|r| r.list).unwrap_or_default();
            let count = items.len();
            all.extend(items);

            if count < PAGE_SIZE {
                break;
            }
            page += 1;
        }

        Ok(all)
    }

    /// Auto-paginate all positions for a wallet address.
    /// Endpoint: GET /openapi/positions/user/{address}?page={page}&limit=20
    async fn fetch_user_positions_all(
        &self,
        address: &str,
    ) -> Result<Vec<serde_json::Value>, OpinionError> {
        const PAGE_SIZE: usize = 20;
        let mut all = Vec::new();
        let mut page: usize = 1;

        loop {
            let endpoint =
                format!("/openapi/positions/user/{address}?page={page}&limit={PAGE_SIZE}");
            let resp: ApiResponse<serde_json::Value> = self.get(&endpoint).await?;

            if resp.code != 0 {
                return Err(OpinionError::Api(
                    resp.msg
                        .unwrap_or_else(|| "fetch user positions failed".into()),
                ));
            }

            let items = resp.result.and_then(|r| r.list).unwrap_or_default();
            let count = items.len();
            all.extend(items);

            if count < PAGE_SIZE {
                break;
            }
            page += 1;
        }

        Ok(all)
    }

    pub async fn fetch_public_trades(
        &self,
        _market: Option<&Market>,
        _limit: Option<usize>,
        _page: Option<usize>,
        _side: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, OpinionError> {
        Ok(vec![])
    }
}

impl Exchange for Opinion {
    fn id(&self) -> &'static str {
        "opinion"
    }

    fn name(&self) -> &'static str {
        "Opinion"
    }

    fn manifest(&self) -> &'static ExchangeManifest {
        &OPINION_MANIFEST
    }

    async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError> {
        // ── event_id short-circuit: fetch by slug and return child markets ──
        if let Some(ref slug) = params.event_id {
            let endpoint = format!("/openapi/market/slug/{slug}");
            let resp: ApiResponse<serde_json::Value> = self
                .get(&endpoint)
                .await
                .map_err(|e| OpenPxError::Exchange(e.into()))?;

            let data = resp.result.and_then(|r| r.data).ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(slug.clone()))
            })?;

            let filter = params.status.unwrap_or(MarketStatusFilter::Active);
            let requested_status = match filter {
                MarketStatusFilter::Active => Some(MarketStatus::Active),
                MarketStatusFilter::Closed => Some(MarketStatus::Closed),
                MarketStatusFilter::Resolved => Some(MarketStatus::Resolved),
                MarketStatusFilter::All => None,
            };

            let mut markets = Vec::new();

            // If categorical with child markets, return each child as an individual market.
            if let Some(children) = data.get("childMarkets").and_then(|v| v.as_array()) {
                if !children.is_empty() {
                    let parent_id = data
                        .get("marketId")
                        .and_then(|v| {
                            v.as_str()
                                .map(String::from)
                                .or_else(|| v.as_i64().map(|n| n.to_string()))
                        })
                        .unwrap_or_default();

                    for child in children {
                        if let Some(mut market) = self.parse_market(child.clone()) {
                            if let Some(ref req) = requested_status {
                                if market.status != *req {
                                    continue;
                                }
                            }
                            if market.group_id.is_none() {
                                market.group_id = Some(parent_id.clone());
                            }
                            if market.event_id.is_none() {
                                market.event_id = canonical_event_id("opinion", &parent_id);
                            }
                            markets.push(market);
                        }
                    }
                    return Ok((markets, None));
                }
            }

            // Binary market or categorical without children — return the parent itself.
            if let Some(market) = self.parse_market(data) {
                if requested_status.is_none() || requested_status == Some(market.status) {
                    markets.push(market);
                }
            }
            return Ok((markets, None));
        }

        self.fetch_markets_page(params).await
    }

    async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError> {
        // Try binary endpoint first
        let binary_endpoint = format!("/openapi/market/{}", market_id);
        match self.get::<serde_json::Value>(&binary_endpoint).await {
            Ok(resp) if resp.code == 0 => {
                if let Some(market) = resp
                    .result
                    .and_then(|r| r.data)
                    .and_then(|d| self.parse_market(d))
                {
                    return Ok(market);
                }
            }
            // Non-recoverable errors — categorical endpoint would fail the same way
            Err(e @ (OpinionError::RateLimited | OpinionError::AuthRequired)) => {
                return Err(OpenPxError::Exchange(e.into()));
            }
            _ => {}
        }

        // Fall back to categorical endpoint
        let categorical_endpoint = format!("/openapi/market/categorical/{}", market_id);
        let resp: ApiResponse<serde_json::Value> = self
            .get(&categorical_endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(
                px_core::ExchangeError::MarketNotFound(market_id.into()),
            ));
        }

        resp.result
            .and_then(|r| r.data)
            .and_then(|d| self.parse_market(d))
            .ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_id.into()))
            })
    }

    async fn fetch_orderbook(
        &self,
        req: px_core::OrderbookRequest,
    ) -> Result<Orderbook, OpenPxError> {
        let token_id = if let Some(token_id) = req.token_id.clone() {
            token_id
        } else {
            let market = self.fetch_market(&req.market_id).await?;
            let token_ids = market.get_token_ids();
            if token_ids.is_empty() {
                return Err(OpenPxError::InvalidInput(
                    "no token IDs found for market".into(),
                ));
            }

            let outcomes = &market.outcomes;
            let is_yes_no = outcomes.len() == 2
                && outcomes.iter().any(|o| o.eq_ignore_ascii_case("yes"))
                && outcomes.iter().any(|o| o.eq_ignore_ascii_case("no"));

            let outcome_idx = match req.outcome.as_deref() {
                Some(raw) => {
                    if raw.trim().is_empty() {
                        return Err(OpenPxError::InvalidInput("invalid outcome".into()));
                    }
                    if let Ok(idx) = raw.parse::<usize>() {
                        idx
                    } else {
                        outcomes
                            .iter()
                            .position(|o| o.eq_ignore_ascii_case(raw))
                            .ok_or_else(|| OpenPxError::InvalidInput("invalid outcome".into()))?
                    }
                }
                None => {
                    if is_yes_no {
                        0
                    } else {
                        return Err(OpenPxError::InvalidInput(
                            "outcome required for non-binary markets".into(),
                        ));
                    }
                }
            };

            token_ids
                .get(outcome_idx)
                .cloned()
                .ok_or_else(|| OpenPxError::InvalidInput("token not found for outcome".into()))?
        };

        let mut orderbook = self
            .get_orderbook(&token_id)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;
        orderbook.market_id = req.market_id.clone();
        orderbook.asset_id = token_id;
        Ok(orderbook)
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
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let token_id = params.get("token_id").ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "token_id required in params".into(),
            ))
        })?;

        if price <= 0.0 || price >= 1.0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "Price must be between 0 and 1".into(),
            )));
        }

        let opinion_order_type = match params
            .get("order_type")
            .map(|s| s.as_str())
            .unwrap_or("gtc")
        {
            "gtc" => "LIMIT",
            "ioc" | "fok" => "MARKET_ORDER",
            other => {
                return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                    format!("invalid order_type '{other}' (allowed: gtc, ioc, fok)"),
                )));
            }
        };

        let order_data = serde_json::json!({
            "market_id": market_id.parse::<i64>().unwrap_or(0),
            "token_id": token_id,
            "side": if side == OrderSide::Buy { 1 } else { 2 },
            "price": price.to_string(),
            "size": size.to_string(),
            "order_type": opinion_order_type,
            "chain_id": self.config.chain_id,
        });

        let resp: ApiResponse<serde_json::Value> = self
            .post("/openapi/order", &order_data)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(
                px_core::ExchangeError::OrderRejected(
                    resp.msg.unwrap_or_else(|| "order failed".into()),
                ),
            ));
        }

        let order_id = resp
            .result
            .and_then(|r| r.data)
            .and_then(|d| d.get("order_id").and_then(|v| v.as_str()).map(String::from))
            .unwrap_or_default();

        Ok(Order {
            id: order_id,
            market_id: market_id.to_string(),
            outcome: outcome.to_string(),
            side,
            price,
            size,
            filled: 0.0,
            status: OrderStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }

    async fn cancel_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let cancel_data = serde_json::json!({
            "order_id": order_id,
            "chain_id": self.config.chain_id,
        });
        let resp: ApiResponse<serde_json::Value> = self
            .post("/openapi/order/cancel", &cancel_data)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
                resp.msg.unwrap_or_else(|| "cancel failed".into()),
            )));
        }

        Ok(Order {
            id: order_id.to_string(),
            market_id: market_id.unwrap_or("").to_string(),
            outcome: String::new(),
            side: OrderSide::Buy,
            price: 0.0,
            size: 0.0,
            filled: 0.0,
            status: OrderStatus::Cancelled,
            created_at: chrono::Utc::now(),
            updated_at: Some(chrono::Utc::now()),
        })
    }

    async fn fetch_order(
        &self,
        order_id: &str,
        _market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let endpoint = format!("/openapi/order/{}", order_id);
        let resp: ApiResponse<serde_json::Value> = self
            .get(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "Order {order_id} not found"
            ))));
        }

        let data = resp.result.and_then(|r| r.data).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Api("order not found".into()))
        })?;

        Ok(self.parse_order(&data))
    }

    async fn fetch_open_orders(
        &self,
        _params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let endpoint = "/openapi/order?status=1&page=1&limit=20".to_string();
        let resp: ApiResponse<serde_json::Value> = self
            .get(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Ok(vec![]);
        }

        let orders_list = resp.result.and_then(|r| r.list).unwrap_or_default();
        Ok(orders_list.iter().map(|o| self.parse_order(o)).collect())
    }

    async fn fetch_positions(
        &self,
        _market_id: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let address = self.config.multi_sig_addr.as_deref().ok_or_else(|| {
            OpenPxError::InvalidInput("multi_sig_addr required for Opinion positions".into())
        })?;

        let items = self
            .fetch_user_positions_all(address)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        Ok(items.iter().map(|p| self.parse_position(p)).collect())
    }

    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let endpoint = "/openapi/user/balance".to_string();

        self.rate_limit().await;

        let url = format!("{}{}", self.config.api_url, endpoint);
        let req = self.auth_headers(
            self.client
                .get(&url)
                .query(&[("chain_id", self.config.chain_id.to_string())]),
        );
        let response = req
            .send()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;

        if response.status() == 429 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
                "rate limited".into(),
            )));
        }

        if response.status() == 401 || response.status() == 403 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
                "authentication required".into(),
            )));
        }

        if !response.status().is_success() {
            let msg = response.text().await.unwrap_or_default();
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(msg)));
        }

        let resp: BalanceResponse = response
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
                resp.msg.unwrap_or_else(|| "fetch balance failed".into()),
            )));
        }

        let balance = resp
            .result
            .and_then(|r| r.balances)
            .and_then(|balances| balances.first().cloned())
            .and_then(|b| b.available_balance)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let mut result = HashMap::new();
        result.insert("USDC".to_string(), balance);
        Ok(result)
    }

    async fn fetch_user_activity(
        &self,
        params: FetchUserActivityParams,
    ) -> Result<serde_json::Value, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let address = &params.address;

        let (trades_result, positions_result) = tokio::join!(
            self.fetch_user_trades_all(address),
            self.fetch_user_positions_all(address),
        );

        let trades = trades_result.map_err(|e| OpenPxError::Exchange(e.into()))?;
        let positions = positions_result.map_err(|e| OpenPxError::Exchange(e.into()))?;

        Ok(serde_json::json!({
            "trades": trades,
            "positions": positions,
        }))
    }

    async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        let token_id = req.token_id.as_deref().ok_or_else(|| {
            OpenPxError::InvalidInput("token_id required for Opinion price history".into())
        })?;

        let points = self
            .fetch_price_history(token_id, req.interval, req.start_ts, req.end_ts)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        // Convert PricePoint (single price) to Candlestick (OHLCV).
        // Opinion's price-history endpoint returns point-in-time prices, not
        // true OHLCV bars, so O=H=L=C=price with volume=0.
        let candles = points
            .into_iter()
            .map(|p| Candlestick {
                timestamp: p.timestamp,
                open: p.price,
                high: p.price,
                low: p.price,
                close: p.price,
                volume: 0.0,
                open_interest: None,
            })
            .collect();

        Ok(candles)
    }

    async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let address = self.config.multi_sig_addr.as_deref().ok_or_else(|| {
            OpenPxError::InvalidInput("multi_sig_addr required for Opinion trades".into())
        })?;

        let limit = req.limit.unwrap_or(20).clamp(1, 100);
        let page = req
            .cursor
            .as_deref()
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(1);

        let endpoint = format!("/openapi/trade/user/{address}?page={page}&limit={limit}");
        let resp: ApiResponse<serde_json::Value> = self
            .get(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        if resp.code != 0 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
                resp.msg.unwrap_or_else(|| "fetch trades failed".into()),
            )));
        }

        let items = resp.result.and_then(|r| r.list).unwrap_or_default();
        let has_more = items.len() >= limit;

        let trades: Vec<MarketTrade> = items
            .iter()
            .filter_map(|item| {
                let obj = item.as_object()?;

                let price = obj.get("price").and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })?;
                let size = obj
                    .get("shares")
                    .or_else(|| obj.get("amount"))
                    .and_then(|v| {
                        v.as_f64()
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .unwrap_or(0.0);

                if size <= 0.0 {
                    return None;
                }

                let timestamp = obj
                    .get("created_at")
                    .or_else(|| obj.get("timestamp"))
                    .and_then(|v| {
                        v.as_i64()
                            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                            .or_else(|| {
                                v.as_str()
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                            })
                    })
                    .unwrap_or_else(chrono::Utc::now);

                let id = obj
                    .get("trade_id")
                    .or_else(|| obj.get("id"))
                    .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())));

                let side = obj.get("side").and_then(|v| {
                    v.as_str()
                        .or_else(|| v.as_i64().map(|n| if n == 1 { "buy" } else { "sell" }))
                });
                let aggressor_side = side.map(|s| {
                    match s.to_lowercase().as_str() {
                        "buy" | "1" => "buy",
                        _ => "sell",
                    }
                    .to_string()
                });

                // Filter by market if requested
                if !req.market_id.is_empty() {
                    let trade_market = obj
                        .get("market_id")
                        .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())));
                    if let Some(ref tm) = trade_market {
                        if tm != &req.market_id {
                            return None;
                        }
                    }
                }

                Some(MarketTrade {
                    id,
                    price,
                    size,
                    side: None,
                    aggressor_side,
                    timestamp,
                    source_channel: Cow::Borrowed("opinion_rest_trade"),
                    tx_hash: None,
                    outcome: obj
                        .get("outcome")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    yes_price: None,
                    no_price: None,
                    taker_address: None,
                })
            })
            .collect();

        let next_cursor = if has_more {
            Some((page + 1).to_string())
        } else {
            None
        };

        Ok((trades, next_cursor))
    }

    async fn fetch_fills(
        &self,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        self.ensure_auth()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let address = self.config.multi_sig_addr.as_deref().ok_or_else(|| {
            OpenPxError::InvalidInput("multi_sig_addr required for Opinion fills".into())
        })?;

        let all_trades = self
            .fetch_user_trades_all(address)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let limit = limit.unwrap_or(100);
        let fills: Vec<Fill> = all_trades
            .iter()
            .filter_map(|item| {
                let obj = item.as_object()?;

                // Filter by market if requested
                if let Some(mid) = market_id {
                    let trade_market = obj
                        .get("market_id")
                        .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())));
                    if trade_market.as_deref() != Some(mid) {
                        return None;
                    }
                }

                let fill_id = obj
                    .get("trade_id")
                    .or_else(|| obj.get("id"))
                    .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())))?;

                let order_id = obj
                    .get("order_id")
                    .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())))
                    .unwrap_or_default();

                let trade_market_id = obj
                    .get("market_id")
                    .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())))
                    .unwrap_or_default();

                let price = obj.get("price").and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })?;
                let size = obj
                    .get("shares")
                    .or_else(|| obj.get("amount"))
                    .and_then(|v| {
                        v.as_f64()
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .unwrap_or(0.0);

                if size <= 0.0 {
                    return None;
                }

                let side_val = obj
                    .get("side")
                    .and_then(|v| v.as_str().map(String::from).or_else(|| Some(v.to_string())));
                let side = match side_val.as_deref() {
                    Some("sell" | "2") => OrderSide::Sell,
                    _ => OrderSide::Buy,
                };

                let outcome = obj
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Yes")
                    .to_string();

                let fee = obj
                    .get("fee")
                    .and_then(|v| {
                        v.as_f64()
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .unwrap_or(0.0);

                let created_at = obj
                    .get("created_at")
                    .or_else(|| obj.get("timestamp"))
                    .and_then(|v| {
                        v.as_i64()
                            .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                            .or_else(|| {
                                v.as_str()
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                            })
                    })
                    .unwrap_or_else(chrono::Utc::now);

                Some(Fill {
                    fill_id,
                    order_id,
                    market_id: trade_market_id,
                    outcome,
                    side,
                    price,
                    size,
                    is_taker: false,
                    fee,
                    created_at,
                })
            })
            .take(limit)
            .collect();

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
            has_fetch_price_history: true,
            has_fetch_trades: true,
            has_fetch_user_activity: true,
            has_fetch_fills: true,
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: self.config.is_authenticated(),
            has_fetch_orderbook_history: false,
        }
    }
}
