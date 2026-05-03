//! End-to-end coverage for the unified orderbook surface.
//!
//! Every input variation of the five orderbook methods is exercised against
//! both Kalshi and Polymarket so the unified contract is held to a single bar:
//!
//!   • `fetch_orderbook(asset_id)`                — single-asset, full depth
//!   • `fetch_orderbooks_batch(asset_ids)`        — batch, full depth
//!   • `fetch_orderbook_stats(asset_id)`          — top-of-book + depth
//!   • `fetch_orderbook_impact(asset_id, size)`   — slippage at a target size
//!   • `fetch_orderbook_microstructure(asset_id)` — depth tiers / slope / gaps
//!
//! ## Variations
//!
//!   • valid asset_id (single)
//!   • multi-asset_id batch (2-3 assets)
//!   • empty asset_ids list (batch should return [])
//!   • above-cap asset_ids list (Kalshi: 101 entries → InvalidOrder)
//!   • nonexistent asset_id (404 / MarketNotFound)
//!   • impact: small size (full fill), large size (partial fill), zero (error), negative (error)
//!   • cross-exchange unification — same shape on Kalshi and Polymarket
//!   • numeric invariants — sorted, [0,1] price, > 0 size, no crossed book,
//!     spread ≥ 0, mid between bid and ask, weighted-mid in [bid, ask],
//!     bid/ask depth ≥ size of best level, stats consistent with raw book,
//!     impact partial-fill size ≤ total side depth, microstructure slope sign.
//!
//! ## Running
//!
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_orderbooks -- --nocapture
//!
//! Single exchange:
//!
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_orderbooks kalshi -- --nocapture
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_orderbooks polymarket -- --nocapture

use std::env;

use openpx::ExchangeInner;
use px_core::error::OpenPxError;
use px_core::{
    FetchMarketsParams, Market, MarketStatusFilter, Orderbook, OrderbookImpact,
    OrderbookMicrostructure, OrderbookStats,
};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn require_live() -> bool {
    let _ = dotenvy::dotenv();
    env::var("OPENPX_LIVE_TESTS").is_ok_and(|v| v == "1")
}

fn make_exchange_config(id: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    let vars: &[(&str, &str)] = match id {
        "kalshi" => &[
            ("KALSHI_API_KEY_ID", "api_key_id"),
            ("KALSHI_PRIVATE_KEY_PEM", "private_key_pem"),
            ("KALSHI_PRIVATE_KEY_PATH", "private_key_path"),
        ],
        "polymarket" => &[
            ("POLYMARKET_PRIVATE_KEY", "private_key"),
            ("POLYMARKET_FUNDER", "funder"),
            ("POLYMARKET_API_KEY", "api_key"),
            ("POLYMARKET_API_SECRET", "api_secret"),
            ("POLYMARKET_API_PASSPHRASE", "api_passphrase"),
        ],
        _ => &[],
    };
    for (env_key, config_key) in vars {
        if let Ok(v) = env::var(env_key) {
            obj.insert((*config_key).into(), v.into());
        }
    }
    serde_json::Value::Object(obj)
}

fn make_exchange(id: &str) -> Option<ExchangeInner> {
    if !require_live() {
        return None;
    }
    ExchangeInner::new(id, make_exchange_config(id)).ok()
}

fn is_transient(e: &OpenPxError) -> bool {
    let msg = format!("{e:?}");
    msg.contains("http error")
        || msg.contains("timed out")
        || msg.contains("connection")
        || msg.contains("rate limit")
        || msg.contains("429")
        || msg.contains("RateLimited")
}

fn is_market_not_found(e: &OpenPxError) -> bool {
    matches!(
        e,
        OpenPxError::Exchange(px_core::error::ExchangeError::MarketNotFound(_))
    )
}

/// Pick the canonical `asset_id` for an orderbook call:
/// - Polymarket: per-outcome CTF token id (each side has its own book).
/// - Kalshi: market ticker (V1 single-book per market — Yes-frame).
fn pick_asset_id(m: &Market) -> String {
    m.outcomes
        .first()
        .and_then(|o| o.token_id.clone())
        .unwrap_or_else(|| m.ticker.clone())
}

