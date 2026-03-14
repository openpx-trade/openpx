use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prediction market.
///
/// # Price Format
///
/// All prices in the `prices` field are normalized to decimal format (0.0 to 1.0).
/// Exchange-specific conversions are handled during parsing:
///
/// - **Kalshi**: Native prices in cents (1-99), converted to decimal by dividing by 100.
/// - **Polymarket, Opinion**: Native prices already in decimal (0.0-1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Market {
    pub id: String,
    pub question: String,
    pub outcomes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<DateTime<Utc>>,
    pub volume: f64,
    pub liquidity: f64,
    /// Outcome prices normalized to decimal format (0.0 to 1.0).
    pub prices: HashMap<String, f64>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub tick_size: f64,
    #[serde(default)]
    pub description: String,
}

impl Market {
    pub fn is_binary(&self) -> bool {
        self.outcomes.len() == 2
    }

    pub fn is_open(&self) -> bool {
        if let Some(metadata) = self.metadata.as_object() {
            if let Some(closed) = metadata.get("closed").and_then(|v| v.as_bool()) {
                return !closed;
            }
        }

        match self.close_time {
            Some(close_time) => Utc::now() < close_time,
            None => true,
        }
    }

    pub fn spread(&self) -> Option<f64> {
        if !self.is_binary() || self.outcomes.len() != 2 {
            return None;
        }

        let prices: Vec<f64> = self.prices.values().copied().collect();
        if prices.len() != 2 {
            return None;
        }

        Some((1.0 - prices.iter().sum::<f64>()).abs())
    }

    pub fn get_token_ids(&self) -> Vec<String> {
        let token_ids = self.metadata.get("clobTokenIds");

        match token_ids {
            Some(serde_json::Value::String(s)) => serde_json::from_str(s).unwrap_or_default(),
            Some(serde_json::Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => vec![],
        }
    }

    pub fn get_outcome_tokens(&self) -> Vec<OutcomeToken> {
        let token_ids = self.get_token_ids();
        self.outcomes
            .iter()
            .enumerate()
            .map(|(i, outcome)| OutcomeToken {
                outcome: outcome.clone(),
                token_id: token_ids.get(i).cloned().unwrap_or_default(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OutcomeToken {
    pub outcome: String,
    pub token_id: String,
}

/// Normalized market status across all exchanges.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    Active,
    Closed,
    Resolved,
}

impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatus::Active => write!(f, "active"),
            MarketStatus::Closed => write!(f, "closed"),
            MarketStatus::Resolved => write!(f, "resolved"),
        }
    }
}

impl std::str::FromStr for MarketStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" | "open" => Ok(MarketStatus::Active),
            "closed" => Ok(MarketStatus::Closed),
            "resolved" | "settled" => Ok(MarketStatus::Resolved),
            _ => Err(format!("Unknown market status: {}", s)),
        }
    }
}

/// Unified market model for the Data Product API.
/// Follows strict normalization with explicit nullable fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UnifiedMarket {
    /// Primary key: {exchange}:{native_id}
    pub openpx_id: String,
    /// Exchange identifier (kalshi, polymarket, etc.)
    pub exchange: String,
    /// Source-native event/group ID from the exchange.
    /// Keep this raw so advanced users can reason about exchange internals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    /// Canonical OpenPX event ID used for cross-exchange event grouping.
    /// SDK users should prefer this over exchange-specific `group_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    /// Native exchange market ID
    pub id: String,
    /// URL-friendly identifier (nullable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Market title
    pub title: String,
    /// Market question (nullable, may differ from title)
    pub question: Option<String>,
    /// Full description/rules
    pub description: String,
    /// Normalized status: Active, Closed, Resolved
    pub status: MarketStatus,
    /// Market type (binary, categorical, etc.)
    pub market_type: String,
    /// Yes outcome token ID (nullable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id_yes: Option<String>,
    /// No outcome token ID (nullable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id_no: Option<String>,
    /// Condition ID for CTF (nullable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<String>,
    /// Total volume (integer, coerced from f64/string)
    pub volume: i64,
    /// Current liquidity (nullable)
    pub liquidity: Option<i64>,
    /// Market close time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_time: Option<DateTime<Utc>>,
    /// Market open time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_time: Option<DateTime<Utc>>,
    /// Outcome labels (e.g., ["Yes", "No"] for binary markets)
    #[serde(default)]
    pub outcomes: Vec<String>,
    /// Outcome-to-token mapping for orderbook subscriptions
    #[serde(default)]
    pub outcome_tokens: Vec<OutcomeToken>,
    /// Outcome prices from the REST API (e.g., {"Yes": 0.65, "No": 0.35})
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outcome_prices: HashMap<String, f64>,
    /// 24-hour trading volume (USDC)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_24h: Option<i64>,
    /// 7-day rolling trading volume (USDC)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_1wk: Option<i64>,
    /// 30-day rolling trading volume (USDC)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_1mo: Option<i64>,
    /// Current open interest (contracts/pairs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
    /// 24-hour YES price change (decimal, e.g. 0.05 = +5%)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1d: Option<f64>,
    /// 1-hour YES price change (decimal, e.g. -0.02 = -2%)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1h: Option<f64>,
    /// 7-day YES price change (decimal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1wk: Option<f64>,
    /// 30-day YES price change (decimal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1mo: Option<f64>,
    /// Last trade price (normalized 0-1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_trade_price: Option<f64>,
    /// Best bid price (normalized 0-1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_bid: Option<f64>,
    /// Best ask price (normalized 0-1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_ask: Option<f64>,
    /// Bid-ask spread (decimal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread: Option<f64>,
    /// Minimum order size (contracts). Exchange-specific:
    /// Polymarket varies per market (e.g. 5, 15); Kalshi defaults to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_order_size: Option<f64>,
    /// Tick size (minimum price increment). Normalized to decimal (e.g. 0.01 = 1 cent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<f64>,
    /// Market image URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Market icon URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

