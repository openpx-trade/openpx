use px_core::{
    manifests::POLYMARKET_MANIFEST, CheckpointCallback, FetchResult, MarketFetcher, OpenPxError,
};
use reqwest::Client;
use std::time::Duration;
use tracing::info;

use crate::error::PolymarketError;

/// Page size for the keyset endpoint. Polymarket caps `/markets/keyset` at
/// 1000 per request; we leave a margin to keep tail latency low.
const PAGE_SIZE: usize = 500;

/// Market fetcher for Polymarket Bronze layer ingestion.
///
/// Uses the gamma `/markets/keyset` endpoint with opaque cursor pagination
/// (the offset-paginated `/markets` endpoint is soft-deprecated as of the
/// 2026-04-10 changelog). The keyset endpoint serializes by definition, so
/// fan-out concurrency is intentionally absent.
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
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(PolymarketError::from)
    }

    /// Fetch one keyset page. Returns the parsed markets and the cursor for
    /// the next page (`None` when the stream is exhausted).
    async fn fetch_page(
        client: &Client,
        base_url: &str,
        cursor: Option<&str>,
        closed: bool,
    ) -> Result<(Vec<serde_json::Value>, Option<String>), OpenPxError> {
        let cursor_clause = match cursor {
            Some(c) if !c.is_empty() => format!("&after_cursor={c}"),
            _ => String::new(),
        };
        let url =
            format!("{base_url}/markets/keyset?limit={PAGE_SIZE}&closed={closed}{cursor_clause}",);

        info!(
            exchange = "polymarket",
            closed,
            ?cursor,
            "fetching keyset page"
        );

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

        let envelope: serde_json::Value = response
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(PolymarketError::from(e).into()))?;

        let markets = envelope
            .get("markets")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let next_cursor = envelope
            .get("next_cursor")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok((markets, next_cursor))
    }

    /// Drain every keyset page for a given `closed` filter.
    async fn drain_stream(
        client: &Client,
        base_url: &str,
        closed: bool,
    ) -> Result<Vec<serde_json::Value>, OpenPxError> {
        let mut cursor: Option<String> = None;
        let mut all = Vec::new();
        loop {
            let (page, next) =
                Self::fetch_page(client, base_url, cursor.as_deref(), closed).await?;
            let count = page.len();
            all.extend(page);
            info!(
                exchange = "polymarket",
                closed,
                page_count = count,
                total = all.len(),
                "drained page"
            );
            match next {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }
        Ok(all)
    }
}

impl MarketFetcher for PolymarketMarketFetcher {
    fn exchange_id(&self) -> &'static str {
        "polymarket"
    }

    async fn fetch_markets(&self) -> Result<Vec<serde_json::Value>, OpenPxError> {
        let client = Self::create_client().map_err(|e| OpenPxError::Exchange(e.into()))?;

        // The keyset endpoint defaults `closed=false` (per the 2026-04-09
        // changelog), so we drain both streams to retain the previous
        // "all markets" semantics.
        let mut all = Self::drain_stream(&client, &self.base_url, false).await?;
        let resolved = Self::drain_stream(&client, &self.base_url, true).await?;
        all.extend(resolved);

        drop(client);
        Ok(all)
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

        let mut cursor = start_cursor;
        let mut buffer: Vec<serde_json::Value> = Vec::new();
        let mut total_fetched = 0usize;

        loop {
            let (page, next) =
                Self::fetch_page(&client, &self.base_url, cursor.as_deref(), false).await?;
            let count = page.len();
            buffer.extend(page);
            total_fetched += count;
            cursor = next;

            info!(
                exchange = "polymarket",
                page_count = count,
                buffer_size = buffer.len(),
                total = total_fetched,
                ?cursor,
                "fetched keyset page"
            );

            while buffer.len() >= checkpoint_interval {
                let checkpoint_data: Vec<_> = buffer.drain(..checkpoint_interval).collect();
                let cursor_str = cursor.clone().unwrap_or_default();
                on_checkpoint(&checkpoint_data, &cursor_str).await?;
                buffer.shrink_to_fit();
            }

            if cursor.is_none() {
                break;
            }
        }

        drop(client);
        buffer.shrink_to_fit();

        Ok(FetchResult {
            markets: buffer,
            final_cursor: cursor,
            total_fetched,
        })
    }
}
