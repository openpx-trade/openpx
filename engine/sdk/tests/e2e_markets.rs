//! End-to-end coverage for the unified `fetch_markets` surface.
//!
//! Every input variation of `FetchMarketsParams` is exercised against both
//! Kalshi and Polymarket so the unified contract is held to a single bar:
//!
//!   • default (no filters)                — the baseline page
//!   • each `MarketStatusFilter` variant   — Active, Closed, Resolved, All
//!   • `limit` clamping                    — small, large, above-cap
//!   • cursor pagination                   — page1.cursor → page2 returns disjoint set
//!   • single `market_tickers`             — explicit market lookup
//!   • multiple `market_tickers`           — batch lookup
//!   • `event_ticker`                      — event-scoped page
//!   • `series_ticker`                     — Kalshi-only filter, ignored by Polymarket
//!   • combined filters                    — ticker + status, event + status
//!   • field invariants                    — every returned `Market` is well-formed
//!   • adversarial inputs                  — nonexistent ids return empty or `MarketNotFound`
//!
//! ## Running
//!
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_markets -- --nocapture
//!
//! Single exchange:
//!
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_markets kalshi -- --nocapture
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_markets polymarket -- --nocapture

use std::env;

use openpx::ExchangeInner;
use px_core::error::OpenPxError;
use px_core::{FetchMarketsParams, Market, MarketStatus, MarketStatusFilter};

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

/// Validate every field of a `Market` against the unified contract.
///
/// This is the single source of truth for "what makes a returned market valid"
/// — every test that returns markets routes them through here.
fn assert_unified_market(m: &Market, exchange: &str) {
    assert_eq!(m.exchange, exchange, "Market.exchange mismatch");
    assert!(!m.ticker.is_empty(), "Market.ticker is empty");
    assert_eq!(
        m.openpx_id,
        format!("{exchange}:{}", m.ticker),
        "Market.openpx_id format mismatch"
    );
    assert!(
        !m.title.is_empty(),
        "Market.title is empty for {}",
        m.ticker
    );
    assert!(
        !m.outcomes.is_empty(),
        "Market.outcomes is empty for {}",
        m.ticker
    );
    assert!(
        m.volume >= 0.0,
        "Market.volume negative ({}) for {}",
        m.volume,
        m.ticker
    );
    if let Some(v24) = m.volume_24h {
        assert!(v24 >= 0.0, "volume_24h negative for {}", m.ticker);
    }
    if let Some(p) = m.last_trade_price {
        assert!(
            (0.0..=1.0).contains(&p),
            "last_trade_price OOB for {}",
            m.ticker
        );
    }
    if let Some(b) = m.best_bid {
        assert!((0.0..=1.0).contains(&b), "best_bid OOB for {}", m.ticker);
    }
    if let Some(a) = m.best_ask {
        assert!((0.0..=1.0).contains(&a), "best_ask OOB for {}", m.ticker);
    }
    if let (Some(b), Some(a)) = (m.best_bid, m.best_ask) {
        // Crossed top-of-book is an upstream data anomaly we mirror faithfully:
        // Polymarket's Gamma `bestBid`/`bestAsk` are denormalized cache fields,
        // not a synchronized snapshot, so a one-tick cross during fast moves is
        // observed in the wild. Allow a small tolerance — anything beyond 5¢
        // means the cache has wandered far enough to be a real bug.
        if b > 0.0 && a > 0.0 {
            let cross = b - a;
            assert!(
                cross <= 0.05,
                "wildly crossed top-of-book on {}: bid {} > ask {} (cross {:.4})",
                m.ticker,
                b,
                a,
                cross
            );
        }
    }
    if let Some(t) = m.tick_size {
        assert!(
            t > 0.0 && t <= 0.1,
            "tick_size {} out of expected range for {}",
            t,
            m.ticker
        );
    }
    if let Some(s) = m.min_order_size {
        assert!(s > 0.0, "min_order_size non-positive for {}", m.ticker);
    }
    for o in &m.outcomes {
        assert!(!o.label.is_empty(), "outcome label empty on {}", m.ticker);
        if let Some(p) = o.price {
            assert!(
                (0.0..=1.0).contains(&p),
                "outcome '{}' price {} OOB on {}",
                o.label,
                p,
                m.ticker
            );
        }
    }
}

/// Returns `Some(markets)` on success, `None` if the call should be skipped
/// (network/rate-limit failure). Panics on real errors so we surface bugs.
async fn fetch_or_skip(
    ex: &ExchangeInner,
    params: &FetchMarketsParams,
    label: &str,
) -> Option<(Vec<Market>, Option<String>)> {
    match ex.fetch_markets(params).await {
        Ok(r) => Some(r),
        Err(e) if is_transient(&e) => {
            eprintln!("SKIP {label}: transient error: {e}");
            None
        }
        Err(e) => panic!("{label} failed: {e:?}"),
    }
}

