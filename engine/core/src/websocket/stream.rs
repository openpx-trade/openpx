//! Concrete stream newtypes for the multiplexed WS surface.
//!
//! These replace the `Pin<Box<dyn Stream<Item = ...>>>` aliases used in the
//! 0.1 trait. The receiver channel is an implementation detail; the public
//! surface is `Stream<Item = T>` and a direct `recv().await` shortcut for
//! hot-path callers that want to avoid `Stream` combinators entirely.

use std::sync::Mutex;

use flume::{Receiver, Sender, TryRecvError};

use crate::websocket::events::{SessionEvent, WsUpdate};

/// Multiplexed per-market update stream. There is exactly one `UpdateStream`
/// per dispatcher; calling `OrderBookWebSocket::updates()` twice returns
/// `None` the second time.
///
/// Why take-once semantics: the underlying channel is MPMC, so cloning the
/// receiver and handing one out per call would split messages between
/// receivers in a way callers cannot detect — every test harness or debug
/// sidecar that "just adds a second consumer" would silently lose half the
/// stream. Take-once turns that footgun into a compile-checked None at the
/// call site.
///
/// Backed by `flume`, an MPMC channel with lighter-weight wakers than
/// `async-channel`'s; the per-message recv on the consumer side is a
/// significant fraction of the WS pipeline p99, and flume's atomic-only
/// fast path stays out of the parking_lot machinery on uncontended
/// recv. The channel is bounded; when full the producer raises
/// `SessionEvent::Lagged` + `SessionEvent::BookInvalidated` rather than
/// dropping deltas silently.
pub struct UpdateStream {
    rx: Receiver<WsUpdate>,
}

impl UpdateStream {
    #[inline]
    pub(crate) fn new(rx: Receiver<WsUpdate>) -> Self {
        Self { rx }
    }

    /// Await the next update. `None` once the producer has been dropped.
    #[inline]
    pub async fn next(&self) -> Option<WsUpdate> {
        self.rx.recv_async().await.ok()
    }

    /// Non-blocking peek. Returns `Ok(Some)` if an update is ready,
    /// `Ok(None)` if the channel is empty, `Err` if closed.
    #[inline]
    pub fn try_next(&self) -> Result<Option<WsUpdate>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(v) => Ok(Some(v)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(e @ TryRecvError::Disconnected) => Err(e),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.rx.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.rx.is_disconnected()
    }
}

/// Connection-level events. Separate from `UpdateStream` so one reconnect is
/// one event regardless of how many markets are subscribed.
pub struct SessionStream {
    rx: Receiver<SessionEvent>,
}

impl SessionStream {
    #[inline]
    pub(crate) fn new(rx: Receiver<SessionEvent>) -> Self {
        Self { rx }
    }

    #[inline]
    pub async fn next(&self) -> Option<SessionEvent> {
        self.rx.recv_async().await.ok()
    }

    #[inline]
    pub fn try_next(&self) -> Result<Option<SessionEvent>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(v) => Ok(Some(v)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(e @ TryRecvError::Disconnected) => Err(e),
        }
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.rx.is_disconnected()
    }
}

/// Producer handle held by exchange WS implementations. Owns the sender
/// halves of both channels so the implementation can emit updates and
/// session events directly.
///
/// The receiver halves are held in `Mutex<Option<...>>` and handed out
/// exactly once via `take_updates()` / `take_session_events()`. This
/// enforces the take-once contract on the consumer side.
pub struct WsDispatcher {
    updates_tx: Sender<WsUpdate>,
    updates_rx: Mutex<Option<Receiver<WsUpdate>>>,
    session_tx: Sender<SessionEvent>,
    session_rx: Mutex<Option<Receiver<SessionEvent>>>,
}

/// Configuration for the dispatcher's bounded channels.
#[derive(Debug, Clone, Copy)]
pub struct WsDispatcherConfig {
    /// Capacity of the per-subscriber update channel. On overflow the
    /// dispatcher emits `SessionEvent::Lagged` + `BookInvalidated` and
    /// drops the offending update. Default 4096.
    pub updates_capacity: usize,
    /// Capacity of the session-event channel. Default 256 — session events
    /// are rare and losing one is a correctness hazard, so oversized.
    pub session_capacity: usize,
}

impl Default for WsDispatcherConfig {
    fn default() -> Self {
        Self {
            updates_capacity: 4096,
            session_capacity: 256,
        }
    }
}

impl WsDispatcher {
    /// Create a new dispatcher. The returned dispatcher owns the send halves
    /// of both channels and the (one-shot) receive halves; consumers fetch
    /// streams exactly once via `take_updates()` / `take_session_events()`.
    pub fn new(config: WsDispatcherConfig) -> Self {
        let (updates_tx, updates_rx) = flume::bounded(config.updates_capacity);
        let (session_tx, session_rx) = flume::bounded(config.session_capacity);
        Self {
            updates_tx,
            updates_rx: Mutex::new(Some(updates_rx)),
            session_tx,
            session_rx: Mutex::new(Some(session_rx)),
        }
    }

    /// Take ownership of the consumer-side update stream. Returns `None` if
    /// already taken — the receiver is single-consumer by contract; cloning
    /// would silently split messages between holders.
    #[inline]
    pub fn take_updates(&self) -> Option<UpdateStream> {
        self.updates_rx
            .lock()
            .ok()
            .and_then(|mut g| g.take())
            .map(UpdateStream::new)
    }

    /// Take ownership of the consumer-side session stream.
    #[inline]
    pub fn take_session_events(&self) -> Option<SessionStream> {
        self.session_rx
            .lock()
            .ok()
            .and_then(|mut g| g.take())
            .map(SessionStream::new)
    }

    /// Emit an update. Returns `true` if delivered. On `Err(TrySendError::Full)`
    /// the update is dropped and the caller is expected to follow up with a
    /// `SessionEvent::Lagged` + one or more `BookInvalidated` events — this
    /// is the correctness contract that distinguishes 0.2 from 0.1's
    /// silent-skip broadcast behavior.
    #[inline]
    pub fn try_send_update(&self, update: WsUpdate) -> bool {
        self.updates_tx.try_send(update).is_ok()
    }

    /// Emit a session event. Unlike updates, these are always delivered via
    /// `send`; the session-event channel is sized generously and losing an
    /// event (e.g. a missed `Reconnected`) is worse than a brief await.
    #[inline]
    pub async fn send_session(&self, event: SessionEvent) {
        let _ = self.session_tx.send_async(event).await;
    }

    #[inline]
    pub fn is_updates_full(&self) -> bool {
        self.updates_tx.is_full()
    }
}
