use std::sync::Arc;

use futures::StreamExt;
use pyo3::prelude::*;
use pythonize::pythonize;
use tokio::sync::{mpsc, Mutex};

use px_core::error::WebSocketError;
use px_core::models::SportResult;
use px_sports::SportsWebSocket;

use crate::error::to_py_err;
use crate::get_runtime;

/// Native SportsWebSocket wrapper for Python.
#[pyclass]
pub struct NativeSportsWebSocket {
    ws: Arc<Mutex<SportsWebSocket>>,
}

#[pymethods]
impl NativeSportsWebSocket {
    #[new]
    fn new() -> Self {
        Self {
            ws: Arc::new(Mutex::new(SportsWebSocket::new())),
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

    #[getter]
    fn state(&self) -> String {
        let ws = self.ws.clone();
        let rt = get_runtime();
        let state = rt.block_on(async { ws.lock().await.state() });
        format!("{state:?}")
    }

    fn stream(&self) -> NativeSportsStream {
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

        NativeSportsStream::new(rx)
    }
}

/// Blocking iterator over sports score updates.
#[pyclass]
pub struct NativeSportsStream {
    rx: Arc<Mutex<mpsc::Receiver<Result<SportResult, WebSocketError>>>>,
}

impl NativeSportsStream {
    pub fn new(rx: mpsc::Receiver<Result<SportResult, WebSocketError>>) -> Self {
        Self {
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

#[pymethods]
impl NativeSportsStream {
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
