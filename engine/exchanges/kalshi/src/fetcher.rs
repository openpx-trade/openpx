use px_core::{
    manifests::KALSHI_MANIFEST, CheckpointCallback, ConcurrentRateLimiter, FetchResult,
    MarketFetcher, OpenPxError, RateLimiter,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::error::KalshiError;
use crate::exchange::to_openpx;

/// Conservative rate limit to avoid hitting Kalshi's longer-window limits.
/// Kalshi's API allows 20 req/s but may have per-minute/hour limits.
const CONSERVATIVE_RATE: u32 = 10;

/// Maximum retry attempts for rate limit errors.
const MAX_RATE_LIMIT_RETRIES: u32 = 3;

/// Initial backoff duration for rate limit retries.
const INITIAL_BACKOFF: Duration = Duration::from_secs(5);

/// All possible Kalshi market statuses for parallel fetching.
const KALSHI_STATUSES: &[&str] = &["unopened", "open", "paused", "closed", "settled"];

/// Per-status cursor state for resumable parallel fetching.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StatusCursors {
    cursors: HashMap<String, Option<String>>,
}

impl StatusCursors {
    fn parse(cursor_str: Option<&str>) -> Self {
        cursor_str
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    fn get(&self, status: &str) -> Option<String> {
        self.cursors.get(status).cloned().flatten()
    }

    fn set(&mut self, status: &str, cursor: Option<String>) {
        self.cursors.insert(status.to_string(), cursor);
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Market fetcher for Kalshi Bronze layer ingestion.
/// Uses parallel status-based fetching: 5 concurrent cursor streams (one per status)
/// to achieve ~5x throughput improvement over sequential fetching.
pub struct KalshiMarketFetcher {
    base_url: String,
}

impl KalshiMarketFetcher {
    pub fn new() -> Result<Self, KalshiError> {
        Ok(Self {
            base_url: KALSHI_MANIFEST.base_url.to_string(),
        })
    }

    /// Create a new HTTP client for fetching.
    fn create_client() -> Result<Client, KalshiError> {
        Client::builder()
            .http2_adaptive_window(true)
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(8)
            .http2_keep_alive_interval(Duration::from_secs(15))
            .build()
            .map_err(KalshiError::from)
    }
}

impl MarketFetcher for KalshiMarketFetcher {
    fn exchange_id(&self) -> &'static str {
        "kalshi"
    }

    async fn fetch_markets(&self) -> Result<Vec<serde_json::Value>, OpenPxError> {
        let client = Self::create_client().map_err(to_openpx)?;
        let mut rate_limiter = RateLimiter::new(KALSHI_MANIFEST.rate_limit.requests_per_second);

        let mut all_markets = Vec::new();
        let mut cursor: Option<String> = None;
        let max_page_size = KALSHI_MANIFEST.pagination.max_page_size;
        let mut iteration = 0;

        loop {
            // Rate limit before each request
            rate_limiter.wait().await;

            let url = match &cursor {
                Some(c) => format!(
                    "{}/markets?limit={}&cursor={}",
                    self.base_url, max_page_size, c
                ),
                None => format!("{}/markets?limit={}", self.base_url, max_page_size),
            };

            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| to_openpx(KalshiError::from(e)))?;

            let status = response.status();
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(OpenPxError::Exchange(KalshiError::Api(body).into()));
            }

            let mut body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| to_openpx(KalshiError::from(e)))?;

            // Take ownership of markets array instead of cloning
            let markets = match body.get_mut("markets").map(serde_json::Value::take) {
                Some(serde_json::Value::Array(arr)) => arr,
                _ => Vec::new(),
            };

            let count = markets.len();
            all_markets.extend(markets);

            info!(
                exchange = "kalshi",
                iteration,
                page_count = count,
                total = all_markets.len(),
                "Fetched page"
            );

            iteration += 1;

            // Check for more pages
            cursor = body
                .get("cursor")
                .and_then(|c| c.as_str())
                .filter(|c| !c.is_empty())
                .map(String::from);

            if cursor.is_none() || count < max_page_size {
                break;
            }
        }

