use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicTrade {
    pub proxy_wallet: String,
    pub side: String,
    pub asset: String,
    pub condition_id: String,
    pub size: f64,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pseudonym: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_image_optimized: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
}

/// A public trade off the market tape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarketTrade {
    /// Globally-unique exchange trade id (e.g. `"t-9c2..."`).
    pub id: String,
    /// Trade price as YES probability in `[0, 1]` (e.g. `0.62`).
    pub price: f64,
    /// Filled size in contracts (e.g. `25.0`).
    pub size: f64,
    /// Direction of the taker relative to YES. Options: `buy`, `sell`; `null` when unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggressor_side: Option<String>,
    /// Upstream trade time in UTC (e.g. `"2026-04-25T12:00:00Z"`).
    pub exchange_ts: DateTime<Utc>,
    /// Wall-clock time OpenPX served the trade (UTC).
    pub openpx_ts: DateTime<Utc>,
    /// Outcome label (e.g. `"Yes"`, `"No"`); `null` when not exposed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// YES-side reference price for binary markets (e.g. `0.62`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<f64>,
    /// NO-side reference price for binary markets (e.g. `0.38`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_price: Option<f64>,
    /// Polymarket taker wallet address (e.g. `"0x..."`); `null` on Kalshi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_address: Option<String>,
}
