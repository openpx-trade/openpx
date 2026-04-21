//! Multiplexed WebSocket event types (openpx 0.2 surface).
//!
//! A single `WsUpdate` stream carries every per-market event — snapshots, deltas,
//! fills, trades, and exchange-specific escape-hatch payloads. Connection-level
//! events (reconnect, lag, book invalidation) are split into a separate
//! `SessionEvent` stream so a reconnect is one event, not 576.
//!
//! Timestamps are dual-clock by design:
//! - `exchange_ts` — exchange-authoritative millis since epoch for cross-stream
//!   ordering and feed-lag measurement.
//! - `local_ts` — `std::time::Instant` captured the moment the socket read
//!   returned, before any parse. Monotonic; correct under NTP adjustments.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::error::WebSocketError;
use crate::models::{ChangeVec, Orderbook};
use crate::websocket::traits::{ActivityFill, ActivityTrade};

/// Every per-market event the WebSocket surface emits. Tagged union with a
/// single escape hatch (`Raw`) for exchange-specific payloads we haven't
/// normalized yet.
#[derive(Debug, Clone)]
pub enum WsUpdate {
    /// Full orderbook snapshot. Caller should replace any cached book for this
    /// `market_id`. Emitted on initial subscribe, after reconnect, and on any
    /// `BookInvalidated` recovery path.
    Snapshot {
        market_id: String,
        book: Arc<Orderbook>,
        exchange_ts: Option<u64>,
        local_ts: Instant,
        seq: u64,
    },
    /// Incremental change to an existing book. Apply in-place, or discard if
    /// the caller has seen a matching `BookInvalidated` without a follow-up
    /// `Snapshot` yet.
    Delta {
        market_id: String,
        changes: ChangeVec,
        exchange_ts: Option<u64>,
        local_ts: Instant,
        seq: u64,
    },
    /// A public trade (any counterparty). Not tied to a local order.
    Trade {
        trade: ActivityTrade,
        local_ts: Instant,
    },
    /// A fill on one of the authenticated user's orders. Emitted in addition
    /// to any `OrderHandle` resolution — the stream is the source of truth for
    /// passive observers; `OrderHandle.await` is the ergonomic path for the
    /// submitter. We deliberately do not dedupe here based on local
    /// attribution — hidden filtering is a debugging hazard in trading
    /// systems.
    Fill {
        fill: ActivityFill,
        local_ts: Instant,
    },
    /// Exchange-specific payload that hasn't been normalized. Consumers should
    /// treat this as best-effort debug surface; structure is not stable.
    Raw {
        exchange: &'static str,
        value: serde_json::Value,
        local_ts: Instant,
    },
}

/// Connection-level events, emitted on a channel separate from `WsUpdate` so
/// a reconnect is observable as a single global signal rather than 576
/// per-market stale flags.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Initial socket establishment.
    Connected,
    /// Socket re-established after an observed outage. `gap` is the wall-clock
    /// interval between the last received message and this event. Callers who
    /// maintain per-market books should discard them and wait for the next
    /// `WsUpdate::Snapshot` for each subscribed market.
    Reconnected {
        gap: Duration,
    },
    /// Outbound dispatch channel overflowed — a slow consumer missed deltas.
    /// Unlike `tokio::sync::broadcast` which silently skips ahead, openpx 0.2
    /// raises this explicitly and invalidates every subscribed book, because
    /// a missed delta corrupts book state in a way the caller cannot detect
    /// from the stream alone.
    Lagged {
        dropped: u64,
        first_seq: u64,
        last_seq: u64,
    },
    /// A specific market's book is no longer trustworthy. Caller should
    /// discard its cache for that `market_id` and wait for the next
    /// `WsUpdate::Snapshot`. Emitted alongside `Lagged`, after reconnects
    /// without auto-resync, or on exchange-side sequence resets.
    BookInvalidated {
        market_id: String,
        reason: InvalidationReason,
    },
    /// A non-fatal error was observed. The session continues; the caller is
    /// informed in case they want to log or alert.
    Error(WebSocketError),
}

/// Why a specific book was invalidated — handed to users so they can decide
/// whether to alert, log, or handle it silently.
#[derive(Debug, Clone)]
pub enum InvalidationReason {
    Reconnect,
    Lag,
    SequenceGap { expected: u64, received: u64 },
    ExchangeReset,
}

impl WsUpdate {
    /// Uniform accessor for the ingest-side monotonic timestamp. Useful for
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

    /// Market ID for events that are scoped to a single market. `None` for
    /// `Raw` payloads that haven't been normalized. `Fill` / `Trade` callers
    /// should read the underlying `ActivityFill::market_id` /
    /// `ActivityTrade::market_id` directly — this accessor stays narrow to
    /// avoid false matches on multi-market events.
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

