use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::models::{
    Candlestick, Event, Fill, LastTrade, Market, MarketTrade, Order, OrderSide, OrderType,
    Orderbook, OrderbookSnapshot, Position, PriceHistoryInterval, Series, Spread, Tag, UserTrade,
};

use super::config::{FetchMarketsParams, FetchOrdersParams, FetchUserActivityParams};
use super::manifest::ExchangeManifest;

#[allow(async_fn_in_trait)]
pub trait Exchange: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    /// Fetch one page of markets from this exchange.
    ///
    /// Returns `(markets, next_cursor)` where `next_cursor` is `None` when no more pages.
    /// Callers paginate externally by passing `next_cursor` back in `params.cursor`.
    /// When `params.status` is `None`, exchanges default to `Active`.
    /// When `params.status` is `Some(All)`, exchanges return all markets regardless of status.
    async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError>;

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

    /// Fetch the exchange's current wall-clock time. Useful for clock-skew
    /// correction in signing-sensitive paths and HFT replay where local
    /// `chrono::Utc::now()` would silently drift from the venue's clock.
    async fn fetch_server_time(&self) -> Result<DateTime<Utc>, OpenPxError> {
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_server_time".into()),
        ))
    }

    /// Fetch a page of events. An event groups one or more markets that share
    /// a resolution (Kalshi) or theme (Polymarket).
    /// Returns `(events, next_cursor)`.
    async fn fetch_events(
        &self,
        req: EventsRequest,
    ) -> Result<(Vec<Event>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_events".into()),
        ))
    }

    /// Fetch a single event by ID or slug (whichever the venue accepts).
    async fn fetch_event(&self, id: &str) -> Result<Event, OpenPxError> {
        let _ = id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_event".into()),
        ))
    }

    /// Fetch L2 orderbooks for multiple markets in one round-trip. Both
    /// venues expose a batch book endpoint (Kalshi `/markets/orderbooks`,
    /// Polymarket `/books`); this method exposes that as a unified primitive.
    async fn fetch_orderbooks_batch(
        &self,
        market_ids: Vec<String>,
    ) -> Result<Vec<Orderbook>, OpenPxError> {
        let _ = market_ids;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbooks_batch".into()),
        ))
    }

    /// Fetch a page of series. A series is a recurring family of events
    /// (weekly inflation prints, monthly NFP, sports seasons, etc.).
    /// Returns `(series, next_cursor)`.
    async fn fetch_series(
        &self,
        req: SeriesRequest,
    ) -> Result<(Vec<Series>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_series".into()),
        ))
    }

    /// Fetch a single series by ID or ticker.
    async fn fetch_series_one(&self, id: &str) -> Result<Series, OpenPxError> {
        let _ = id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_series_one".into()),
        ))
    }

    /// Fetch the midpoint price for a single market outcome. Cheaper than
    /// pulling a full orderbook when only the mark price is needed.
    async fn fetch_midpoint(&self, req: MidpointRequest) -> Result<f64, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_midpoint".into()),
        ))
    }

    /// Fetch midpoints for multiple market outcomes in one round-trip.
    /// Returns a map keyed by the input identifier (token_id on Polymarket,
    /// ticker on Kalshi).
    async fn fetch_midpoints_batch(
        &self,
        market_ids: Vec<String>,
    ) -> Result<HashMap<String, f64>, OpenPxError> {
        let _ = market_ids;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_midpoints_batch".into()),
        ))
    }

    /// Fetch the top-of-book spread (best bid, best ask, ask - bid).
    async fn fetch_spread(&self, req: MidpointRequest) -> Result<Spread, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_spread".into()),
        ))
    }

    /// Fetch the most recent public trade for a market outcome.
    async fn fetch_last_trade_price(&self, req: MidpointRequest) -> Result<LastTrade, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_last_trade_price".into()),
        ))
    }

    /// Fetch the open interest (total outstanding contracts) for a market.
    async fn fetch_open_interest(&self, market_id: &str) -> Result<f64, OpenPxError> {
        let _ = market_id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_open_interest".into()),
        ))
    }

    /// Fetch a user's trade history. Distinct from `fetch_fills` because some
    /// venues (Polymarket) carry on-chain fields like `tx_hash` and a
    /// `realized_pnl` that don't fit the `Fill` model. When
    /// `req.user_address` is `None`, returns the authenticated caller's own
    /// trades. When `Some(addr)`, returns a public lookup for that wallet
    /// (Polymarket only — Kalshi returns `NotSupported` in that case).
    async fn fetch_user_trades(
        &self,
        req: UserTradesRequest,
    ) -> Result<(Vec<UserTrade>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_user_trades".into()),
        ))
    }

    /// Fetch the tag set associated with a market (or its parent event).
    async fn fetch_market_tags(&self, market_id: &str) -> Result<Vec<Tag>, OpenPxError> {
        let _ = market_id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_market_tags".into()),
        ))
    }

    /// Cancel all of the caller's open orders. When `market_id.is_some()`,
    /// cancel only orders on that market.
    async fn cancel_all_orders(&self, market_id: Option<&str>) -> Result<Vec<Order>, OpenPxError> {
        let _ = market_id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("cancel_all_orders".into()),
        ))
    }

    /// Submit multiple orders in a single round-trip. Both venues cap the
    /// batch size — Polymarket at 15, Kalshi at a token-budget-dependent
    /// number. Implementations should reject batches that exceed their cap
    /// up-front rather than splitting silently.
    async fn create_orders_batch(&self, orders: Vec<NewOrder>) -> Result<Vec<Order>, OpenPxError> {
        let _ = orders;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("create_orders_batch".into()),
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
            has_fetch_user_activity: false,
            has_fetch_fills: false,
            has_fetch_server_time: false,
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: false,
            has_fetch_orderbook_history: false,
            has_fetch_events: false,
            has_fetch_event: false,
            has_fetch_orderbooks_batch: false,
            has_fetch_series: false,
            has_fetch_series_one: false,
            has_fetch_midpoint: false,
            has_fetch_midpoints_batch: false,
            has_fetch_spread: false,
            has_fetch_last_trade_price: false,
            has_fetch_open_interest: false,
            has_fetch_user_trades: false,
            has_fetch_market_tags: false,
            has_cancel_all_orders: false,
            has_create_orders_batch: false,
        }
    }

    /// Returns the exchange manifest containing connection and data mapping configuration.
    fn manifest(&self) -> &'static ExchangeManifest;
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    pub has_fetch_user_activity: bool,
    pub has_fetch_fills: bool,
    pub has_fetch_server_time: bool,
    pub has_approvals: bool,
    pub has_refresh_balance: bool,
    pub has_websocket: bool,
    pub has_fetch_orderbook_history: bool,
    pub has_fetch_events: bool,
    pub has_fetch_event: bool,
    pub has_fetch_orderbooks_batch: bool,
    pub has_fetch_series: bool,
    pub has_fetch_series_one: bool,
    pub has_fetch_midpoint: bool,
    pub has_fetch_midpoints_batch: bool,
    pub has_fetch_spread: bool,
    pub has_fetch_last_trade_price: bool,
    pub has_fetch_open_interest: bool,
    pub has_fetch_user_trades: bool,
    pub has_fetch_market_tags: bool,
    pub has_cancel_all_orders: bool,
    pub has_create_orders_batch: bool,
}

