use futures::stream::{FuturesUnordered, StreamExt};
use px_core::{
    manifests::POLYMARKET_MANIFEST, CheckpointCallback, FetchResult, MarketFetcher, OpenPxError,
};
use reqwest::Client;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::info;

use crate::error::PolymarketError;

/// Number of concurrent requests to maintain for optimal throughput.
/// With ~300-500ms latency and 30 req/s limit, we need ~15 concurrent requests.
const CONCURRENCY: usize = 15;

/// Type alias for boxed fetch future (stream_id, offset, markets)
type FetchFuture = Pin<
    Box<dyn Future<Output = Result<(usize, usize, Vec<serde_json::Value>), OpenPxError>> + Send>,
>;

/// Market fetcher for Polymarket Bronze layer ingestion.
/// Uses the Gamma API for public market data with concurrent fetching.
pub struct PolymarketMarketFetcher {
    base_url: String,
}

impl PolymarketMarketFetcher {
    pub fn new() -> Result<Self, PolymarketError> {
        Ok(Self {
            base_url: POLYMARKET_MANIFEST.base_url.to_string(),
        })
    }

    /// Create a new HTTP client for fetching.
    /// Uses the shared `px_core::http::tuned_client_builder()` and then
    /// overrides `pool_max_idle_per_host` to match our CONCURRENCY (15),
    /// since the fetcher opens more parallel streams than normal REST.
    fn create_client() -> Result<Client, PolymarketError> {
        px_core::http::tuned_client_builder()
            .pool_max_idle_per_host(CONCURRENCY)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(PolymarketError::from)
    }

    /// Fetch a single page of markets.
    async fn fetch_page(
        client: Client,
        base_url: String,
        offset: usize,
        page_size: usize,
        stream_id: usize,
    ) -> Result<(usize, usize, Vec<serde_json::Value>), OpenPxError> {
        info!(
            exchange = "polymarket",
            stream_id, offset, "Stream fetching page"
        );

        let url = format!("{}/markets?limit={}&offset={}", base_url, page_size, offset);

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

        let markets: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(PolymarketError::from(e).into()))?;

        Ok((stream_id, offset, markets))
    }

    /// Create a fetch task that respects the semaphore and done flag.
    fn create_fetch_task(
        client: Client,
        base_url: String,
        offset: usize,
        page_size: usize,
        semaphore: Arc<Semaphore>,
        done: Arc<AtomicBool>,
        stream_id: usize,
    ) -> FetchFuture {
        Box::pin(async move {
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|_| px_core::OpenPxError::Other("semaphore closed".into()))?;
            if done.load(Ordering::SeqCst) {
                return Ok((stream_id, offset, vec![]));
            }
            Self::fetch_page(client, base_url, offset, page_size, stream_id).await
        })
    }
}