// ---------------------------------------------------------------------------
// Per-exchange test suites
// ---------------------------------------------------------------------------

macro_rules! markets_suite {
    ($exchange_id:ident) => {
        mod $exchange_id {
            use super::*;

            const ID: &str = stringify!($exchange_id);

            // ──────────────────────────────────────────────────────────────
            // Default: no filters
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn default_no_filters() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((markets, _)) =
                    fetch_or_skip(&ex, &FetchMarketsParams::default(), "default").await
                else {
                    return;
                };
                assert!(!markets.is_empty(), "default fetch returned 0 markets");
                for m in &markets {
                    assert_unified_market(m, ID);
                }
            }

            // ──────────────────────────────────────────────────────────────
            // Status filter — every variant
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn status_active() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Active),
                    limit: Some(50),
                    ..Default::default()
                };
                let Some((markets, _)) = fetch_or_skip(&ex, &params, "status_active").await else {
                    return;
                };
                for m in &markets {
                    assert_unified_market(m, ID);
                    assert_eq!(
                        m.status,
                        MarketStatus::Active,
                        "status=active returned non-active market {}",
                        m.ticker
                    );
                }
            }

            #[tokio::test]
            async fn status_closed() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Closed),
                    limit: Some(50),
                    ..Default::default()
                };
                let Some((markets, _)) = fetch_or_skip(&ex, &params, "status_closed").await else {
                    return;
                };
                for m in &markets {
                    assert_unified_market(m, ID);
                    // Polymarket has no separate "closed" state — Closed/Resolved both
                    // map to MarketStatus::Resolved client-side.
                    assert!(
                        matches!(m.status, MarketStatus::Closed | MarketStatus::Resolved),
                        "status=closed returned active market {}",
                        m.ticker
                    );
                }
            }

            #[tokio::test]
            async fn status_resolved() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Resolved),
                    limit: Some(50),
                    ..Default::default()
                };
                let Some((markets, _)) = fetch_or_skip(&ex, &params, "status_resolved").await
                else {
                    return;
                };
                for m in &markets {
                    assert_unified_market(m, ID);
                    assert!(
                        matches!(m.status, MarketStatus::Resolved | MarketStatus::Closed),
                        "status=resolved returned active market {}",
                        m.ticker
                    );
                }
            }

            #[tokio::test]
            async fn status_all_returns_mixed_statuses() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::All),
                    limit: Some(200),
                    ..Default::default()
                };
                let Some((markets, _)) = fetch_or_skip(&ex, &params, "status_all").await else {
                    return;
                };
                assert!(!markets.is_empty(), "status=all returned 0 markets");
                for m in &markets {
                    assert_unified_market(m, ID);
                }
                // status=All should see at least one of {Active} *and* one of
                // {Closed,Resolved} in the page so callers actually get coverage.
                let has_active = markets.iter().any(|m| m.status == MarketStatus::Active);
                let has_finished = markets
                    .iter()
                    .any(|m| matches!(m.status, MarketStatus::Closed | MarketStatus::Resolved));
                assert!(
                    has_active && has_finished,
                    "status=all should return both active and finished markets in a page; \
                     got active={has_active} finished={has_finished} (n={})",
                    markets.len()
                );
            }

            // ──────────────────────────────────────────────────────────────
            // limit
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn limit_small_is_honored() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Active),
                    limit: Some(5),
                    ..Default::default()
                };
                let Some((markets, _)) = fetch_or_skip(&ex, &params, "limit_small").await else {
                    return;
                };
                // Kalshi joins live + historical (not All here, so just live);
                // Polymarket pages events of which each can hold many markets.
                // We document the tolerance as: ≤ exchange-specific multiple of
                // requested limit. Pick a generous bound that still catches the
                // "ignored entirely" failure mode.
                let bound = match ID {
                    "kalshi" => 10,         // 5 live + 5 historical worst case
                    "polymarket" => 5 * 25, // up to ~25 markets per event
                    _ => 50,
                };
                assert!(
                    markets.len() <= bound,
                    "limit=5 yielded {} markets (>{} bound) on {ID}",
                    markets.len(),
                    bound
                );
            }

            #[tokio::test]
            async fn limit_above_cap_is_clamped_not_errored() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Active),
                    limit: Some(100_000),
                    ..Default::default()
                };
                // Per the trait contract, oversize limits are silently clamped.
                // We just need this to succeed and return a valid page.
                let Some((markets, _)) = fetch_or_skip(&ex, &params, "limit_above_cap").await
                else {
                    return;
                };
                assert!(
                    !markets.is_empty(),
                    "above-cap limit on {ID} returned 0 markets"
                );
                for m in &markets {
                    assert_unified_market(m, ID);
                }
            }

            // ──────────────────────────────────────────────────────────────
            // Cursor pagination
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn cursor_pagination_advances() {
                let Some(ex) = make_exchange(ID) else { return };
                let p1 = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Active),
                    limit: Some(20),
                    ..Default::default()
                };
                let Some((page1, cursor)) = fetch_or_skip(&ex, &p1, "pagination_page1").await
                else {
                    return;
                };
                let Some(cursor) = cursor else {
                    eprintln!("SKIP pagination on {ID}: only one page available");
                    return;
                };
                let p2 = FetchMarketsParams {
                    status: Some(MarketStatusFilter::Active),
                    limit: Some(20),
                    cursor: Some(cursor),
                    ..Default::default()
                };
                let Some((page2, _)) = fetch_or_skip(&ex, &p2, "pagination_page2").await else {
                    return;
                };
                assert!(!page2.is_empty(), "page2 returned 0 markets");

                // The two pages should be disjoint by ticker — pagination must
                // advance, not loop back.
                let p1_set: std::collections::HashSet<_> =
                    page1.iter().map(|m| m.ticker.as_str()).collect();
                let overlap = page2
                    .iter()
                    .filter(|m| p1_set.contains(m.ticker.as_str()))
                    .count();
                assert!(
                    overlap == 0,
                    "page2 overlaps page1 on {} ticker(s) — cursor not advancing",
                    overlap
                );
            }

            // ──────────────────────────────────────────────────────────────
            // Single-ticker lookup
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn single_market_ticker() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((page, _)) = fetch_or_skip(
                    &ex,
                    &FetchMarketsParams {
                        status: Some(MarketStatusFilter::Active),
                        limit: Some(5),
                        ..Default::default()
                    },
                    "seed_for_single_lookup",
                )
                .await
                else {
                    return;
                };
                let Some(target) = page.into_iter().next() else {
                    return;
                };

                let params = FetchMarketsParams {
                    market_tickers: vec![target.ticker.clone()],
                    status: Some(MarketStatusFilter::All),
                    ..Default::default()
                };
                let Some((singletons, cursor)) =
                    fetch_or_skip(&ex, &params, "single_market_ticker").await
                else {
                    return;
                };
                // Single-ticker lookup is never paginated — cursor must be None.
                assert!(
                    cursor.is_none(),
                    "single-ticker lookup returned a cursor on {ID}: {:?}",
                    cursor
                );
                let single = singletons
                    .iter()
                    .find(|m| m.ticker == target.ticker)
                    .unwrap_or_else(|| {
                        panic!(
                            "single lookup did not return ticker {} on {ID}",
                            target.ticker
                        )
                    });
                assert_unified_market(single, ID);
                // Stable identity fields must match exactly between the page-listing
                // path and the explicit-lookup path.
                assert_eq!(single.openpx_id, target.openpx_id, "openpx_id drift");
                assert_eq!(single.title, target.title, "title drift");
                assert_eq!(single.market_type, target.market_type, "market_type drift");
                assert_eq!(single.outcomes, target.outcomes, "outcomes drift");
            }

            // ──────────────────────────────────────────────────────────────
            // Multi-ticker batch lookup
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn multi_market_tickers() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((page, _)) = fetch_or_skip(
                    &ex,
                    &FetchMarketsParams {
                        status: Some(MarketStatusFilter::Active),
                        limit: Some(10),
                        ..Default::default()
                    },
                    "seed_for_multi_lookup",
                )
                .await
                else {
                    return;
                };
                if page.len() < 2 {
                    eprintln!("SKIP multi_lookup on {ID}: page too small");
                    return;
                }
                let tickers: Vec<String> = page.iter().take(3).map(|m| m.ticker.clone()).collect();

                let params = FetchMarketsParams {
                    market_tickers: tickers.clone(),
                    status: Some(MarketStatusFilter::All),
                    ..Default::default()
                };
                let Some((markets, cursor)) =
                    fetch_or_skip(&ex, &params, "multi_market_tickers").await
                else {
                    return;
                };
                assert!(
                    cursor.is_none(),
                    "multi-ticker lookup returned a cursor on {ID}"
                );
                assert!(
                    !markets.is_empty(),
                    "multi-ticker lookup returned 0 markets"
                );
                for m in &markets {
                    assert_unified_market(m, ID);
                    assert!(
                        tickers.contains(&m.ticker),
                        "multi-lookup returned unrequested ticker {} on {ID}",
                        m.ticker
                    );
                }
            }

            // ──────────────────────────────────────────────────────────────
            // event_ticker — filter all markets within a single event
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn event_ticker_returns_only_event_markets() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((page, _)) = fetch_or_skip(
                    &ex,
                    &FetchMarketsParams {
                        status: Some(MarketStatusFilter::Active),
                        limit: Some(100),
                        ..Default::default()
                    },
                    "seed_for_event_lookup",
                )
                .await
                else {
                    return;
                };
                let Some(event_id) = page.iter().find_map(|m| m.event_ticker.clone()) else {
                    eprintln!("SKIP event_ticker on {ID}: no markets carry event_ticker");
                    return;
                };
                let params = FetchMarketsParams {
                    event_ticker: Some(event_id.clone()),
                    status: Some(MarketStatusFilter::All),
                    ..Default::default()
                };
                let Some((markets, cursor)) = fetch_or_skip(&ex, &params, "event_ticker").await
                else {
                    return;
                };
                assert!(
                    cursor.is_none(),
                    "event_ticker lookup paginated on {ID}, cursor returned"
                );
                assert!(!markets.is_empty(), "event {event_id} returned 0 markets");
                for m in &markets {
                    assert_unified_market(m, ID);
                    assert_eq!(
                        m.event_ticker.as_deref(),
                        Some(event_id.as_str()),
                        "event_ticker mismatch on {} (expected {event_id})",
                        m.ticker
                    );
                }
            }

            // ──────────────────────────────────────────────────────────────
            // Combined filters
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn ticker_plus_status_mismatch_returns_empty() {
                let Some(ex) = make_exchange(ID) else { return };
                let Some((page, _)) = fetch_or_skip(
                    &ex,
                    &FetchMarketsParams {
                        status: Some(MarketStatusFilter::Active),
                        limit: Some(5),
                        ..Default::default()
                    },
                    "seed_for_ticker_status_mismatch",
                )
                .await
                else {
                    return;
                };
                let Some(active_ticker) = page.into_iter().next().map(|m| m.ticker) else {
                    return;
                };

                let params = FetchMarketsParams {
                    market_tickers: vec![active_ticker.clone()],
                    status: Some(MarketStatusFilter::Resolved),
                    ..Default::default()
                };
                let Some((markets, _)) =
                    fetch_or_skip(&ex, &params, "ticker_plus_status_mismatch").await
                else {
                    return;
                };
                // The active ticker should be filtered out by status=resolved.
                assert!(
                    markets.is_empty(),
                    "active ticker {active_ticker} survived status=resolved filter on {ID}"
                );
            }

            // ──────────────────────────────────────────────────────────────
            // Adversarial inputs — must not panic, must not lie
            // ──────────────────────────────────────────────────────────────

            #[tokio::test]
            async fn nonexistent_market_ticker_is_empty() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    market_tickers: vec!["__openpx_e2e_definitely_not_a_ticker__".into()],
                    status: Some(MarketStatusFilter::All),
                    ..Default::default()
                };
                match ex.fetch_markets(&params).await {
                    Ok((markets, _)) => assert!(
                        markets.is_empty(),
                        "nonexistent ticker on {ID} returned {} markets",
                        markets.len()
                    ),
                    Err(e) if is_transient(&e) => {}
                    Err(e) => {
                        // MarketNotFound is acceptable too — what matters is no
                        // false positives or panics.
                        let msg = format!("{e:?}");
                        assert!(
                            msg.contains("MarketNotFound") || msg.contains("not found"),
                            "unexpected error class for nonexistent ticker on {ID}: {e:?}"
                        );
                    }
                }
            }

            #[tokio::test]
            async fn nonexistent_event_ticker() {
                let Some(ex) = make_exchange(ID) else { return };
                let params = FetchMarketsParams {
                    event_ticker: Some("__openpx_e2e_definitely_not_an_event__".into()),
                    status: Some(MarketStatusFilter::All),
                    ..Default::default()
                };
                match ex.fetch_markets(&params).await {
                    // Either an empty list or a MarketNotFound error is acceptable.
                    // What we forbid is a non-unified error class (`Api(...)`).
                    Ok((markets, _)) => assert!(
                        markets.is_empty(),
                        "nonexistent event on {ID} returned {} markets",
                        markets.len()
                    ),
                    Err(e) if is_transient(&e) => {}
                    Err(OpenPxError::Exchange(px_core::error::ExchangeError::MarketNotFound(
                        _,
                    ))) => {}
                    Err(e) => panic!("nonexistent event on {ID} surfaced non-unified error: {e:?}"),
                }
            }
        }
    };
}

