use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Order time-in-force / execution type.
///
/// Normalized across all exchanges:
/// - `Gtc` (good-til-cancelled) — rests on the book until filled or cancelled.
/// - `Ioc` (immediate-or-cancel) — fills what it can immediately, cancels the rest.
/// - `Fok` (fill-or-kill) — must fill entirely in one shot or is cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    #[default]
    Gtc,
    Ioc,
    Fok,
}

impl FromStr for OrderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gtc" => Ok(Self::Gtc),
            "ioc" => Ok(Self::Ioc),
            "fok" => Ok(Self::Fok),
            other => Err(format!(
                "invalid order_type '{other}' (allowed: gtc, ioc, fok)"
            )),
        }
    }
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gtc => f.write_str("gtc"),
            Self::Ioc => f.write_str("ioc"),
            Self::Fok => f.write_str("fok"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Selects which outcome of a market an order targets.
///
/// `Yes` / `No` cover binary markets on both exchanges. The remaining variants
/// only resolve on Polymarket (multi-outcome categorical markets); Kalshi
/// markets are always binary, so anything other than `Yes` / `No` is rejected
/// with `InvalidInput` at the Kalshi adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderOutcome {
    /// Binary YES outcome (Kalshi YES side, Polymarket `outcomes[0]`).
    Yes,
    /// Binary NO outcome (Kalshi NO side, Polymarket `outcomes[1]`).
    No,
    /// Match a Polymarket outcome by its `label` (case-insensitive).
    Label(String),
    /// Match a Polymarket outcome by its zero-based ordinal in `Market.outcomes`.
    Index(usize),
    /// Polymarket CTF token id — bypass label/index lookup.
    TokenId(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LiquidityRole {
    Maker,
    Taker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Open,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}

/// Lean input shape for `Exchange::create_order`. All cross-venue and
/// per-venue knobs are deliberately absent — `post_only`, `expiration_ts`,
/// `client_order_id`, `reduce_only`, `neg_risk`, builder/metadata, and
/// subaccounts are not exposed. Anything an exchange requires that is not
/// modelled here is generated internally (e.g. Kalshi V2's required
/// `client_order_id` is filled with a per-call UUID).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CreateOrderRequest {
    /// Unified market identifier — Kalshi market ticker or Polymarket slug.
    pub market_ticker: String,
    /// Which outcome of the market to trade.
    pub outcome: OrderOutcome,
    /// Buy or sell.
    pub side: OrderSide,
    /// Limit price as YES probability in `(0.0, 1.0)`.
    pub price: f64,
    /// Order size in contracts.
    pub size: f64,
    /// Time-in-force / execution type.
    #[serde(default)]
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Order {
    pub id: String,
    pub market_ticker: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub filled: f64,
    /// Volume-weighted per-contract fee paid for fills resulting from this
    /// order, in quote-currency dollars. Kalshi reports it on `create_order`
    /// when fills occur immediately; Polymarket charges fees at trade
    /// settlement, so this stays `None` on initial create response.
    #[serde(default)]
    pub fee: Option<f64>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn remaining(&self) -> f64 {
        self.size - self.filled
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled || self.filled >= self.size
    }

    pub fn fill_percentage(&self) -> f64 {
        if self.size == 0.0 {
            return 0.0;
        }
        self.filled / self.size
    }
}

/// A single fill (trade execution) from a user's order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Fill {
    pub fill_id: String,
    pub order_id: String,
    pub market_ticker: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub is_taker: bool,
    pub fee: f64,
    pub created_at: DateTime<Utc>,
}

// TODO(fill-sim): Add local fill simulation for backtesting strategies offline.
// Sketch of a FillEngine that simulates order execution against a local
// orderbook copy:
//   - execute_market_order(order, book) → FillResult with fills, fees, slippage check
//   - execute_limit_order(order, book) → checks immediate fillability
//   - Configurable: min_fill_size, max_slippage_pct, fee_rate_bps
//   - Tracks fill history with get_fills(order_id), get_stats()
// Pro Traders (user type B) would use this for backtesting without hitting live APIs.
// Could be implemented as a standalone utility crate or SDK-side helper.
//
// See also: TODO(historical-orderbook) for the data
// ingestion side. NautilusTrader (nautechsystems/nautilus_trader) takes a similar approach:
// L2 snapshots replayed as CLEAR+ADD delta sequences into a simulated matching engine.
// Key caveat: without real trade tape, fill simulation is approximate — no queue priority
// or true latency modeling. Good for strategy development, not precise PnL attribution.
