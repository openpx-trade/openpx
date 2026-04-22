use std::sync::Arc;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use tokio::sync::Mutex;

use openpx::WebSocketInner;
use px_core::websocket::OrderBookWebSocket;

use crate::error::to_py_err;
use crate::get_runtime;
use crate::stream::{NativeSessionStream, NativeUpdateStream};

/// Native WebSocket wrapper for Python.
/// Created via `Exchange.websocket()` or standalone `WebSocket(id, config)`.
#[pyclass]
pub struct NativeWebSocket {
    ws: Arc<Mutex<WebSocketInner>>,
}

#[pymethods]
impl NativeWebSocket {
    #[new]
    fn new(id: &str, config: &Bound<'_, PyDict>) -> PyResult<Self> {
        let config_json: serde_json::Value =
            pythonize::depythonize(config).map_err(|e| to_py_err(e.to_string()))?;
        let ws = WebSocketInner::new(id, config_json).map_err(|e| to_py_err(e.to_string()))?;
        Ok(Self {
            ws: Arc::new(Mutex::new(ws)),
        })
    }

    fn connect(&self, py: Python<'_>) -> PyResult<()> {
        let ws = self.ws.clone();
        let rt = get_runtime();
        py.detach(|| rt.block_on(async { ws.lock().await.connect().await }))
            .map_err(|e| to_py_err(e.to_string()))
    }

    fn disconnect(&self, py: Python<'_>) -> PyResult<()> {
        let ws = self.ws.clone();
        let rt = get_runtime();
        py.detach(|| rt.block_on(async { ws.lock().await.disconnect().await }))
            .map_err(|e| to_py_err(e.to_string()))
    }

    fn subscribe(&self, py: Python<'_>, market_id: &str) -> PyResult<()> {
        let ws = self.ws.clone();
        let market_id = market_id.to_string();
        let rt = get_runtime();
        py.detach(|| rt.block_on(async { ws.lock().await.subscribe(&market_id).await }))
            .map_err(|e| to_py_err(e.to_string()))
    }

    fn unsubscribe(&self, py: Python<'_>, market_id: &str) -> PyResult<()> {
        let ws = self.ws.clone();
        let market_id = market_id.to_string();
        let rt = get_runtime();
        py.detach(|| rt.block_on(async { ws.lock().await.unsubscribe(&market_id).await }))
            .map_err(|e| to_py_err(e.to_string()))
    }

    fn state(&self) -> &'static str {
        let ws = self.ws.clone();
        let rt = get_runtime();
        rt.block_on(async { ws.lock().await.state() }).as_str()
    }

    /// Iterator over the multiplexed `WsUpdate` stream. Each item is one of:
    /// Snapshot, Delta, Trade, Fill, Raw — distinguished by the `kind` field.
    fn updates(&self, py: Python<'_>) -> PyResult<NativeUpdateStream> {
        let ws = self.ws.clone();
        let rt = get_runtime();
        let stream = py.detach(|| rt.block_on(async { ws.lock().await.updates() }));
        Ok(NativeUpdateStream::new(stream))
    }

    /// Iterator over connection-level session events
    /// (Connected, Reconnected, Lagged, BookInvalidated, Error).
    fn session_events(&self, py: Python<'_>) -> PyResult<NativeSessionStream> {
        let ws = self.ws.clone();
        let rt = get_runtime();
        let stream = py.detach(|| rt.block_on(async { ws.lock().await.session_events() }));
        Ok(NativeSessionStream::new(stream))
    }
}