impl UnifiedMarket {
    /// Create openpx_id from exchange and native id
    pub fn make_openpx_id(exchange: &str, id: &str) -> String {
        format!("{}:{}", exchange, id)
    }

    /// Parse openpx_id into (exchange, native_id)
    pub fn parse_openpx_id(openpx_id: &str) -> Option<(&str, &str)> {
        let (exchange, id) = openpx_id.split_once(':')?;
        if exchange.is_empty() || id.is_empty() {
            return None;
        }
        Some((exchange, id))
    }

    /// Check if market matches search query (case-insensitive)
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.title.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self
                .question
                .as_ref()
                .is_some_and(|q| q.to_lowercase().contains(&query_lower))
    }
}

#[cfg(test)]
mod tests {
    use super::UnifiedMarket;

    #[test]
    fn parse_openpx_id_valid() {
        let parsed = UnifiedMarket::parse_openpx_id("kalshi:TICKER-123");
        assert_eq!(parsed, Some(("kalshi", "TICKER-123")));
    }

    #[test]
    fn parse_openpx_id_invalid() {
        assert_eq!(UnifiedMarket::parse_openpx_id("invalid"), None);
        assert_eq!(UnifiedMarket::parse_openpx_id("kalshi:"), None);
        assert_eq!(UnifiedMarket::parse_openpx_id(":TICKER"), None);
        assert_eq!(UnifiedMarket::parse_openpx_id(""), None);
    }

    #[test]
    fn volume_1wk_omitted_when_none() {
        let market = UnifiedMarket {
            openpx_id: "test:1".into(),
            exchange: "test".into(),
            group_id: None,
            event_id: None,
            id: "1".into(),
            slug: None,
            title: "Test".into(),
            question: None,
            description: String::new(),
            status: super::MarketStatus::Active,
            market_type: "binary".into(),
            token_id_yes: None,
            token_id_no: None,
            condition_id: None,
            volume: 0,
            liquidity: None,
            close_time: None,
            open_time: None,
            outcomes: vec![],
            outcome_tokens: vec![],
            outcome_prices: std::collections::HashMap::new(),
            volume_24h: None,
            volume_1wk: None,
            volume_1mo: None,
            open_interest: None,
            price_change_1d: None,
            price_change_1h: None,
            price_change_1wk: None,
            price_change_1mo: None,
            last_trade_price: None,
            best_bid: None,
            best_ask: None,
            spread: None,
            min_order_size: None,
            tick_size: None,
            image_url: None,
            icon_url: None,
        };
        let json = serde_json::to_value(&market).unwrap();
        assert!(json.get("volume_1wk").is_none());
        assert!(json.get("volume_24h").is_none());
        assert!(json.get("volume_1mo").is_none());
        assert!(json.get("min_order_size").is_none());
    }

    // TODO(fee-rate): Add fee_rate (basis points) to market data responses. Pro traders need
    // fee rates for accurate PnL calculations and cost-optimal routing between exchanges.
    // polyfill-rs has get_fee_rate_bps(token_id) returning the maker fee rate.
    // Implementation: add fee_rate_bps field alongside tick_size in the market data pipeline.
    // Note: fee rates may vary per user tier on some exchanges, so document as "base fee rate."

    #[test]
    fn volume_1wk_present_when_some() {
        let market = UnifiedMarket {
            openpx_id: "test:1".into(),
            exchange: "test".into(),
            group_id: None,
            event_id: None,
            id: "1".into(),
            slug: None,
            title: "Test".into(),
            question: None,
            description: String::new(),
            status: super::MarketStatus::Active,
            market_type: "binary".into(),
            token_id_yes: None,
            token_id_no: None,
            condition_id: None,
            volume: 0,
            liquidity: None,
            close_time: None,
            open_time: None,
            outcomes: vec![],
            outcome_tokens: vec![],
            outcome_prices: std::collections::HashMap::new(),
            volume_24h: Some(1000),
            volume_1wk: Some(7000),
            volume_1mo: Some(30000),
            open_interest: None,
            price_change_1d: None,
            price_change_1h: None,
            price_change_1wk: None,
            price_change_1mo: None,
            last_trade_price: None,
            best_bid: None,
            best_ask: None,
            spread: None,
            min_order_size: Some(15.0),
            tick_size: None,
            image_url: None,
            icon_url: None,
        };
        let json = serde_json::to_value(&market).unwrap();
        assert_eq!(json["volume_24h"], 1000);
        assert_eq!(json["volume_1wk"], 7000);
        assert_eq!(json["volume_1mo"], 30000);
        assert_eq!(json["min_order_size"], 15.0);
    }
}