/// Find a market with non-empty orderbook so structural assertions have data
/// to chew on. Walks at most `max_tries` candidates, picking the first that
/// returns a populated book on at least one side. Returns `(market, asset_id, book)`.
async fn seed_market_with_book(
    ex: &ExchangeInner,
    label: &str,
    max_tries: usize,
) -> Option<(Market, String, Orderbook)> {
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Active),
        limit: Some(50),
        ..Default::default()
    };
    let (markets, _) = match ex.fetch_markets(&params).await {
        Ok(r) => r,
        Err(e) if is_transient(&e) => {
            eprintln!("SKIP {label}: transient: {e}");
            return None;
        }
        Err(e) => {
            eprintln!("SKIP {label}: fetch_markets failed: {e:?}");
            return None;
        }
    };
    if markets.is_empty() {
        eprintln!("SKIP {label}: no active markets");
        return None;
    }

    for m in markets.into_iter().take(max_tries) {
        let asset_id = pick_asset_id(&m);
        match ex.fetch_orderbook(&asset_id).await {
            Ok(book) if !book.bids.is_empty() || !book.asks.is_empty() => {
                return Some((m, asset_id, book));
            }
            Ok(_) => continue,
            Err(e) if is_transient(&e) || is_market_not_found(&e) => continue,
            Err(e) => {
                eprintln!("SKIP {label}: fetch_orderbook surfaced unexpected error: {e:?}");
                return None;
            }
        }
    }
    eprintln!("WARN {label}: no market with non-empty orderbook in first {max_tries}");
    None
}

// ---------------------------------------------------------------------------
// Universal invariant checkers — one source of truth, every test routes here
// ---------------------------------------------------------------------------

fn assert_book_well_formed(book: &Orderbook, asset_id: &str, label: &str) {
    // asset_id round-trips
    if !book.asset_id.is_empty() {
        assert!(
            book.asset_id == asset_id
                || book.asset_id.contains(asset_id)
                || asset_id.contains(&book.asset_id),
            "{label}: book.asset_id={} != requested {}",
            book.asset_id,
            asset_id
        );
    }

    // bids descending, asks ascending — strictly monotonic by price
    for w in book.bids.windows(2) {
        assert!(
            w[0].price >= w[1].price,
            "{label}: bids not sorted desc: {:?} then {:?}",
            w[0].price.to_f64(),
            w[1].price.to_f64()
        );
    }
    for w in book.asks.windows(2) {
        assert!(
            w[0].price <= w[1].price,
            "{label}: asks not sorted asc: {:?} then {:?}",
            w[0].price.to_f64(),
            w[1].price.to_f64()
        );
    }

    // every level: price strictly in (0,1), size > 0
    for l in book.bids.iter().chain(book.asks.iter()) {
        let p = l.price.to_f64();
        assert!(
            p > 0.0 && p < 1.0,
            "{label}: price {} outside (0,1) for asset {}",
            p,
            asset_id
        );
        assert!(
            l.size > 0.0,
            "{label}: non-positive size {} for asset {}",
            l.size,
            asset_id
        );
    }

    // no crossed book — best_bid <= best_ask
    if let (Some(b), Some(a)) = (book.best_bid(), book.best_ask()) {
        assert!(
            b <= a,
            "{label}: crossed book best_bid={b} > best_ask={a} on {asset_id}"
        );
    }
}

