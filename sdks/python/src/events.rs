#![allow(non_upper_case_globals)] // Python dunder convention requires lowercase `__match_args__`

//! Pyclass wrappers for `WsUpdate` and `SessionEvent`.
//!
//! The Rust side is a tagged enum; the Python side used to receive a
//! `dict[str, Any]` with a `kind` discriminator. That put consumer code
//! right back on the untyped-dict path the 0.2 surface was trying to
//! escape. These wrappers make `isinstance(update, Snapshot)` work and
//! give `match update: case Snapshot(market_id, book, ...)` proper
//! positional binding via `__match_args__`.
//!
//! Nested payloads (`book`, `changes`, `trade`, `fill`) stay as
//! `dict[str, Any]` for now — a full `Orderbook` / `PriceLevelChange`
//! pyclass surface is a separate cut. Top-level isinstance matching
//! delivers the biggest DX win per line of Rust.

use pyo3::prelude::*;
use pythonize::pythonize;

use px_core::websocket::{InvalidationReason, SessionEvent, WsUpdate};

// ---------- WsUpdate variants ----------

#[pyclass(module = "openpx", frozen)]
pub struct Snapshot {
    #[pyo3(get)]
    pub market_id: String,
    #[pyo3(get)]
    pub book: Py<PyAny>,
    #[pyo3(get)]
    pub exchange_ts: Option<u64>,
    #[pyo3(get)]
    pub local_ts_ms: u64,
    #[pyo3(get)]
    pub seq: u64,
}

#[pymethods]
impl Snapshot {
    #[classattr]
    const __match_args__: (&str, &str, &str, &str, &str) =
        ("market_id", "book", "exchange_ts", "local_ts_ms", "seq");

    #[getter]
    fn kind(&self) -> &'static str {
        "Snapshot"
    }

    fn __repr__(&self) -> String {
        format!(
            "Snapshot(market_id={:?}, seq={}, exchange_ts={:?}, local_ts_ms={})",
            self.market_id, self.seq, self.exchange_ts, self.local_ts_ms
        )
    }
}

#[pyclass(module = "openpx", frozen)]
pub struct Delta {
    #[pyo3(get)]
    pub market_id: String,
    #[pyo3(get)]
    pub changes: Py<PyAny>,
    #[pyo3(get)]
    pub exchange_ts: Option<u64>,
    #[pyo3(get)]
    pub local_ts_ms: u64,
    #[pyo3(get)]
    pub seq: u64,
}

#[pymethods]
impl Delta {
    #[classattr]
    const __match_args__: (&str, &str, &str, &str, &str) =
        ("market_id", "changes", "exchange_ts", "local_ts_ms", "seq");

    #[getter]
    fn kind(&self) -> &'static str {
        "Delta"
    }

    fn __repr__(&self) -> String {
        format!(
            "Delta(market_id={:?}, seq={}, exchange_ts={:?}, local_ts_ms={})",
            self.market_id, self.seq, self.exchange_ts, self.local_ts_ms
        )
    }
}

#[pyclass(module = "openpx", frozen)]
pub struct Trade {
    #[pyo3(get)]
    pub trade: Py<PyAny>,
    #[pyo3(get)]
    pub local_ts_ms: u64,
}

#[pymethods]
impl Trade {
    #[classattr]
    const __match_args__: (&str, &str) = ("trade", "local_ts_ms");

    #[getter]
    fn kind(&self) -> &'static str {
        "Trade"
    }

    fn __repr__(&self) -> String {
        format!("Trade(local_ts_ms={})", self.local_ts_ms)
    }
}

#[pyclass(module = "openpx", frozen)]
pub struct Fill {
    #[pyo3(get)]
    pub fill: Py<PyAny>,
    #[pyo3(get)]
    pub local_ts_ms: u64,
}

#[pymethods]
impl Fill {
    #[classattr]
    const __match_args__: (&str, &str) = ("fill", "local_ts_ms");

    #[getter]
    fn kind(&self) -> &'static str {
        "Fill"
    }

    fn __repr__(&self) -> String {
        format!("Fill(local_ts_ms={})", self.local_ts_ms)
    }
}

pub fn ws_update_into_py(py: Python<'_>, update: WsUpdate) -> PyResult<Py<PyAny>> {
    match update {
        WsUpdate::Snapshot {
            market_id,
            book,
            exchange_ts,
            local_ts_ms,
            seq,
            ..
        } => {
            let book_py = pythonize(py, &*book)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .into();
            Py::new(
                py,
                Snapshot {
                    market_id,
                    book: book_py,
                    exchange_ts,
                    local_ts_ms,
                    seq,
                },
            )
            .map(Py::into_any)
        }
        WsUpdate::Delta {
            market_id,
            changes,
            exchange_ts,
            local_ts_ms,
            seq,
            ..
        } => {
            let changes_py = pythonize(py, &changes)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .into();
            Py::new(
                py,
                Delta {
                    market_id,
                    changes: changes_py,
                    exchange_ts,
                    local_ts_ms,
                    seq,
                },
            )
            .map(Py::into_any)
        }
        WsUpdate::Trade {
            trade,
            local_ts_ms,
            ..
        } => {
            let trade_py = pythonize(py, &trade)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .into();
            Py::new(
                py,
                Trade {
                    trade: trade_py,
                    local_ts_ms,
                },
            )
            .map(Py::into_any)
        }
        WsUpdate::Fill {
            fill, local_ts_ms, ..
        } => {
            let fill_py = pythonize(py, &fill)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?
                .into();
            Py::new(
                py,
                Fill {
                    fill: fill_py,
                    local_ts_ms,
                },
            )
            .map(Py::into_any)
        }
    }
}

