use px_core::{
    manifests::POLYMARKET_MANIFEST, CheckpointCallback, FetchResult, MarketFetcher, OpenPxError,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

use crate::error::PolymarketError;

/// /markets/keyset is cursor-paginated, so we can't fan out within a single
/// stream. Two-phase drain — `closed=false` then `closed=true` — gives us the
/// only natural parallelism axis Polymarket exposes for "all markets".
const PHASES: &[(bool, &str)] = &[(false, "o"), (true, "c")];

/// Per-phase cursor state for resumable bulk fetches. Serialized as the
/// fetcher's compound cursor string.
/// - Key absent  → phase not yet started (begin from no cursor).
/// - Value `None` → phase exhausted.
/// - Value `Some(_)` → continue from that `after_cursor`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PhaseCursors {
    cursors: HashMap<String, Option<String>>,
}

impl PhaseCursors {
    fn parse(s: Option<&str>) -> Self {
        s.and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

pub struct PolymarketMarketFetcher {
    base_url: String,
}

impl PolymarketMarketFetcher {
    pub fn new() -> Result<Self, PolymarketError> {
        Ok(Self {
            base_url: POLYMARKET_MANIFEST.base_url.to_string(),
        })
    }

    fn create_client() -> Result<Client, PolymarketError> {
        px_core::http::tuned_client_builder()
            .pool_max_idle_per_host(PHASES.len())
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(PolymarketError::from)
    }

    /// Fetch one keyset page for a given phase.
    /// Returns (markets, next_cursor). `next_cursor` is `None` when the phase
    /// is exhausted (server omits or empties the field).
    async fn fetch_phase_page(
        client: &Client,
        base_url: &str,
        closed: bool,
        after_cursor: Option<&str>,
        page_size: usize,
    ) -> Result<(Vec<serde_json::Value>, Option<String>), OpenPxError> {
        let mut url = format!(
            "{}/markets/keyset?limit={}&closed={}",
            base_url, page_size, closed
        );
        if let Some(c) = after_cursor.filter(|s| !s.is_empty()) {
            url.push_str(&format!("&after_cursor={c}"));
        }

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| OpenPxError::Exchange(PolymarketError::from(e).into()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(OpenPxError::Exchange(PolymarketError::Api(body).into()));
        }

        let mut body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(PolymarketError::from(e).into()))?;

        let markets = match body.get_mut("markets").map(serde_json::Value::take) {
            Some(serde_json::Value::Array(arr)) => arr,
            _ => Vec::new(),
        };
        let next_cursor = body
            .get("next_cursor")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok((markets, next_cursor))
    }
}

impl MarketFetcher for PolymarketMarketFetcher {
    fn exchange_id(&self) -> &'static str {
        "polymarket"
    }

    async fn fetch_markets(&self) -> Result<Vec<serde_json::Value>, OpenPxError> {
        let client = Self::create_client().map_err(|e| OpenPxError::Exchange(e.into()))?;
        let max_page_size = POLYMARKET_MANIFEST.pagination.max_page_size;

        let mut all_markets = Vec::new();

        for &(closed, label) in PHASES {
            let mut after: Option<String> = None;
            let mut iteration = 0;
            loop {
                let (mut markets, next) = Self::fetch_phase_page(
                    &client,
                    &self.base_url,
                    closed,
                    after.as_deref(),
                    max_page_size,
                )
                .await?;

                let count = markets.len();
                all_markets.append(&mut markets);

                info!(
                    exchange = "polymarket",
                    phase = label,
                    iteration,
                    page_count = count,
                    total = all_markets.len(),
                    "Fetched page"
                );

                iteration += 1;
                after = next;
                if after.is_none() {
                    break;
                }
            }
        }

        drop(client);

        Ok(all_markets)
    }

    fn extract_status(&self, raw: &serde_json::Value) -> String {
        let closed = raw.get("closed").and_then(|v| v.as_bool()).unwrap_or(false);
        let active = raw.get("active").and_then(|v| v.as_bool()).unwrap_or(true);

        if closed || !active {
            "resolved".to_string()
        } else {
            "active".to_string()
        }
    }

    async fn fetch_markets_with_checkpoints(
        &self,
        start_cursor: Option<String>,
        checkpoint_interval: usize,
        on_checkpoint: CheckpointCallback,
    ) -> Result<FetchResult, OpenPxError> {
        let client = Self::create_client().map_err(|e| OpenPxError::Exchange(e.into()))?;
        let max_page_size = POLYMARKET_MANIFEST.pagination.max_page_size;

        let mut state = PhaseCursors::parse(start_cursor.as_deref());
        let mut buffer: Vec<serde_json::Value> = Vec::new();
        let mut total_fetched = 0;

        for &(closed, label) in PHASES {
            // Skip phases the caller has already drained.
            let initial_after = match state.cursors.get(label) {
                Some(None) => continue,
                Some(Some(c)) => Some(c.clone()),
                None => None,
            };

            let mut after = initial_after;
            let mut iteration = 0;
            loop {
                let (mut markets, next) = Self::fetch_phase_page(
                    &client,
                    &self.base_url,
                    closed,
                    after.as_deref(),
                    max_page_size,
                )
                .await?;

                let count = markets.len();
                total_fetched += count;
                buffer.append(&mut markets);

                state.cursors.insert(label.to_string(), next.clone());
                after = next;

                info!(
                    exchange = "polymarket",
                    phase = label,
                    iteration,
                    page_count = count,
                    buffer_size = buffer.len(),
                    total = total_fetched,
                    "Fetched page"
                );
                iteration += 1;

                while buffer.len() >= checkpoint_interval {
                    let chunk: Vec<_> = buffer.drain(..checkpoint_interval).collect();
                    let cursor_str = state.serialize();
                    on_checkpoint(&chunk, &cursor_str).await?;
                    buffer.shrink_to_fit();
                }

                if after.is_none() {
                    break;
                }
            }
        }

        drop(client);
        buffer.shrink_to_fit();

        Ok(FetchResult {
            markets: buffer,
            final_cursor: Some(state.serialize()),
            total_fetched,
        })
    }
}
