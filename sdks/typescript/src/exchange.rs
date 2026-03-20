use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use openpx::ExchangeInner;

use crate::error::to_napi_err;
use crate::websocket::WebSocket;

/// Tokio runtime shared across all Exchange instances.
fn get_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime")
    })
}

/// Expose the shared runtime to other modules.
pub fn get_runtime_ref() -> &'static tokio::runtime::Runtime {
    get_runtime()
}

#[napi]
pub struct Exchange {
    inner: Arc<ExchangeInner>,
    config: serde_json::Value,
}

#[napi]
impl Exchange {
    #[napi(constructor)]
    pub fn new(id: String, config: serde_json::Value) -> Result<Self> {
        let inner = ExchangeInner::new(&id, config.clone()).map_err(to_napi_err)?;
        Ok(Self {
            inner: Arc::new(inner),
            config,
        })
    }

    /// Create a WebSocket connection using this exchange's credentials.
    #[napi]
    pub fn websocket(&self) -> Result<WebSocket> {
        WebSocket::new(self.inner.id().to_string(), self.config.clone())
    }

    #[napi(getter)]
    pub fn id(&self) -> &'static str {
        self.inner.id()
    }

    #[napi(getter)]
    pub fn name(&self) -> &'static str {
        self.inner.name()
    }

    #[napi]
    pub fn describe(&self) -> Result<serde_json::Value> {
        let info = self.inner.describe();
        serde_json::to_value(&info).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_markets(
        &self,
        status: Option<String>,
        cursor: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let fetch_params = px_core::FetchMarketsParams {
            status: status
                .map(|s| s.parse::<px_core::MarketStatusFilter>())
                .transpose()
                .map_err(to_napi_err)?,
            cursor,
            ..Default::default()
        };
        let result = rt
            .spawn(async move { inner.fetch_markets(&fetch_params).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        let (markets, next_cursor) = result;
        Ok(serde_json::json!({ "markets": markets, "cursor": next_cursor }))
    }

    #[napi]
    pub async fn fetch_market(&self, market_id: String) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_market(&market_id).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn create_order(
        &self,
        market_id: String,
        outcome: String,
        side: String,
        price: f64,
        size: f64,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let order_side: px_core::OrderSide =
            serde_json::from_value(serde_json::Value::String(side)).map_err(to_napi_err)?;
        let extra: std::collections::HashMap<String, String> = params
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let rt = get_runtime();
        let result = rt
            .spawn(async move {
                inner
                    .create_order(&market_id, &outcome, order_side, price, size, extra)
                    .await
            })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn cancel_order(
        &self,
        order_id: String,
        market_id: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.cancel_order(&order_id, market_id.as_deref()).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_order(
        &self,
        order_id: String,
        market_id: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_order(&order_id, market_id.as_deref()).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_open_orders(&self, market_id: Option<String>) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move {
                let params = market_id.map(|mid| px_core::FetchOrdersParams {
                    market_id: Some(mid),
                });
                inner.fetch_open_orders(params).await
            })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_positions(&self, market_id: Option<String>) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_positions(market_id.as_deref()).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_balance(&self) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_balance().await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_orderbook(
        &self,
        market_id: String,
        outcome: Option<String>,
        token_id: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let req = px_core::OrderbookRequest {
            market_id,
            outcome,
            token_id,
        };
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_orderbook(req).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    pub async fn fetch_fills(
        &self,
        market_id: Option<String>,
        limit: Option<u32>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move {
                inner
                    .fetch_fills(market_id.as_deref(), limit.map(|l| l as usize))
                    .await
            })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    #[allow(clippy::too_many_arguments)]
    pub async fn fetch_price_history(
        &self,
        market_id: String,
        interval: String,
        outcome: Option<String>,
        token_id: Option<String>,
        condition_id: Option<String>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let parsed_interval: px_core::PriceHistoryInterval =
            interval.parse().map_err(|e: String| to_napi_err(e))?;
        let req = px_core::PriceHistoryRequest {
            market_id,
            outcome,
            token_id,
            condition_id,
            interval: parsed_interval,
            start_ts,
            end_ts,
        };
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_price_history(req).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
    }

    #[napi]
    #[allow(clippy::too_many_arguments)]
    pub async fn fetch_trades(
        &self,
        market_id: String,
        market_ref: Option<String>,
        outcome: Option<String>,
        token_id: Option<String>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let req = px_core::TradesRequest {
            market_id,
            market_ref,
            outcome,
            token_id,
            start_ts,
            end_ts,
            limit: limit.map(|l| l as usize),
            cursor,
        };
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_trades(req).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        let (trades, next_cursor) = result;
        Ok(serde_json::json!({ "trades": trades, "cursor": next_cursor }))
    }

    #[napi]
    pub async fn fetch_orderbook_history(
        &self,
        market_id: String,
        token_id: Option<String>,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let req = px_core::OrderbookHistoryRequest {
            market_id,
            token_id,
            start_ts,
            end_ts,
            limit: limit.map(|l| l as usize),
            cursor,
        };
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_orderbook_history(req).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        let (snapshots, next_cursor) = result;
        Ok(serde_json::json!({ "snapshots": snapshots, "cursor": next_cursor }))
    }
}
