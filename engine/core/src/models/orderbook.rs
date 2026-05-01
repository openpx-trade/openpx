use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

// ---------------------------------------------------------------------------
// FixedPrice — integer-backed price for orderbook hot paths
// ---------------------------------------------------------------------------

/// Fixed-point price representation. 1 tick = 0.0001 (scale factor 10,000).
/// Eliminates f64 comparison issues (no PRICE_EPSILON), enables `Ord` (no NaN),
/// and uses integer arithmetic (1-5ns vs 20-100ns for f64 ops).
///
/// Serializes as f64 on the wire for JSON backward compatibility.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedPrice(u64);

impl FixedPrice {
    pub const SCALE: u64 = 10_000;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(Self::SCALE);

    #[inline]
    pub fn from_f64(price: f64) -> Self {
        Self((price * Self::SCALE as f64).round() as u64)
    }

    #[inline]
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }

    #[inline]
    pub fn raw(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// 1.0 - self, exact in fixed-point. Used for NO-side price inversion.
    #[inline]
    pub fn complement(self) -> Self {
        Self(Self::SCALE.saturating_sub(self.0))
    }

    #[inline]
    pub fn midpoint(self, other: Self) -> Self {
        Self((self.0 + other.0) / 2)
    }
}

impl std::fmt::Debug for FixedPrice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FixedPrice({})", self.to_f64())
    }
}

impl std::fmt::Display for FixedPrice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

impl Default for FixedPrice {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<f64> for FixedPrice {
    #[inline]
    fn from(v: f64) -> Self {
        Self::from_f64(v)
    }
}

impl From<FixedPrice> for f64 {
    #[inline]
    fn from(v: FixedPrice) -> Self {
        v.to_f64()
    }
}

impl Serialize for FixedPrice {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.to_f64())
    }
}

impl<'de> Deserialize<'de> for FixedPrice {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = f64::deserialize(deserializer)?;
        Ok(Self::from_f64(v))
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for FixedPrice {
    fn schema_name() -> String {
        "number".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        f64::json_schema(gen)
    }
}

// ---------------------------------------------------------------------------
// Orderbook types
// ---------------------------------------------------------------------------

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
    pub price: FixedPrice,
    pub size: f64,
}

/// Stack-allocated change list. Kalshi = 1 change, Polymarket typically 1-3.
/// Falls back to heap only if > 4 changes in a single update (rare).
pub type ChangeVec = SmallVec<[PriceLevelChange; 4]>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PriceLevel {
    pub price: FixedPrice,
    pub size: f64,
}

impl PriceLevel {
    #[inline]
    pub fn new(price: f64, size: f64) -> Self {
        Self {
            price: FixedPrice::from_f64(price),
            size,
        }
    }

    #[inline]
    pub fn with_fixed(price: FixedPrice, size: f64) -> Self {
        Self { price, size }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Orderbook {
    pub asset_id: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Exchange-provided hash for verifying book state integrity during replay.
    /// Polymarket: present on `book` snapshot events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl Orderbook {
    #[inline]
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|l| l.price.to_f64())
    }

    #[inline]
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|l| l.price.to_f64())
    }

    #[inline]
    pub fn mid_price(&self) -> Option<f64> {
        match (self.bids.first(), self.asks.first()) {
            (Some(bid), Some(ask)) => Some(bid.price.midpoint(ask.price).to_f64()),
            _ => None,
        }
    }

    #[inline]
    pub fn spread(&self) -> Option<f64> {
        match (self.bids.first(), self.asks.first()) {
            (Some(bid), Some(ask)) => Some(ask.price.to_f64() - bid.price.to_f64()),
            _ => None,
        }
    }

    #[inline]
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
            asset_id: asset_id.into(),
            bids: parsed_bids,
            asks: parsed_asks,
            last_update_id: None,
            timestamp: Some(Utc::now()),
            hash: None,
        }
    }
}

/// Sort price levels in descending order (highest price first) -- bid side ordering.
/// Uses integer comparison via FixedPrice::Ord (no partial_cmp/NaN handling).
pub fn sort_bids(levels: &mut [PriceLevel]) {
    levels.sort_unstable_by_key(|l| std::cmp::Reverse(l.price));
}

/// Sort price levels in ascending order (lowest price first) -- ask side ordering.
/// Uses integer comparison via FixedPrice::Ord (no partial_cmp/NaN handling).
pub fn sort_asks(levels: &mut [PriceLevel]) {
    levels.sort_unstable_by_key(|l| l.price);
}

/// Insert a price level into a bid-sorted (descending) list.
/// Binary-search for the insert position (O(log n)), then Vec::insert
/// (O(n) memcpy shift on average). Net O(log n + n) per op vs the old
/// push+sort's O(n log n). Equal-price entries go AFTER existing entries.
#[inline]
pub fn insert_bid(levels: &mut Vec<PriceLevel>, level: PriceLevel) {
    let idx = levels.partition_point(|l| l.price > level.price);
    levels.insert(idx, level);
}

/// Insert a price level into an ask-sorted (ascending) list.
/// Binary-search + Vec::insert; same complexity profile as `insert_bid`.
#[inline]
pub fn insert_ask(levels: &mut Vec<PriceLevel>, level: PriceLevel) {
    let idx = levels.partition_point(|l| l.price < level.price);
    levels.insert(idx, level);
}

/// Apply a price-level delta to a bid-sorted list with replace-or-insert
/// semantics (matches the behaviour of a sorted associative map):
///   - `size > 0.0` and price exists: replace in place (O(log n)).
///   - `size > 0.0` and price is new: insert at sorted position (O(log n + n)).
///   - `size == 0.0`: remove the level if present (no-op otherwise).
pub fn apply_bid_level(levels: &mut Vec<PriceLevel>, level: PriceLevel) {
    match levels.binary_search_by(|l| level.price.cmp(&l.price)) {
        Ok(idx) => {
            if level.size > 0.0 {
                levels[idx] = level;
            } else {
                levels.remove(idx);
            }
        }
        Err(idx) => {
            if level.size > 0.0 {
                levels.insert(idx, level);
            }
        }
    }
}

/// See `apply_bid_level`. Same semantics, ascending ordering.
pub fn apply_ask_level(levels: &mut Vec<PriceLevel>, level: PriceLevel) {
    match levels.binary_search_by(|l| l.price.cmp(&level.price)) {
        Ok(idx) => {
            if level.size > 0.0 {
                levels[idx] = level;
            } else {
                levels.remove(idx);
            }
        }
        Err(idx) => {
            if level.size > 0.0 {
                levels.insert(idx, level);
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RestPriceLevel {
    pub price: String,
    pub size: String,
}
