use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::models::{
    Fill, Market, MarketLineage, MarketTrade, Order, OrderSide, OrderType, Orderbook,
    OrderbookImpact, OrderbookMicrostructure, OrderbookStats, Position,
};

use super::config::{FetchMarketsParams, FetchOrdersParams};
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

    /// Fetch the full-depth L2 orderbook (bids + asks) for a single asset.
    /// `asset_id` is the per-outcome identifier — Kalshi market ticker or Polymarket token id.
    async fn fetch_orderbook(&self, asset_id: &str) -> Result<Orderbook, OpenPxError> {
        let _ = asset_id;
        Err(OpenPxError::Exchange(
            crate::error::ExchangeError::NotSupported("fetch_orderbook".into()),
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

    /// Snapshot stats: top-of-book bid/ask, mid, spread (bps), size-weighted
    /// mid, top-10 imbalance, and total bid/ask depth — pure functions of the
    /// full-depth orderbook. `exchange_ts` carries the upstream snapshot time;
    /// `openpx_ts` is set to wall-clock when OpenPX served the response.
    async fn fetch_orderbook_stats(&self, asset_id: &str) -> Result<OrderbookStats, OpenPxError> {
        let book = self.fetch_orderbook(asset_id).await?;
        Ok(crate::models::orderbook_stats(&book))
    }

    /// Slippage curve at a single requested size. Walks the book consuming
    /// levels until `size` is filled or the side exhausts; returns partial
    /// fills with `fill_pct < 100.0`. `size` must be > 0.
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

    /// Microstructure signals: cumulative depth within 10/50/100 bps tiers,
    /// linear-regression slope of cumulative size vs distance-from-mid (bps),
    /// largest consecutive-level price gap, and per-side level counts.
    async fn fetch_orderbook_microstructure(
        &self,
        asset_id: &str,
    ) -> Result<OrderbookMicrostructure, OpenPxError> {
        let book = self.fetch_orderbook(asset_id).await?;
        Ok(crate::models::orderbook_microstructure(&book))
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
