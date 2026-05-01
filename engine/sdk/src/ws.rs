use px_core::error::{OpenPxError, WebSocketError};
use px_core::websocket::{OrderBookWebSocket, SessionStream, UpdateStream, WebSocketState};
use px_exchange_kalshi::KalshiWebSocket;
use px_exchange_polymarket::PolymarketWebSocket;

use crate::config;

/// Enum dispatch over exchange-specific WebSocket implementations.
/// Mirrors `ExchangeInner` but for real-time streaming connections.
pub enum WebSocketInner {
    Kalshi(KalshiWebSocket),
    Polymarket(PolymarketWebSocket),
}

impl WebSocketInner {
    /// Create a WebSocket instance from an exchange ID and JSON config.
    /// Uses the same config format as `ExchangeInner::new()`.
    pub fn new(id: &str, config: serde_json::Value) -> Result<Self, OpenPxError> {
        match id {
            "kalshi" => {
                let cfg = config::parse_kalshi(&config)?;
                Ok(Self::Kalshi(
                    KalshiWebSocket::new(cfg).map_err(|e| OpenPxError::Config(e.to_string()))?,
                ))
            }
            "polymarket" => {
                let cfg = config::parse_polymarket(&config)?;
                Ok(Self::Polymarket(PolymarketWebSocket::from_config(&cfg)))
            }
            _ => Err(OpenPxError::Config(format!("unknown exchange: {id}"))),
        }
    }

    /// Register outcome names for Polymarket token IDs so activity events
    /// include "Yes"/"No". No-op for other exchanges.
    pub async fn register_outcomes(&self, yes_token_id: &str, no_token_id: &str) {
        if let Self::Polymarket(ws) = self {
            ws.register_outcomes(yes_token_id, no_token_id).await;
        }
    }
}

macro_rules! ws_dispatch_async {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            WebSocketInner::Kalshi(ws) => ws.$method($($arg),*).await,
            WebSocketInner::Polymarket(ws) => ws.$method($($arg),*).await,
        }
    };
}

impl OrderBookWebSocket for WebSocketInner {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        ws_dispatch_async!(self, connect)
    }

    async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        ws_dispatch_async!(self, disconnect)
    }

    async fn subscribe(&mut self, market_ticker: &str) -> Result<(), WebSocketError> {
        ws_dispatch_async!(self, subscribe, market_ticker)
    }

    async fn unsubscribe(&mut self, market_ticker: &str) -> Result<(), WebSocketError> {
        ws_dispatch_async!(self, unsubscribe, market_ticker)
    }

    fn state(&self) -> WebSocketState {
        match self {
            Self::Kalshi(ws) => ws.state(),
            Self::Polymarket(ws) => ws.state(),
        }
    }

    fn updates(&self) -> Option<UpdateStream> {
        match self {
            Self::Kalshi(ws) => ws.updates(),
            Self::Polymarket(ws) => ws.updates(),
        }
    }

    fn session_events(&self) -> Option<SessionStream> {
        match self {
            Self::Kalshi(ws) => ws.session_events(),
            Self::Polymarket(ws) => ws.session_events(),
        }
    }
}
