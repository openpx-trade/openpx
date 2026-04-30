use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// One outcome of a prediction market — label, current price, and (where the
/// exchange exposes one) the on-chain token id needed to subscribe to its
/// orderbook stream. Replaces the previous trio of `outcome_prices` map +
/// `outcome_tokens` vec + binary-only `token_id_yes`/`token_id_no` with a
/// single ordered list keyed by index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Outcome {
    /// Outcome label (e.g. "Yes", "No", or a categorical option name).
    pub label: String,
    /// Current outcome price in [0.0, 1.0]. None when not exposed (e.g. Kalshi
    /// markets pre-fill, or Polymarket categorical markets without a price).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    /// CTF token id (Polymarket only). Used to subscribe to per-outcome
    /// orderbook streams. None on Kalshi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
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
/// - **Polymarket**: Native prices already in decimal (0.0-1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Market {
    // ── Identity ──────────────────────────────────────────────────────────
    /// Primary key: {exchange}:{ticker}
    pub openpx_id: String,
    /// Exchange identifier (kalshi, polymarket)
    pub exchange: String,
    /// Native exchange ticker. Kalshi: `ticker`. Polymarket: `slug`.
    pub ticker: String,
    /// Native event identifier this market belongs to. Kalshi: upstream
    /// `event_ticker`. Polymarket: the parent event's `slug`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_ticker: Option<String>,
    /// Polymarket's numeric DB id (e.g. "1031769"). Exposed for callers that
    /// need to build UI deep-links or cross-reference Polymarket's REST-only
    /// numeric surface. Not used for trading or subscription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_id: Option<String>,

    // ── Display ───────────────────────────────────────────────────────────
    /// Market title
    pub title: String,
    /// Resolution rules. Kalshi: `{rules_primary} | {rules_secondary}`.
    /// Polymarket: `description`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,

    // ── Status ─────────────────────────────────────────────────────────────
    /// Normalized status: Active, Closed, Resolved
    pub status: MarketStatus,
    /// Market type classification
    pub market_type: MarketType,

    // ── Outcomes ───────────────────────────────────────────────────────────
    /// Ordered list of outcomes. Each entry carries label, price, and (if
    /// exposed) token id. Polymarket categorical markets have N entries;
    /// Kalshi binary markets always have 2 ("Yes" / "No").
    #[serde(default)]
    pub outcomes: Vec<Outcome>,

    // ── CTF ───────────────────────────────────────────────────────────────
    /// Polymarket CTF condition id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<String>,

    // ── Volume ─────────────────────────────────────────────────────────────
    /// Total volume (USD)
    pub volume: f64,
    /// 24-hour trading volume (USD)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_24h: Option<f64>,

    // ── Pricing ────────────────────────────────────────────────────────────
    /// Last trade price (normalized 0-1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_trade_price: Option<f64>,
    /// Best bid price (normalized 0-1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_bid: Option<f64>,
    /// Best ask price (normalized 0-1)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_ask: Option<f64>,

    // ── Trading Params ─────────────────────────────────────────────────────
    /// Minimum price increment in dollars. Polymarket:
    /// `orderPriceMinTickSize` directly. Kalshi: the smallest `step` across
    /// the tiered `price_ranges` array (the finest precision the market
    /// accepts anywhere on its price curve — orders in coarser tiers must
    /// still round to that tier's step).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<f64>,
    /// Minimum order size (contracts).
    /// Kalshi: synthesized — 1.0, or 0.01 when fractional_trading_enabled.
    /// Polymarket: read directly from `orderMinSize`.
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
    /// Settlement / resolution time. Kalshi: `settlement_ts`. Polymarket:
    /// `closedTime` (populated only after on-chain settlement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_time: Option<DateTime<Utc>>,

    // ── Polymarket-Specific ────────────────────────────────────────────────
    /// Polymarket: neg-risk flag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neg_risk: Option<bool>,
    /// Polymarket: neg-risk market ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub neg_risk_market_id: Option<String>,

    // ── Resolution ─────────────────────────────────────────────────────────
    /// Resolution result. Kalshi: enum string from upstream. Polymarket:
    /// derived winning-outcome label (the `outcomes[i]` whose `outcomePrices[i]`
    /// is 1.0 after settlement). None for unresolved markets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl Market {
    /// Create openpx_id from exchange and ticker
    #[inline]
    pub fn make_openpx_id(exchange: &str, ticker: &str) -> String {
        format!("{}:{}", exchange, ticker)
    }

    /// Parse openpx_id into (exchange, ticker)
    pub fn parse_openpx_id(openpx_id: &str) -> Option<(&str, &str)> {
        let (exchange, ticker) = openpx_id.split_once(':')?;
        if exchange.is_empty() || ticker.is_empty() {
            return None;
        }
        Some((exchange, ticker))
    }

    /// Check if market matches search query (case-insensitive)
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.title.to_lowercase().contains(&query_lower)
            || self
                .rules
                .as_ref()
                .is_some_and(|r| r.to_lowercase().contains(&query_lower))
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

    /// Find an outcome by label (case-insensitive).
    pub fn outcome(&self, label: &str) -> Option<&Outcome> {
        self.outcomes
            .iter()
            .find(|o| o.label.eq_ignore_ascii_case(label))
    }

    /// Yes-side token id, when exposed (Polymarket binary markets).
    pub fn token_id_yes(&self) -> Option<&str> {
        self.outcome("Yes").and_then(|o| o.token_id.as_deref())
    }

    /// No-side token id, when exposed (Polymarket binary markets).
    pub fn token_id_no(&self) -> Option<&str> {
        self.outcome("No").and_then(|o| o.token_id.as_deref())
    }

    /// All exposed token ids in outcome order (skips outcomes with no token id).
    pub fn token_ids(&self) -> Vec<String> {
        self.outcomes
            .iter()
            .filter_map(|o| o.token_id.clone())
            .collect()
    }
}

