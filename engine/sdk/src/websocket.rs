use async_trait::async_trait;

use px_core::error::{OpenPxError, WebSocketError};
use px_core::websocket::{
    ActivityStream, OrderBookWebSocket, OrderbookStream as CoreOrderbookStream, WebSocketState,
};
use px_exchange_kalshi::{KalshiConfig, KalshiWebSocket};
use px_exchange_limitless::{LimitlessConfig, LimitlessWebSocket};
use px_exchange_opinion::{OpinionConfig, OpinionWebSocket};
use px_exchange_predictfun::{PredictFunConfig, PredictFunWebSocket};
use px_exchange_polymarket::PolymarketWebSocket;

/// Enum dispatch over exchange-specific WebSocket implementations.
/// Mirrors `ExchangeInner` but for real-time streaming connections.
pub enum WebSocketInner {
    Kalshi(KalshiWebSocket),
    Polymarket(PolymarketWebSocket),
    Limitless(LimitlessWebSocket),
    Opinion(OpinionWebSocket),
    PredictFun(PredictFunWebSocket),
}

impl WebSocketInner {
    /// Create a WebSocket instance from an exchange ID and JSON config.
    /// Uses the same config format as `ExchangeInner::new()`.
    pub fn new(id: &str, config: serde_json::Value) -> Result<Self, OpenPxError> {
        let obj = config.as_object();
        match id {
            "kalshi" => {
                let mut cfg = KalshiConfig::new();
                if let Some(obj) = obj {
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
                }
                Ok(Self::Kalshi(
                    KalshiWebSocket::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
            }
            "polymarket" => {
                if let Some(obj) = obj {
                    if let (Some(key), Some(secret), Some(passphrase)) = (
                        obj.get("api_key").and_then(|v| v.as_str()),
                        obj.get("api_secret").and_then(|v| v.as_str()),
                        obj.get("api_passphrase").and_then(|v| v.as_str()),
                    ) {
                        return Ok(Self::Polymarket(PolymarketWebSocket::with_auth(
                            key.to_string(),
                            secret.to_string(),
                            passphrase.to_string(),
                        )));
                    }
                }
                Ok(Self::Polymarket(PolymarketWebSocket::new()))
            }
            "limitless" => {
                let mut cfg = LimitlessConfig::new();
                if let Some(obj) = obj {
                    if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_key(v);
                    }
                    if let Some(v) = obj.get("ws_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_ws_url(v);
                    }
                }
                Ok(Self::Limitless(LimitlessWebSocket::new(cfg)))
            }
            "opinion" => {
                let mut cfg = OpinionConfig::new();
                if let Some(obj) = obj {
                    if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_key(v);
                    }
                    if let Some(v) = obj.get("ws_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_ws_url(v);
                    }
                    if let Some(v) = obj.get("api_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_url(v);
                    }
                }
                Ok(Self::Opinion(
                    OpinionWebSocket::new(cfg)
                        .map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
            }
            "predictfun" => {
                let mut cfg = PredictFunConfig::new();
                if let Some(obj) = obj {
                    if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_key(v);
                    }
                    if let Some(v) = obj.get("ws_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_ws_url(v);
                    }
                    if let Some(v) = obj.get("api_url").and_then(|v| v.as_str()) {
                        cfg = cfg.with_api_url(v);
                    }
                    if obj.get("testnet").and_then(|v| v.as_bool()).unwrap_or(false) {
                        cfg = PredictFunConfig::testnet();
                        if let Some(v) = obj.get("api_key").and_then(|v| v.as_str()) {
                            cfg = cfg.with_api_key(v);
                        }
                    }
                    let jwt = obj.get("jwt").and_then(|v| v.as_str()).map(str::to_string);
                    if let Some(jwt) = jwt {
                        return Ok(Self::PredictFun(PredictFunWebSocket::with_jwt(cfg, jwt)));
                    }
                }
                Ok(Self::PredictFun(PredictFunWebSocket::new(cfg)))
            }
            _ => Err(OpenPxError::Config(format!("unknown exchange: {id}"))),
        }
    }
}

macro_rules! ws_dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            WebSocketInner::Kalshi(ws) => ws.$method($($arg),*).await,
            WebSocketInner::Polymarket(ws) => ws.$method($($arg),*).await,
            WebSocketInner::Limitless(ws) => ws.$method($($arg),*).await,
            WebSocketInner::Opinion(ws) => ws.$method($($arg),*).await,
            WebSocketInner::PredictFun(ws) => ws.$method($($arg),*).await,
        }
    };
}

#[async_trait]
impl OrderBookWebSocket for WebSocketInner {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        ws_dispatch!(self, connect)
    }

    async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        ws_dispatch!(self, disconnect)
    }

    async fn subscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        ws_dispatch!(self, subscribe, market_id)
    }

    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        ws_dispatch!(self, unsubscribe, market_id)
    }

    fn state(&self) -> WebSocketState {
        match self {
            Self::Kalshi(ws) => ws.state(),
            Self::Polymarket(ws) => ws.state(),
            Self::Limitless(ws) => ws.state(),
            Self::Opinion(ws) => ws.state(),
            Self::PredictFun(ws) => ws.state(),
        }
    }

    async fn orderbook_stream(
        &mut self,
        market_id: &str,
    ) -> Result<CoreOrderbookStream, WebSocketError> {
        ws_dispatch!(self, orderbook_stream, market_id)
    }

    async fn activity_stream(&mut self, market_id: &str) -> Result<ActivityStream, WebSocketError> {
        ws_dispatch!(self, activity_stream, market_id)
    }
}