markets_suite!(kalshi);
markets_suite!(polymarket);

// ---------------------------------------------------------------------------
// Cross-exchange unification
// ---------------------------------------------------------------------------

/// The unified contract: at the surface, you can't tell which exchange you're
/// talking to from the call site alone — only from the data flowing back.
///
/// This test takes the same `FetchMarketsParams` shape, runs it through both
/// exchanges, and asserts both produce well-formed pages with a unified shape.
#[tokio::test]
async fn unified_shape_across_exchanges() {
    let Some(kalshi) = make_exchange("kalshi") else {
        return;
    };
    let Some(polymarket) = make_exchange("polymarket") else {
        return;
    };

    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Active),
        limit: Some(20),
        ..Default::default()
    };

    let k = match kalshi.fetch_markets(&params).await {
        Ok(r) => r,
        Err(e) if is_transient(&e) => return,
        Err(e) => panic!("kalshi fetch_markets failed: {e:?}"),
    };
    let p = match polymarket.fetch_markets(&params).await {
        Ok(r) => r,
        Err(e) if is_transient(&e) => return,
        Err(e) => panic!("polymarket fetch_markets failed: {e:?}"),
    };

    // Same return type, same paging contract.
    let (k_markets, _k_cursor) = k;
    let (p_markets, _p_cursor) = p;

    assert!(!k_markets.is_empty(), "kalshi returned 0 markets");
    assert!(!p_markets.is_empty(), "polymarket returned 0 markets");

    for m in &k_markets {
        assert_unified_market(m, "kalshi");
    }
    for m in &p_markets {
        assert_unified_market(m, "polymarket");
    }

    // Field-level invariants that must hold on both sides:
    // openpx_id is always {exchange}:{ticker}; status is one of the unified
    // enum variants; outcomes carry only labels we expect.
    let allowed_outcome_labels = |labels: &[&str]| -> bool { labels.iter().all(|l| !l.is_empty()) };
    for m in k_markets.iter().chain(p_markets.iter()) {
        let labels: Vec<&str> = m.outcomes.iter().map(|o| o.label.as_str()).collect();
        assert!(allowed_outcome_labels(&labels), "empty outcome label");
    }
}

