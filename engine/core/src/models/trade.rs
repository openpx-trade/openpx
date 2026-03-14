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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    #[serde(default)]
    pub raw: serde_json::Value,
}

/// OHLCV candlestick, normalized across all exchanges.
/// Prices are decimals (0.0 to 1.0). Timestamp is the period START (not end).
/// Serialized over the wire as RFC3339 (DateTime<Utc>) for API consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Candlestick {
    /// Period start timestamp (UTC). lightweight-charts expects start-of-period.
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    /// Trade volume in contracts. 0.0 if exchange doesn't provide volume.
    pub volume: f64,
    /// Open interest at this candle's close. Only available from exchanges that report it (e.g., Kalshi).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PriceHistoryInterval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "6h")]
    SixHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "1w")]
    OneWeek,
    #[serde(rename = "max")]
    Max,
}

impl PriceHistoryInterval {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::OneHour => "1h",
            Self::SixHours => "6h",
            Self::OneDay => "1d",
            Self::OneWeek => "1w",
            Self::Max => "max",
        }
    }

    /// Approximate duration of one interval in seconds.
    pub fn seconds(&self) -> i64 {
        match self {
            Self::OneMinute => 60,
            Self::OneHour => 3600,
            Self::SixHours => 21600,
            Self::OneDay => 86400,
            Self::OneWeek => 604_800,
            Self::Max => 86400,
        }
    }
}

impl std::str::FromStr for PriceHistoryInterval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1m" => Ok(Self::OneMinute),
            "1h" => Ok(Self::OneHour),
            "6h" => Ok(Self::SixHours),
            "1d" => Ok(Self::OneDay),
            "1w" => Ok(Self::OneWeek),
            "max" => Ok(Self::Max),
            _ => Err(format!("unknown interval: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candlestick_omits_open_interest_when_none() {
        let c = Candlestick {
            timestamp: chrono::Utc::now(),
            open: 0.5,
            high: 0.6,
            low: 0.4,
            close: 0.55,
            volume: 100.0,
            open_interest: None,
        };
        let json = serde_json::to_value(&c).unwrap();
        assert!(json.get("open_interest").is_none());
    }

    #[test]
    fn candlestick_includes_open_interest_when_some() {
        let c = Candlestick {
            timestamp: chrono::Utc::now(),
            open: 0.5,
            high: 0.6,
            low: 0.4,
            close: 0.55,
            volume: 100.0,
            open_interest: Some(42000.0),
        };
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["open_interest"], 42000.0);
    }

    #[test]
    fn candlestick_roundtrip_with_open_interest() {
        let c = Candlestick {
            timestamp: chrono::Utc::now(),
            open: 0.5,
            high: 0.6,
            low: 0.4,
            close: 0.55,
            volume: 100.0,
            open_interest: Some(1234.0),
        };
        let serialized = serde_json::to_string(&c).unwrap();
        let deserialized: Candlestick = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.open_interest, Some(1234.0));
    }

    #[test]
    fn candlestick_deserialize_without_open_interest_defaults_none() {
        // Simulates old relay/exchange response without the field
        let json = r#"{"timestamp":"2026-01-01T00:00:00Z","open":0.5,"high":0.6,"low":0.4,"close":0.55,"volume":100.0}"#;
        let c: Candlestick = serde_json::from_str(json).unwrap();
        assert_eq!(c.open_interest, None);
    }
}
