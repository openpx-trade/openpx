use std::future::Future;
use std::pin::Pin;

use crate::OpenPxError;

/// Result of a checkpointed fetch operation.
#[derive(Debug)]
pub struct FetchResult {
    /// Remaining markets that weren't flushed to a checkpoint (< checkpoint_interval)
    pub markets: Vec<serde_json::Value>,
    /// Final cursor value for logging/debugging
    pub final_cursor: Option<String>,
    /// Total number of markets fetched across all pages
    pub total_fetched: usize,
}

/// Callback type for checkpoint operations.
/// Receives the batch of markets and the current cursor value.
pub type CheckpointCallback = Box<
    dyn Fn(
            &[serde_json::Value],
            &str,
        ) -> Pin<Box<dyn Future<Output = Result<(), OpenPxError>> + Send>>
        + Send
        + Sync,
>;

/// Trait for fetching raw market data from exchanges.
/// Used by the Bronze layer to collect complete API responses.
#[allow(async_fn_in_trait)]
pub trait MarketFetcher: Send + Sync {
    /// Exchange identifier (e.g., "kalshi", "polymarket")
    fn exchange_id(&self) -> &'static str;

    /// Fetch all markets as raw JSON values.
    /// Handles pagination internally and returns all available markets.
    async fn fetch_markets(&self) -> Result<Vec<serde_json::Value>, OpenPxError>;

    /// Fetch markets with checkpoint callback.
    /// Called every `checkpoint_interval` records with accumulated data.
    ///
    /// # Arguments
    /// * `start_cursor` - Optional cursor/offset to resume from
    /// * `checkpoint_interval` - Number of records before triggering checkpoint
    /// * `on_checkpoint` - Async callback invoked with batch data and cursor
    ///
    /// # Returns
    /// FetchResult containing remaining unflushed markets and metadata
    async fn fetch_markets_with_checkpoints(
        &self,
        start_cursor: Option<String>,
        checkpoint_interval: usize,
        on_checkpoint: CheckpointCallback,
    ) -> Result<FetchResult, OpenPxError>;

    /// Extract the exchange-specific status from a raw market JSON.
    fn extract_status(&self, raw: &serde_json::Value) -> String;
}
