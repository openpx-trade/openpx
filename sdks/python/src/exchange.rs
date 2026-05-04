use pyo3::prelude::*;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyDict;
use pythonize::pythonize;

use openpx::ExchangeInner;
use std::sync::Arc;

use crate::error::to_py_err;
use crate::get_runtime;

/// Native exchange wrapper. Returns Python dicts via pythonize.
/// The pure-Python `Exchange` wrapper unpacks these into Pydantic models.
#[pyclass]
pub struct NativeExchange {
    inner: Arc<ExchangeInner>,
    described: PyOnceLock<Py<PyAny>>,
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
            described: PyOnceLock::new(),
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
        let cached = self.described.get_or_try_init(py, || {
            let info = self.inner.describe();
            let dict = pythonize(py, &info).map_err(|e| to_py_err(e.to_string()))?;
            Ok::<_, PyErr>(dict.unbind())
        })?;
        Ok(cached.bind(py).clone())
    }

    #[pyo3(signature = (status=None, cursor=None, market_tickers=None, series_ticker=None, event_ticker=None, limit=None))]
    #[allow(clippy::too_many_arguments)]
    fn fetch_markets<'py>(
        &self,
        py: Python<'py>,
        status: Option<&str>,
        cursor: Option<&str>,
        market_tickers: Option<Vec<String>>,
        series_ticker: Option<&str>,
        event_ticker: Option<&str>,
        limit: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let fetch_params = px_core::FetchMarketsParams {
            status: status
                .map(|s| s.parse::<px_core::MarketStatusFilter>())
                .transpose()
                .map_err(to_py_err)?,
            cursor: cursor.map(String::from),
            market_tickers: market_tickers.unwrap_or_default(),
            series_ticker: series_ticker.map(String::from),
            event_ticker: event_ticker.map(String::from),
            limit,
        };
        let result = py.detach(|| rt.block_on(inner.fetch_markets(&fetch_params)));
        let (markets, next_cursor) = result.map_err(|e| to_py_err(e.to_string()))?;
        let val = serde_json::json!({ "markets": markets, "cursor": next_cursor });
        pythonize(py, &val).map_err(|e| to_py_err(e.to_string()))
    }

    fn fetch_market_lineage<'py>(
        &self,
        py: Python<'py>,
        market_ticker: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_ticker = market_ticker.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_market_lineage(&market_ticker)));
        let lineage = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &lineage).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id, outcome, side, price, size, order_type="gtc"))]
    #[allow(clippy::too_many_arguments)]
    fn create_order<'py>(
        &self,
        py: Python<'py>,
        asset_id: &str,
        outcome: &str,
        side: &str,
        price: f64,
        size: f64,
        order_type: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let order_side: px_core::OrderSide =
            serde_json::from_value(serde_json::Value::String(side.to_string()))
                .map_err(|e| to_py_err(e.to_string()))?;
        let order_type_enum: px_core::OrderType = match order_type.to_ascii_lowercase().as_str() {
            "gtc" => px_core::OrderType::Gtc,
            "ioc" => px_core::OrderType::Ioc,
            "fok" => px_core::OrderType::Fok,
            other => {
                return Err(to_py_err(format!(
                    "invalid order_type '{other}' (allowed: gtc, ioc, fok)"
                )))
            }
        };
        let order_outcome = match outcome.to_ascii_lowercase().as_str() {
            "yes" => px_core::OrderOutcome::Yes,
            "no" => px_core::OrderOutcome::No,
            _ => px_core::OrderOutcome::Label(outcome.to_string()),
        };
        let req = px_core::CreateOrderRequest {
            asset_id: asset_id.to_string(),
            outcome: order_outcome,
            side: order_side,
            price,
            size,
            order_type: order_type_enum,
        };
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.create_order(req)));
        let order = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &order).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (order_id))]
    fn cancel_order<'py>(&self, py: Python<'py>, order_id: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let order_id = order_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.cancel_order(&order_id)));
        let order = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &order).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id=None))]
    fn cancel_all_orders<'py>(
        &self,
        py: Python<'py>,
        asset_id: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.cancel_all_orders(asset_id.as_deref())));
        let cancelled = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &cancelled).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (orders))]
    fn create_orders_batch<'py>(
        &self,
        py: Python<'py>,
        orders: Vec<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Each order is a dict with the same fields as `create_order` args.
        let mut reqs = Vec::with_capacity(orders.len());
        for entry in orders {
            let val: serde_json::Value =
                pythonize::depythonize(&entry).map_err(|e| to_py_err(e.to_string()))?;
            let req: px_core::CreateOrderRequest =
                serde_json::from_value(val).map_err(|e| to_py_err(e.to_string()))?;
            reqs.push(req);
        }
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.create_orders_batch(reqs)));
        let placed = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &placed).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (order_id))]
    fn fetch_order<'py>(&self, py: Python<'py>, order_id: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let order_id = order_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_order(&order_id)));
        let order = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &order).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id=None))]
    fn fetch_open_orders<'py>(
        &self,
        py: Python<'py>,
        asset_id: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_open_orders(asset_id.as_deref())));
        let orders = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &orders).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_ticker=None))]
    fn fetch_positions<'py>(
        &self,
        py: Python<'py>,
        market_ticker: Option<&str>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_ticker = market_ticker.map(String::from);
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_positions(market_ticker.as_deref())));
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

    fn refresh_balance(&self, py: Python<'_>) -> PyResult<()> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.refresh_balance()));
        result.map_err(|e| to_py_err(e.to_string()))
    }

    fn fetch_server_time<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_server_time()));
        let ts = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &ts.to_rfc3339()).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id))]
    fn fetch_orderbook<'py>(&self, py: Python<'py>, asset_id: &str) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let asset_id = asset_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbook(&asset_id)));
        let book = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &book).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_ids))]
    fn fetch_orderbooks_batch<'py>(
        &self,
        py: Python<'py>,
        asset_ids: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbooks_batch(asset_ids)));
        let books = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &books).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id))]
    fn fetch_orderbook_stats<'py>(
        &self,
        py: Python<'py>,
        asset_id: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let asset_id = asset_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbook_stats(&asset_id)));
        let stats = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &stats).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id, size))]
    fn fetch_orderbook_impact<'py>(
        &self,
        py: Python<'py>,
        asset_id: &str,
        size: f64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let asset_id = asset_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbook_impact(&asset_id, size)));
        let impact = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &impact).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id))]
    fn fetch_orderbook_microstructure<'py>(
        &self,
        py: Python<'py>,
        asset_id: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let asset_id = asset_id.to_string();
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_orderbook_microstructure(&asset_id)));
        let micro = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &micro).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (market_ticker=None, limit=None))]
    fn fetch_fills<'py>(
        &self,
        py: Python<'py>,
        market_ticker: Option<&str>,
        limit: Option<usize>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let market_ticker = market_ticker.map(String::from);
        let rt = get_runtime();
        let result = py.detach(|| rt.block_on(inner.fetch_fills(market_ticker.as_deref(), limit)));
        let fills = result.map_err(|e| to_py_err(e.to_string()))?;
        pythonize(py, &fills).map_err(|e| to_py_err(e.to_string()))
    }

    #[pyo3(signature = (asset_id, start_ts=None, end_ts=None, limit=None, cursor=None))]
    fn fetch_trades<'py>(
        &self,
        py: Python<'py>,
        asset_id: &str,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = self.inner.clone();
        let req = px_core::TradesRequest {
            asset_id: asset_id.to_string(),
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
}
