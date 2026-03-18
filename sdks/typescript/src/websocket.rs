use std::sync::Arc;

use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use tokio::sync::Mutex;

use px_core::websocket::OrderBookWebSocket;
use openpx::WebSocketInner;

use crate::error::to_napi_err;

fn get_runtime() -> &'static tokio::runtime::Runtime {
    crate::exchange::get_runtime_ref()
}

#[napi]
pub struct WebSocket {
    inner: Arc<Mutex<WebSocketInner>>,
}

#[napi]
impl WebSocket {
    #[napi(constructor)]
    pub fn new(id: String, config: serde_json::Value) -> Result<Self> {
        let ws = WebSocketInner::new(&id, config).map_err(to_napi_err)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(ws)),
        })
    }

    #[napi]
    pub async fn connect(&self) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.spawn(async move { ws.lock().await.connect().await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)
    }

    #[napi]
    pub async fn disconnect(&self) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.spawn(async move { ws.lock().await.disconnect().await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)
    }

    #[napi]
    pub async fn subscribe(&self, market_id: String) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.spawn(async move { ws.lock().await.subscribe(&market_id).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)
    }

    #[napi]
    pub async fn unsubscribe(&self, market_id: String) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.spawn(async move { ws.lock().await.unsubscribe(&market_id).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)
    }

    #[napi(getter)]
    pub fn state(&self) -> String {
        let ws = self.inner.clone();
        let rt = get_runtime();
        let state = rt.block_on(async { ws.lock().await.state() });
        format!("{state:?}")
    }

    /// Subscribe to orderbook updates via callback.
    /// The callback receives each update as a JSON object.
    /// Returns when the stream ends or an error occurs.
    #[napi]
    pub async fn on_orderbook_update(
        &self,
        market_id: String,
        #[napi(ts_arg_type = "(err: Error | null, update: any) => void")]
        callback: ThreadsafeFunction<serde_json::Value, ErrorStrategy::CalleeHandled>,
    ) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();

        let stream = rt
            .spawn(async move { ws.lock().await.orderbook_stream(&market_id).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;

        rt.spawn(async move {
            let mut stream = stream;
            while let Some(item) = stream.next().await {
                match item {
                    Ok(update) => {
                        let val = serde_json::to_value(&update).unwrap_or_default();
                        callback.call(Ok(val), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    Err(e) => {
                        callback.call(
                            Err(Error::from_reason(e.to_string())),
                            ThreadsafeFunctionCallMode::NonBlocking,
                        );
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Subscribe to activity events (trades, fills) via callback.
    #[napi]
    pub async fn on_activity_update(
        &self,
        market_id: String,
        #[napi(ts_arg_type = "(err: Error | null, event: any) => void")]
        callback: ThreadsafeFunction<serde_json::Value, ErrorStrategy::CalleeHandled>,
    ) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();

        let stream = rt
            .spawn(async move { ws.lock().await.activity_stream(&market_id).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;

        rt.spawn(async move {
            let mut stream = stream;
            while let Some(item) = stream.next().await {
                match item {
                    Ok(event) => {
                        let val = serde_json::to_value(&event).unwrap_or_default();
                        callback.call(Ok(val), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    Err(e) => {
                        callback.call(
                            Err(Error::from_reason(e.to_string())),
                            ThreadsafeFunctionCallMode::NonBlocking,
                        );
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}
