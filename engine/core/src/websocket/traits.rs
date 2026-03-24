use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use crate::error::WebSocketError;
use crate::models::{CryptoPrice, LiquidityRole, OrderbookUpdate, SportResult};

/// Shared WebSocket reconnect/keepalive constants for all exchange implementations.
pub const WS_PING_INTERVAL: Duration = Duration::from_secs(20);
pub const WS_CRYPTO_PING_INTERVAL: Duration = Duration::from_secs(5);
pub const WS_RECONNECT_BASE_DELAY: Duration = Duration::from_millis(3000);
pub const WS_RECONNECT_MAX_DELAY: Duration = Duration::from_millis(60000);
pub const WS_MAX_RECONNECT_ATTEMPTS: u32 = 10;

/// Envelope wrapping every WebSocket stream item for HFT feed-latency measurement.
///
/// - `exchange_time` — server-authoritative UTC timestamp for event ordering.
///   Always prefer this for trade sequencing and cross-exchange correlation.
/// - `received_at` — local UTC timestamp captured at socket read for measuring
///   wire-to-process latency. Compare `received_at - exchange_time` for feed lag.
/// - `seq` — per-market monotonic sequence number for gap detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage<T> {
    pub seq: u64,
    pub exchange_time: Option<DateTime<Utc>>,
    pub received_at: DateTime<Utc>,
    pub data: T,
}

pub type OrderbookStream =
    Pin<Box<dyn Stream<Item = Result<WsMessage<OrderbookUpdate>, WebSocketError>> + Send>>;
pub type ActivityStream =
    Pin<Box<dyn Stream<Item = Result<WsMessage<ActivityEvent>, WebSocketError>> + Send>>;
pub type SportsStream = Pin<Box<dyn Stream<Item = Result<SportResult, WebSocketError>> + Send>>;
pub type CryptoPriceStream =
    Pin<Box<dyn Stream<Item = Result<CryptoPrice, WebSocketError>> + Send>>;

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
    /// Fee rate in basis points (e.g. 0 = no fee, 200 = 2%).
    /// Polymarket: present on `last_trade_price` events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_rate_bps: Option<u32>,
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
    /// On-chain transaction hash. Opinion: `txHash` from `trade.record.new`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    /// Fee charged for this fill. Opinion: `fee` from `trade.record.new`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
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
