//! Multiplexed WebSocket event types.
//!
//! A single `WsUpdate` stream carries every per-market event — snapshots, deltas,
//! fills, trades, and exchange-specific escape-hatch payloads. Connection-level
//! events (reconnect, lag, book invalidation) are split into a separate
//! `SessionEvent` stream so a reconnect is one event, not 576.
//!
//! Timestamps are dual-clock by design:
//! - `exchange_ts: Option<u64>` — exchange-authoritative millis since epoch for
//!   cross-stream ordering and feed-lag measurement.
//! - `local_ts: Instant` — captured the moment the socket read returned, before
//!   any parse. Monotonic; correct under NTP adjustments. Skipped during
//!   serialization since `Instant` has no portable representation.
//! - `local_ts_ms: u64` — wall-clock millis captured alongside `local_ts`,
//!   suitable for FFI / NDJSON archival when serialization is required.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::WebSocketError;
use crate::models::{ChangeVec, Orderbook};
use crate::websocket::traits::{ActivityFill, ActivityTrade};

/// Every per-market event the WebSocket surface emits. Tagged union with a
/// single escape hatch (`Raw`) for exchange-specific payloads we haven't
/// normalized yet.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind")]
pub enum WsUpdate {
    /// Full orderbook snapshot. Caller should replace any cached book for this
    /// `market_id`. Emitted on initial subscribe and after any
    /// `BookInvalidated` recovery path.
    Snapshot {
        market_id: String,
        book: Arc<Orderbook>,
        exchange_ts: Option<u64>,
        #[serde(skip)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        local_ts: Instant,
        local_ts_ms: u64,
        seq: u64,
    },
    /// Incremental change to an existing book. Apply in-place, or discard if
    /// the caller has seen a matching `BookInvalidated` without a follow-up
    /// `Snapshot` yet.
    Delta {
        market_id: String,
        changes: ChangeVec,
        exchange_ts: Option<u64>,
        #[serde(skip)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        local_ts: Instant,
        local_ts_ms: u64,
        seq: u64,
    },
    /// A public trade (any counterparty). Not tied to a local order.
    Trade {
        trade: ActivityTrade,
        #[serde(skip)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        local_ts: Instant,
        local_ts_ms: u64,
    },
    /// A fill on one of the authenticated user's orders.
    Fill {
        fill: ActivityFill,
        #[serde(skip)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        local_ts: Instant,
        local_ts_ms: u64,
    },
    /// Exchange-specific payload that hasn't been normalized. Treat as
    /// best-effort debug surface; structure is not stable.
    Raw {
        exchange: String,
        value: serde_json::Value,
        #[serde(skip)]
        #[cfg_attr(feature = "schema", schemars(skip))]
        local_ts: Instant,
        local_ts_ms: u64,
    },
}

/// Connection-level events, emitted on a channel separate from `WsUpdate` so
/// a reconnect is observable as a single global signal rather than per-market.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind")]
pub enum SessionEvent {
    /// Initial socket establishment.
    Connected,
    /// Socket re-established after an observed outage. `gap_ms` is the
    /// wall-clock interval between the last received message and this event.
    /// Callers who maintain per-market books should discard them and wait for
    /// the next `WsUpdate::Snapshot` for each subscribed market.
    Reconnected {
        gap_ms: u64,
    },
    /// Outbound dispatch channel overflowed — a slow consumer missed deltas.
    /// Unlike `tokio::sync::broadcast` which silently skips ahead, openpx
    /// raises this explicitly and invalidates every affected book, because
    /// a missed delta corrupts book state in a way the caller cannot detect
    /// from the stream alone.
    Lagged {
        dropped: u64,
        first_seq: u64,
        last_seq: u64,
    },
    /// A specific market's book is no longer trustworthy. Caller should
    /// discard its cache for that `market_id` and wait for the next
    /// `WsUpdate::Snapshot`.
    BookInvalidated {
        market_id: String,
        reason: InvalidationReason,
    },
    /// A non-fatal error observed on the connection. The session continues;
    /// the caller is informed in case they want to log or alert.
    Error {
        message: String,
    },
}

impl SessionEvent {
    /// Construct a `Reconnected` event from a `Duration`-shaped gap. Saturating
    /// cast at u64::MAX keeps the type stable for callers serializing to JSON.
    #[inline]
    pub fn reconnected(gap: Duration) -> Self {
        Self::Reconnected {
            gap_ms: u64::try_from(gap.as_millis()).unwrap_or(u64::MAX),
        }
    }

    /// Convenience constructor that stringifies the upstream error.
    #[inline]
    pub fn error(err: WebSocketError) -> Self {
        Self::Error {
            message: err.to_string(),
        }
    }
}

/// Why a specific book was invalidated — handed to users so they can decide
/// whether to alert, log, or handle it silently.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum InvalidationReason {
    Reconnect,
    Lag,
    SequenceGap { expected: u64, received: u64 },
    ExchangeReset,
}

impl WsUpdate {
    /// Uniform accessor for the ingest-side monotonic timestamp. Use for
    /// metrics and cross-update ordering; for per-market sequencing prefer
    /// the `seq` field on `Snapshot` / `Delta`.
    #[inline]
    pub fn local_ts(&self) -> Instant {
        match self {
            Self::Snapshot { local_ts, .. }
            | Self::Delta { local_ts, .. }
            | Self::Trade { local_ts, .. }
            | Self::Fill { local_ts, .. }
            | Self::Raw { local_ts, .. } => *local_ts,
        }
    }

    /// Wall-clock millis paired with `local_ts`. Use for serialization and
    /// any cross-process correlation; not monotonic.
    #[inline]
    pub fn local_ts_ms(&self) -> u64 {
        match self {
            Self::Snapshot { local_ts_ms, .. }
            | Self::Delta { local_ts_ms, .. }
            | Self::Trade { local_ts_ms, .. }
            | Self::Fill { local_ts_ms, .. }
            | Self::Raw { local_ts_ms, .. } => *local_ts_ms,
        }
    }

    /// Market ID for events scoped to a single market. `None` for `Raw`
    /// payloads that haven't been normalized.
    #[inline]
    pub fn market_id(&self) -> Option<&str> {
        match self {
            Self::Snapshot { market_id, .. } | Self::Delta { market_id, .. } => Some(market_id),
            Self::Trade { trade, .. } => Some(&trade.market_id),
            Self::Fill { fill, .. } => Some(&fill.market_id),
            Self::Raw { .. } => None,
        }
    }
}

/// Capture both clocks at the same call site. Use this at the socket-read
/// boundary so every emitted update carries paired monotonic + wall-clock
/// timestamps.
#[inline]
pub fn now_pair() -> (Instant, u64) {
    let local_ts = Instant::now();
    let local_ts_ms = chrono::Utc::now().timestamp_millis() as u64;
    (local_ts, local_ts_ms)
}
