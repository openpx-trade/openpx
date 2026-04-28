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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
// TODO(order-fees): Add fee fields (e.g. `fee: Option<f64>`, `fee_rate_bps: Option<u32>`).
// Kalshi returns fees in create_order and fill responses — capture them here.
// Polymarket fees are protocol-level and can be computed from trade data.
// OpenPX does not charge fees; only the underlying exchange does.
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub filled: f64,
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
    pub market_id: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub is_taker: bool,
    pub fee: f64,
    pub created_at: DateTime<Utc>,
}

/// A user-facing trade row: distinct from `Fill` because it carries
/// auxiliary fields some venues expose (realized PnL, on-chain tx hash,
/// owner wallet) but others do not. Polymarket Data `/trades` returns
/// these natively; Kalshi `/portfolio/fills` returns a subset, with
/// `realized_pnl` and `tx_hash` left as `None`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UserTrade {
    pub id: String,
    pub market_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    pub side: OrderSide,
    pub size: f64,
    pub price: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<LiquidityRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realized_pnl: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub ts_ms: i64,
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