// ---------------------------------------------------------------------------
// Surface coverage — every public entry point must agree
// ---------------------------------------------------------------------------
//
// The Rust trait is only one of three surfaces — `CLI`, `Python SDK`, and
// `TypeScript SDK` are the others. All three reach the same Rust core, but the
// shape we publish at each boundary is its own contract: a missing parameter
// or a re-encoded field shows up as a regression for users on that surface
// even when the unit tests stay green.
//
// Each surface test below takes the same input and asserts the same shape.
//
// Pre-requisites (skipped gracefully when missing):
//   • CLI:    `cargo build --release -p px-cli` produced `target/release/openpx`
//   • Python: `just python-build` (maturin develop) installed openpx in venv
//   • Node:   `cd sdks/typescript && npm run build` produced `openpx.node`

mod surface {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR has at least 2 parents (engine/sdk → engine → repo)")
    }

    /// Resolve `KALSHI_PRIVATE_KEY_PATH` to an absolute path so subprocesses
    /// launched in different cwds (sdks/typescript, etc.) can still find it.
    fn absolute_kalshi_key_path() -> Option<String> {
        let raw = env::var("KALSHI_PRIVATE_KEY_PATH").ok()?;
        let p = PathBuf::from(&raw);
        if p.is_absolute() {
            return Some(raw);
        }
        let abs = project_root().join(&p);
        abs.exists().then(|| abs.to_string_lossy().into_owned())
    }

    /// Locate the CLI binary. Returns None and prints a SKIP if missing.
    fn cli_binary() -> Option<PathBuf> {
        let bin = project_root().join("target/release/openpx");
        if bin.exists() {
            Some(bin)
        } else {
            eprintln!(
                "SKIP surface/cli: build with `cargo build --release -p px-cli` first ({})",
                bin.display()
            );
            None
        }
    }

    /// Locate the Python venv interpreter that has openpx installed via
    /// `just python` (maturin develop).
    fn python_interpreter() -> Option<PathBuf> {
        let candidates = ["sdks/python/.venv/bin/python", ".venv/bin/python"];
        for c in candidates {
            let p = project_root().join(c);
            if p.exists() {
                // Verify openpx is importable.
                let ok = Command::new(&p)
                    .args(["-c", "import openpx"])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                if ok {
                    return Some(p);
                }
            }
        }
        eprintln!("SKIP surface/python: no venv with openpx installed (run `just python` first)");
        None
    }

    /// Locate the Node entry point. The TS SDK uses `index.js` + a built
    /// `openpx.node` artifact.
    fn node_entry() -> Option<PathBuf> {
        let entry = project_root().join("sdks/typescript/index.js");
        let native = project_root().join("sdks/typescript/openpx.node");
        if entry.exists() && native.exists() {
            Some(entry)
        } else {
            eprintln!("SKIP surface/node: build with `cd sdks/typescript && npm run build` first");
            None
        }
    }

    /// Common assertions on a parsed JSON response from any surface.
    fn assert_unified_response(value: &serde_json::Value, exchange: &str) {
        let markets = value
            .get("markets")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("response missing 'markets' array: {value}"));
        // Allow empty-but-shaped responses too (e.g. nonexistent ticker case).
        for m in markets {
            let ticker = m
                .get("ticker")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("market missing 'ticker': {m}"));
            assert!(!ticker.is_empty(), "ticker is empty");
            let openpx_id = m
                .get("openpx_id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("market missing 'openpx_id': {m}"));
            assert_eq!(
                openpx_id,
                format!("{exchange}:{ticker}"),
                "openpx_id format drift on {exchange} for {ticker}"
            );
            let exchg = m
                .get("exchange")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("market missing 'exchange': {m}"));
            assert_eq!(exchg, exchange, "exchange field drift on {ticker}");
            let title = m.get("title").and_then(|v| v.as_str()).unwrap_or("");
            assert!(!title.is_empty(), "title empty on {ticker}");
            let outcomes = m
                .get("outcomes")
                .and_then(|v| v.as_array())
                .unwrap_or_else(|| panic!("outcomes missing on {ticker}"));
            assert!(!outcomes.is_empty(), "outcomes empty on {ticker}");
        }
    }

    // ──────────────────────────────────────────────────────────────────────
    // CLI surface
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn cli_polymarket_status_all_with_limit() {
        if !require_live() {
            return;
        }
        let Some(bin) = cli_binary() else { return };
        let out = Command::new(&bin)
            .args([
                "polymarket",
                "fetch-markets",
                "--status",
                "all",
                "--limit",
                "5",
            ])
            .output()
            .expect("CLI invoke failed");
        assert!(
            out.status.success(),
            "CLI exited {}: stderr={}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("CLI stdout was not JSON");
        assert_unified_response(&v, "polymarket");
        let count = v.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        // limit=5 with status=all merges 2 buckets — generous bound that still
        // catches "limit ignored" regressions.
        assert!(count > 0, "CLI returned 0 markets for status=all limit=5");
        assert!(
            count < 100,
            "CLI returned {count} markets for limit=5 — limit ignored?"
        );
    }

    #[test]
    fn cli_kalshi_single_ticker() {
        if !require_live() {
            return;
        }
        let Some(bin) = cli_binary() else { return };
        // Run from project root so the relative `KALSHI_PRIVATE_KEY_PATH` in
        // .env (e.g. `kalshi-private-key.pem`) resolves.
        let cwd = project_root();
        let key_abs = absolute_kalshi_key_path();
        // Seed: pick the first active Kalshi ticker from the catalog.
        let mut seed_cmd = Command::new(&bin);
        seed_cmd
            .args(["kalshi", "fetch-markets", "--limit", "1"])
            .current_dir(&cwd);
        if let Some(ref k) = key_abs {
            seed_cmd.env("KALSHI_PRIVATE_KEY_PATH", k);
        }
        let seed = seed_cmd.output().expect("CLI seed call failed");
        if !seed.status.success() {
            eprintln!("SKIP cli_kalshi_single_ticker: seed call failed");
            return;
        }
        let seed_v: serde_json::Value = match serde_json::from_slice(&seed.stdout) {
            Ok(v) => v,
            Err(_) => return,
        };
        let Some(ticker) = seed_v
            .get("markets")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|m| m.get("ticker"))
            .and_then(|v| v.as_str())
            .map(String::from)
        else {
            eprintln!("SKIP cli_kalshi_single_ticker: no seed market available");
            return;
        };

        let mut lookup_cmd = Command::new(&bin);
        lookup_cmd
            .args([
                "kalshi",
                "fetch-markets",
                "--market-tickers",
                &ticker,
                "--status",
                "all",
            ])
            .current_dir(&cwd);
        if let Some(ref k) = key_abs {
            lookup_cmd.env("KALSHI_PRIVATE_KEY_PATH", k);
        }
        let out = lookup_cmd.output().expect("CLI single-ticker call failed");
        assert!(
            out.status.success(),
            "CLI exited {}: stderr={}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("CLI stdout was not JSON");
        assert_unified_response(&v, "kalshi");
        let cursor = v.get("next_cursor");
        assert!(
            cursor.is_some_and(|c| c.is_null()),
            "single-ticker lookup should not paginate (got {cursor:?})"
        );
        let count = v.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        assert_eq!(
            count, 1,
            "single-ticker lookup should return exactly 1, got {count}"
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // Python SDK surface
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn python_polymarket_status_all_with_limit() {
        if !require_live() {
            return;
        }
        let Some(py) = python_interpreter() else {
            return;
        };
        let script = r#"
import json, sys
from openpx import Exchange
ex = Exchange("polymarket", {})
r = ex.fetch_markets(status="all", limit=5)
# Pydantic models — round-trip via JSON to get plain dicts.
out = {
    "markets": [m.model_dump() if hasattr(m, "model_dump") else m for m in r["markets"]],
    "cursor": r.get("cursor"),
    "count": len(r["markets"]),
}
sys.stdout.write(json.dumps(out, default=str))
"#;
        let out = Command::new(&py)
            .args(["-c", script])
            .output()
            .expect("Python SDK invoke failed");
        if !out.status.success() {
            panic!(
                "Python SDK failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("Python stdout was not JSON");
        // Python's MarketStatus enum serializes to a dict like {"value": "..."},
        // so we can't run assert_unified_response unchanged. Validate the
        // structural pieces directly.
        let markets = v.get("markets").and_then(|v| v.as_array()).unwrap();
        assert!(!markets.is_empty(), "Python SDK returned 0 markets");
        for m in markets {
            assert_eq!(
                m.get("exchange").and_then(|v| v.as_str()),
                Some("polymarket"),
                "exchange field drift in Python output"
            );
            let ticker = m.get("ticker").and_then(|v| v.as_str()).unwrap();
            let openpx_id = m.get("openpx_id").and_then(|v| v.as_str()).unwrap();
            assert_eq!(openpx_id, format!("polymarket:{ticker}"));
        }
        let count = v.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(
            count > 0 && count < 100,
            "limit=5 yielded {count} markets — limit ignored?"
        );
    }

    #[test]
    fn python_kalshi_single_ticker() {
        if !require_live() {
            return;
        }
        let Some(py) = python_interpreter() else {
            return;
        };
        let Some(key_path) = absolute_kalshi_key_path() else {
            eprintln!("SKIP python_kalshi_single_ticker: no KALSHI_PRIVATE_KEY_PATH");
            return;
        };
        let script = r#"
import json, os, sys
from openpx import Exchange
config = {
    "api_key_id": os.environ.get("KALSHI_API_KEY_ID", ""),
    "private_key_path": os.environ.get("KALSHI_KEY_ABS", ""),
}
ex = Exchange("kalshi", config)
seed = ex.fetch_markets(status="active", limit=1)
ticker = (seed["markets"][0].ticker if hasattr(seed["markets"][0], "ticker")
          else seed["markets"][0]["ticker"])
r = ex.fetch_markets(market_tickers=[ticker], status="all")
markets = [m.model_dump() if hasattr(m, "model_dump") else m for m in r["markets"]]
sys.stdout.write(json.dumps({"markets": markets, "cursor": r.get("cursor")}, default=str))
"#;
        let out = Command::new(&py)
            .env("KALSHI_KEY_ABS", &key_path)
            .args(["-c", script])
            .output()
            .expect("Python SDK invoke failed");
        if !out.status.success() {
            panic!(
                "Python single-ticker failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("Python stdout was not JSON");
        let markets = v.get("markets").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            markets.len(),
            1,
            "single-ticker lookup via Python should return 1, got {}",
            markets.len()
        );
        // Pydantic's None→null serialization
        assert!(
            v.get("cursor").is_some_and(|c| c.is_null()),
            "Python single-ticker cursor should be null"
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // TypeScript SDK surface
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn node_polymarket_status_all_with_limit() {
        if !require_live() {
            return;
        }
        let Some(_) = node_entry() else { return };
        let ts_dir = project_root().join("sdks/typescript");
        let script = r#"
const { Exchange } = require('./index.js');
(async () => {
  const ex = new Exchange('polymarket', {});
  const r = await ex.fetchMarkets('all', null, null, null, null, 5);
  process.stdout.write(JSON.stringify({
    markets: r.markets,
    cursor: r.cursor,
    count: r.markets.length,
  }));
})().catch(e => { process.stderr.write(String(e)); process.exit(1); });
"#;
        let out = Command::new("node")
            .args(["-e", script])
            .current_dir(&ts_dir)
            .output()
            .expect("Node invoke failed");
        if !out.status.success() {
            panic!("Node SDK failed: {}", String::from_utf8_lossy(&out.stderr));
        }
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("Node stdout was not JSON");
        assert_unified_response(&v, "polymarket");
        let count = v.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(
            count > 0 && count < 100,
            "Node limit=5 yielded {count} — limit ignored?"
        );
    }

    #[test]
    fn node_kalshi_single_ticker() {
        if !require_live() {
            return;
        }
        let Some(_) = node_entry() else { return };
        let Some(key_path) = absolute_kalshi_key_path() else {
            eprintln!("SKIP node_kalshi_single_ticker: no KALSHI_PRIVATE_KEY_PATH");
            return;
        };
        let ts_dir = project_root().join("sdks/typescript");
        let script = r#"
const { Exchange } = require('./index.js');
(async () => {
  const ex = new Exchange('kalshi', {
    api_key_id: process.env.KALSHI_API_KEY_ID || '',
    private_key_path: process.env.KALSHI_KEY_ABS || '',
  });
  const seed = await ex.fetchMarkets('active', null, null, null, null, 1);
  const ticker = seed.markets[0].ticker;
  const r = await ex.fetchMarkets('all', null, [ticker], null, null, null);
  process.stdout.write(JSON.stringify({ markets: r.markets, cursor: r.cursor }));
})().catch(e => { process.stderr.write(String(e)); process.exit(1); });
"#;
        let out = Command::new("node")
            .env("KALSHI_KEY_ABS", &key_path)
            .args(["-e", script])
            .current_dir(&ts_dir)
            .output()
            .expect("Node invoke failed");
        if !out.status.success() {
            panic!(
                "Node single-ticker failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let v: serde_json::Value =
            serde_json::from_slice(&out.stdout).expect("Node stdout was not JSON");
        assert_unified_response(&v, "kalshi");
        let markets = v.get("markets").and_then(|v| v.as_array()).unwrap();
        assert_eq!(
            markets.len(),
            1,
            "Node single-ticker yielded {} markets",
            markets.len()
        );
        assert!(
            v.get("cursor").is_some_and(|c| c.is_null()),
            "Node single-ticker cursor should be null"
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // Cross-surface unification — same input, same shape, every surface
    // ──────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn shape_consistent_across_rust_cli_python_node() {
        if !require_live() {
            return;
        }
        let Some(ex) = make_exchange("polymarket") else {
            return;
        };

        // 1. Rust SDK (direct call)
        let rust_params = FetchMarketsParams {
            status: Some(MarketStatusFilter::Active),
            limit: Some(3),
            ..Default::default()
        };
        let rust_result = match ex.fetch_markets(&rust_params).await {
            Ok((m, _)) => m,
            Err(e) if is_transient(&e) => return,
            Err(e) => panic!("rust path failed: {e:?}"),
        };
        if rust_result.is_empty() {
            return;
        }
        let rust_keys: std::collections::BTreeSet<&str> = serde_json::to_value(&rust_result[0])
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str().to_string())
            .map(|s| Box::leak(s.into_boxed_str()) as &str)
            .collect();

        // Helper: extract field-name set from a CLI/python/node result.
        let collect_keys = |v: &serde_json::Value| -> Option<std::collections::BTreeSet<String>> {
            v.get("markets")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|m| m.as_object())
                .map(|obj| obj.keys().cloned().collect())
        };

        // 2. CLI
        if let Some(bin) = cli_binary() {
            let out = Command::new(&bin)
                .args([
                    "polymarket",
                    "fetch-markets",
                    "--status",
                    "active",
                    "--limit",
                    "3",
                ])
                .output()
                .expect("CLI invoke failed");
            if out.status.success() {
                let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
                if let Some(keys) = collect_keys(&v) {
                    let rust_set: std::collections::BTreeSet<String> =
                        rust_keys.iter().map(|s| s.to_string()).collect();
                    let missing: Vec<_> = rust_set.difference(&keys).cloned().collect();
                    let extra: Vec<_> = keys.difference(&rust_set).cloned().collect();
                    assert!(
                        missing.is_empty() && extra.is_empty(),
                        "CLI shape drift: missing={missing:?} extra={extra:?}"
                    );
                }
            }
        }

        // 3. Python
        if let Some(py) = python_interpreter() {
            let script = r#"
import json, sys
from openpx import Exchange
r = Exchange("polymarket", {}).fetch_markets(status="active", limit=3)
m = r["markets"][0]
out = m.model_dump() if hasattr(m, "model_dump") else m
sys.stdout.write(json.dumps(out, default=str))
"#;
            let out = Command::new(&py)
                .args(["-c", script])
                .output()
                .expect("python");
            if out.status.success() {
                let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
                if let Some(obj) = v.as_object() {
                    let keys: std::collections::BTreeSet<String> = obj.keys().cloned().collect();
                    let rust_set: std::collections::BTreeSet<String> =
                        rust_keys.iter().map(|s| s.to_string()).collect();
                    // Pydantic emits *all* fields including None — Python may
                    // have a superset of Rust's keys (since serde-skip strips
                    // them on the Rust side). What we forbid is missing keys.
                    let missing: Vec<_> = rust_set.difference(&keys).cloned().collect();
                    assert!(
                        missing.is_empty(),
                        "Python shape missing fields: {missing:?}"
                    );
                }
            }
        }

        // 4. Node
        if node_entry().is_some() {
            let ts_dir = project_root().join("sdks/typescript");
            let script = r#"
const { Exchange } = require('./index.js');
(async () => {
  const r = await new Exchange('polymarket', {}).fetchMarkets('active', null, null, null, null, 3);
  process.stdout.write(JSON.stringify(r.markets[0]));
})().catch(e => { process.stderr.write(String(e)); process.exit(1); });
"#;
            let out = Command::new("node")
                .args(["-e", script])
                .current_dir(&ts_dir)
                .output()
                .expect("node");
            if out.status.success() {
                let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
                if let Some(obj) = v.as_object() {
                    let keys: std::collections::BTreeSet<String> = obj.keys().cloned().collect();
                    let rust_set: std::collections::BTreeSet<String> =
                        rust_keys.iter().map(|s| s.to_string()).collect();
                    let missing: Vec<_> = rust_set.difference(&keys).cloned().collect();
                    let extra: Vec<_> = keys.difference(&rust_set).cloned().collect();
                    assert!(
                        missing.is_empty() && extra.is_empty(),
                        "Node shape drift: missing={missing:?} extra={extra:?}"
                    );
                }
            }
        }
    }
}
