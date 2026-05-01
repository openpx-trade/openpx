use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A grouping of related markets — one Kalshi event_ticker or one Polymarket event slug.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Event {
    /// Native event identifier — Kalshi event ticker or Polymarket event slug (e.g. `"KXPRES-2028"`).
    pub ticker: String,
    /// Polymarket numeric event id (e.g. `"12585"`); `null` on Kalshi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_id: Option<String>,
    /// Human-readable event title (e.g. `"2028 US Presidential Election"`).
    pub title: String,
    /// Long-form event description; `null` when upstream omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Topical category (e.g. `"Politics"`); `null` when upstream omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Parent series ticker (e.g. `"KXPRES"`); `null` when the event has no parent series.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    /// Upstream lifecycle string (e.g. `"open"`); `null` when upstream omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Tickers of markets under this event (e.g. `["KXBTCD-25APR1517"]`).
    #[serde(default)]
    pub market_tickers: Vec<String>,
    /// Event start time in UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_ts: Option<DateTime<Utc>>,
    /// Event end time in UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_ts: Option<DateTime<Utc>>,
    /// Lifetime trading volume in USD (e.g. `12345.67`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    /// Open interest in USD (e.g. `5000.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
    /// `true` if exactly one child market resolves YES.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutually_exclusive: Option<bool>,
    /// Last upstream update time in UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated_ts: Option<DateTime<Utc>>,
}
