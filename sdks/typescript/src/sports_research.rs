//! Native NAPI binding for the `Sports` research facade. Exposes the ESPN
//! catalog + the venue-bridge primitive.

use std::sync::Arc;

use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use openpx::{GameFilter, GameId, Sports};

use crate::error::to_napi_err;

fn rt() -> &'static tokio::runtime::Runtime {
    crate::exchange::get_runtime_ref()
}

#[napi(js_name = "Sports")]
pub struct JsSports {
    inner: Arc<Sports>,
}

#[napi]
impl JsSports {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let sports = Sports::new().map_err(|e| to_napi_err(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(sports),
        })
    }

    #[napi]
    pub async fn list_sports(&self) -> Result<serde_json::Value> {
        let sports = self.inner.clone();
        let v = rt()
            .spawn(async move { sports.list_sports().await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        Ok(serde_json::to_value(&v).unwrap_or_default())
    }

    #[napi(ts_args_type = "sportId?: string")]
    pub async fn list_leagues(&self, sport_id: Option<String>) -> Result<serde_json::Value> {
        let sports = self.inner.clone();
        let v = rt()
            .spawn(async move { sports.list_leagues(sport_id.as_deref()).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        Ok(serde_json::to_value(&v).unwrap_or_default())
    }

    /// `filter` should match the `GameFilter` schema (see TS types).
    #[napi(ts_args_type = "filter: GameFilter")]
    pub async fn list_games(&self, filter: serde_json::Value) -> Result<serde_json::Value> {
        let f: GameFilter = serde_json::from_value(filter).map_err(to_napi_err)?;
        let sports = self.inner.clone();
        let v = rt()
            .spawn(async move { sports.list_games(f).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        Ok(serde_json::to_value(&v).unwrap_or_default())
    }

    #[napi]
    pub async fn get_game(&self, league: String, id: String) -> Result<serde_json::Value> {
        let sports = self.inner.clone();
        let v = rt()
            .spawn(async move { sports.get_game(&league, &GameId::new(id)).await })
            .await
            .map_err(to_napi_err)?
            .map_err(to_napi_err)?;
        Ok(serde_json::to_value(&v).unwrap_or_default())
    }

    /// Subscribe to live state updates for a league. The callback receives
    /// each `GameState` delta as a JSON object.
    #[napi]
    pub async fn on_game_state(
        &self,
        league: String,
        #[napi(ts_arg_type = "(err: Error | null, state: any) => void")]
        callback: ThreadsafeFunction<serde_json::Value, ErrorStrategy::CalleeHandled>,
    ) -> Result<()> {
        let sports = self.inner.clone();
        let stream = sports.subscribe_game_state(&league);
        rt().spawn(async move {
            let mut s = stream;
            while let Some(item) = s.next().await {
                match item {
                    Ok(state) => {
                        let val = serde_json::to_value(&state).unwrap_or_default();
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
