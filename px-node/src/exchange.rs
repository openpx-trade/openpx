use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use px_sdk::ExchangeInner;

use crate::error::to_napi_err;

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

#[napi]
pub struct Exchange {
    inner: Arc<ExchangeInner>,
}

#[napi]
impl Exchange {
    #[napi(constructor)]
    pub fn new(id: String, config: serde_json::Value) -> Result<Self> {
        let inner = ExchangeInner::new(&id, config).map_err(to_napi_err)?;
        Ok(Self {
            inner: Arc::new(inner),
        })
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
        limit: Option<u32>,
        cursor: Option<String>,
    ) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move {
                let params = if limit.is_some() || cursor.is_some() {
                    Some(px_core::FetchMarketsParams {
                        limit: limit.map(|l| l as usize),
                        cursor,
                    })
                } else {
                    None
                };
                inner.fetch_markets(params).await
            })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        serde_json::to_value(&result).map_err(to_napi_err)
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
    pub async fn fetch_all_unified_markets(&self) -> Result<serde_json::Value> {
        let inner = self.inner.clone();
        let rt = get_runtime();
        let result = rt
            .spawn(async move { inner.fetch_all_unified_markets().await })
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
}