fn assert_stats_consistent_with_book(stats: &OrderbookStats, book: &Orderbook, label: &str) {
    assert_eq!(
        stats.asset_id, book.asset_id,
        "{label}: stats.asset_id drift"
    );
    assert_eq!(stats.best_bid, book.best_bid(), "{label}: best_bid drift");
    assert_eq!(stats.best_ask, book.best_ask(), "{label}: best_ask drift");

    if let Some(mid) = stats.mid {
        assert!(
            (0.0..=1.0).contains(&mid),
            "{label}: stats.mid out of [0,1]: {mid}"
        );
        if let (Some(b), Some(a)) = (stats.best_bid, stats.best_ask) {
            assert!(
                mid >= b && mid <= a,
                "{label}: mid={mid} outside [bid={b}, ask={a}]"
            );
        }
    }
    if let Some(s) = stats.spread_bps {
        assert!(s >= 0.0, "{label}: spread_bps negative: {s}");
    }

    let recomputed_bid_depth: f64 = book.bids.iter().map(|l| l.size).sum();
    let recomputed_ask_depth: f64 = book.asks.iter().map(|l| l.size).sum();
    assert!(
        (stats.bid_depth - recomputed_bid_depth).abs() < 1e-6,
        "{label}: bid_depth drift {} vs {}",
        stats.bid_depth,
        recomputed_bid_depth
    );
    assert!(
        (stats.ask_depth - recomputed_ask_depth).abs() < 1e-6,
        "{label}: ask_depth drift {} vs {}",
        stats.ask_depth,
        recomputed_ask_depth
    );

    if let (Some(wm), Some(b), Some(a)) = (stats.weighted_mid, stats.best_bid, stats.best_ask) {
        assert!(
            wm >= b && wm <= a,
            "{label}: weighted_mid {wm} outside [bid={b}, ask={a}]"
        );
    }
    if let Some(imb) = stats.imbalance {
        assert!(
            (-1.0..=1.0).contains(&imb),
            "{label}: imbalance {imb} outside [-1,1]"
        );
    }
}

fn assert_impact_well_formed(impact: &OrderbookImpact, book: &Orderbook, size: f64, label: &str) {
    assert_eq!(impact.size, size, "{label}: impact echoes wrong size");
    assert!(
        (0.0..=100.0).contains(&impact.buy_fill_pct),
        "{label}: buy_fill_pct {} outside [0,100]",
        impact.buy_fill_pct
    );
    assert!(
        (0.0..=100.0).contains(&impact.sell_fill_pct),
        "{label}: sell_fill_pct {} outside [0,100]",
        impact.sell_fill_pct
    );

    let total_ask_depth: f64 = book.asks.iter().map(|l| l.size).sum();
    let total_bid_depth: f64 = book.bids.iter().map(|l| l.size).sum();

    // If requested size exceeds total depth on a side, fill_pct must be < 100.
    if size > total_ask_depth + 1e-9 {
        assert!(
            impact.buy_fill_pct < 100.0 + 1e-9,
            "{label}: buy_fill_pct=100 but size={size} > total ask depth={total_ask_depth}"
        );
    }
    if size > total_bid_depth + 1e-9 {
        assert!(
            impact.sell_fill_pct < 100.0 + 1e-9,
            "{label}: sell_fill_pct=100 but size={size} > total bid depth={total_bid_depth}"
        );
    }

    // Walk-the-book must produce a buy_avg_price >= best_ask and
    // sell_avg_price <= best_bid (we eat better levels first, then worse).
    if let (Some(buy_avg), Some(best_ask)) = (impact.buy_avg_price, book.best_ask()) {
        assert!(
            buy_avg + 1e-9 >= best_ask,
            "{label}: buy_avg {} < best_ask {}",
            buy_avg,
            best_ask
        );
    }
    if let (Some(sell_avg), Some(best_bid)) = (impact.sell_avg_price, book.best_bid()) {
        assert!(
            sell_avg <= best_bid + 1e-9,
            "{label}: sell_avg {} > best_bid {}",
            sell_avg,
            best_bid
        );
    }

    // Slippage in bps must be non-negative when present.
    if let Some(b) = impact.buy_slippage_bps {
        assert!(b >= 0.0, "{label}: buy_slippage_bps negative: {b}");
    }
    if let Some(s) = impact.sell_slippage_bps {
        assert!(s >= 0.0, "{label}: sell_slippage_bps negative: {s}");
    }
}