impl MarketFetcher for PolymarketMarketFetcher {
    fn exchange_id(&self) -> &'static str {
        "polymarket"
    }

    async fn fetch_markets(&self) -> Result<Vec<serde_json::Value>, OpenPxError> {
        let client = Self::create_client().map_err(|e| OpenPxError::Exchange(e.into()))?;
        let max_page_size = POLYMARKET_MANIFEST.pagination.max_page_size;

        // Shared state for concurrent fetching
        let next_offset = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(AtomicBool::new(false));
        let semaphore = Arc::new(Semaphore::new(CONCURRENCY));

        // Results stored by offset for ordering
        let mut results: BTreeMap<usize, Vec<serde_json::Value>> = BTreeMap::new();
        let mut futures: FuturesUnordered<FetchFuture> = FuturesUnordered::new();

        // Start initial batch of concurrent requests
        for stream_id in 0..CONCURRENCY {
            let offset = next_offset.fetch_add(max_page_size, Ordering::SeqCst);
            futures.push(Self::create_fetch_task(
                client.clone(),
                self.base_url.clone(),
                offset,
                max_page_size,
                Arc::clone(&semaphore),
                Arc::clone(&done),
                stream_id,
            ));
        }

        let mut total_fetched = 0;
        let mut highest_complete_offset = 0;

        while let Some(result) = futures.next().await {
            let (stream_id, offset, markets) = result?;
            let count = markets.len();

            if count > 0 {
                results.insert(offset, markets);
                total_fetched += count;
            }

            // Check if this page signals end of data
            if count < max_page_size {
                done.store(true, Ordering::SeqCst);
            }

            // Queue next request if not done (reuse stream_id)
            if !done.load(Ordering::SeqCst) {
                let next = next_offset.fetch_add(max_page_size, Ordering::SeqCst);
                futures.push(Self::create_fetch_task(
                    client.clone(),
                    self.base_url.clone(),
                    next,
                    max_page_size,
                    Arc::clone(&semaphore),
                    Arc::clone(&done),
                    stream_id,
                ));
            }

            // Log progress periodically
            while results.contains_key(&highest_complete_offset) {
                let page_count = results
                    .get(&highest_complete_offset)
                    .map(|v| v.len())
                    .unwrap_or(0);
                info!(
                    exchange = "polymarket",
                    offset = highest_complete_offset,
                    page_count,
                    total = total_fetched,
                    "Fetched page"
                );
                highest_complete_offset += max_page_size;
            }
        }

        // Collect results in order
        let mut all_markets = Vec::with_capacity(total_fetched);
        for (_, markets) in results {
            all_markets.extend(markets);
        }

        // Client is dropped here, closing connections
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

        // Parse start offset from cursor
        let start_offset: usize = start_cursor
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Shared state for concurrent fetching
        let next_offset = Arc::new(AtomicUsize::new(start_offset));
        let done = Arc::new(AtomicBool::new(false));
        let semaphore = Arc::new(Semaphore::new(CONCURRENCY));

        // Results stored by offset for ordering
        let mut pending_results: BTreeMap<usize, Vec<serde_json::Value>> = BTreeMap::new();
        let mut futures: FuturesUnordered<FetchFuture> = FuturesUnordered::new();

        // Buffer for checkpointing - process in order
        let mut buffer: Vec<serde_json::Value> = Vec::new();
        let mut next_expected_offset = start_offset;
        let mut total_fetched = 0;

        // Start initial batch of concurrent requests
        for stream_id in 0..CONCURRENCY {
            let offset = next_offset.fetch_add(max_page_size, Ordering::SeqCst);
            futures.push(Self::create_fetch_task(
                client.clone(),
                self.base_url.clone(),
                offset,
                max_page_size,
                Arc::clone(&semaphore),
                Arc::clone(&done),
                stream_id,
            ));
        }

        while let Some(result) = futures.next().await {
            let (stream_id, offset, markets) = result?;
            let count = markets.len();

            if count > 0 {
                pending_results.insert(offset, markets);
            }

            // Check if this page signals end of data
            if count < max_page_size {
                done.store(true, Ordering::SeqCst);
            }

            // Queue next request if not done (reuse stream_id)
            if !done.load(Ordering::SeqCst) {
                let next = next_offset.fetch_add(max_page_size, Ordering::SeqCst);
                futures.push(Self::create_fetch_task(
                    client.clone(),
                    self.base_url.clone(),
                    next,
                    max_page_size,
                    Arc::clone(&semaphore),
                    Arc::clone(&done),
                    stream_id,
                ));
            }

            // Process results in order
            while let Some(markets) = pending_results.remove(&next_expected_offset) {
                let page_count = markets.len();
                total_fetched += page_count;

                info!(
                    exchange = "polymarket",
                    offset = next_expected_offset,
                    page_count,
                    buffer_size = buffer.len(),
                    total = total_fetched,
                    "Fetched page"
                );

                buffer.extend(markets);
                next_expected_offset += max_page_size;

                // Checkpoint if buffer exceeds interval
                while buffer.len() >= checkpoint_interval {
                    let checkpoint_data: Vec<_> = buffer.drain(..checkpoint_interval).collect();
                    let cursor_str = next_expected_offset.to_string();
                    on_checkpoint(&checkpoint_data, &cursor_str).await?;

                    // Explicitly release memory
                    buffer.shrink_to_fit();
                }
            }
        }

        // Get final offset for cursor
        let final_offset = next_expected_offset;

        // Drop client to close connections
        drop(client);

        // Shrink buffer to release unused memory
        buffer.shrink_to_fit();

        Ok(FetchResult {
            markets: buffer,
            final_cursor: Some(final_offset.to_string()),
            total_fetched,
        })
    }
}
