use std::sync::Arc;

use futures::StreamExt;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use tokio::sync::Mutex;

use px_core::websocket::OrderBookWebSocket;
use px_sdk::WebSocketInner;

use crate::error::to_py_err;
use crate::get_runtime;
use crate::stream::{NativeActivityStream, NativeOrderbookStream};

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

    fn state(&self) -> String {
        let ws = self.ws.clone();
        let rt = get_runtime();
        let state = rt.block_on(async { ws.lock().await.state() });
        format!("{state:?}")
    }

    fn orderbook_stream(&self, py: Python<'_>, market_id: &str) -> PyResult<NativeOrderbookStream> {
        let ws = self.ws.clone();
        let market_id = market_id.to_string();
        let rt = get_runtime();

        let stream = py
            .detach(|| rt.block_on(async { ws.lock().await.orderbook_stream(&market_id).await }))
            .map_err(|e: px_core::error::WebSocketError| to_py_err(e.to_string()))?;

        let (tx, rx) = tokio::sync::mpsc::channel(256);
        rt.spawn(async move {
            let mut stream = stream;
            while let Some(item) = stream.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        Ok(NativeOrderbookStream::new(rx))
    }

    fn activity_stream(&self, py: Python<'_>, market_id: &str) -> PyResult<NativeActivityStream> {
        let ws = self.ws.clone();
        let market_id = market_id.to_string();
        let rt = get_runtime();

        let stream = py
            .detach(|| rt.block_on(async { ws.lock().await.activity_stream(&market_id).await }))
            .map_err(|e: px_core::error::WebSocketError| to_py_err(e.to_string()))?;

        let (tx, rx) = tokio::sync::mpsc::channel(256);
        rt.spawn(async move {
            let mut stream = stream;
            while let Some(item) = stream.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        Ok(NativeActivityStream::new(rx))
    }
}