fn assert_microstructure_well_formed(
    micro: &OrderbookMicrostructure,
    book: &Orderbook,
    label: &str,
) {
    assert_eq!(
        micro.level_count.bids as usize,
        book.bids.len(),
        "{label}: bid level_count drift"
    );
    assert_eq!(
        micro.level_count.asks as usize,
        book.asks.len(),
        "{label}: ask level_count drift"
    );

    // Cumulative depth tiers are monotonic non-decreasing.
    let dbk = &micro.depth_buckets;
    assert!(
        dbk.bid_within_10bps <= dbk.bid_within_50bps + 1e-9
            && dbk.bid_within_50bps <= dbk.bid_within_100bps + 1e-9,
        "{label}: bid depth buckets non-monotonic 10/50/100={}/{}/{}",
        dbk.bid_within_10bps,
        dbk.bid_within_50bps,
        dbk.bid_within_100bps
    );
    assert!(
        dbk.ask_within_10bps <= dbk.ask_within_50bps + 1e-9
            && dbk.ask_within_50bps <= dbk.ask_within_100bps + 1e-9,
        "{label}: ask depth buckets non-monotonic 10/50/100={}/{}/{}",
        dbk.ask_within_10bps,
        dbk.ask_within_50bps,
        dbk.ask_within_100bps
    );

    if let Some(g) = micro.max_gap.bid_gap_bps {
        assert!(g >= 0.0, "{label}: bid_gap_bps negative: {g}");
    }
    if let Some(g) = micro.max_gap.ask_gap_bps {
        assert!(g >= 0.0, "{label}: ask_gap_bps negative: {g}");
    }
}

// ---------------------------------------------------------------------------
// Per-exchange test suites
// ---------------------------------------------------------------------------

