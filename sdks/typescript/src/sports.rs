use std::sync::Arc;

use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use tokio::sync::Mutex;

use px_sports::SportsWebSocket as InnerSportsWebSocket;

use crate::error::to_napi_err;

fn get_runtime() -> &'static tokio::runtime::Runtime {
    crate::exchange::get_runtime_ref()
}

#[napi]
pub struct SportsWebSocket {
    inner: Arc<Mutex<InnerSportsWebSocket>>,
}

impl Default for SportsWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl SportsWebSocket {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerSportsWebSocket::new())),
        }
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

    #[napi(getter)]
    pub fn state(&self) -> String {
        let ws = self.inner.clone();
        let rt = get_runtime();
        let state = rt.block_on(async { ws.lock().await.state() });
        format!("{state:?}")
    }

    /// Subscribe to score updates via callback.
    /// The callback receives each SportResult as a JSON object.
    #[napi]
    pub async fn on_score_update(
        &self,
        #[napi(ts_arg_type = "(err: Error | null, score: any) => void")]
        callback: ThreadsafeFunction<serde_json::Value, ErrorStrategy::CalleeHandled>,
    ) -> Result<()> {
        let ws = self.inner.clone();
        let rt = get_runtime();

        let stream = rt
            .spawn(async move { ws.lock().await.stream() })
            .await
            .map_err(to_napi_err)?;

        rt.spawn(async move {
            let mut stream = stream;
            while let Some(item) = stream.next().await {
                match item {
                    Ok(data) => {
                        let val = serde_json::to_value(&data).unwrap_or_default();
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
