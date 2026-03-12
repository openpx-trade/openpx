// TODO(zero-alloc-book): Evaluate fixed-point integer price representation for orderbook hot paths.
// polyfill-rs uses u32 prices (1 tick = 0.0001, scale factor 10,000) and BTreeMap<Price, Qty>
// with integer arithmetic for spread/midpoint/market-impact calculations (1-5ns per op vs 20-100ns
// for Decimal). They prove zero heap allocations via #[global_allocator] counting tests. Our
// Orderbook uses f64 PriceLevel with Vec<PriceLevel> which allocates on every update. For the WS
// hot path (applying orderbook deltas at high frequency), consider a FastOrderbook variant with
// stack-allocated SmallVec levels and integer prices. Benchmark first — this matters most if we
// ever publish a Rust SDK or if WS orderbook throughput becomes a bottleneck.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Epsilon for floating-point price comparison.
/// Centralized here to prevent per-crate divergence (f64::EPSILON is too small after arithmetic).
pub const PRICE_EPSILON: f64 = 1e-9;

/// Bid or ask side. Serializes as "bid"/"ask" on the wire.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PriceLevelSide {
    Bid,
    Ask,
}

/// A single price level change. Absolute replacement semantics:
/// size > 0 = set level to this size, size == 0 = remove level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PriceLevelChange {
    pub side: PriceLevelSide,
    pub price: f64,
    pub size: f64,
}

/// Stack-allocated change list. Kalshi = 1 change, Polymarket typically 1-3.
/// Falls back to heap only if > 4 changes in a single update (rare).
pub type ChangeVec = SmallVec<[PriceLevelChange; 4]>;

/// Emitted by exchange WS implementations through OrderbookStream.
#[derive(Debug, Clone)]
pub enum OrderbookUpdate {
    /// Full orderbook snapshot (initial connect, reconnect).
    Snapshot(Orderbook),
    /// Incremental change. Changes only — NO full book clone.
    /// WsManager maintains its own cached book and applies changes in-place.
    Delta {
        changes: ChangeVec,
        timestamp: Option<DateTime<Utc>>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}

impl PriceLevel {
    pub fn new(price: f64, size: f64) -> Self {
        Self { price, size }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Orderbook {
    pub market_id: String,
    pub asset_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

impl Orderbook {
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price)
    }

    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price)
    }

    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn has_data(&self) -> bool {
        !self.bids.is_empty() && !self.asks.is_empty()
    }

    /// Sort bids descending and asks ascending by price
    pub fn sort(&mut self) {
        sort_bids(&mut self.bids);
        sort_asks(&mut self.asks);
    }

    pub fn from_rest_response(
        bids: &[RestPriceLevel],
        asks: &[RestPriceLevel],
        asset_id: impl Into<String>,
    ) -> Self {
        let mut parsed_bids: Vec<PriceLevel> = bids
            .iter()
            .filter_map(|b| {
                let price = b.price.parse::<f64>().ok()?;
                let size = b.size.parse::<f64>().ok()?;
                if price > 0.0 && size > 0.0 {
                    Some(PriceLevel::new(price, size))
                } else {
                    None
                }
            })
            .collect();

        let mut parsed_asks: Vec<PriceLevel> = asks
            .iter()
            .filter_map(|a| {
                let price = a.price.parse::<f64>().ok()?;
                let size = a.size.parse::<f64>().ok()?;
                if price > 0.0 && size > 0.0 {
                    Some(PriceLevel::new(price, size))
                } else {
                    None
                }
            })
            .collect();

        sort_bids(&mut parsed_bids);
        sort_asks(&mut parsed_asks);

        Self {
            market_id: String::new(),
            asset_id: asset_id.into(),
            bids: parsed_bids,
            asks: parsed_asks,
            last_update_id: None,
            timestamp: Some(Utc::now()),
        }
    }
}

/// A point-in-time L2 orderbook snapshot, used for historical orderbook data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrderbookSnapshot {
    pub timestamp: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

/// Sort price levels in descending order (highest price first) -- bid side ordering
pub fn sort_bids(levels: &mut [PriceLevel]) {
    levels.sort_by(|a, b| {
        b.price
            .partial_cmp(&a.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Sort price levels in ascending order (lowest price first) -- ask side ordering
pub fn sort_asks(levels: &mut [PriceLevel]) {
    levels.sort_by(|a, b| {
        a.price
            .partial_cmp(&b.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[derive(Debug, Clone, Deserialize)]
pub struct RestPriceLevel {
    pub price: String,
    pub size: String,
}
