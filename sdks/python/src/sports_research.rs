//! Native Python binding for the `Sports` research facade. Exposes the ESPN
//! catalog + the venue-bridge primitive.

use std::sync::Arc;

use futures::StreamExt;
use openpx::{Game, GameFilter, GameId, Sports};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pythonize::{depythonize, pythonize};
use tokio::sync::{mpsc, Mutex};

use px_core::error::OpenPxError;
use px_sports::GameState;

use crate::error::to_py_err;
use crate::get_runtime;

#[pyclass]
pub struct NativeSports {
    inner: Arc<Sports>,
}

#[pymethods]
impl NativeSports {
    #[new]
    fn new() -> PyResult<Self> {
        let sports = Sports::new().map_err(|e| to_py_err(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(sports),
        })
    }

    fn list_sports(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let sports = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(async { sports.list_sports().await }));
        let v = result.map_err(|e| to_py_err(e.to_string()))?;
        Ok(pythonize(py, &v)
            .map_err(|e| to_py_err(e.to_string()))?
            .into())
    }

    #[pyo3(signature = (sport_id=None))]
    fn list_leagues(&self, py: Python<'_>, sport_id: Option<String>) -> PyResult<Py<PyAny>> {
        let sports = self.inner.clone();
        let rt = get_runtime();
        let result =
            py.detach(|| rt.block_on(async { sports.list_leagues(sport_id.as_deref()).await }));
        let v = result.map_err(|e| to_py_err(e.to_string()))?;
        Ok(pythonize(py, &v)
            .map_err(|e| to_py_err(e.to_string()))?
            .into())
    }

    /// `filter` is a dict matching the `GameFilter` schema (see Pydantic model).
    fn list_games(&self, py: Python<'_>, filter: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let f: GameFilter = depythonize(&filter).map_err(|e| to_py_err(e.to_string()))?;
        let sports = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(async { sports.list_games(f).await }));
        let v = result.map_err(|e| to_py_err(e.to_string()))?;
        Ok(pythonize(py, &v)
            .map_err(|e| to_py_err(e.to_string()))?
            .into())
    }

    fn get_game(&self, py: Python<'_>, league: String, id: String) -> PyResult<Py<PyAny>> {
        let sports = self.inner.clone();
        let rt = get_runtime();
        let result =
            py.detach(|| rt.block_on(async { sports.get_game(&league, &GameId::new(id)).await }));
        let v = result.map_err(|e| to_py_err(e.to_string()))?;
        Ok(pythonize(py, &v)
            .map_err(|e| to_py_err(e.to_string()))?
            .into())
    }

    /// `game` is a dict matching the `Game` schema. Returns a dict with
    /// `kalshi` and `polymarket` keys, each a list of matching events.
    fn markets_for_game(&self, py: Python<'_>, game: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let g: Game = depythonize(&game).map_err(|e| to_py_err(e.to_string()))?;
        let sports = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(async { sports.markets_for_game(&g).await }));
        let v = result.map_err(|e| to_py_err(e.to_string()))?;
        Ok(pythonize(py, &v)
            .map_err(|e| to_py_err(e.to_string()))?
            .into())
    }

    /// Subscribe to live state updates for a league. Returns an iterator
    /// that yields one dict per game state delta.
    fn subscribe_game_state(&self, league: String) -> NativeGameStateStream {
        let stream = self.inner.subscribe_game_state(&league);
        let (tx, rx) = mpsc::channel::<Result<GameState, OpenPxError>>(256);
        let rt = get_runtime();
        rt.spawn(async move {
            let mut s = stream;
            while let Some(item) = s.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });
        NativeGameStateStream {
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

#[pyclass]
pub struct NativeGameStateStream {
    rx: Arc<Mutex<mpsc::Receiver<Result<GameState, OpenPxError>>>>,
}

#[pymethods]
impl NativeGameStateStream {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let rx = self.rx.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(async { rx.lock().await.recv().await }));
        match result {
            Some(Ok(state)) => {
                let val = pythonize(py, &state).map_err(|e| to_py_err(e.to_string()))?;
                Ok(Some(val.into()))
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
