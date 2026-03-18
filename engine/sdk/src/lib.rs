// Re-export everything from px-core so users only need `openpx`.
pub use px_core::*;

mod ws;
pub use ws::WebSocketInner;

pub use px_sports::SportsWebSocket;
pub use px_crypto::CryptoPriceWebSocket;

// Exchange implementations (re-export for direct construction)
pub use px_exchange_kalshi::{Kalshi, KalshiConfig};
pub use px_exchange_opinion::{Opinion, OpinionConfig};
pub use px_exchange_polymarket::{Polymarket, PolymarketConfig};

use std::collections::HashMap;

/// Enum dispatch over all supported exchanges.
/// Direct match dispatch eliminates vtable indirection and allows the compiler to
/// monomorphize and inline each exchange method — no heap-allocated `Pin<Box<dyn Future>>`.
pub enum ExchangeInner {
    Kalshi(Box<Kalshi>),
    Polymarket(Box<Polymarket>),
    Opinion(Opinion),
}

/// Dispatch macro: calls a trait method on the inner exchange via match + UFCS,
/// avoiding `&dyn Exchange` vtable overhead while ensuring trait methods are called
/// (not inherent methods that may shadow them with different signatures).
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ExchangeInner::Kalshi(e) => Exchange::$method(e.as_ref(), $($arg),*).await,
            ExchangeInner::Polymarket(e) => Exchange::$method(e.as_ref(), $($arg),*).await,
            ExchangeInner::Opinion(e) => Exchange::$method(e, $($arg),*).await,
        }
    };
}

/// Sync dispatch macro for non-async trait methods.
macro_rules! dispatch_sync {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            ExchangeInner::Kalshi(e) => Exchange::$method(e.as_ref(), $($arg),*),
            ExchangeInner::Polymarket(e) => Exchange::$method(e.as_ref(), $($arg),*),
            ExchangeInner::Opinion(e) => Exchange::$method(e, $($arg),*),
        }
    };
}

impl ExchangeInner {
    /// Create an exchange instance from an ID string and a JSON config.
    pub fn new(id: &str, config: serde_json::Value) -> Result<Self, OpenPxError> {
        match id {
            "kalshi" => {
                let mut cfg = KalshiConfig::new();
                if let Some(obj) = config.as_object() {
                    if let Some(v) = obj.get("api_key_id").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_key_id(v);
                    }
                    if let Some(v) = obj.get("private_key_pem").and_then(|v| v.as_str()) {
                        cfg = cfg.with_private_key_pem(v);
                    }
                    if let Some(v) = obj.get("private_key_path").and_then(|v| v.as_str()) {
                        cfg = cfg.with_private_key_path(v);
                    }
                    if let Some(v) = obj.get("api_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_url(v);
                    }
                    if obj.get("demo").and_then(|v| v.as_bool()).unwrap_or(false) {
                        cfg = KalshiConfig::demo();
                        if let Some(v) = obj.get("api_key_id").and_then(|v| v.as_str()) {
                            cfg = cfg.with_api_key_id(v);
                        }
                        if let Some(v) = obj.get("private_key_pem").and_then(|v| v.as_str()) {
                            cfg = cfg.with_private_key_pem(v);
                        }
                    }
                    if let Some(v) = obj.get("verbose").and_then(|v| v.as_bool()) {
                        cfg = cfg.with_verbose(v);
                    }
                }
                Ok(Self::Kalshi(Box::new(
                    Kalshi::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                )))
            }
            "polymarket" => {
                let mut cfg = PolymarketConfig::new();
                if let Some(obj) = config.as_object() {
                    if let Some(v) = obj.get("private_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_private_key(v);
                    }
                    if let Some(v) = obj.get("funder").and_then(|v| v.as_str()) {
                        cfg = cfg.with_funder(v);
                    }
                    if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                        if let (Some(secret), Some(passphrase)) = (
                            obj.get("api_secret").and_then(|v| v.as_str()),
                            obj.get("api_passphrase").and_then(|v| v.as_str()),
                        ) {
                            cfg = cfg.with_api_credentials(v, secret, passphrase);
                        }
                    }
                    if let Some(v) = obj.get("gamma_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_gamma_url(v);
                    }
                    if let Some(v) = obj.get("clob_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_clob_url(v);
                    }
                    if let Some(v) = obj.get("verbose").and_then(|v| v.as_bool()) {
                        cfg = cfg.with_verbose(v);
                    }
                }
                Ok(Self::Polymarket(Box::new(
                    Polymarket::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                )))
            }
            "opinion" => {
                let mut cfg = OpinionConfig::new();
                if let Some(obj) = config.as_object() {
                    if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_key(v);
                    }
                    if let Some(v) = obj.get("private_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_private_key(v);
                    }
                    if let Some(v) = obj.get("multi_sig_addr").and_then(|v| v.as_str()) {
                        cfg = cfg.with_multi_sig(v);
                    }
                    if let Some(v) = obj.get("api_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_url(v);
                    }
                    if let Some(v) = obj.get("verbose").and_then(|v| v.as_bool()) {
                        cfg = cfg.with_verbose(v);
                    }
                }
                Ok(Self::Opinion(
                    Opinion::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
            }
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

    pub async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError> {
        dispatch!(self, fetch_market, market_id)
    }

    pub async fn create_order(
        &self,
        market_id: &str,
        outcome: &str,
        side: OrderSide,
        price: f64,
        size: f64,
        params: HashMap<String, String>,
    ) -> Result<Order, OpenPxError> {
        dispatch!(
            self,
            create_order,
            market_id,
            outcome,
            side,
            price,
            size,
            params
        )
    }

    pub async fn cancel_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        dispatch!(self, cancel_order, order_id, market_id)
    }

    pub async fn fetch_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        dispatch!(self, fetch_order, order_id, market_id)
    }

    pub async fn fetch_open_orders(
        &self,
        params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError> {
        dispatch!(self, fetch_open_orders, params)
    }

    pub async fn fetch_positions(
        &self,
        market_id: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError> {
        dispatch!(self, fetch_positions, market_id)
    }

    pub async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
        dispatch!(self, fetch_balance)
    }

    pub async fn fetch_orderbook(&self, req: OrderbookRequest) -> Result<Orderbook, OpenPxError> {
        dispatch!(self, fetch_orderbook, req)
    }

    pub async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        dispatch!(self, fetch_price_history, req)
    }

    pub async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        dispatch!(self, fetch_trades, req)
    }

    pub async fn fetch_orderbook_history(
        &self,
        req: OrderbookHistoryRequest,
    ) -> Result<(Vec<OrderbookSnapshot>, Option<String>), OpenPxError> {
        dispatch!(self, fetch_orderbook_history, req)
    }

    pub async fn fetch_fills(
        &self,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        dispatch!(self, fetch_fills, market_id, limit)
    }

    pub async fn fetch_balance_raw(&self) -> Result<serde_json::Value, OpenPxError> {
        dispatch!(self, fetch_balance_raw)
    }

    pub async fn fetch_user_activity(
        &self,
        params: FetchUserActivityParams,
    ) -> Result<serde_json::Value, OpenPxError> {
        dispatch!(self, fetch_user_activity, params)
    }

    pub async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        dispatch!(self, refresh_balance)
    }
}
