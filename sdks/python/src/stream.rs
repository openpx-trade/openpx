use pyo3::prelude::*;
use pythonize::pythonize;

use px_core::websocket::{SessionStream, UpdateStream};

use crate::error::to_py_err;
use crate::get_runtime;

/// Blocking iterator over the multiplexed WebSocket update stream.
/// Each `__next__` call releases the GIL and waits for the next update.
#[pyclass]
pub struct NativeUpdateStream {
    rx: UpdateStream,
}

impl NativeUpdateStream {
    pub fn new(rx: UpdateStream) -> Self {
        Self { rx }
    }
}

#[pymethods]
impl NativeUpdateStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(self.rx.next()));
        match result {
            Some(update) => {
                let py_val = pythonize(py, &update).map_err(|e| to_py_err(e.to_string()))?;
                Ok(Some(py_val.into()))
            }
            None => Ok(None),
        }
    }
}

/// Blocking iterator over connection-level session events
/// (Connected, Reconnected, Lagged, BookInvalidated, Error).
#[pyclass]
pub struct NativeSessionStream {
    rx: SessionStream,
}

impl NativeSessionStream {
    pub fn new(rx: SessionStream) -> Self {
        Self { rx }
    }
}

#[pymethods]
impl NativeSessionStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(self.rx.next()));
        match result {
            Some(event) => {
                let py_val = pythonize(py, &event).map_err(|e| to_py_err(e.to_string()))?;
                Ok(Some(py_val.into()))
            }
            None => Ok(None),
        }
    }
}
