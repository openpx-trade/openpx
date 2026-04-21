//! Concrete stream newtypes for the multiplexed WS surface.
//!
//! These replace the `Pin<Box<dyn Stream<Item = ...>>>` aliases used in the
//! 0.1 trait. The receiver channel is an implementation detail; the public
//! surface is `Stream<Item = T>` and a direct `recv().await` shortcut for
//! hot-path callers that want to avoid `Stream` combinators entirely.

use async_channel::{Receiver, TryRecvError};

use crate::websocket::events::{SessionEvent, WsUpdate};

/// Multiplexed per-market update stream. One instance per subscriber. The
/// underlying channel is bounded; when full the producer raises
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
        self.rx.recv().await.ok()
    }

    /// Non-blocking peek. Returns `Ok(Some)` if an update is ready,
    /// `Ok(None)` if the channel is empty, `Err` if closed.
    #[inline]
    pub fn try_next(&self) -> Result<Option<WsUpdate>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(v) => Ok(Some(v)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(e @ TryRecvError::Closed) => Err(e),
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
        self.rx.is_closed()
    }

    /// Escape hatch for consumers that genuinely need `futures::Stream`
    /// combinators. The receiver already implements `Stream`; returning it
    /// by clone is cheap (channels are reference-counted).
    #[inline]
    pub fn into_stream(self) -> Receiver<WsUpdate> {
        self.rx
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
        self.rx.recv().await.ok()
    }

    #[inline]
    pub fn try_next(&self) -> Result<Option<SessionEvent>, TryRecvError> {
        match self.rx.try_recv() {
            Ok(v) => Ok(Some(v)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(e @ TryRecvError::Closed) => Err(e),
        }
    }

    #[inline]
    pub fn is_closed(&self) -> bool {
        self.rx.is_closed()
    }

    #[inline]
    pub fn into_stream(self) -> Receiver<SessionEvent> {
        self.rx
    }
}

/// Producer handle held by exchange WS implementations. Owns both channels
/// so the implementation can emit updates and session events without a
/// second routing layer.
///
/// The dispatcher also retains the receiver halves so the trait method
/// `updates()` can hand out cloned receivers on demand. Cloned receivers
/// are co-consumers of the same queue (each message goes to one
/// receiver, first-grabbed) — this is the documented single-consumer
/// pattern. Callers that need fan-out should run one consumer that
/// re-broadcasts.
pub struct WsDispatcher {
    updates_tx: async_channel::Sender<WsUpdate>,
    updates_rx: Receiver<WsUpdate>,
    session_tx: async_channel::Sender<SessionEvent>,
    session_rx: Receiver<SessionEvent>,
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
    /// Create a new dispatcher. The returned dispatcher owns both send and
    /// receive halves of its channels; consumers fetch streams via
    /// `updates()` / `session_events()`, which clone the receivers.
    pub fn new(config: WsDispatcherConfig) -> Self {
        let (updates_tx, updates_rx) = async_channel::bounded(config.updates_capacity);
        let (session_tx, session_rx) = async_channel::bounded(config.session_capacity);
        Self {
            updates_tx,
            updates_rx,
            session_tx,
            session_rx,
        }
    }

    /// Hand out a consumer-side update stream. Cheap (cloning an
    /// `async-channel::Receiver` is an atomic-refcount bump).
    #[inline]
    pub fn updates(&self) -> UpdateStream {
        UpdateStream::new(self.updates_rx.clone())
    }

    /// Hand out a consumer-side session stream.
    #[inline]
    pub fn session_events(&self) -> SessionStream {
        SessionStream::new(self.session_rx.clone())
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
        let _ = self.session_tx.send(event).await;
    }

    #[inline]
    pub fn is_updates_full(&self) -> bool {
        self.updates_tx.is_full()
    }
}
