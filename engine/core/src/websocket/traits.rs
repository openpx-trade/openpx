use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::error::WebSocketError;
use crate::models::{CryptoPrice, LiquidityRole, SportResult};
use crate::websocket::stream::{SessionStream, UpdateStream};

/// Shared WebSocket reconnect/keepalive constants for all exchange implementations.
pub const WS_PING_INTERVAL: Duration = Duration::from_secs(20);
pub const WS_CRYPTO_PING_INTERVAL: Duration = Duration::from_secs(5);
pub const WS_RECONNECT_BASE_DELAY: Duration = Duration::from_millis(3000);
pub const WS_RECONNECT_MAX_DELAY: Duration = Duration::from_millis(60000);
pub const WS_MAX_RECONNECT_ATTEMPTS: u32 = 10;

/// Force a reconnect if no message has been received for this long. Sized
/// for the quietest live venue: a 50-market Kalshi subscription at
/// ~0.5 msg/s hit a 40s max-quiet window in smoke testing, so 60s was
/// only 33% above observed worst-case. 90s gives ~2× headroom while
/// still shrinking macOS's ~30-minute silent-death window by ~20×.
pub const WS_STALL_TIMEOUT: Duration = Duration::from_secs(90);
pub const WS_STALL_CHECK_INTERVAL: Duration = Duration::from_secs(10);

/// Returns once `last_message_at` is older than `WS_STALL_TIMEOUT`.
///
/// Drop into a `tokio::select!` alongside the read loop to break out when
/// the socket goes silent (half-open connection, server-side hang, NAT
/// timeout). The existing reconnect path then handles it. Without this,
/// a dead socket can sit there for 30+ minutes before OS keepalive
/// surfaces an error — observed in production smoke tests.
pub async fn stall_watchdog(last_message_at: Arc<RwLock<Option<DateTime<Utc>>>>) {
    let mut tick = interval(WS_STALL_CHECK_INTERVAL);
    tick.tick().await; // first tick fires immediately; skip
    loop {
        tick.tick().await;
        let last = *last_message_at.read().await;
        if let Some(last) = last {
            let age = Utc::now() - last;
            if age.to_std().is_ok_and(|d| d > WS_STALL_TIMEOUT) {
                tracing::warn!(
                    stall_secs = age.num_seconds(),
                    "no messages for >{}s; forcing reconnect",
                    WS_STALL_TIMEOUT.as_secs()
                );
                return;
            }
        }
    }
}

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

    /// Stable string label, matching the `Display` impl. Bindings should use
    /// this rather than `Debug` formatting, which is not a stability guarantee.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting",
            Self::Closed => "Closed",
        }
    }
}

impl std::fmt::Display for WebSocketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Lock-free atomic wrapper for WebSocketState. O(1) reads without acquiring
/// any async lock.
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

/// Activity events still carried inside `WsUpdate::Trade` / `WsUpdate::Fill`.
/// Retained as a typed payload alongside the stream rather than as a separate
/// per-token surface.
///
/// `exchange_ts_ms` is exchange-authoritative millis since epoch — uniform
/// with `WsUpdate::{Snapshot, Delta}::exchange_ts`, so every timestamp on
/// the WS surface is the same type (`u64` millis). `chrono::DateTime` was
/// more expressive but cost a representation mismatch at every FFI boundary.
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
    /// Fee rate in basis points (e.g. 0 = no fee, 200 = 2%). Polymarket
    /// `last_trade_price` events populate this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_rate_bps: Option<u32>,
    /// Exchange-authoritative timestamp (millis since epoch).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange_ts_ms: Option<u64>,
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
    /// Exchange-authoritative timestamp (millis since epoch).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange_ts_ms: Option<u64>,
    pub source_channel: Cow<'static, str>,
    pub liquidity_role: Option<LiquidityRole>,
}