// ---------- SessionEvent variants ----------

#[pyclass(module = "openpx", frozen)]
pub struct Connected;

#[pymethods]
impl Connected {
    #[classattr]
    const __match_args__: () = ();

    #[getter]
    fn kind(&self) -> &'static str {
        "Connected"
    }

    fn __repr__(&self) -> String {
        "Connected()".into()
    }
}

#[pyclass(module = "openpx", frozen)]
pub struct Reconnected {
    #[pyo3(get)]
    pub gap_ms: u64,
}

#[pymethods]
impl Reconnected {
    #[classattr]
    const __match_args__: (&str,) = ("gap_ms",);

    #[getter]
    fn kind(&self) -> &'static str {
        "Reconnected"
    }

    fn __repr__(&self) -> String {
        format!("Reconnected(gap_ms={})", self.gap_ms)
    }
}

#[pyclass(module = "openpx", frozen)]
pub struct Lagged {
    #[pyo3(get)]
    pub dropped: u64,
    #[pyo3(get)]
    pub first_seq: u64,
    #[pyo3(get)]
    pub last_seq: u64,
}

#[pymethods]
impl Lagged {
    #[classattr]
    const __match_args__: (&str, &str, &str) = ("dropped", "first_seq", "last_seq");

    #[getter]
    fn kind(&self) -> &'static str {
        "Lagged"
    }

    fn __repr__(&self) -> String {
        format!(
            "Lagged(dropped={}, first_seq={}, last_seq={})",
            self.dropped, self.first_seq, self.last_seq
        )
    }
}

#[pyclass(module = "openpx", frozen)]
pub struct BookInvalidated {
    #[pyo3(get)]
    pub market_id: String,
    #[pyo3(get)]
    pub reason: String,
    #[pyo3(get)]
    pub expected_seq: Option<u64>,
    #[pyo3(get)]
    pub received_seq: Option<u64>,
}

#[pymethods]
impl BookInvalidated {
    #[classattr]
    const __match_args__: (&str, &str) = ("market_id", "reason");

    #[getter]
    fn kind(&self) -> &'static str {
        "BookInvalidated"
    }

    fn __repr__(&self) -> String {
        format!(
            "BookInvalidated(market_id={:?}, reason={:?})",
            self.market_id, self.reason
        )
    }
}

#[pyclass(module = "openpx", name = "SessionError", frozen)]
pub struct SessionErrorEvent {
    #[pyo3(get)]
    pub message: String,
}

#[pymethods]
impl SessionErrorEvent {
    #[classattr]
    const __match_args__: (&str,) = ("message",);

    #[getter]
    fn kind(&self) -> &'static str {
        "Error"
    }

    fn __repr__(&self) -> String {
        format!("SessionError(message={:?})", self.message)
    }
}

fn invalidation_reason_label(reason: &InvalidationReason) -> (&'static str, Option<u64>, Option<u64>) {
    match reason {
        InvalidationReason::Reconnect => ("Reconnect", None, None),
        InvalidationReason::Lag => ("Lag", None, None),
        InvalidationReason::SequenceGap { expected, received } => {
            ("SequenceGap", Some(*expected), Some(*received))
        }
        InvalidationReason::ExchangeReset => ("ExchangeReset", None, None),
    }
}

pub fn session_event_into_py(py: Python<'_>, event: SessionEvent) -> PyResult<Py<PyAny>> {
    match event {
        SessionEvent::Connected => Py::new(py, Connected).map(Py::into_any),
        SessionEvent::Reconnected { gap_ms } => {
            Py::new(py, Reconnected { gap_ms }).map(Py::into_any)
        }
        SessionEvent::Lagged {
            dropped,
            first_seq,
            last_seq,
        } => Py::new(
            py,
            Lagged {
                dropped,
                first_seq,
                last_seq,
            },
        )
        .map(Py::into_any),
        SessionEvent::BookInvalidated { market_id, reason } => {
            let (label, expected, received) = invalidation_reason_label(&reason);
            Py::new(
                py,
                BookInvalidated {
                    market_id,
                    reason: label.to_string(),
                    expected_seq: expected,
                    received_seq: received,
                },
            )
            .map(Py::into_any)
        }
        SessionEvent::Error { message } => {
            Py::new(py, SessionErrorEvent { message }).map(Py::into_any)
        }
    }
}