        // Drop client to close connections
        drop(client);

        Ok(all_markets)
    }

    fn extract_status(&self, raw: &serde_json::Value) -> String {
        raw.get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("open")
            .to_string()
    }

    async fn fetch_markets_with_checkpoints(
        &self,
        start_cursor: Option<String>,
        checkpoint_interval: usize,
        on_checkpoint: CheckpointCallback,
    ) -> Result<FetchResult, OpenPxError> {
        let client = Self::create_client().map_err(to_openpx)?;

        // Parse per-status cursors from start_cursor (JSON format)
        let status_cursors = StatusCursors::parse(start_cursor.as_deref());

        // Shared state for parallel fetching
        // Use conservative rate to avoid hitting Kalshi's longer-window limits
        let rate_limiter = Arc::new(ConcurrentRateLimiter::new(
            CONSERVATIVE_RATE,
            KALSHI_STATUSES.len(),
        ));
        let buffer = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let total_fetched = Arc::new(AtomicUsize::new(0));
        let current_cursors = Arc::new(tokio::sync::Mutex::new(status_cursors.clone()));
        let on_checkpoint = Arc::new(on_checkpoint);
        let cancelled = Arc::new(AtomicBool::new(false));

        // Spawn one task per status
        let mut handles = Vec::new();
        for &status in KALSHI_STATUSES {
            let cursor = status_cursors.get(status);
            let handle = tokio::spawn(Self::fetch_status_stream(
                client.clone(),
                self.base_url.clone(),
                status.to_string(),
                cursor,
                Arc::clone(&rate_limiter),
                Arc::clone(&buffer),
                Arc::clone(&total_fetched),
                Arc::clone(&current_cursors),
                checkpoint_interval,
                Arc::clone(&on_checkpoint),
                Arc::clone(&cancelled),
            ));
            handles.push(handle);
        }

        // Wait for all status streams, abort all on first error
        let mut first_error: Option<OpenPxError> = None;
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    if first_error.is_none() {
                        cancelled.store(true, Ordering::SeqCst);
                        first_error = Some(e);
                    }
                }
                Err(e) => {
                    if first_error.is_none() {
                        cancelled.store(true, Ordering::SeqCst);
                        first_error = Some(OpenPxError::Other(format!("task join error: {}", e)));
                    }
                }
            }
        }

        // Drop client to close connections
        drop(client);

        // Return error if any stream failed
        if let Some(e) = first_error {
            return Err(e);
        }

        // Get remaining buffer and final cursor state
        let mut remaining = buffer.lock().await;
        remaining.shrink_to_fit();
        let markets = std::mem::take(&mut *remaining);
        drop(remaining);

        let final_cursor = current_cursors.lock().await.serialize();
        let total = total_fetched.load(Ordering::SeqCst);

        Ok(FetchResult {
            markets,
            final_cursor: Some(final_cursor),
            total_fetched: total,
        })
    }
}

impl KalshiMarketFetcher {
    /// Fetch a URL with retry for rate limit errors.
    async fn fetch_with_retry(
        client: &Client,
        url: &str,
        status: &str,
    ) -> Result<serde_json::Value, OpenPxError> {
        let mut backoff = INITIAL_BACKOFF;

        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|e| to_openpx(KalshiError::from(e)))?;

            let http_status = response.status();

            // Check for rate limit (429)
            if http_status == reqwest::StatusCode::TOO_MANY_REQUESTS
                && attempt < MAX_RATE_LIMIT_RETRIES
            {
                warn!(
                    exchange = "kalshi",
                    stream = %status,
                    attempt = attempt + 1,
                    backoff_secs = backoff.as_secs(),
                    "Rate limited, backing off"
                );
                sleep(backoff).await;
                backoff *= 2;
                continue;
            }