/// WebSocket driver trait. Surface is deliberately small: connect/disconnect,
/// subscribe/unsubscribe per market, and hand out two multiplexed streams.
///
/// `updates()` carries per-market book and activity events; `session_events()`
/// carries connection-level signals that a consumer wants to observe exactly
/// once regardless of how many markets are subscribed.
///
/// Both stream methods are take-once: subsequent calls return `None`. The
/// underlying channel is single-consumer by contract — handing out cloned
/// receivers would split messages silently between holders, so a second
/// "debug sidecar" consumer is rejected at the call site instead. For
/// fan-out, run one consumer that re-dispatches.
#[allow(async_fn_in_trait)]
pub trait OrderBookWebSocket: Send + Sync {
    async fn connect(&mut self) -> Result<(), WebSocketError>;
    async fn disconnect(&mut self) -> Result<(), WebSocketError>;
    async fn subscribe(&mut self, market_id: &str) -> Result<(), WebSocketError>;
    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError>;
    fn state(&self) -> WebSocketState;
    /// Take ownership of the multiplexed update stream. Returns `None` if
    /// already taken.
    fn updates(&self) -> Option<UpdateStream>;
    /// Take ownership of the connection-level session event stream. Returns
    /// `None` if already taken.
    fn session_events(&self) -> Option<SessionStream>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TDuration};

    /// Watchdog must fire after WS_STALL_TIMEOUT of silence even if
    /// `last_message_at` was never written (stays `None`)? Actually no —
    /// `None` means "we haven't received anything yet, so don't trigger";
    /// fresh connections need a grace period before traffic starts.
    #[tokio::test]
    async fn watchdog_does_not_fire_when_last_message_is_none() {
        let last = Arc::new(RwLock::new(None::<DateTime<Utc>>));
        // Race the watchdog against a 1.5x check-interval timeout — the
        // watchdog must NOT return in that window.
        let result = tokio::time::timeout(
            WS_STALL_CHECK_INTERVAL + TDuration::from_secs(5),
            stall_watchdog(last),
        )
        .await;
        assert!(
            result.is_err(),
            "watchdog returned {result:?} despite last_message_at being None"
        );
    }

    /// If a recent message has been recorded, watchdog stays asleep.
    #[tokio::test]
    async fn watchdog_does_not_fire_with_recent_activity() {
        let last = Arc::new(RwLock::new(Some(Utc::now())));
        let result = tokio::time::timeout(
            WS_STALL_CHECK_INTERVAL + TDuration::from_secs(5),
            stall_watchdog(last),
        )
        .await;
        assert!(result.is_err(), "watchdog fired despite fresh activity");
    }

    /// If `last_message_at` is older than the threshold at the moment of the
    /// next check tick, watchdog must return promptly. We pre-age the
    /// timestamp by `WS_STALL_TIMEOUT + 5s` so the very first post-`tick`
    /// read sees it as stale.
    #[tokio::test]
    async fn watchdog_fires_when_last_message_is_stale() {
        let stale = Utc::now() - chrono::Duration::from_std(WS_STALL_TIMEOUT).unwrap()
            - chrono::Duration::seconds(5);
        let last = Arc::new(RwLock::new(Some(stale)));
        // Should fire within one check interval (10s) plus slack.
        let result = tokio::time::timeout(
            WS_STALL_CHECK_INTERVAL + TDuration::from_secs(2),
            stall_watchdog(last),
        )
        .await;
        assert!(result.is_ok(), "watchdog did not fire on stale timestamp");
    }

    /// Continuous activity (refresh `last_message_at` every 1s) must keep
    /// the watchdog asleep indefinitely. Models the "active market"
    /// scenario.
    #[tokio::test]
    async fn watchdog_stays_asleep_under_continuous_activity() {
        let last = Arc::new(RwLock::new(Some(Utc::now())));
        let last_clone = last.clone();
        let refresher = tokio::spawn(async move {
            for _ in 0..15 {
                sleep(TDuration::from_secs(1)).await;
                *last_clone.write().await = Some(Utc::now());
            }
        });
        let result = tokio::time::timeout(TDuration::from_secs(15), stall_watchdog(last)).await;
        refresher.abort();
        assert!(
            result.is_err(),
            "watchdog fired despite messages every 1s for 15s"
        );
    }
}
