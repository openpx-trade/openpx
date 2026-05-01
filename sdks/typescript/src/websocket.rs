use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use tokio::sync::Mutex;

use openpx::WebSocketInner;
use px_core::websocket::OrderBookWebSocket;

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
    pub async fn subscribe(&self, market_ticker: String) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.spawn(async move { ws.lock().await.subscribe(&market_ticker).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)
    }

    #[napi]
    pub async fn unsubscribe(&self, market_ticker: String) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.spawn(async move { ws.lock().await.unsubscribe(&market_ticker).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)
    }

    #[napi(getter)]
    pub fn state(&self) -> &'static str {
        let ws = self.inner.clone();
        let rt = get_runtime();
        rt.block_on(async { ws.lock().await.state() }).as_str()
    }

    /// Subscribe to the multiplexed update stream via callback. Each call
    /// delivers one `WsUpdate` (Snapshot, Delta, Trade, Fill) as a JSON
    /// object with a `kind` discriminator. Single-consumer: registering a
    /// second callback on the same WebSocket rejects.
    #[napi]
    pub async fn on_update(
        &self,
        #[napi(ts_arg_type = "(err: Error | null, update: any) => void")]
        callback: ThreadsafeFunction<serde_json::Value, ErrorStrategy::CalleeHandled>,
    ) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();

        let stream = rt
            .spawn(async move { ws.lock().await.updates() })
            .await
            .map_err(to_napi_err)?
            .ok_or_else(|| to_napi_err("updates() already taken; the stream is single-consumer"))?;

        rt.spawn(async move {
            while let Some(update) = stream.next().await {
                let val = serde_json::to_value(&update).unwrap_or_default();
                callback.call(Ok(val), ThreadsafeFunctionCallMode::NonBlocking);
            }
        });

        Ok(())
    }

    /// Subscribe to connection-level session events via callback
    /// (Connected, Reconnected, Lagged, BookInvalidated, Error).
    /// Single-consumer: registering a second callback on the same WebSocket
    /// rejects.
    #[napi]
    pub async fn on_session_event(
        &self,
        #[napi(ts_arg_type = "(err: Error | null, event: any) => void")]
        callback: ThreadsafeFunction<serde_json::Value, ErrorStrategy::CalleeHandled>,
    ) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();

        let stream = rt
            .spawn(async move { ws.lock().await.session_events() })
            .await
            .map_err(to_napi_err)?
            .ok_or_else(|| {
                to_napi_err("session_events() already taken; the stream is single-consumer")
            })?;

        rt.spawn(async move {
            while let Some(event) = stream.next().await {
                let val = serde_json::to_value(&event).unwrap_or_default();
                callback.call(Ok(val), ThreadsafeFunctionCallMode::NonBlocking);
            }
        });

        Ok(())
    }
}
