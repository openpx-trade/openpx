use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::OrderSide;

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
/// - `price` is normalized to [0.0, 1.0] across all exchanges.
/// - `timestamp` is the exchange-provided trade timestamp (UTC).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarketTrade {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub price: f64,
    pub size: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggressor_side: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub source_channel: std::borrow::Cow<'static, str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_address: Option<String>,
}

/// Last public trade price + side + size for a market outcome. Distinct from
/// the full `MarketTrade` tape: this is just "what just printed?" — common UI
/// need that doesn't require the full trade history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LastTrade {
    pub price: f64,
    pub side: OrderSide,
    pub size: f64,
    pub ts_ms: i64,
}
