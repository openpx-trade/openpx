use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A grouping of related markets. On Kalshi, an Event is a single resolution
/// (e.g. "Will Candidate X win State Y?") with one or more contracts. On
/// Polymarket, an Event is the parent of one or more Markets sharing a common
/// theme (e.g. "2028 US Presidential Election" with markets per candidate).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Event {
    /// Native event identifier. Kalshi: `event_ticker` (e.g. `"KXPRES-2028"`).
    /// Polymarket: the event slug (e.g. `"presidential-election-winner-2024"`).
    pub ticker: String,
    /// Polymarket's numeric REST id for the event (e.g. `"12585"`). None on
    /// Kalshi (no separate numeric event surface).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_id: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Parent series ticker. Kalshi: upstream `series_ticker`. Polymarket: the
    /// embedded series's `ticker` (or `slug` if `ticker` is null).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_ticker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Sibling market market_tickers under this event — unified `Market.ticker`
    /// values (Kalshi market market_tickers; Polymarket slugs).
    #[serde(default)]
    pub market_tickers: Vec<String>,
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
