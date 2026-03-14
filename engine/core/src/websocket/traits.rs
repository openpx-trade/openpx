use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use crate::error::WebSocketError;
use crate::models::{LiquidityRole, OrderbookUpdate};

/// Shared WebSocket reconnect/keepalive constants for all exchange implementations.
pub const WS_PING_INTERVAL: Duration = Duration::from_secs(20);
pub const WS_RECONNECT_BASE_DELAY: Duration = Duration::from_millis(3000);
pub const WS_RECONNECT_MAX_DELAY: Duration = Duration::from_millis(60000);
pub const WS_MAX_RECONNECT_ATTEMPTS: u32 = 10;

pub type OrderbookStream =
    Pin<Box<dyn Stream<Item = Result<OrderbookUpdate, WebSocketError>> + Send>>;
pub type ActivityStream = Pin<Box<dyn Stream<Item = Result<ActivityEvent, WebSocketError>> + Send>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WebSocketState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Reconnecting = 3,
    Closed = 4,
}

impl WebSocketState {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Connecting,
            2 => Self::Connected,
            3 => Self::Reconnecting,
            4 => Self::Closed,
            _ => Self::Disconnected,
        }
    }
}

/// Lock-free atomic wrapper for WebSocketState.
/// Enables O(1) reads without acquiring any async lock.
pub struct AtomicWebSocketState(AtomicU8);

impl AtomicWebSocketState {
    pub fn new(state: WebSocketState) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    #[inline]
    pub fn load(&self) -> WebSocketState {
        WebSocketState::from_u8(self.0.load(Ordering::Acquire))
    }

    #[inline]
    pub fn store(&self, state: WebSocketState) {
        self.0.store(state as u8, Ordering::Release);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum ActivityEvent {
    Trade(ActivityTrade),
    Fill(ActivityFill),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ActivityTrade {
    pub market_id: String,
    pub asset_id: String,
    pub trade_id: Option<String>,
    pub price: f64,
    pub size: f64,
    pub side: Option<String>,
    pub aggressor_side: Option<String>,
    pub outcome: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub source_channel: Cow<'static, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ActivityFill {
    pub market_id: String,
    pub asset_id: String,
    pub fill_id: Option<String>,
    pub order_id: Option<String>,
    pub price: f64,
    pub size: f64,
    pub side: Option<String>,
    pub outcome: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub source_channel: Cow<'static, str>,
    pub liquidity_role: Option<LiquidityRole>,
}

#[allow(async_fn_in_trait)]
pub trait OrderBookWebSocket: Send + Sync {
    async fn connect(&mut self) -> Result<(), WebSocketError>;

    async fn disconnect(&mut self) -> Result<(), WebSocketError>;

    async fn subscribe(&mut self, market_id: &str) -> Result<(), WebSocketError>;

    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError>;

    fn state(&self) -> WebSocketState;

    async fn orderbook_stream(
        &mut self,
        market_id: &str,
    ) -> Result<OrderbookStream, WebSocketError>;

    async fn activity_stream(
        &mut self,
        _market_id: &str,
    ) -> Result<ActivityStream, WebSocketError> {
        Err(WebSocketError::Subscription(
            "activity stream not supported".to_string(),
        ))
    }
}
