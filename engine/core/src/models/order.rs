use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Time-in-force. Options: `gtc` (rests on book), `ioc` (fill-now-or-cancel-rest), `fok` (all-or-nothing).
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

/// Order direction. Options: `buy`, `sell`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Outcome targeted by an order. Options: `yes`, `no`, or `label: "<name>"` for categorical Polymarket markets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum OrderOutcome {
    Yes,
    No,
    Label(String),
}

/// Whether a fill provided liquidity (`maker`) or took it (`taker`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LiquidityRole {
    Maker,
    Taker,
}

/// Order lifecycle state. Options: `pending`, `open`, `filled`, `partially_filled`, `cancelled`, `rejected`.
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

/// Input for `create_order`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CreateOrderRequest {
    /// The orderable asset — Kalshi market ticker or Polymarket CTF token id (e.g. `"KXBTCD-25APR1517"`).
    pub asset_id: String,
    /// Outcome to trade. Options: `yes`, `no`, or `label: "<name>"` (Polymarket categorical markets only).
    pub outcome: OrderOutcome,
    /// Order direction. Options: `buy`, `sell`.
    pub side: OrderSide,
    /// Limit price as YES probability in `(0, 1)` (e.g. `0.62`).
    pub price: f64,
    /// Order size in contracts (e.g. `100.0`).
    pub size: f64,
    /// Time-in-force; defaults to `gtc` (options: `gtc`, `ioc`, `fok`).
    #[serde(default)]
    pub order_type: OrderType,
}

/// An order on the unified surface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Order {
    /// Globally-unique exchange order id (e.g. `"a1b2c3d4-..."`).
    pub id: String,
    /// Unified market ticker the order belongs to (e.g. `"KXBTCD-25APR1517"`).
    pub market_ticker: String,
    /// Outcome label as published by the exchange (e.g. `"Yes"`, `"No"`, or a categorical label).
    pub outcome: String,
    /// Order direction. Options: `buy`, `sell`.
    pub side: OrderSide,
    /// Limit price as YES probability in `(0, 1)` (e.g. `0.62`).
    pub price: f64,
    /// Order size in contracts (e.g. `100.0`).
    pub size: f64,
    /// Cumulative filled size in contracts (e.g. `25.0`).
    pub filled: f64,
    /// Volume-weighted per-contract fee in quote dollars; `null` on Polymarket and on unfilled orders.
    #[serde(default)]
    pub fee: Option<f64>,
    /// Order lifecycle state. Options: `pending`, `open`, `filled`, `partially_filled`, `cancelled`, `rejected`.
    pub status: OrderStatus,
    /// Order creation time (UTC) (e.g. `"2026-04-25T12:00:00Z"`).
    pub created_at: DateTime<Utc>,
    /// Last update time (UTC); `null` if untouched since creation.
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

/// A single fill (trade execution) from one of the caller's orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Fill {
    /// Globally-unique fill id (e.g. `"f-9c2..."`).
    pub fill_id: String,
    /// Parent order id (e.g. `"a1b2c3d4-..."`).
    pub order_id: String,
    /// Unified market ticker the fill belongs to (e.g. `"KXBTCD-25APR1517"`).
    pub market_ticker: String,
    /// Outcome label as published by the exchange (e.g. `"Yes"`, `"No"`).
    pub outcome: String,
    /// Order direction. Options: `buy`, `sell`.
    pub side: OrderSide,
    /// Fill price as YES probability in `(0, 1)` (e.g. `0.62`).
    pub price: f64,
    /// Filled size in contracts (e.g. `25.0`).
    pub size: f64,
    /// `true` if the caller took liquidity, `false` if they made it.
    pub is_taker: bool,
    /// Fee paid in quote dollars (e.g. `0.07`).
    pub fee: f64,
    /// Fill execution time (UTC) (e.g. `"2026-04-25T12:00:00Z"`).
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
