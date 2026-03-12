use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::events::canonical_event_id;
use crate::models::{
    Candlestick, Fill, Market, MarketStatus, MarketTrade, Order, OrderSide, Orderbook,
    OrderbookSnapshot, Position, PriceHistoryInterval, UnifiedMarket,
};

use super::config::{FetchMarketsParams, FetchOrdersParams, FetchUserActivityParams};
use super::manifest::ExchangeManifest;
use super::normalizers::extract_string;

#[async_trait]
pub trait Exchange: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    async fn fetch_markets(
        &self,
        params: Option<FetchMarketsParams>,
    ) -> Result<Vec<Market>, OpenPxError>;

    async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError>;

    async fn create_order(
        &self,
        market_id: &str,
        outcome: &str,
        side: OrderSide,
        price: f64,
        size: f64,
        params: HashMap<String, String>,
    ) -> Result<Order, OpenPxError>;

    async fn cancel_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError>;

    async fn fetch_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError>;

    async fn fetch_open_orders(
        &self,
        params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError>;

    async fn fetch_positions(&self, market_id: Option<&str>) -> Result<Vec<Position>, OpenPxError>;

    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError>;

    /// Refresh cached balance/allowance state if supported by the exchange.
    async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        Ok(())
    }

    /// Fetch L2 orderbook for a market outcome.
    /// Uses owned types for async compatibility.
    async fn fetch_orderbook(&self, req: OrderbookRequest) -> Result<Orderbook, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbook".into()),
        ))
    }

    /// Fetch historical OHLCV price history / candlestick data for a market outcome.
    async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_price_history".into()),
        ))
    }

    /// Fetch recent public trades ("tape") for a market outcome.
    /// Returns `(trades, next_cursor)` where `next_cursor` supports pagination.
    async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_trades".into()),
        ))
    }

    /// Fetch historical L2 orderbook snapshots for a market.
    /// Returns `(snapshots, next_cursor)` for pagination.
    async fn fetch_orderbook_history(
        &self,
        req: OrderbookHistoryRequest,
    ) -> Result<(Vec<OrderbookSnapshot>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbook_history".into()),
        ))
    }

    /// Fetch all markets within one source-native exchange event/group.
    ///
    /// Default behavior preserves compatibility by scanning all markets and filtering
    /// by `group_id`. Exchanges with native event endpoints should override this
    /// for performance.
    async fn fetch_event_markets(&self, group_id: &str) -> Result<Vec<UnifiedMarket>, OpenPxError> {
        let group_id = group_id.trim();
        if group_id.is_empty() {
            return Ok(Vec::new());
        }

        let markets = self.fetch_all_unified_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| m.group_id.as_deref() == Some(group_id))
            .collect())
    }

    /// Fetch raw balance response from exchange API
    async fn fetch_balance_raw(&self) -> Result<Value, OpenPxError> {
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_balance_raw".into()),
        ))
    }

    /// Fetch user activity (positions, trades, portfolio data) for a given address.
    // TODO(trade-history): Implement per-exchange. No exchange currently implements this.
    // Kalshi: GET /portfolio/fills returns user's fill history with fees, timestamps, maker/taker.
    // Polymarket: activity API provides user trade history.
    // Wire this up to a /api/v1/fills or /api/v1/trade-history endpoint and surface
    // in the terminal UI as a "My Fills" / "Trade History" view.
    async fn fetch_user_activity(
        &self,
        params: FetchUserActivityParams,
    ) -> Result<Value, OpenPxError> {
        let _ = params;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_user_activity".into()),
        ))
    }

    /// Fetch user's fill (trade execution) history for a market.
    async fn fetch_fills(
        &self,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        let _ = (market_id, limit);
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_fills".into()),
        ))
    }

    fn describe(&self) -> ExchangeInfo {
        ExchangeInfo {
            id: self.id(),
            name: self.name(),
            has_fetch_markets: true,
            has_create_order: true,
            has_cancel_order: true,
            has_fetch_positions: true,
            has_fetch_balance: true,
            has_fetch_orderbook: false,
            has_fetch_price_history: false,
            has_fetch_trades: false,
            has_fetch_events: false,
            has_fetch_user_activity: false,
            has_fetch_fills: false,
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: false,
            has_fetch_orderbook_history: false,
        }
    }

    /// Returns the exchange manifest containing connection and data mapping configuration.
    fn manifest(&self) -> &'static ExchangeManifest;

    /// Fetch ALL markets from exchange with internal pagination.
    /// Returns UnifiedMarket directly (normalized at source).
    async fn fetch_all_unified_markets(&self) -> Result<Vec<UnifiedMarket>, OpenPxError> {
        let mut all_markets = Vec::new();
        let mut cursor: Option<String> = None;
        let manifest = self.manifest();
        let page_size = manifest.pagination.max_page_size;

        loop {
            let sent_cursor = cursor.clone();
            let params = FetchMarketsParams {
                limit: Some(page_size),
                cursor: cursor.clone(),
            };
            let batch = self.fetch_markets(Some(params)).await?;
            let count = batch.len();

            if count == 0 {
                break; // Empty page = normal end of pagination
            }

            // Convert Market -> UnifiedMarket
            for market in batch {
                all_markets.push(self.to_unified_market(market)?);
            }

            if count < page_size {
                break; // Partial page = last page
            }

            // Update cursor for next iteration
            let next_cursor = Some(
                (cursor
                    .as_ref()
                    .and_then(|c| c.parse::<usize>().ok())
                    .unwrap_or(0)
                    + count)
                    .to_string(),
            );

            // Stuck-cursor guard: cursor didn't advance from what we just sent
            if next_cursor == sent_cursor {
                return Err(OpenPxError::Exchange(crate::ExchangeError::Api(format!(
                    "Pagination stuck at cursor={:?}, collected {} markets for {}",
                    next_cursor,
                    all_markets.len(),
                    self.id()
                ))));
            }

            cursor = next_cursor;
        }
        Ok(all_markets)
    }

    /// Convert internal Market to UnifiedMarket.
    /// Each exchange should override this with its specific field mapping logic.
    fn to_unified_market(&self, market: Market) -> Result<UnifiedMarket, OpenPxError> {
        let exchange_id = self.id();
        let openpx_id = UnifiedMarket::make_openpx_id(exchange_id, &market.id);

        // Default implementation - extract from metadata when available
        let metadata = &market.metadata;

        let status = if market.is_open() {
            MarketStatus::Active
        } else {
            MarketStatus::Closed
        };

        let market_type = metadata
            .get("market_type")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                if market.outcomes.len() == 2 {
                    "binary".to_string()
                } else {
                    "categorical".to_string()
                }
            });

        let token_ids = market.get_token_ids();
        let token_id_yes = token_ids.first().cloned();
        let token_id_no = token_ids.get(1).cloned();
        let outcomes = market.outcomes.clone();
        let outcome_tokens = market.get_outcome_tokens();
        // Support both top-level fields (e.g., Kalshi's `event_ticker`) and nested event IDs
        // (e.g., Polymarket's `events[0].id` in raw market JSON).
        let group_id = extract_string(metadata, &["event_ticker", "group_id", "events.0.id"]);
        let event_id = group_id
            .as_deref()
            .and_then(|gid| canonical_event_id(exchange_id, gid));

        Ok(UnifiedMarket {
            openpx_id,
            exchange: exchange_id.to_string(),
            group_id,
            event_id,
            id: market.id,
            slug: metadata
                .get("slug")
                .and_then(|v| v.as_str())
                .map(String::from),
            title: market.question.clone(),
            question: Some(market.question),
            description: market.description,
            status,
            market_type,
            token_id_yes,
            token_id_no,
            condition_id: metadata
                .get("conditionId")
                .and_then(|v| v.as_str())
                .map(String::from),
            volume: market.volume as i64,
            liquidity: Some(market.liquidity as i64),
            close_time: market.close_time,
            open_time: metadata
                .get("open_time")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            outcomes,
            outcome_tokens,
            outcome_prices: market.prices,
            volume_24h: metadata
                .get("volume24hr")
                .or_else(|| metadata.get("volume_24h"))
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .map(|v| v as i64),
            volume_1wk: metadata
                .get("volume1wk")
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .map(|v| v as i64),
            volume_1mo: metadata
                .get("volume1mo")
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .map(|v| v as i64),
            open_interest: metadata
                .get("open_interest")
                .or_else(|| metadata.get("openInterest"))
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                }),
            price_change_1d: metadata.get("oneDayPriceChange").and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            }),
            price_change_1h: metadata.get("oneHourPriceChange").and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            }),
            price_change_1wk: metadata.get("oneWeekPriceChange").and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            }),
            price_change_1mo: metadata.get("oneMonthPriceChange").and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            }),
            last_trade_price: metadata
                .get("lastTradePrice")
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .or_else(|| {
                    // Kalshi: last_price in cents → divide by 100
                    metadata
                        .get("last_price")
                        .and_then(|v| v.as_f64())
                        .map(|c| c / 100.0)
                }),
            best_bid: metadata
                .get("bestBid")
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .or_else(|| {
                    metadata
                        .get("yes_bid")
                        .and_then(|v| v.as_f64())
                        .map(|c| c / 100.0)
                }),
            best_ask: metadata
                .get("bestAsk")
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .or_else(|| {
                    metadata
                        .get("yes_ask")
                        .and_then(|v| v.as_f64())
                        .map(|c| c / 100.0)
                }),
            spread: metadata
                .get("spread")
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .or_else(|| {
                    // Auto-compute from bid/ask when not directly available
                    let bid = metadata
                        .get("bestBid")
                        .and_then(|v| v.as_f64())
                        .or_else(|| {
                            metadata
                                .get("yes_bid")
                                .and_then(|v| v.as_f64())
                                .map(|c| c / 100.0)
                        });
                    let ask = metadata
                        .get("bestAsk")
                        .and_then(|v| v.as_f64())
                        .or_else(|| {
                            metadata
                                .get("yes_ask")
                                .and_then(|v| v.as_f64())
                                .map(|c| c / 100.0)
                        });
                    bid.zip(ask).map(|(b, a)| a - b)
                }),
            min_order_size: metadata
                .get("minimum_order_size")
                .or_else(|| metadata.get("min_order_size"))
                .or_else(|| metadata.get("orderMinSize"))
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                }),
            tick_size: Some(market.tick_size),
            image_url: metadata
                .get("image")
                .and_then(|v| v.as_str())
                .map(String::from),
            icon_url: metadata
                .get("icon")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub has_fetch_markets: bool,
    pub has_create_order: bool,
    pub has_cancel_order: bool,
    pub has_fetch_positions: bool,
    pub has_fetch_balance: bool,
    pub has_fetch_orderbook: bool,
    pub has_fetch_price_history: bool,
    pub has_fetch_trades: bool,
    pub has_fetch_events: bool,
    pub has_fetch_user_activity: bool,
    pub has_fetch_fills: bool,
    pub has_approvals: bool,
    pub has_refresh_balance: bool,
    pub has_websocket: bool,
    pub has_fetch_orderbook_history: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Market;
    use serde_json::json;

    /// Minimal Exchange impl for testing `to_unified_market()`.
    struct StubExchange;

    #[async_trait]
    impl Exchange for StubExchange {
        fn id(&self) -> &'static str {
            "stub"
        }
        fn name(&self) -> &'static str {
            "Stub"
        }
        fn manifest(&self) -> &'static ExchangeManifest {
            &crate::manifests::KALSHI_MANIFEST
        }
        async fn fetch_markets(
            &self,
            _: Option<FetchMarketsParams>,
        ) -> Result<Vec<Market>, OpenPxError> {
            Ok(vec![])
        }
        async fn fetch_market(&self, _: &str) -> Result<Market, OpenPxError> {
            Err(OpenPxError::Other("stub".into()))
        }
        async fn create_order(
            &self,
            _: &str,
            _: &str,
            _: OrderSide,
            _: f64,
            _: f64,
            _: HashMap<String, String>,
        ) -> Result<Order, OpenPxError> {
            Err(OpenPxError::Other("stub".into()))
        }
        async fn cancel_order(&self, _: &str, _: Option<&str>) -> Result<Order, OpenPxError> {
            Err(OpenPxError::Other("stub".into()))
        }
        async fn fetch_order(&self, _: &str, _: Option<&str>) -> Result<Order, OpenPxError> {
            Err(OpenPxError::Other("stub".into()))
        }
        async fn fetch_open_orders(
            &self,
            _: Option<FetchOrdersParams>,
        ) -> Result<Vec<Order>, OpenPxError> {
            Ok(vec![])
        }
        async fn fetch_positions(&self, _: Option<&str>) -> Result<Vec<Position>, OpenPxError> {
            Ok(vec![])
        }
        async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
            Ok(HashMap::new())
        }
    }

    fn market_with_metadata(metadata: serde_json::Value) -> Market {
        Market {
            id: "test-1".to_string(),
            question: "Test?".to_string(),
            outcomes: vec!["Yes".to_string(), "No".to_string()],
            close_time: None,
            volume: 1000.0,
            liquidity: 100.0,
            prices: HashMap::new(),
            metadata,
            tick_size: 0.01,
            description: String::new(),
        }
    }

    #[test]
    fn volume_24h_polymarket_key() {
        let market = market_with_metadata(json!({ "volume24hr": 5000.0 }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_24h, Some(5000));
    }

    #[test]
    fn volume_24h_kalshi_snake_case_key() {
        let market = market_with_metadata(json!({ "volume_24h": 3000.0 }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_24h, Some(3000));
    }

    #[test]
    fn volume_24h_polymarket_key_takes_precedence() {
        // If both keys exist, Polymarket's `volume24hr` wins (checked first)
        let market = market_with_metadata(json!({ "volume24hr": 9000.0, "volume_24h": 1000.0 }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_24h, Some(9000));
    }

    #[test]
    fn volume_1wk_extracted() {
        let market = market_with_metadata(json!({ "volume1wk": 25000.0 }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_1wk, Some(25000));
    }

    #[test]
    fn volume_1wk_from_string() {
        let market = market_with_metadata(json!({ "volume1wk": "42000" }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_1wk, Some(42000));
    }

    #[test]
    fn volume_1wk_none_when_missing() {
        let market = market_with_metadata(json!({}));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_1wk, None);
    }

    #[test]
    fn min_order_size_extracted_from_polymarket() {
        let market = market_with_metadata(json!({ "minimum_order_size": 15.0 }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.min_order_size, Some(15.0));
    }

    #[test]
    fn min_order_size_from_string() {
        let market = market_with_metadata(json!({ "minimum_order_size": "5" }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.min_order_size, Some(5.0));
    }

    #[test]
    fn min_order_size_none_when_missing() {
        let market = market_with_metadata(json!({}));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.min_order_size, None);
    }

    #[test]
    fn all_volume_buckets_extracted() {
        let market = market_with_metadata(json!({
            "volume24hr": 1000.0,
            "volume1wk": 7000.0,
            "volume1mo": 30000.0,
        }));
        let unified = StubExchange.to_unified_market(market).unwrap();
        assert_eq!(unified.volume_24h, Some(1000));
        assert_eq!(unified.volume_1wk, Some(7000));
        assert_eq!(unified.volume_1mo, Some(30000));
    }
}

/// Request for fetching an L2 orderbook.
#[derive(Debug, Clone, Default)]
pub struct OrderbookRequest {
    pub market_id: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
}

/// Request for fetching price history / candlestick data.
#[derive(Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_id: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
    /// Condition ID for OI enrichment (Polymarket).
    pub condition_id: Option<String>,
    pub interval: PriceHistoryInterval,
    /// Unix seconds
    pub start_ts: Option<i64>,
    /// Unix seconds
    pub end_ts: Option<i64>,
}

/// Request for fetching recent public trades ("tape") for a market outcome.
#[derive(Debug, Clone, Default)]
pub struct TradesRequest {
    /// Exchange-native market identifier (as used by `UnifiedMarket.id` / `openpx_id`).
    pub market_id: String,
    /// Optional alternate market identifier for trade endpoints (e.g., Polymarket conditionId).
    /// When provided, exchanges should prefer this over `market_id`.
    pub market_ref: Option<String>,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
    /// Unix seconds (inclusive)
    pub start_ts: Option<i64>,
    /// Unix seconds (inclusive)
    pub end_ts: Option<i64>,
    /// Max number of trades to return (exchange-specific caps may apply).
    pub limit: Option<usize>,
    /// Opaque pagination cursor from a previous response.
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OrderbookHistoryRequest {
    pub market_id: String,
    pub token_id: Option<String>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

// TODO(rfq): Add Request-for-Quote methods to Exchange trait for institutional-size trades.
// RFQ workflow:
//   1. Requester posts RFQ: "I want to buy 100K YES shares on market X"
//   2. Market makers see the RFQ and submit competing quotes (price + size)
//   3. Requester reviews quotes, accepts the best one
//   4. Trade executes bilaterally off the order book — zero market impact
// Reduces slippage on large orders by avoiding walking the book. Polymarket CLOB API supports
// this natively. polyfill-rs has full RFQ support: create_rfq_request, cancel_rfq_request,
// get_rfq_requests, get_rfq_requester_quotes, get_rfq_best_quote, accept_rfq_quote, plus
// quoter-side: create_rfq_quote, cancel_rfq_quote, get_rfq_quoter_quotes, approve_rfq_order.
// Would need: Exchange trait methods, relay protocol additions, API handlers, SDK regeneration.

// TODO(more-exchanges): Add support for more prediction market exchanges.
// Current (full): Kalshi, Polymarket. Partial: Opinion, Limitless, Predictfun.
// Candidates: Manifold Markets, Metaculus, PredictIt, Probable, Myriad.
// See pmxt (github.com/pmxt-dev/pmxt) for reference implementations and data format conventions.

// TODO(order-scoring): Add is_order_scoring(order_id) method for Polymarket maker rewards.
// Polymarket has a maker rewards program where qualifying limit orders (resting on book,
// sufficiently tight spread, minimum size) earn reward tokens. Pro market makers (user type B)
// use this to optimize which orders to keep active and which to cancel.
// polyfill-rs has is_order_scoring(order_id) -> bool and are_orders_scoring(order_ids) ->
// HashMap<String, bool>. Polymarket CLOB API endpoint: GET /order-scoring/{order_id}.
// Also useful: batch variant for checking multiple orders at once.
