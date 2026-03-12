use std::sync::Arc;

use pyo3::prelude::*;
use pythonize::pythonize;
use tokio::sync::{mpsc, Mutex};

use px_core::error::WebSocketError;
use px_core::models::OrderbookUpdate;
use px_core::websocket::ActivityEvent;

use crate::error::to_py_err;
use crate::get_runtime;

/// Blocking iterator over orderbook updates.
/// Each `__next__` call releases the GIL and waits for the next update.
#[pyclass]
pub struct NativeOrderbookStream {
    rx: Arc<Mutex<mpsc::Receiver<Result<OrderbookUpdate, WebSocketError>>>>,
}

impl NativeOrderbookStream {
    pub fn new(rx: mpsc::Receiver<Result<OrderbookUpdate, WebSocketError>>) -> Self {
        Self {
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

#[pymethods]
impl NativeOrderbookStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let rx = self.rx.clone();
        let rt = get_runtime();

        let result = py.allow_threads(|| rt.block_on(async { rx.lock().await.recv().await }));

        match result {
            Some(Ok(update)) => {
                let val = serde_json::to_value(&update).map_err(|e| to_py_err(e.to_string()))?;
                let py_val = pythonize(py, &val).map_err(|e| to_py_err(e.to_string()))?;
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

/// Blocking iterator over activity events (trades, fills).
#[pyclass]
pub struct NativeActivityStream {
    rx: Arc<Mutex<mpsc::Receiver<Result<ActivityEvent, WebSocketError>>>>,
}

impl NativeActivityStream {
    pub fn new(rx: mpsc::Receiver<Result<ActivityEvent, WebSocketError>>) -> Self {
        Self {
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

#[pymethods]
impl NativeActivityStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let rx = self.rx.clone();
        let rt = get_runtime();

        let result = py.allow_threads(|| rt.block_on(async { rx.lock().await.recv().await }));

        match result {
            Some(Ok(event)) => {
                let py_val = pythonize(py, &event).map_err(|e| to_py_err(e.to_string()))?;
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