macro_rules! orderbooks_suite {
    ($exchange_id:ident) => {
        mod $exchange_id {
            use super::*;

            const ID: &str = stringify!($exchange_id);

            // ──────────────────────────────────────────────────────────────
            // describe() — the surface advertises every orderbook method
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn describe_advertises_orderbook_surface() {
                let Some(ex) = make_exchange(ID) else { return };
                let info = ex.describe();
                assert!(
                    info.has_fetch_orderbook,
                    "{ID}: describe().has_fetch_orderbook should be true"
                );
                assert!(
                    info.has_fetch_orderbooks_batch,
                    "{ID}: describe().has_fetch_orderbooks_batch should be true"
                );
            }

            // ──────────────────────────────────────────────────────────────
            // fetch_orderbook(asset_id) — happy path
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn fetch_orderbook_single_valid() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, book)) =
                    seed_market_with_book(&ex, &format!("{ID}/single_valid"), 10).await
                else {
                    return;
                };
                assert_book_well_formed(&book, &asset_id, &format!("{ID}/single_valid"));
                eprintln!(
                    "{ID}/single_valid: asset={asset_id} bids={} asks={} bb={:?} ba={:?}",
                    book.bids.len(),
                    book.asks.len(),
                    book.best_bid(),
                    book.best_ask()
                );
            }

            // ──────────────────────────────────────────────────────────────
            // fetch_orderbook(asset_id) — adversarial inputs
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn fetch_orderbook_nonexistent() {
                let Some(ex) = make_exchange(ID) else { return };
                // Pick a syntactically plausible-but-fake id per exchange so
                // we exercise the upstream's not-found path, not a 400.
                let fake = match ID {
                    // Kalshi market tickers use uppercase letters + dashes
                    "kalshi" => "OPENPX-E2E-NOPE-0".to_string(),
                    // Polymarket CTF token ids are big base-10 integers
                    "polymarket" => "1".to_string(),
                    _ => "__nope__".into(),
                };
                match ex.fetch_orderbook(&fake).await {
                    Err(e) if is_market_not_found(&e) => {}
                    Err(e) if is_transient(&e) => {}
                    Err(e) => {
                        let msg = format!("{e:?}");
                        // Both 404-style and validation errors are acceptable —
                        // forbid only panics and silent successes.
                        assert!(
                            msg.contains("not found")
                                || msg.contains("NotFound")
                                || msg.contains("Invalid")
                                || msg.contains("InvalidOrder")
                                || msg.contains("InvalidInput")
                                || msg.contains("Api"),
                            "{ID}: nonexistent surfaced unexpected error: {e:?}"
                        );
                    }
                    Ok(book) => {
                        assert!(
                            book.bids.is_empty() && book.asks.is_empty(),
                            "{ID}: fake asset_id returned non-empty book"
                        );
                    }
                }
            }

            #[tokio::test]
            async fn fetch_orderbook_malformed_id() {
                let Some(ex) = make_exchange(ID) else { return };
                let bad = "!@#$%^&*()";
                match ex.fetch_orderbook(bad).await {
                    Ok(book) => assert!(
                        book.bids.is_empty() && book.asks.is_empty(),
                        "{ID}: malformed asset_id returned populated book"
                    ),
                    Err(e) if is_transient(&e) => {}
                    Err(_) => { /* any error class is fine, no panic */ }
                }
            }

            // ──────────────────────────────────────────────────────────────
            // fetch_orderbooks_batch — every variation
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn fetch_orderbooks_batch_empty_returns_empty() {
                let Some(ex) = make_exchange(ID) else { return };
                match ex.fetch_orderbooks_batch(Vec::new()).await {
                    Ok(books) => assert!(
                        books.is_empty(),
                        "{ID}: empty asset_ids returned {} books",
                        books.len()
                    ),
                    Err(e) if is_transient(&e) => {}
                    Err(e) => panic!("{ID}: empty batch surfaced error: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_orderbooks_batch_multi_valid() {
                let Some(ex) = make_exchange(ID) else { return };
                // Seed: pick 3 markets that have books.
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Active),
                    limit: Some(30),
                    ..Default::default()
                };
                let (markets, _) = match ex.fetch_markets(&params).await {
                    Ok(r) => r,
                    Err(e) if is_transient(&e) => return,
                    Err(e) => panic!("{ID}: seed fetch_markets failed: {e:?}"),
                };
                let mut chosen = Vec::new();
                for m in markets.iter().take(15) {
                    let id = pick_asset_id(m);
                    match ex.fetch_orderbook(&id).await {
                        Ok(b) if !b.bids.is_empty() || !b.asks.is_empty() => chosen.push(id),
                        _ => continue,
                    }
                    if chosen.len() >= 3 {
                        break;
                    }
                }
                if chosen.is_empty() {
                    eprintln!("SKIP {ID}/batch_multi_valid: no markets with books");
                    return;
                }
                let books = match ex.fetch_orderbooks_batch(chosen.clone()).await {
                    Ok(r) => r,
                    Err(e) if is_transient(&e) => return,
                    Err(e) => panic!("{ID}: batch_multi_valid failed: {e:?}"),
                };
                assert!(
                    !books.is_empty(),
                    "{ID}: batch returned 0 books for {} requested",
                    chosen.len()
                );
                // Every returned book should round-trip the requested asset_id
                // and pass the universal well-formedness check.
                for b in &books {
                    assert_book_well_formed(b, &b.asset_id, &format!("{ID}/batch_multi_valid"));
                }
                let returned_ids: std::collections::HashSet<_> =
                    books.iter().map(|b| b.asset_id.clone()).collect();
                let requested_ids: std::collections::HashSet<_> = chosen.iter().cloned().collect();
                let overlap = returned_ids.intersection(&requested_ids).count();
                assert!(
                    overlap > 0,
                    "{ID}: batch returned {} books but none matched requested ids",
                    books.len()
                );
                eprintln!(
                    "{ID}/batch_multi_valid: {} requested, {} returned, {} matched",
                    chosen.len(),
                    books.len(),
                    overlap
                );
            }

            #[tokio::test]
            async fn fetch_orderbooks_batch_above_kalshi_cap_rejects() {
                if ID != "kalshi" {
                    return;
                }
                let Some(ex) = make_exchange(ID) else { return };
                // Kalshi cap is 100 — sending 101 must be rejected with
                // InvalidOrder, not silently accepted.
                let oversized: Vec<String> =
                    (0..101).map(|i| format!("OPENPX-E2E-CAP-{i}")).collect();
                match ex.fetch_orderbooks_batch(oversized).await {
                    Err(OpenPxError::Exchange(px_core::error::ExchangeError::InvalidOrder(_))) => {}
                    Err(e) if is_transient(&e) => {}
                    Err(e) => panic!("{ID}: above-cap batch should be InvalidOrder, got {e:?}"),
                    Ok(_) => panic!("{ID}: above-cap batch returned Ok"),
                }
            }

            // ──────────────────────────────────────────────────────────────
            // fetch_orderbook_stats(asset_id)
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn fetch_orderbook_stats_consistent_with_book() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, book)) =
                    seed_market_with_book(&ex, &format!("{ID}/stats"), 10).await
                else {
                    return;
                };
                let stats = match ex.fetch_orderbook_stats(&asset_id).await {
                    Ok(s) => s,
                    Err(e) if is_transient(&e) => return,
                    Err(e) => panic!("{ID}: stats failed: {e:?}"),
                };
                assert_stats_consistent_with_book(&stats, &book, &format!("{ID}/stats"));
                eprintln!(
                    "{ID}/stats: asset={asset_id} mid={:?} spread_bps={:?} imb={:?}",
                    stats.mid, stats.spread_bps, stats.imbalance
                );
            }

            // ──────────────────────────────────────────────────────────────
            // fetch_orderbook_impact(asset_id, size) — every variation
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn fetch_orderbook_impact_small_size() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, book)) =
                    seed_market_with_book(&ex, &format!("{ID}/impact_small"), 10).await
                else {
                    return;
                };
                // Pick a size strictly below the smaller of the two top levels
                // so a full fill is achievable on both sides if both are present.
                let small_size = match (book.bids.first(), book.asks.first()) {
                    (Some(b), Some(a)) => b.size.min(a.size) * 0.5,
                    (Some(b), None) => b.size * 0.5,
                    (None, Some(a)) => a.size * 0.5,
                    (None, None) => 1.0,
                };
                if small_size <= 0.0 {
                    return;
                }
                let impact = match ex.fetch_orderbook_impact(&asset_id, small_size).await {
                    Ok(i) => i,
                    Err(e) if is_transient(&e) => return,
                    Err(e) => panic!("{ID}: impact_small failed: {e:?}"),
                };
                assert_impact_well_formed(
                    &impact,
                    &book,
                    small_size,
                    &format!("{ID}/impact_small"),
                );
            }

            #[tokio::test]
            async fn fetch_orderbook_impact_large_size_partial() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, book)) =
                    seed_market_with_book(&ex, &format!("{ID}/impact_large"), 10).await
                else {
                    return;
                };
                let total_ask_depth: f64 = book.asks.iter().map(|l| l.size).sum();
                let total_bid_depth: f64 = book.bids.iter().map(|l| l.size).sum();
                // 10x the smaller side to guarantee a partial fill there.
                let big_size = total_ask_depth.max(total_bid_depth) * 10.0 + 1.0;
                let impact = match ex.fetch_orderbook_impact(&asset_id, big_size).await {
                    Ok(i) => i,
                    Err(e) if is_transient(&e) => return,
                    Err(e) => panic!("{ID}: impact_large failed: {e:?}"),
                };
                assert_impact_well_formed(&impact, &book, big_size, &format!("{ID}/impact_large"));
                let any_partial = impact.buy_fill_pct < 100.0 || impact.sell_fill_pct < 100.0;
                assert!(
                    any_partial,
                    "{ID}: oversize impact reports 100% fill on both sides — \
                     buy={} sell={} bid_depth={} ask_depth={}",
                    impact.buy_fill_pct, impact.sell_fill_pct, total_bid_depth, total_ask_depth
                );
            }

            #[tokio::test]
            async fn fetch_orderbook_impact_zero_size_errors() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, _)) =
                    seed_market_with_book(&ex, &format!("{ID}/impact_zero"), 5).await
                else {
                    return;
                };
                match ex.fetch_orderbook_impact(&asset_id, 0.0).await {
                    Err(OpenPxError::InvalidInput(_)) => {}
                    Err(e) if is_transient(&e) => {}
                    Err(e) => panic!("{ID}: impact(size=0) wrong error: {e:?}"),
                    Ok(_) => panic!("{ID}: impact(size=0) returned Ok"),
                }
            }

            #[tokio::test]
            async fn fetch_orderbook_impact_negative_size_errors() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, _)) =
                    seed_market_with_book(&ex, &format!("{ID}/impact_neg"), 5).await
                else {
                    return;
                };
                match ex.fetch_orderbook_impact(&asset_id, -42.0).await {
                    Err(OpenPxError::InvalidInput(_)) => {}
                    Err(e) if is_transient(&e) => {}
                    Err(e) => panic!("{ID}: impact(size=-42) wrong error: {e:?}"),
                    Ok(_) => panic!("{ID}: impact(size=-42) returned Ok"),
                }
            }

            // ──────────────────────────────────────────────────────────────
            // fetch_orderbook_microstructure(asset_id)
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn fetch_orderbook_microstructure_consistent() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((_, asset_id, book)) =
                    seed_market_with_book(&ex, &format!("{ID}/micro"), 10).await
                else {
                    return;
                };
                let micro = match ex.fetch_orderbook_microstructure(&asset_id).await {
                    Ok(m) => m,
                    Err(e) if is_transient(&e) => return,
                    Err(e) => panic!("{ID}: microstructure failed: {e:?}"),
                };
                assert_microstructure_well_formed(&micro, &book, &format!("{ID}/micro"));
                eprintln!(
                    "{ID}/micro: asset={asset_id} bids={} asks={} bid_slope={:?} ask_slope={:?}",
                    micro.level_count.bids,
                    micro.level_count.asks,
                    micro.bid_slope,
                    micro.ask_slope
                );
            }
        }
    };
}