            if !http_status.is_success() {
                let body = response.text().await.unwrap_or_default();

                // Check for rate limit error in response body
                if body.contains("too_many_requests") && attempt < MAX_RATE_LIMIT_RETRIES {
                    warn!(
                        exchange = "kalshi",
                        stream = %status,
                        attempt = attempt + 1,
                        backoff_secs = backoff.as_secs(),
                        "Rate limited (body), backing off"
                    );
                    sleep(backoff).await;
                    backoff *= 2;
                    continue;
                }

                error!(
                    exchange = "kalshi",
                    stream = %status,
                    error = %body,
                    "API error"
                );
                return Err(OpenPxError::Exchange(KalshiError::Api(body).into()));
            }

            return response
                .json()
                .await
                .map_err(|e| to_openpx(KalshiError::from(e)));
        }

        Err(OpenPxError::Exchange(
            KalshiError::Api("Max rate limit retries exceeded".to_string()).into(),
        ))
    }

    /// Fetch all pages for a single status stream.
    #[allow(clippy::too_many_arguments)]
    async fn fetch_status_stream(
        client: Client,
        base_url: String,
        status: String,
        mut cursor: Option<String>,
        rate_limiter: Arc<ConcurrentRateLimiter>,
        buffer: Arc<tokio::sync::Mutex<Vec<serde_json::Value>>>,
        total_fetched: Arc<AtomicUsize>,
        current_cursors: Arc<tokio::sync::Mutex<StatusCursors>>,
        checkpoint_interval: usize,
        on_checkpoint: Arc<CheckpointCallback>,
        cancelled: Arc<AtomicBool>,
    ) -> Result<(), OpenPxError> {
        let max_page_size = KALSHI_MANIFEST.pagination.max_page_size;
        let mut iteration = 0;

        info!(
            exchange = "kalshi",
            stream = %status,
            "Starting stream"
        );

        loop {
            // Check for cancellation before making request
            if cancelled.load(Ordering::SeqCst) {
                info!(
                    exchange = "kalshi",
                    stream = %status,
                    "Stream cancelled"
                );
                return Ok(());
            }

            // Acquire rate limit permit (blocks until rate limit allows)
            let _permit = rate_limiter.acquire().await;

            // Build URL with status filter
            let url = match &cursor {
                Some(c) => format!(
                    "{}/markets?status={}&limit={}&cursor={}",
                    base_url, status, max_page_size, c
                ),
                None => format!(
                    "{}/markets?status={}&limit={}",
                    base_url, status, max_page_size
                ),
            };

            // Fetch with retry for rate limit errors
            let mut body = Self::fetch_with_retry(&client, &url, &status).await?;

            // Take ownership of markets array instead of cloning
            let markets = match body.get_mut("markets").map(serde_json::Value::take) {
                Some(serde_json::Value::Array(arr)) => arr,
                _ => Vec::new(),
            };

            let count = markets.len();
            total_fetched.fetch_add(count, Ordering::SeqCst);

            // Update cursor for next page
            cursor = body
                .get("cursor")
                .and_then(|c| c.as_str())
                .filter(|c| !c.is_empty())
                .map(String::from);

            // Update current cursor state for checkpointing
            current_cursors.lock().await.set(&status, cursor.clone());

            info!(
                exchange = "kalshi",
                status = %status,
                iteration,
                page_count = count,
                total = total_fetched.load(Ordering::SeqCst),
                "Fetched page"
            );

            iteration += 1;

            // Add to shared buffer and checkpoint if needed
            {
                let mut buf = buffer.lock().await;
                buf.extend(markets);

                // Checkpoint if buffer exceeds interval
                while buf.len() >= checkpoint_interval {
                    let checkpoint_data: Vec<_> = buf.drain(..checkpoint_interval).collect();
                    let cursor_str = current_cursors.lock().await.serialize();
                    on_checkpoint(&checkpoint_data, &cursor_str).await?;
                    buf.shrink_to_fit();
                }
            }

            // Done if no more pages
            if cursor.is_none() || count < max_page_size {
                break;
            }
        }

        info!(
            exchange = "kalshi",
            stream = %status,
            iteration,
            "Stream completed"
        );

        Ok(())
    }
}
