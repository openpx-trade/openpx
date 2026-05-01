use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A recurring family of events (e.g. a weekly inflation reading or sports season).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Series {
    /// Native series identifier — Kalshi series ticker or Polymarket series ticker/slug (e.g. `"KXPRES"`).
    pub ticker: String,
    /// Polymarket numeric series id (e.g. `"10345"`); `null` on Kalshi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_id: Option<String>,
    /// Human-readable series title (e.g. `"US Presidential Election"`).
    pub title: String,
    /// Topical category (e.g. `"Politics"`); `null` when upstream omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Cadence string (e.g. `"weekly"`); `null` when upstream omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    /// Free-form tags (e.g. `["macro", "fed"]`).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Resolution sources used by the exchange (e.g. `[{"name": "BLS", "url": "..."}]`).
    #[serde(default)]
    pub settlement_sources: Vec<SettlementSource>,
    /// Fee schedule label (e.g. `"flat"`); `null` when upstream omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_type: Option<String>,
    /// Lifetime trading volume in USD across the series (e.g. `123456.78`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    /// Last upstream update time in UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_ts: Option<DateTime<Utc>>,
}

/// Reference used by the exchange to settle a series.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SettlementSource {
    /// Display name of the source (e.g. `"Bureau of Labor Statistics"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Source URL (e.g. `"https://www.bls.gov/cpi/"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
