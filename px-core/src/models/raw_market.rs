use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Raw market row for Bronze layer storage.
/// Stores the complete API JSON with minimal processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMarketRow {
    /// Exchange identifier (e.g., "kalshi", "polymarket")
    pub exchange: String,
    /// Exchange-specific status (raw, not normalized)
    pub status: String,
    /// Timestamp when data was collected
    pub collection_date: DateTime<Utc>,
    /// Complete raw JSON from the exchange API
    pub raw_json: String,
}

impl RawMarketRow {
    /// Create a new RawMarketRow from raw JSON data.
    pub fn new(exchange: &str, status: &str, raw_json: String) -> Self {
        Self {
            exchange: exchange.to_string(),
            status: status.to_string(),
            collection_date: Utc::now(),
            raw_json,
        }
    }
}
