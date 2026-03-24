use std::sync::Arc;

use pyo3::prelude::*;
use pythonize::pythonize;
use tokio::sync::{mpsc, Mutex};

use px_core::error::WebSocketError;
use px_core::models::OrderbookUpdate;
use px_core::websocket::{ActivityEvent, WsMessage};

use crate::error::to_py_err;
use crate::get_runtime;

type ObReceiver = mpsc::Receiver<Result<WsMessage<OrderbookUpdate>, WebSocketError>>;
type ActivityReceiver = mpsc::Receiver<Result<WsMessage<ActivityEvent>, WebSocketError>>;

/// Blocking iterator over orderbook updates.
/// Each `__next__` call releases the GIL and waits for the next update.
#[pyclass]
pub struct NativeOrderbookStream {
    rx: Arc<Mutex<ObReceiver>>,
}

impl NativeOrderbookStream {
    pub fn new(rx: ObReceiver) -> Self {
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

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let rx = self.rx.clone();
        let rt = get_runtime();

        let result = py.detach(|| rt.block_on(async { rx.lock().await.recv().await }));

        match result {
            Some(Ok(msg)) => {
                let py_val = pythonize(py, &msg).map_err(|e| to_py_err(e.to_string()))?;
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
    rx: Arc<Mutex<ActivityReceiver>>,
}

impl NativeActivityStream {
    pub fn new(rx: ActivityReceiver) -> Self {
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

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let rx = self.rx.clone();
        let rt = get_runtime();

        let result = py.detach(|| rt.block_on(async { rx.lock().await.recv().await }));

        match result {
            Some(Ok(msg)) => {
                let py_val = pythonize(py, &msg).map_err(|e| to_py_err(e.to_string()))?;
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
