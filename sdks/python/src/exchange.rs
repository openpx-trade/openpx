use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::pythonize;

use px_sdk::ExchangeInner;
use std::sync::Arc;

use crate::error::to_py_err;
use crate::get_runtime;

/// Native exchange wrapper. Returns Python dicts via pythonize.
/// The pure-Python `Exchange` wrapper unpacks these into Pydantic models.
#[pyclass]
pub struct NativeExchange {
    inner: Arc<ExchangeInner>,
}

#[pymethods]
impl NativeExchange {
    #[new]
    fn new(id: &str, config: &Bound<'_, PyDict>) -> PyResult<Self> {
        let config_json: serde_json::Value =
            pythonize::depythonize(config).map_err(|e| to_py_err(e.to_string()))?;
        let inner = ExchangeInner::new(id, config_json).map_err(|e| to_py_err(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[getter]
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    #[getter]
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn describe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let info = self.inner.describe();
        pythonize(py, &info).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (status=None, cursor=None))]
    fn fetch_markets<'py>(
        &self,
        py: Python<'py>,
        status: Option<&str>,
        cursor: Option<&str>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let fetch_params = px_core::FetchMarketsParams {
            status: status
                .map(|s| s.parse::<px_core::MarketStatus>())
                .transpose()
                .map_err(to_py_err)?,
            cursor: cursor.map(String::from),
            ..Default::default()
        };
        let result = py.detach(|| rt.block_on(inner.fetch_markets(&fetch_params)));
        let (markets, next_cursor) = result.map_err(|e| to_py_err(e.to_string()))?;
        let val = serde_json::json!({ "markets": markets, "cursor": next_cursor });
        pythonize(py, &val).map_err(|e| to_py_err(e.to_string()))
    }

    fn fetch_market<'py>(&self, py: Python<'py>, market_id: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_id = market_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_market(&market_id)));
        let market = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &market).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id, outcome, side, price, size, params=None))]
    #[allow(clippy::too_many_arguments)]
    fn create_order<'py>(
        &self,
        py: Python<'py>,
        market_id: &str,
        outcome: &str,
        side: &str,
        price: f64,
        size: f64,
        params: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_id = market_id.to_string();
        let outcome = outcome.to_string();
        let order_side: px_core::OrderSide =
            serde_json::from_value(serde_json::Value::String(side.to_string()))
                .map_err(|e| to_py_err(e.to_string()))?;
        let extra: std::collections::HashMap<String, String> = match params {
            Some(d) => pythonize::depythonize(d).unwrap_or_default(),
            None => std::collections::HashMap::new(),
        };
        let rt = get_runtime();
        let result = py.detach(|| {
            rt.block_on(inner.create_order(&market_id, &outcome, order_side, price, size, extra))
        });
        let order = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &order).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (order_id, market_id=None))]
    fn cancel_order<'py>(
        &self,
        py: Python<'py>,
        order_id: &str,
        market_id: Option<&str>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let order_id = order_id.to_string();
        let market_id = market_id.map(String::from);
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.cancel_order(&order_id, market_id.as_deref())));
        let order = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &order).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (order_id, market_id=None))]
    fn fetch_order<'py>(
        &self,
        py: Python<'py>,
        order_id: &str,
        market_id: Option<&str>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let order_id = order_id.to_string();
        let market_id = market_id.map(String::from);
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_order(&order_id, market_id.as_deref())));
        let order = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &order).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id=None))]
    fn fetch_open_orders<'py>(
        &self,
        py: Python<'py>,
        market_id: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| {
            let params = market_id.map(|mid| px_core::FetchOrdersParams {
                market_id: Some(mid),
            });
            rt.block_on(inner.fetch_open_orders(params))
        });
        let orders = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &orders).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id=None))]
    fn fetch_positions<'py>(
        &self,
        py: Python<'py>,
        market_id: Option<&str>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_id = market_id.map(String::from);
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_positions(market_id.as_deref())));
        let positions = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &positions).map_err(|e| to_py_err(e.to_string()))
    }

    fn fetch_balance<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_balance()));
        let balance = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &balance).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id, outcome=None, token_id=None))]
    fn fetch_orderbook<'py>(
        &self,
        py: Python<'py>,
        market_id: &str,
        outcome: Option<String>,
        token_id: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let req = px_core::OrderbookRequest {
            market_id: market_id.to_string(),
            outcome,
            token_id,
        };
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbook(req)));
        let book = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &book).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id=None, limit=None))]
    fn fetch_fills<'py>(
        &self,
        py: Python<'py>,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_id = market_id.map(String::from);
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_fills(market_id.as_deref(), limit)));
        let fills = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &fills).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id, interval, outcome=None, token_id=None, condition_id=None, start_ts=None, end_ts=None))]
    #[allow(clippy::too_many_arguments)]
    fn fetch_price_history<'py>(
        &self,
        py: Python<'py>,
        market_id: &str,
        interval: &str,
        outcome: Option<String>,
        token_id: Option<String>,
        condition_id: Option<String>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let parsed_interval: px_core::PriceHistoryInterval =
            interval.parse().map_err(|e: String| to_py_err(e))?;
        let req = px_core::PriceHistoryRequest {
            market_id: market_id.to_string(),
            outcome,
            token_id,
            condition_id,
            interval: parsed_interval,
            start_ts,
            end_ts,
        };
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_price_history(req)));
        let candles = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &candles).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id, market_ref=None, outcome=None, token_id=None, start_ts=None, end_ts=None, limit=None, cursor=None))]
    #[allow(clippy::too_many_arguments)]
    fn fetch_trades<'py>(
        &self,
        py: Python<'py>,
        market_id: &str,
        market_ref: Option<String>,
        outcome: Option<String>,
        token_id: Option<String>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let req = px_core::TradesRequest {
            market_id: market_id.to_string(),
            market_ref,
            outcome,
            token_id,
            start_ts,
            end_ts,
            limit,
            cursor,
        };
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_trades(req)));
        let (trades, next_cursor) = result.map_err(|e| to_py_err(e.to_string()))?;
        let val = serde_json::json!({ "trades": trades, "cursor": next_cursor });
        pythonize(py, &val).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_id, token_id=None, start_ts=None, end_ts=None, limit=None, cursor=None))]
    #[allow(clippy::too_many_arguments)]
    fn fetch_orderbook_history<'py>(
        &self,
        py: Python<'py>,
        market_id: &str,
        token_id: Option<String>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let req = px_core::OrderbookHistoryRequest {
            market_id: market_id.to_string(),
            token_id,
            start_ts,
            end_ts,
            limit,
            cursor,
        };
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbook_history(req)));
        let (snapshots, next_cursor) = result.map_err(|e| to_py_err(e.to_string()))?;
        let val = serde_json::json!({ "snapshots": snapshots, "cursor": next_cursor });
        pythonize(py, &val).map_err(|e| to_py_err(e.to_string()))
    }
}
