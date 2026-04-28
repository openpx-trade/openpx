use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A grouping of related markets. On Kalshi, an Event is a single resolution
/// (e.g. "Will Candidate X win State Y?") with one or more contracts. On
/// Polymarket, an Event is the parent of one or more Markets sharing a common
/// theme (e.g. "2028 US Presidential Election" with markets per candidate).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Event {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default)]
    pub market_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_ts: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_ts: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutually_exclusive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_ts: Option<DateTime<Utc>>,
}
