use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::models::{
    Candlestick, Fill, Market, MarketTrade, Order, OrderSide, Orderbook, OrderbookSnapshot,
    Position, PriceHistoryInterval,
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
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: false,
            has_fetch_orderbook_history: false,
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
    pub has_approvals: bool,
    pub has_refresh_balance: bool,
    pub has_websocket: bool,
    pub has_fetch_orderbook_history: bool,
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