impl Default for Market {
    fn default() -> Self {
        Self {
            openpx_id: String::new(),
            exchange: String::new(),
            ticker: String::new(),
            event_ticker: None,
            numeric_id: None,
            title: String::new(),
            rules: None,
            status: MarketStatus::Active,
            market_type: MarketType::Binary,
            outcomes: vec![],
            condition_id: None,
            volume: 0.0,
            volume_24h: None,
            last_trade_price: None,
            best_bid: None,
            best_ask: None,
            tick_size: None,
            min_order_size: None,
            close_time: None,
            open_time: None,
            created_at: None,
            settlement_time: None,
            neg_risk: None,
            neg_risk_market_id: None,
            result: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(label: &str, price: Option<f64>, token: Option<&str>) -> Outcome {
        Outcome {
            label: label.into(),
            price,
            token_id: token.map(String::from),
        }
    }

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
            ticker: "1".into(),
            title: "Test".into(),
            ..Default::default()
        };
        let json = serde_json::to_value(&market).unwrap();
        assert!(json.get("volume_24h").is_none());
        assert!(json.get("min_order_size").is_none());
        assert!(json.get("event_ticker").is_none());
        assert!(json.get("tick_size").is_none());
    }

    #[test]
    fn optional_fields_present_when_some() {
        let market = Market {
            openpx_id: "test:1".into(),
            exchange: "test".into(),
            ticker: "1".into(),
            title: "Test".into(),
            volume_24h: Some(1000.0),
            min_order_size: Some(15.0),
            tick_size: Some(0.01),
            event_ticker: Some("EV-1".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&market).unwrap();
        assert_eq!(json["volume_24h"], 1000.0);
        assert_eq!(json["min_order_size"], 15.0);
        assert_eq!(json["tick_size"], 0.01);
        assert_eq!(json["event_ticker"], "EV-1");
    }

    #[test]
    fn matches_search_title_and_rules() {
        let market = Market {
            title: "Will Bitcoin reach $100k?".into(),
            rules: Some("Resolves yes if BTC closes above 100000 USD on Coinbase".into()),
            ..Default::default()
        };
        assert!(market.matches_search("bitcoin"));
        assert!(market.matches_search("100k"));
        assert!(market.matches_search("coinbase"));
        assert!(!market.matches_search("ethereum"));
    }

    #[test]
    fn token_id_yes_no_lookup() {
        let market = Market {
            outcomes: vec![
                outcome("Yes", Some(0.65), Some("yes_tok")),
                outcome("No", Some(0.35), Some("no_tok")),
            ],
            ..Default::default()
        };
        assert_eq!(market.token_id_yes(), Some("yes_tok"));
        assert_eq!(market.token_id_no(), Some("no_tok"));
        assert_eq!(market.token_ids(), vec!["yes_tok", "no_tok"]);
    }

    #[test]
    fn token_id_yes_no_absent_for_kalshi() {
        let market = Market {
            outcomes: vec![
                outcome("Yes", Some(0.65), None),
                outcome("No", Some(0.35), None),
            ],
            ..Default::default()
        };
        assert_eq!(market.token_id_yes(), None);
        assert_eq!(market.token_id_no(), None);
        assert!(market.token_ids().is_empty());
    }

    #[test]
    fn is_binary_and_is_open() {
        let market = Market {
            outcomes: vec![outcome("Yes", None, None), outcome("No", None, None)],
            status: MarketStatus::Active,
            ..Default::default()
        };
        assert!(market.is_binary());
        assert!(market.is_open());

        let closed = Market {
            outcomes: vec![outcome("Yes", None, None), outcome("No", None, None)],
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
            ticker: "1".into(),
            title: "Test".into(),
            market_type: MarketType::Categorical,
            ..Default::default()
        };
        let json = serde_json::to_value(&market).unwrap();
        assert_eq!(json["market_type"], "categorical");
    }
}