/// Request for fetching an L2 orderbook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrderbookRequest {
    pub market_id: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
}

/// Request for fetching price history / candlestick data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TradesRequest {
    /// Exchange-native market identifier.
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrderbookHistoryRequest {
    pub market_id: String,
    pub token_id: Option<String>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Request for fetching events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct EventsRequest {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
    /// Filter by lifecycle status — venue-specific values (e.g. `open`,
    /// `closed`, `settled` on Kalshi; `closed=true/false` on Polymarket).
    pub status: Option<String>,
    /// Filter by series ID / ticker.
    pub series_id: Option<String>,
    /// Include the events' nested `Market` objects when supported.
    pub with_nested_markets: Option<bool>,
    /// Unix seconds — only events closing at or after this timestamp.
    pub min_close_ts: Option<i64>,
    /// Unix seconds — only events updated at or after this timestamp.
    pub min_updated_ts: Option<i64>,
}

/// Request for fetching series.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SeriesRequest {
    pub limit: Option<usize>,
    pub cursor: Option<String>,
    /// Filter by category (e.g. `Politics`, `Sports`).
    pub category: Option<String>,
    /// Include traded volume in the response when the venue supports it.
    pub include_volume: Option<bool>,
}

/// Request for midpoint / spread / last-trade-price methods. The same shape
/// is reused for all three since they target the same outcome and accept the
/// same identifier inputs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MidpointRequest {
    pub market_id: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
}

/// Request for `fetch_user_trades`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UserTradesRequest {
    /// `None` = caller's own trades (auth required on both venues).
    /// `Some(addr)` = public lookup for `addr` (Polymarket only; Kalshi
    /// returns `NotSupported`).
    pub user_address: Option<String>,
    pub market_id: Option<String>,
    pub side: Option<OrderSide>,
    /// Unix seconds (inclusive)
    pub start_ts: Option<i64>,
    /// Unix seconds (inclusive)
    pub end_ts: Option<i64>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// One order in a `create_orders_batch` call. Each venue caps the batch size
/// (Polymarket: 15; Kalshi: token-budget-dependent).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NewOrder {
    pub market_id: String,
    pub outcome: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: f64,
    pub size: f64,
    /// Polymarket: pin maker-only. Ignored on Kalshi.
    pub post_only: Option<bool>,
    /// Kalshi: only allow size reductions. Maps to `reduce_only=true`.
    pub reduce_only: Option<bool>,
    /// Kalshi-specific idempotency key.
    pub client_order_id: Option<String>,
    /// Unix seconds. Required for `OrderType::Gtc` orders that should expire.
    pub expiration_ts: Option<i64>,
}
