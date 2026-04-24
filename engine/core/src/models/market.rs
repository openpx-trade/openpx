use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Market type classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MarketType {
    Binary,
    Categorical,
    Scalar,
}

impl std::fmt::Display for MarketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketType::Binary => write!(f, "binary"),
            MarketType::Categorical => write!(f, "categorical"),
            MarketType::Scalar => write!(f, "scalar"),
        }
    }
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
            "closed" | "initialized" | "inactive" | "paused" | "unopened" | "disputed"
            | "amended" => Ok(MarketStatus::Closed),
            "resolved" | "settled" | "determined" | "finalized" => Ok(MarketStatus::Resolved),
            _ => Err(format!("Unknown market status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OutcomeToken {
    pub outcome: String,
    pub token_id: String,
}

/// Unified prediction market model.
///
/// All exchanges produce this single type directly — no intermediate conversion.
///
/// # Price Format
///
/// All prices are normalized to decimal format (0.0 to 1.0).
/// Exchange-specific conversions are handled during parsing:
///
/// - **Kalshi**: Fixed-point dollar strings parsed directly (post March 2026 migration).
/// - **Polymarket, Opinion**: Native prices already in decimal (0.0-1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Market {
    // ── Identity ──────────────────────────────────────────────────────────
    /// Primary key: {exchange}:{native_id}
    pub openpx_id: String,
    /// Exchange identifier (kalshi, polymarket, opinion)
    pub exchange: String,
    /// Native exchange market ID
    pub id: String,
    /// Source-native event/group ID from the exchange.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    /// Canonical OpenPX event ID for cross-exchange event grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,

    // ── Display ───────────────────────────────────────────────────────────
    /// Market title
    pub title: String,
    /// Market question (may differ from title)
    pub question: Option<String>,
    /// Full description
    pub description: String,
    /// URL-friendly identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Resolution rules
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,

    // ── Status ─────────────────────────────────────────────────────────────
    /// Normalized status: Active, Closed, Resolved
    pub status: MarketStatus,
    /// Market type classification
    pub market_type: MarketType,
    /// Whether the market is currently accepting orders
    #[serde(default)]
    pub accepting_orders: bool,

    // ── Outcomes ───────────────────────────────────────────────────────────
    /// Outcome labels (e.g., ["Yes", "No"] for binary markets)
    #[serde(default)]
    pub outcomes: Vec<String>,
    /// Outcome-to-token mapping for orderbook subscriptions
    #[serde(default)]
    pub outcome_tokens: Vec<OutcomeToken>,
    /// Outcome prices from the REST API (e.g., {"Yes": 0.65, "No": 0.35})
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outcome_prices: HashMap<String, f64>,

    // ── Token / CTF ───────────────────────────────────────────────────────
    /// Yes outcome token ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id_yes: Option<String>,
    /// No outcome token ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id_no: Option<String>,
    /// Condition ID for CTF
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<String>,
    /// Question ID (Opinion, Polymarket)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question_id: Option<String>,
    /// Polymarket's numeric DB id (e.g. "1031769"). Exposed for callers that
    /// need to build UI deep-links or cross-reference Polymarket's REST-only
    /// numeric surface. Not used for trading or subscription — `id` (the
    /// condition_id on Polymarket) is the canonical identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_numeric_id: Option<String>,

    // ── Volume ─────────────────────────────────────────────────────────────
    /// Total volume (USD)
    pub volume: f64,
    /// 24-hour trading volume (USD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_24h: Option<f64>,
    /// 7-day rolling trading volume (USD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_1wk: Option<f64>,
    /// 30-day rolling trading volume (USD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_1mo: Option<f64>,

    // ── Pricing / Liquidity ────────────────────────────────────────────────
    /// Current liquidity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liquidity: Option<f64>,
    /// Current open interest
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<f64>,
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

    // ── Price Changes ──────────────────────────────────────────────────────
    /// 24-hour YES price change (decimal, e.g. 0.05 = +5%)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1d: Option<f64>,
    /// 1-hour YES price change
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1h: Option<f64>,
    /// 7-day YES price change
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1wk: Option<f64>,
    /// 30-day YES price change
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_change_1mo: Option<f64>,

    // ── Trading Params ─────────────────────────────────────────────────────
    /// Tick size (minimum price increment, normalized decimal e.g. 0.01)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<f64>,
    /// Minimum order size (contracts)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_order_size: Option<f64>,

    // ── Time ───────────────────────────────────────────────────────────────
    /// Market close time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_time: Option<DateTime<Utc>>,
    /// Market open time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_time: Option<DateTime<Utc>>,
    /// Market creation time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// Settlement / resolution time
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_time: Option<DateTime<Utc>>,

    // ── Media ──────────────────────────────────────────────────────────────
    /// Market image URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Market icon URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,

    // ── Exchange-Specific ──────────────────────────────────────────────────
    /// Polymarket: neg-risk flag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<bool>,
    /// Polymarket: neg-risk market ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neg_risk_market_id: Option<String>,
    /// Maker fee rate (basis points)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maker_fee_bps: Option<f64>,
    /// Taker fee rate (basis points)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taker_fee_bps: Option<f64>,
    /// Denomination token (e.g. USDC address)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denomination_token: Option<String>,
    /// Chain ID for on-chain markets
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
    /// Notional value per contract (Kalshi)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notional_value: Option<f64>,
    /// Kalshi sub-penny pricing structure
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_level_structure: Option<String>,
    /// Kalshi: settlement value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_value: Option<f64>,
    /// Kalshi: previous price
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_price: Option<f64>,
    /// Kalshi: can close early
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_close_early: Option<bool>,
    /// Resolution result
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl Market {
    /// Create openpx_id from exchange and native id
    #[inline]
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

    #[inline]
    pub fn is_binary(&self) -> bool {
        self.outcomes.len() == 2
    }

    #[inline]
    pub fn is_open(&self) -> bool {
        if self.status != MarketStatus::Active {
            return false;
        }
        match self.close_time {
            Some(close_time) => Utc::now() < close_time,
            None => true,
        }
    }

    /// Compute bid-ask spread from outcome prices for binary markets.
    pub fn computed_spread(&self) -> Option<f64> {
        if let Some(s) = self.spread {
            return Some(s);
        }
        if let (Some(bid), Some(ask)) = (self.best_bid, self.best_ask) {
            return Some(ask - bid);
        }
        if !self.is_binary() || self.outcome_prices.len() != 2 {
            return None;
        }
        Some((1.0 - self.outcome_prices.values().copied().sum::<f64>()).abs())
    }

    pub fn get_token_ids(&self) -> Vec<String> {
        if !self.outcome_tokens.is_empty() {
            return self
                .outcome_tokens
                .iter()
                .map(|t| t.token_id.clone())
                .collect();
        }
        let mut ids = Vec::new();
        if let Some(ref id) = self.token_id_yes {
            ids.push(id.clone());
        }
        if let Some(ref id) = self.token_id_no {
            ids.push(id.clone());
        }
        ids
    }

    pub fn get_outcome_tokens(&self) -> Vec<OutcomeToken> {
        if !self.outcome_tokens.is_empty() {
            return self.outcome_tokens.clone();
        }
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

impl Default for Market {
    fn default() -> Self {
        Self {
            openpx_id: String::new(),
            exchange: String::new(),
            id: String::new(),
            group_id: None,
            event_id: None,
            title: String::new(),
            question: None,
            description: String::new(),
            slug: None,
            rules: None,
            status: MarketStatus::Active,
            market_type: MarketType::Binary,
            accepting_orders: true,
            outcomes: vec![],
            outcome_tokens: vec![],
            outcome_prices: HashMap::new(),
            token_id_yes: None,
            token_id_no: None,
            condition_id: None,
            question_id: None,
            native_numeric_id: None,
            volume: 0.0,
            volume_24h: None,
            volume_1wk: None,
            volume_1mo: None,
            liquidity: None,
            open_interest: None,
            last_trade_price: None,
            best_bid: None,
            best_ask: None,
            spread: None,
            price_change_1d: None,
            price_change_1h: None,
            price_change_1wk: None,
            price_change_1mo: None,
            tick_size: None,
            min_order_size: None,
            close_time: None,
            open_time: None,
            created_at: None,
            settlement_time: None,
            image_url: None,
            icon_url: None,
            neg_risk: None,
            neg_risk_market_id: None,
            maker_fee_bps: None,
            taker_fee_bps: None,
            denomination_token: None,
            chain_id: None,
            notional_value: None,
            price_level_structure: None,
            settlement_value: None,
            previous_price: None,
            can_close_early: None,
            result: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_openpx_id_valid() {
        let parsed = Market::parse_openpx_id("kalshi:TICKER-123");
        assert_eq!(parsed, Some(("kalshi", "TICKER-123")));
    }

    #[test]
    fn parse_openpx_id_invalid() {
        assert_eq!(Market::parse_openpx_id("invalid"), None);
        assert_eq!(Market::parse_openpx_id("kalshi:"), None);
        assert_eq!(Market::parse_openpx_id(":TICKER"), None);
        assert_eq!(Market::parse_openpx_id(""), None);
    }

    #[test]
    fn optional_fields_omitted_when_none() {
        let market = Market {
            openpx_id: "test:1".into(),
            exchange: "test".into(),
            id: "1".into(),
            title: "Test".into(),
            ..Default::default()
        };
        let json = serde_json::to_value(&market).unwrap();
        assert!(json.get("volume_1wk").is_none());
        assert!(json.get("volume_24h").is_none());
        assert!(json.get("volume_1mo").is_none());
        assert!(json.get("min_order_size").is_none());
    }

    // TODO(fee-rate): Add fee_rate (basis points) to market data responses. Pro traders need
    // fee rates for accurate PnL calculations and cost-optimal routing between exchanges.
    // Per-token `get_fee_rate_bps(token_id)` returning the maker fee rate.
    // Implementation: add fee_rate_bps field alongside tick_size in the market data pipeline.
    // Note: fee rates may vary per user tier on some exchanges, so document as "base fee rate."

    #[test]
    fn optional_fields_present_when_some() {
        let market = Market {
            openpx_id: "test:1".into(),
            exchange: "test".into(),
            id: "1".into(),
            title: "Test".into(),
            volume_24h: Some(1000.0),
            volume_1wk: Some(7000.0),
            volume_1mo: Some(30000.0),
            min_order_size: Some(15.0),
            ..Default::default()
        };
        let json = serde_json::to_value(&market).unwrap();
        assert_eq!(json["volume_24h"], 1000.0);
        assert_eq!(json["volume_1wk"], 7000.0);
        assert_eq!(json["volume_1mo"], 30000.0);
        assert_eq!(json["min_order_size"], 15.0);
    }

    #[test]
    fn matches_search_title() {
        let market = Market {
            title: "Will Bitcoin reach $100k?".into(),
            ..Default::default()
        };
        assert!(market.matches_search("bitcoin"));
        assert!(market.matches_search("100k"));
        assert!(!market.matches_search("ethereum"));
    }

    #[test]
    fn get_token_ids_from_outcome_tokens() {
        let market = Market {
            outcome_tokens: vec![
                OutcomeToken {
                    outcome: "Yes".into(),
                    token_id: "tok1".into(),
                },
                OutcomeToken {
                    outcome: "No".into(),
                    token_id: "tok2".into(),
                },
            ],
            ..Default::default()
        };
        assert_eq!(market.get_token_ids(), vec!["tok1", "tok2"]);
    }

    #[test]
    fn get_token_ids_from_yes_no_fields() {
        let market = Market {
            token_id_yes: Some("yes_tok".into()),
            token_id_no: Some("no_tok".into()),
            ..Default::default()
        };
        assert_eq!(market.get_token_ids(), vec!["yes_tok", "no_tok"]);
    }

    #[test]
    fn is_binary_and_is_open() {
        let market = Market {
            outcomes: vec!["Yes".into(), "No".into()],
            status: MarketStatus::Active,
            ..Default::default()
        };
        assert!(market.is_binary());
        assert!(market.is_open());

        let closed = Market {
            outcomes: vec!["Yes".into(), "No".into()],
            status: MarketStatus::Closed,
            ..Default::default()
        };
        assert!(!closed.is_open());
    }

    #[test]
    fn market_type_serialization() {
        let market = Market {
            openpx_id: "test:1".into(),
            exchange: "test".into(),
            id: "1".into(),
            title: "Test".into(),
            market_type: MarketType::Categorical,
            ..Default::default()
        };
        let json = serde_json::to_value(&market).unwrap();
        assert_eq!(json["market_type"], "categorical");
    }
}