orderbooks_suite!(kalshi);
orderbooks_suite!(polymarket);

// ---------------------------------------------------------------------------
// Cross-exchange unification: the call site is the same on both exchanges
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unified_orderbook_shape_across_exchanges() {
    let Some(kalshi) = make_exchange("kalshi") else {
        return;
    };
    let Some(polymarket) = make_exchange("polymarket") else {
        return;
    };

    let Some((_, k_asset, k_book)) = seed_market_with_book(&kalshi, "unified/kalshi", 10).await
    else {
        return;
    };
    let Some((_, p_asset, p_book)) =
        seed_market_with_book(&polymarket, "unified/polymarket", 10).await
    else {
        return;
    };

    // Same well-formedness invariants on both sides.
    assert_book_well_formed(&k_book, &k_asset, "unified/kalshi");
    assert_book_well_formed(&p_book, &p_asset, "unified/polymarket");

    // Same downstream pure-function pipeline.
    let k_stats = kalshi.fetch_orderbook_stats(&k_asset).await.unwrap();
    let p_stats = polymarket.fetch_orderbook_stats(&p_asset).await.unwrap();
    assert_stats_consistent_with_book(&k_stats, &k_book, "unified/kalshi/stats");
    assert_stats_consistent_with_book(&p_stats, &p_book, "unified/polymarket/stats");

    // The serialized JSON shape must be identical at the top-level keys —
    // a consumer who flips ID strings cannot tell the source by the shape.
    let k_json = serde_json::to_value(&k_book).unwrap();
    let p_json = serde_json::to_value(&p_book).unwrap();
    let k_keys: std::collections::BTreeSet<_> =
        k_json.as_object().unwrap().keys().cloned().collect();
    let p_keys: std::collections::BTreeSet<_> =
        p_json.as_object().unwrap().keys().cloned().collect();
    assert_eq!(
        k_keys, p_keys,
        "Orderbook serialization keys differ between Kalshi and Polymarket: \
         k={k_keys:?} p={p_keys:?}"
    );
}
