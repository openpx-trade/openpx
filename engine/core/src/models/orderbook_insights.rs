use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::Orderbook;

const TOP_N_FOR_WEIGHTED: usize = 10;
const SLOPE_MAX_LEVELS: usize = 20;
const SLOPE_BPS_WINDOW: f64 = 200.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrderbookStats {
    pub exchange_ts: Option<DateTime<Utc>>,
    pub openpx_ts: DateTime<Utc>,
    pub asset_id: String,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub mid: Option<f64>,
    pub spread_bps: Option<f64>,
    pub weighted_mid: Option<f64>,
    pub imbalance: Option<f64>,
    pub bid_depth: f64,
    pub ask_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrderbookImpact {
    pub exchange_ts: Option<DateTime<Utc>>,
    pub openpx_ts: DateTime<Utc>,
    pub asset_id: String,
    pub size: f64,
    pub mid: Option<f64>,
    pub buy_avg_price: Option<f64>,
    pub buy_slippage_bps: Option<f64>,
    pub buy_fill_pct: f64,
    pub sell_avg_price: Option<f64>,
    pub sell_slippage_bps: Option<f64>,
    pub sell_fill_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrderbookMicrostructure {
    pub exchange_ts: Option<DateTime<Utc>>,
    pub openpx_ts: DateTime<Utc>,
    pub asset_id: String,
    pub depth_buckets: DepthBuckets,
    pub bid_slope: Option<f64>,
    pub ask_slope: Option<f64>,
    pub max_gap: MaxGap,
    pub level_count: LevelCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DepthBuckets {
    pub bid_within_10bps: f64,
    pub ask_within_10bps: f64,
    pub bid_within_50bps: f64,
    pub ask_within_50bps: f64,
    pub bid_within_100bps: f64,
    pub ask_within_100bps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MaxGap {
    pub bid_gap_bps: Option<f64>,
    pub ask_gap_bps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LevelCount {
    pub bids: u32,
    pub asks: u32,
}

/// Snapshot stats: top-of-book, weighted mid, imbalance, total depth.
/// Pure function over the unified orderbook — no upstream calls.
pub fn orderbook_stats(book: &Orderbook) -> OrderbookStats {
    let best_bid = book.best_bid();
    let best_ask = book.best_ask();
    let mid = book.mid_price();

    let spread_bps = match (best_bid, best_ask, mid) {
        (Some(b), Some(a), Some(m)) if m > 0.0 => Some((a - b) / m * 10_000.0),
        _ => None,
    };

    let q_b: f64 = book
        .bids
        .iter()
        .take(TOP_N_FOR_WEIGHTED)
        .map(|l| l.size)
        .sum();
    let q_a: f64 = book
        .asks
        .iter()
        .take(TOP_N_FOR_WEIGHTED)
        .map(|l| l.size)
        .sum();
    let total_top_n = q_b + q_a;

    let weighted_mid = match (best_bid, best_ask) {
        (Some(b), Some(a)) if total_top_n > 0.0 => Some((b * q_a + a * q_b) / total_top_n),
        _ => None,
    };

    let imbalance = if total_top_n > 0.0 {
        Some((q_b - q_a) / total_top_n)
    } else {
        None
    };

    let bid_depth: f64 = book.bids.iter().map(|l| l.size).sum();
    let ask_depth: f64 = book.asks.iter().map(|l| l.size).sum();

    OrderbookStats {
        exchange_ts: book.timestamp,
        openpx_ts: Utc::now(),
        asset_id: book.asset_id.clone(),
        best_bid,
        best_ask,
        mid,
        spread_bps,
        weighted_mid,
        imbalance,
        bid_depth,
        ask_depth,
    }
}

/// Slippage curve at a single requested size. Walks both sides of the book
/// (asks ascending for buy, bids descending for sell) consuming levels until
/// `size` is filled or the side exhausts.
///
/// Note: `bps` are mid-relative; interpretability degrades for prices near 0
/// or 1, where small absolute moves represent very large bps.
pub fn orderbook_impact(book: &Orderbook, size: f64) -> OrderbookImpact {
    let mid = book.mid_price();
    let (buy_avg, buy_fill) = walk_side(&book.asks, size);
    let (sell_avg, sell_fill) = walk_side(&book.bids, size);

    let buy_slippage_bps = match (buy_avg, mid) {
        (Some(p), Some(m)) if m > 0.0 => Some((p - m).abs() / m * 10_000.0),
        _ => None,
    };
    let sell_slippage_bps = match (sell_avg, mid) {
        (Some(p), Some(m)) if m > 0.0 => Some((p - m).abs() / m * 10_000.0),
        _ => None,
    };

    OrderbookImpact {
        exchange_ts: book.timestamp,
        openpx_ts: Utc::now(),
        asset_id: book.asset_id.clone(),
        size,
        mid,
        buy_avg_price: buy_avg,
        buy_slippage_bps,
        buy_fill_pct: pct(buy_fill, size),
        sell_avg_price: sell_avg,
        sell_slippage_bps,
        sell_fill_pct: pct(sell_fill, size),
    }
}

/// Microstructure signals: cumulative depth at standard bps tiers, slope of
/// cumulative size vs distance-from-mid, largest consecutive-level gap, and
/// raw level counts per side.
pub fn orderbook_microstructure(book: &Orderbook) -> OrderbookMicrostructure {
    let mid = book.mid_price();

    let depth_buckets = match mid {
        Some(m) if m > 0.0 => DepthBuckets {
            bid_within_10bps: cumulative_within(&book.bids, m, 10.0),
            ask_within_10bps: cumulative_within(&book.asks, m, 10.0),
            bid_within_50bps: cumulative_within(&book.bids, m, 50.0),
            ask_within_50bps: cumulative_within(&book.asks, m, 50.0),
            bid_within_100bps: cumulative_within(&book.bids, m, 100.0),
            ask_within_100bps: cumulative_within(&book.asks, m, 100.0),
        },
        _ => DepthBuckets {
            bid_within_10bps: 0.0,
            ask_within_10bps: 0.0,
            bid_within_50bps: 0.0,
            ask_within_50bps: 0.0,
            bid_within_100bps: 0.0,
            ask_within_100bps: 0.0,
        },
    };

    let bid_slope = mid.and_then(|m| slope(&book.bids, m));
    let ask_slope = mid.and_then(|m| slope(&book.asks, m));

    let max_gap = MaxGap {
        bid_gap_bps: mid.and_then(|m| max_gap_bps(&book.bids, m)),
        ask_gap_bps: mid.and_then(|m| max_gap_bps(&book.asks, m)),
    };

    OrderbookMicrostructure {
        exchange_ts: book.timestamp,
        openpx_ts: Utc::now(),
        asset_id: book.asset_id.clone(),
        depth_buckets,
        bid_slope,
        ask_slope,
        max_gap,
        level_count: LevelCount {
            bids: book.bids.len() as u32,
            asks: book.asks.len() as u32,
        },
    }
}

fn walk_side(levels: &[crate::models::PriceLevel], size: f64) -> (Option<f64>, f64) {
    if size <= 0.0 || levels.is_empty() {
        return (None, 0.0);
    }
    let mut filled = 0.0;
    let mut notional = 0.0;
    for l in levels {
        let take = (size - filled).min(l.size);
        notional += take * l.price.to_f64();
        filled += take;
        if filled >= size {
            break;
        }
    }
    if filled <= 0.0 {
        (None, 0.0)
    } else {
        (Some(notional / filled), filled)
    }
}

fn pct(filled: f64, size: f64) -> f64 {
    if size <= 0.0 {
        return 0.0;
    }
    (filled / size).min(1.0) * 100.0
}

fn cumulative_within(levels: &[crate::models::PriceLevel], mid: f64, bps_window: f64) -> f64 {
    levels
        .iter()
        .take_while(|l| (l.price.to_f64() - mid).abs() / mid * 10_000.0 <= bps_window)
        .map(|l| l.size)
        .sum()
}

/// OLS slope of cumulative size (y) vs distance-from-mid in bps (x), over the
/// closer of: top SLOPE_MAX_LEVELS levels, or all levels within
/// SLOPE_BPS_WINDOW bps. Returns `None` if fewer than 2 points qualify.
fn slope(levels: &[crate::models::PriceLevel], mid: f64) -> Option<f64> {
    if mid <= 0.0 {
        return None;
    }
    let mut points: Vec<(f64, f64)> = Vec::with_capacity(SLOPE_MAX_LEVELS);
    let mut cum = 0.0;
    for l in levels.iter().take(SLOPE_MAX_LEVELS) {
        let dist_bps = (l.price.to_f64() - mid).abs() / mid * 10_000.0;
        if dist_bps > SLOPE_BPS_WINDOW {
            break;
        }
        cum += l.size;
        points.push((dist_bps, cum));
    }
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f64;
    let mean_x = points.iter().map(|(x, _)| x).sum::<f64>() / n;
    let mean_y = points.iter().map(|(_, y)| y).sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for (x, y) in &points {
        num += (x - mean_x) * (y - mean_y);
        den += (x - mean_x).powi(2);
    }
    if den == 0.0 {
        None
    } else {
        Some(num / den)
    }
}

fn max_gap_bps(levels: &[crate::models::PriceLevel], mid: f64) -> Option<f64> {
    if mid <= 0.0 || levels.len() < 2 {
        return None;
    }
    let mut max = 0.0_f64;
    for w in levels.windows(2) {
        let gap = (w[0].price.to_f64() - w[1].price.to_f64()).abs();
        let bps = gap / mid * 10_000.0;
        if bps > max {
            max = bps;
        }
    }
    Some(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PriceLevel;

    fn book(bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) -> Orderbook {
        Orderbook {
            asset_id: "test-asset".into(),
            bids: bids.into_iter().map(|(p, s)| PriceLevel::new(p, s)).collect(),
            asks: asks.into_iter().map(|(p, s)| PriceLevel::new(p, s)).collect(),
            last_update_id: None,
            timestamp: None,
            hash: None,
        }
    }

    #[test]
    fn stats_tight_book_around_half() {
        let bids: Vec<(f64, f64)> = (0..10).map(|i| (0.49 - 0.001 * i as f64, 100.0)).collect();
        let asks: Vec<(f64, f64)> = (0..10).map(|i| (0.51 + 0.001 * i as f64, 100.0)).collect();
        let s = orderbook_stats(&book(bids, asks));
        assert_eq!(s.best_bid, Some(0.49));
        assert_eq!(s.best_ask, Some(0.51));
        assert_eq!(s.mid, Some(0.50));
        assert!((s.spread_bps.unwrap() - 400.0).abs() < 1e-6);
        assert!((s.imbalance.unwrap()).abs() < 1e-9);
        assert!((s.weighted_mid.unwrap() - 0.50).abs() < 1e-9);
        assert!((s.bid_depth - 1000.0).abs() < 1e-9);
        assert!((s.ask_depth - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn impact_skewed_book() {
        let b = book(
            vec![(0.49, 1000.0), (0.48, 1000.0), (0.47, 1000.0)],
            vec![(0.51, 10.0)],
        );
        let s = orderbook_stats(&b);
        assert!(s.imbalance.unwrap() > 0.9);

        let small_buy = orderbook_impact(&b, 5.0);
        assert!((small_buy.buy_fill_pct - 100.0).abs() < 1e-9);
        assert_eq!(small_buy.buy_avg_price, Some(0.51));

        let big_sell = orderbook_impact(&b, 5_000.0);
        assert!(big_sell.sell_fill_pct < 100.0);
        assert!(big_sell.sell_avg_price.is_some());

        let oversize_buy = orderbook_impact(&b, 1_000.0);
        assert!(oversize_buy.buy_fill_pct < 100.0);
    }

    #[test]
    fn microstructure_single_level_each() {
        let b = book(vec![(0.49, 100.0)], vec![(0.51, 100.0)]);
        let m = orderbook_microstructure(&b);
        assert_eq!(m.bid_slope, None);
        assert_eq!(m.ask_slope, None);
        assert_eq!(m.max_gap.bid_gap_bps, None);
        assert_eq!(m.max_gap.ask_gap_bps, None);
        assert_eq!(m.level_count.bids, 1);
        assert_eq!(m.level_count.asks, 1);
    }

    #[test]
    fn empty_one_side() {
        let b = book(vec![(0.49, 100.0), (0.48, 50.0)], vec![]);
        let s = orderbook_stats(&b);
        assert_eq!(s.mid, None);
        assert_eq!(s.spread_bps, None);
        assert_eq!(s.weighted_mid, None);
        assert!((s.bid_depth - 150.0).abs() < 1e-9);
        assert!((s.ask_depth).abs() < 1e-9);

        let i = orderbook_impact(&b, 50.0);
        assert_eq!(i.buy_avg_price, None);
        assert!((i.buy_fill_pct).abs() < 1e-9);
        assert_eq!(i.sell_avg_price, Some(0.49));
        assert!((i.sell_fill_pct - 100.0).abs() < 1e-9);
    }

    #[test]
    fn microstructure_gappy_asks() {
        let b = book(vec![(0.49, 100.0)], vec![(0.51, 100.0), (0.55, 100.0), (0.56, 100.0)]);
        let m = orderbook_microstructure(&b);
        // mid = 0.50; ask gap from 0.51 -> 0.55 = 0.04; bps = 0.04/0.50 * 10_000 = 800.
        assert!((m.max_gap.ask_gap_bps.unwrap() - 800.0).abs() < 1e-6);

        // oversize buy: total ask depth = 300, request 500 → partial.
        let i = orderbook_impact(&b, 500.0);
        assert!(i.buy_fill_pct < 100.0);
        assert!(i.buy_avg_price.is_some());
    }
}
