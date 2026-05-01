// Re-export everything from px-core so users only need `openpx`.
pub use px_core::*;

mod config;
mod ws;
pub use ws::WebSocketInner;

pub use px_crypto::CryptoPriceWebSocket;
pub use px_sports::SportsWebSocket;

// Exchange implementations (re-export for direct construction)
pub use px_exchange_kalshi::{Kalshi, KalshiConfig};
pub use px_exchange_polymarket::{Polymarket, PolymarketConfig, PolymarketSignatureType};

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Enum dispatch over all supported exchanges.
/// Direct match dispatch eliminates vtable indirection and allows the compiler to
/// monomorphize and inline each exchange method — no heap-allocated `Pin<Box<dyn Future>>`.
pub enum ExchangeInner {
    Kalshi(Box<Kalshi>),
    Polymarket(Box<Polymarket>),
}

/// Dispatch macro: calls a trait method on the inner exchange via match + UFCS,
/// avoiding `&dyn Exchange` vtable overhead while ensuring trait methods are called
/// (not inherent methods that may shadow them with different signatures).
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ExchangeInner::Kalshi(e) => Exchange::$method(e.as_ref(), $($arg),*).await,
            ExchangeInner::Polymarket(e) => Exchange::$method(e.as_ref(), $($arg),*).await,
        }
    };
}

/// Sync dispatch macro for non-async trait methods.
macro_rules! dispatch_sync {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ExchangeInner::Kalshi(e) => Exchange::$method(e.as_ref(), $($arg),*),
            ExchangeInner::Polymarket(e) => Exchange::$method(e.as_ref(), $($arg),*),
        }
    };
}

impl ExchangeInner {
    /// Create an exchange instance from an ID string and a JSON config.
    pub fn new(id: &str, config: serde_json::Value) -> Result<Self, OpenPxError> {
        match id {
            "kalshi" => Ok(Self::Kalshi(Box::new(
                Kalshi::new(config::parse_kalshi(&config)?)
                    .map_err(|e| OpenPxError::Config(e.to_string()))?,
            ))),
            "polymarket" => Ok(Self::Polymarket(Box::new(
                Polymarket::new(config::parse_polymarket(&config)?)
                    .map_err(|e| OpenPxError::Config(e.to_string()))?,
            ))),
            _ => Err(OpenPxError::Config(format!("unknown exchange: {id}"))),
        }
    }

    pub fn id(&self) -> &'static str {
        dispatch_sync!(self, id)
    }

    pub fn name(&self) -> &'static str {
        dispatch_sync!(self, name)
    }

    pub fn describe(&self) -> ExchangeInfo {
        dispatch_sync!(self, describe)
    }

    pub async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError> {
        dispatch!(self, fetch_markets, params)
    }

    pub async fn create_order(&self, req: CreateOrderRequest) -> Result<Order, OpenPxError> {
        dispatch!(self, create_order, req)
    }

    pub async fn cancel_order(
        &self,
        order_id: &str,
        market_ticker: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        dispatch!(self, cancel_order, order_id, market_ticker)
    }

    pub async fn fetch_order(
        &self,
        order_id: &str,
        market_ticker: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        dispatch!(self, fetch_order, order_id, market_ticker)
    }

    pub async fn fetch_open_orders(
        &self,
        params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError> {
        dispatch!(self, fetch_open_orders, params)
    }

    pub async fn fetch_positions(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError> {
        dispatch!(self, fetch_positions, market_ticker)
    }

    pub async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
        dispatch!(self, fetch_balance)
    }

    pub async fn fetch_orderbook(&self, asset_id: &str) -> Result<Orderbook, OpenPxError> {
        dispatch!(self, fetch_orderbook, asset_id)
    }

    pub async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        dispatch!(self, fetch_trades, req)
    }

    pub async fn fetch_fills(
        &self,
        market_ticker: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        dispatch!(self, fetch_fills, market_ticker, limit)
    }

    pub async fn fetch_server_time(&self) -> Result<DateTime<Utc>, OpenPxError> {
        dispatch!(self, fetch_server_time)
    }

    pub async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        dispatch!(self, refresh_balance)
    }

    pub async fn fetch_market_lineage(
        &self,
        market_ticker: &str,
    ) -> Result<MarketLineage, OpenPxError> {
        dispatch!(self, fetch_market_lineage, market_ticker)
    }

    pub async fn fetch_orderbooks_batch(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<Vec<Orderbook>, OpenPxError> {
        dispatch!(self, fetch_orderbooks_batch, asset_ids)
    }

    pub async fn fetch_orderbook_stats(
        &self,
        asset_id: &str,
    ) -> Result<OrderbookStats, OpenPxError> {
        dispatch!(self, fetch_orderbook_stats, asset_id)
    }

    pub async fn fetch_orderbook_impact(
        &self,
        asset_id: &str,
        size: f64,
    ) -> Result<OrderbookImpact, OpenPxError> {
        dispatch!(self, fetch_orderbook_impact, asset_id, size)
    }

    pub async fn fetch_orderbook_microstructure(
        &self,
        asset_id: &str,
    ) -> Result<OrderbookMicrostructure, OpenPxError> {
        dispatch!(self, fetch_orderbook_microstructure, asset_id)
    }

    pub async fn cancel_all_orders(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Order>, OpenPxError> {
        dispatch!(self, cancel_all_orders, market_ticker)
    }

    pub async fn create_orders_batch(
        &self,
        reqs: Vec<CreateOrderRequest>,
    ) -> Result<Vec<Order>, OpenPxError> {
        dispatch!(self, create_orders_batch, reqs)
    }
}
