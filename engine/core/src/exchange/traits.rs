use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::models::{
    Candlestick, Fill, LastTrade, Market, MarketLineage, MarketTrade, Order, OrderSide, OrderType,
    Orderbook, OrderbookSnapshot, Position, PriceHistoryInterval, Spread, Tag, UserTrade,
};

use super::config::{FetchMarketsParams, FetchOrdersParams, FetchUserActivityParams};
use super::manifest::ExchangeManifest;

#[allow(async_fn_in_trait)]
pub trait Exchange: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    /// Fetch markets — `status` filter accepts `active`, `closed`, `resolved`, or `all` (defaults to `active`).
    /// Single-market and multi-market lookups both go through this surface: pass a single ticker via
    /// `params.market_tickers` to fetch one market, or omit it to page through the full catalog.
    async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError>;

    /// Submit a new order — `side` is `buy` or `sell`; `params["order_type"]` accepts `gtc`, `ioc`, or `fok`.
    async fn create_order(
        &self,
        market_ticker: &str,
        outcome: &str,
        side: OrderSide,
        price: f64,
        size: f64,
        params: HashMap<String, String>,
    ) -> Result<Order, OpenPxError>;

    /// Cancel an existing order by ID, optionally scoped to a market.
    async fn cancel_order(
        &self,
        order_id: &str,
        market_ticker: Option<&str>,
    ) -> Result<Order, OpenPxError>;

    /// Fetch a single order by ID, optionally scoped to a market.
    async fn fetch_order(
        &self,
        order_id: &str,
        market_ticker: Option<&str>,
    ) -> Result<Order, OpenPxError>;

    /// Fetch the caller's currently open orders, optionally filtered by market.
    async fn fetch_open_orders(
        &self,
        params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError>;

    /// Fetch the caller's open positions, optionally filtered by market.
    async fn fetch_positions(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError>;

    /// Fetch the caller's account balance, keyed by currency (USD on Kalshi, USDC on Polymarket).
    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError>;

    /// Refresh cached balance and allowance state from the exchange.
    async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        Ok(())
    }

    /// Fetch the L2 orderbook (bids + asks) for a single market outcome.
    async fn fetch_orderbook(&self, req: OrderbookRequest) -> Result<Orderbook, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbook".into()),
        ))
    }

    /// Fetch OHLCV candlestick history — `interval` is `1m`, `1h`, `6h`, `1d`, `1w`, or `max`.
    async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_price_history".into()),
        ))
    }

    /// Fetch a paginated tape of recent public trades for a market outcome.
    async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_trades".into()),
        ))
    }

    /// Fetch a paginated list of historical L2 orderbook snapshots for a market.
    async fn fetch_orderbook_history(
        &self,
        req: OrderbookHistoryRequest,
    ) -> Result<(Vec<OrderbookSnapshot>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbook_history".into()),
        ))
    }

    /// Fetch the raw upstream balance response (unprocessed JSON) from the exchange.
    async fn fetch_balance_raw(&self) -> Result<Value, OpenPxError> {
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_balance_raw".into()),
        ))
    }

    /// Fetch user activity (positions, trades, portfolio data) for a wallet address.
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

    /// Fetch the caller's fill (trade execution) history, optionally filtered by market.
    async fn fetch_fills(
        &self,
        market_ticker: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        let _ = (market_ticker, limit);
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_fills".into()),
        ))
    }

    /// Fetch the exchange's current wall-clock time (UTC).
    async fn fetch_server_time(&self) -> Result<DateTime<Utc>, OpenPxError> {
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_server_time".into()),
        ))
    }

    /// Fetch a market plus its parent event and series in one call. The
    /// `event` and `series` fields are `Option` — a dangling parent reference
    /// returns `None` rather than failing the whole call. `market_ticker` is
    /// the unified `Market.ticker` (Kalshi market ticker or Polymarket slug).
    async fn fetch_market_lineage(
        &self,
        market_ticker: &str,
    ) -> Result<MarketLineage, OpenPxError> {
        let _ = market_ticker;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_market_lineage".into()),
        ))
    }

    /// Fetch L2 orderbooks for multiple markets in one round-trip.
    async fn fetch_orderbooks_batch(
        &self,
        market_tickers: Vec<String>,
    ) -> Result<Vec<Orderbook>, OpenPxError> {
        let _ = market_tickers;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbooks_batch".into()),
        ))
    }

    /// Fetch the midpoint price for a single market outcome.
    async fn fetch_midpoint(&self, req: MidpointRequest) -> Result<f64, OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_midpoint".into()),
        ))
    }

    /// Fetch midpoint prices for multiple market outcomes in one round-trip.
    async fn fetch_midpoints_batch(
        &self,
        market_tickers: Vec<String>,
    ) -> Result<HashMap<String, f64>, OpenPxError> {
        let _ = market_tickers;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_midpoints_batch".into()),
        ))
    }

    /// Fetch the top-of-book spread (best bid, best ask, ask − bid) for a market outcome.
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
    async fn fetch_open_interest(&self, market_ticker: &str) -> Result<f64, OpenPxError> {
        let _ = market_ticker;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_open_interest".into()),
        ))
    }

    /// Fetch a wallet's trade history — optional `side` filter is `buy` or `sell`.
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
    async fn fetch_market_tags(&self, market_ticker: &str) -> Result<Vec<Tag>, OpenPxError> {
        let _ = market_ticker;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_market_tags".into()),
        ))
    }

    /// Cancel all of the caller's open orders, optionally scoped to a market.
    async fn cancel_all_orders(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Order>, OpenPxError> {
        let _ = market_ticker;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("cancel_all_orders".into()),
        ))
    }

    /// Submit multiple orders in one round-trip — each order's `side` is `buy`/`sell` and `order_type` is `gtc`, `ioc`, or `fok` (cap: 15 on Polymarket; token-budget on Kalshi).
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
            has_fetch_market_lineage: false,
            has_fetch_orderbooks_batch: false,
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
    pub has_fetch_market_lineage: bool,
    pub has_fetch_orderbooks_batch: bool,
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
    pub market_ticker: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
}

/// Request for fetching price history / candlestick data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PriceHistoryRequest {
    pub market_ticker: String,
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
    pub market_ticker: String,
    /// Optional alternate market identifier for trade endpoints (e.g., Polymarket conditionId).
    /// When provided, exchanges should prefer this over `market_ticker`.
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
    pub market_ticker: String,
    pub token_id: Option<String>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// Request for midpoint / spread / last-trade-price methods. The same shape
/// is reused for all three since they target the same outcome and accept the
/// same identifier inputs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MidpointRequest {
    pub market_ticker: String,
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
    pub market_ticker: Option<String>,
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
    pub market_ticker: String,
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
