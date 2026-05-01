use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::OpenPxError;
use crate::models::{
    CreateOrderRequest, Fill, Market, MarketLineage, MarketTrade, Order, Orderbook,
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

    /// Submit a new order. The unified surface exposes only the six fields on
    /// `CreateOrderRequest`; cross-venue and per-venue knobs (post-only,
    /// expiration, idempotency keys, neg-risk overrides, builder/metadata,
    /// subaccounts) are not modelled. Each adapter generates whatever its
    /// upstream requires (e.g. Kalshi V2's required `client_order_id` and
    /// `self_trade_prevention_type`) internally.
    async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, OpenPxError>;

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

    /// Fetch a paginated tape of recent public trades for a market.
    /// Both Yes and No outcomes' trades are returned interleaved; consumers
    /// distinguish via `MarketTrade.outcome`.
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

    /// Submit multiple orders in one round-trip. Each request shares the
    /// same shape as `create_order`. Cap: 15 on Polymarket; token-budget on
    /// Kalshi.
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

/// Request for fetching recent public trades ("tape") for a market.
///
/// `asset_id` is the market identifier — Kalshi market ticker or Polymarket
/// Gamma slug. Both Yes and No outcomes' trades are returned interleaved;
/// use `MarketTrade.outcome` to distinguish.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TradesRequest {
    pub asset_id: String,
    /// Unix seconds (inclusive lower bound).
    pub start_ts: Option<i64>,
    /// Unix seconds (inclusive upper bound).
    pub end_ts: Option<i64>,
    /// Max trades to return. Capped per exchange (Kalshi 1000, Polymarket 500).
    pub limit: Option<usize>,
    /// Opaque cursor from a prior response.
    pub cursor: Option<String>,
}
