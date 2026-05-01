use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::models::{
    CreateOrderRequest, Fill, Market, MarketLineage, MarketTrade, Order, Orderbook,
    OrderbookImpact, OrderbookMicrostructure, OrderbookStats, Position,
};

use super::config::FetchMarketsParams;
use super::manifest::ExchangeManifest;

#[allow(async_fn_in_trait)]
pub trait Exchange: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    /// List markets, optionally filtered by status, ticker, event, or series.
    async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError>;

    /// Submit a new order.
    async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, OpenPxError>;

    /// Cancel one open order by its globally-unique `order_id`.
    async fn cancel_order(&self, order_id: &str) -> Result<Order, OpenPxError>;

    /// Fetch one order by its globally-unique `order_id`.
    async fn fetch_order(&self, order_id: &str) -> Result<Order, OpenPxError>;

    /// List the caller's open orders, optionally scoped to one `asset_id`.
    async fn fetch_open_orders(&self, asset_id: Option<&str>) -> Result<Vec<Order>, OpenPxError>;

    /// List the caller's open positions, optionally scoped to one `market_ticker`.
    async fn fetch_positions(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError>;

    /// Fetch the caller's account balance, keyed by currency code (`USD` on Kalshi, `USDC` on Polymarket).
    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError>;

    /// Refresh cached balance and on-chain allowance state from the exchange.
    async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        Ok(())
    }

    /// Fetch the full-depth L2 orderbook (bids + asks) for one `asset_id`.
    async fn fetch_orderbook(&self, asset_id: &str) -> Result<Orderbook, OpenPxError> {
        let _ = asset_id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbook".into()),
        ))
    }

    /// Fetch a paginated tape of recent public trades for one market.
    async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        let _ = req;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_trades".into()),
        ))
    }

    /// List the caller's fill (trade execution) history, optionally scoped to one `market_ticker`.
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

    /// Fetch the exchange's current wall-clock time in UTC.
    async fn fetch_server_time(&self) -> Result<DateTime<Utc>, OpenPxError> {
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_server_time".into()),
        ))
    }

    /// Fetch one market plus its parent event and series in a single round-trip.
    async fn fetch_market_lineage(
        &self,
        market_ticker: &str,
    ) -> Result<MarketLineage, OpenPxError> {
        let _ = market_ticker;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_market_lineage".into()),
        ))
    }

    /// Fetch full-depth L2 orderbooks for multiple assets in one round-trip.
    async fn fetch_orderbooks_batch(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<Vec<Orderbook>, OpenPxError> {
        let _ = asset_ids;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbooks_batch".into()),
        ))
    }

    /// Snapshot stats: top-of-book, mid, spread (bps), size-weighted mid, top-10 imbalance, and total depth.
    async fn fetch_orderbook_stats(&self, asset_id: &str) -> Result<OrderbookStats, OpenPxError> {
        let book = self.fetch_orderbook(asset_id).await?;
        Ok(crate::models::orderbook_stats(&book))
    }

    /// Average fill price and slippage (bps) for a market order of `size` contracts on each side.
    async fn fetch_orderbook_impact(
        &self,
        asset_id: &str,
        size: f64,
    ) -> Result<OrderbookImpact, OpenPxError> {
        if size <= 0.0 {
            return Err(OpenPxError::InvalidInput("size must be > 0".into()));
        }
        let book = self.fetch_orderbook(asset_id).await?;
        Ok(crate::models::orderbook_impact(&book, size))
    }

    /// Microstructure signals: depth within 10/50/100 bps, slope, max gap, and per-side level counts.
    async fn fetch_orderbook_microstructure(
        &self,
        asset_id: &str,
    ) -> Result<OrderbookMicrostructure, OpenPxError> {
        let book = self.fetch_orderbook(asset_id).await?;
        Ok(crate::models::orderbook_microstructure(&book))
    }

    /// Cancel all of the caller's open orders, optionally scoped to one `asset_id`.
    async fn cancel_all_orders(&self, asset_id: Option<&str>) -> Result<Vec<Order>, OpenPxError> {
        let _ = asset_id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("cancel_all_orders".into()),
        ))
    }

    /// Submit multiple orders in one round-trip (max 15 on Polymarket; token-budget on Kalshi).
    async fn create_orders_batch(
        &self,
        reqs: Vec<CreateOrderRequest>,
    ) -> Result<Vec<Order>, OpenPxError> {
        let _ = reqs;
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
            has_fetch_trades: false,
            has_fetch_fills: false,
            has_fetch_server_time: false,
            has_approvals: false,
            has_refresh_balance: false,
            has_websocket: false,
            has_fetch_market_lineage: false,
            has_fetch_orderbooks_batch: false,
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
    pub has_fetch_trades: bool,
    pub has_fetch_fills: bool,
    pub has_fetch_server_time: bool,
    pub has_approvals: bool,
    pub has_refresh_balance: bool,
    pub has_websocket: bool,
    pub has_fetch_market_lineage: bool,
    pub has_fetch_orderbooks_batch: bool,
    pub has_cancel_all_orders: bool,
    pub has_create_orders_batch: bool,
}

/// Request for `fetch_trades` — recent public trades ("tape") for one market.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TradesRequest {
    /// Market identifier — Kalshi market ticker or Polymarket Gamma slug (e.g. `"KXBTCD-25APR1517"`).
    pub asset_id: String,
    /// Inclusive lower bound, unix seconds (e.g. `1714521600`).
    pub start_ts: Option<i64>,
    /// Inclusive upper bound, unix seconds (e.g. `1714608000`).
    pub end_ts: Option<i64>,
    /// Max trades to return; capped at 1000 (Kalshi) / 500 (Polymarket) (e.g. `100`).
    pub limit: Option<usize>,
    /// Opaque cursor returned by a prior page (e.g. `"eyJvIjoxMDB9"`).
    pub cursor: Option<String>,
}
