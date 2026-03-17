use std::sync::Arc;

use futures::StreamExt;
use pyo3::prelude::*;
use pythonize::pythonize;
use tokio::sync::{mpsc, Mutex};

use px_core::error::WebSocketError;
use px_core::models::{CryptoPrice, CryptoPriceSource};
use px_crypto::CryptoPriceWebSocket;

use crate::error::to_py_err;
use crate::get_runtime;

fn parse_source(s: &str) -> PyResult<CryptoPriceSource> {
    match s.to_lowercase().as_str() {
        "binance" => Ok(CryptoPriceSource::Binance),
        "chainlink" => Ok(CryptoPriceSource::Chainlink),
        _ => Err(to_py_err(format!(
            "unknown crypto source: {s} (expected 'binance' or 'chainlink')"
        ))),
    }
}

/// Native CryptoPriceWebSocket wrapper for Python.
#[pyclass]
pub struct NativeCryptoPriceWebSocket {
    ws: Arc<Mutex<CryptoPriceWebSocket>>,
}

#[pymethods]
impl NativeCryptoPriceWebSocket {
    #[new]
    fn new() -> Self {
        Self {
            ws: Arc::new(Mutex::new(CryptoPriceWebSocket::new())),
        }
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

    fn subscribe(&self, py: Python<'_>, source: &str, symbols: Vec<String>) -> PyResult<()> {
        let source = parse_source(source)?;
        let ws = self.ws.clone();
        let rt = get_runtime();
        py.detach(|| rt.block_on(async { ws.lock().await.subscribe(source, &symbols).await }))
            .map_err(|e| to_py_err(e.to_string()))
    }

    fn unsubscribe(&self, py: Python<'_>, source: &str, symbols: Vec<String>) -> PyResult<()> {
        let source = parse_source(source)?;
        let ws = self.ws.clone();
        let rt = get_runtime();
        py.detach(|| rt.block_on(async { ws.lock().await.unsubscribe(source, &symbols).await }))
            .map_err(|e| to_py_err(e.to_string()))
    }

    #[getter]
    fn state(&self) -> String {
        let ws = self.ws.clone();
        let rt = get_runtime();
        let state = rt.block_on(async { ws.lock().await.state() });
        format!("{state:?}")
    }

    fn stream(&self) -> NativeCryptoPriceStream {
        let ws = self.ws.clone();
        let rt = get_runtime();

        let stream = rt.block_on(async { ws.lock().await.stream() });

        let (tx, rx) = mpsc::channel(256);
        rt.spawn(async move {
            let mut stream = stream;
            while let Some(item) = stream.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        NativeCryptoPriceStream::new(rx)
    }
}

/// Blocking iterator over crypto price updates.
#[pyclass]
pub struct NativeCryptoPriceStream {
    rx: Arc<Mutex<mpsc::Receiver<Result<CryptoPrice, WebSocketError>>>>,
}

impl NativeCryptoPriceStream {
    pub fn new(rx: mpsc::Receiver<Result<CryptoPrice, WebSocketError>>) -> Self {
        Self {
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

#[pymethods]
impl NativeCryptoPriceStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let rx = self.rx.clone();
        let rt = get_runtime();

        let result = py.detach(|| rt.block_on(async { rx.lock().await.recv().await }));

        match result {
            Some(Ok(data)) => {
                let py_val = pythonize(py, &data).map_err(|e| to_py_err(e.to_string()))?;
                Ok(Some(py_val.into()))
            }
            Some(Err(e)) => Err(to_py_err(e.to_string())),
            None => Ok(None),
        }
    }

    fn close(&self) {
        let rx = self.rx.clone();
        let rt = get_runtime();
        rt.block_on(async {
            rx.lock().await.close();
        });
    }
}
