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

/// Normalized public market trade, suitable for "tape" UIs.
///
/// - `price` is normalized to [0.0, 1.0]; anchored to the Yes side on
///   binary markets — use `no_price` for the No-side reference.
/// - `aggressor_side` is "buy" / "sell" relative to the Yes side on binary
///   markets — "buy" means upward pressure on Yes, "sell" downward.
/// - `exchange_ts` is the upstream-provided trade timestamp.
/// - `openpx_ts` is wall-clock when OpenPX served the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarketTrade {
    pub id: String,
    pub price: f64,
    pub size: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggressor_side: Option<String>,
    pub exchange_ts: DateTime<Utc>,
    pub openpx_ts: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_address: Option<String>,
}
