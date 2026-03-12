use std::collections::HashMap;

use px_core::error::OpenPxError;
use px_core::models::{
    Candlestick, Fill, Market, MarketTrade, Order, OrderSide, Orderbook, OrderbookSnapshot,
    Position, UnifiedMarket,
};
use px_core::{
    ExchangeInfo, FetchMarketsParams, FetchOrdersParams, FetchUserActivityParams,
    OrderbookHistoryRequest, OrderbookRequest, PriceHistoryRequest, TradesRequest,
};

use px_exchange_kalshi::{Kalshi, KalshiConfig};
use px_exchange_limitless::{Limitless, LimitlessConfig};
use px_exchange_opinion::{Opinion, OpinionConfig};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};
use px_exchange_predictfun::{PredictFun, PredictFunConfig};

use px_core::Exchange;

/// Enum dispatch over all supported exchanges.
/// Both `px-python` and `px-node` depend on this to avoid duplicating exchange construction logic.
pub enum ExchangeInner {
    Kalshi(Kalshi),
    Polymarket(Box<Polymarket>),
    Opinion(Opinion),
    Limitless(Limitless),
    PredictFun(PredictFun),
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
                Ok(Self::Kalshi(
                    Kalshi::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
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
            "limitless" => {
                let mut cfg = LimitlessConfig::new();
                if let Some(obj) = config.as_object() {
                    if let Some(v) = obj.get("private_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_private_key(v);
                    }
                    if let Some(v) = obj.get("api_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_url(v);
                    }
                    if let Some(v) = obj.get("verbose").and_then(|v| v.as_bool()) {
                        cfg = cfg.with_verbose(v);
                    }
                }
                Ok(Self::Limitless(
                    Limitless::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
            }
            "predictfun" => {
                let mut cfg = PredictFunConfig::new();
                if let Some(obj) = config.as_object() {
                    if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_key(v);
                    }
                    if let Some(v) = obj.get("private_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_private_key(v);
                    }
                    if let Some(v) = obj.get("api_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_url(v);
                    }
                    if obj
                        .get("testnet")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        cfg = cfg.with_testnet(true);
                    }
                    if let Some(v) = obj.get("verbose").and_then(|v| v.as_bool()) {
                        cfg = cfg.with_verbose(v);
                    }
                }
                Ok(Self::PredictFun(
                    PredictFun::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
            }
            _ => Err(OpenPxError::Config(format!("unknown exchange: {id}"))),
        }
    }

    /// Returns a trait object reference — ensures we always call the Exchange trait methods
    /// rather than any inherent methods that may shadow them.
    fn as_exchange(&self) -> &dyn Exchange {
        match self {
            Self::Kalshi(e) => e,
            Self::Polymarket(e) => e.as_ref(),
            Self::Opinion(e) => e,
            Self::Limitless(e) => e,
            Self::PredictFun(e) => e,
        }
    }

    pub fn id(&self) -> &'static str {
        self.as_exchange().id()
    }

    pub fn name(&self) -> &'static str {
        self.as_exchange().name()
    }

    pub fn describe(&self) -> ExchangeInfo {
        self.as_exchange().describe()
    }

    pub async fn fetch_markets(
        &self,
        params: Option<FetchMarketsParams>,
    ) -> Result<Vec<Market>, OpenPxError> {
        self.as_exchange().fetch_markets(params).await
    }

    pub async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError> {
        self.as_exchange().fetch_market(market_id).await
    }

    pub async fn fetch_all_unified_markets(&self) -> Result<Vec<UnifiedMarket>, OpenPxError> {
        self.as_exchange().fetch_all_unified_markets().await
    }

    pub async fn fetch_event_markets(
        &self,
        group_id: &str,
    ) -> Result<Vec<UnifiedMarket>, OpenPxError> {
        self.as_exchange().fetch_event_markets(group_id).await
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
        self.as_exchange()
            .create_order(market_id, outcome, side, price, size, params)
            .await
    }

    pub async fn cancel_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.as_exchange().cancel_order(order_id, market_id).await
    }

    pub async fn fetch_order(
        &self,
        order_id: &str,
        market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        self.as_exchange().fetch_order(order_id, market_id).await
    }

    pub async fn fetch_open_orders(
        &self,
        params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError> {
        self.as_exchange().fetch_open_orders(params).await
    }

    pub async fn fetch_positions(
        &self,
        market_id: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError> {
        self.as_exchange().fetch_positions(market_id).await
    }

    pub async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
        self.as_exchange().fetch_balance().await
    }

    pub async fn fetch_orderbook(&self, req: OrderbookRequest) -> Result<Orderbook, OpenPxError> {
        self.as_exchange().fetch_orderbook(req).await
    }

    pub async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        self.as_exchange().fetch_price_history(req).await
    }

    pub async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        self.as_exchange().fetch_trades(req).await
    }

    pub async fn fetch_orderbook_history(
        &self,
        req: OrderbookHistoryRequest,
    ) -> Result<(Vec<OrderbookSnapshot>, Option<String>), OpenPxError> {
        self.as_exchange().fetch_orderbook_history(req).await
    }

    pub async fn fetch_fills(
        &self,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        self.as_exchange().fetch_fills(market_id, limit).await
    }

    pub async fn fetch_balance_raw(&self) -> Result<serde_json::Value, OpenPxError> {
        self.as_exchange().fetch_balance_raw().await
    }

    pub async fn fetch_user_activity(
        &self,
        params: FetchUserActivityParams,
    ) -> Result<serde_json::Value, OpenPxError> {
        self.as_exchange().fetch_user_activity(params).await
    }

    pub async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        self.as_exchange().refresh_balance().await
    }
}
