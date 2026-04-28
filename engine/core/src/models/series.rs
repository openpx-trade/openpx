use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A recurring family of events. Examples: a weekly inflation reading,
/// a monthly nonfarm payrolls release, a regular sports season. On Kalshi,
/// a Series is identified by `series_ticker` (e.g. `KXPRES`). On Polymarket,
/// a Series wraps multiple Events with shared metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Series {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub settlement_sources: Vec<SettlementSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SettlementSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
