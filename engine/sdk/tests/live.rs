//! Live integration tests against real exchange APIs.
//!
//! These tests hit production endpoints — they are **not** run in CI.
//!
//! ## Running
//!
//! Unauthenticated (market data only):
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test live -- --nocapture
//!
//! Single exchange:
//!   OPENPX_LIVE_TESTS=1 cargo test -p openpx --test live kalshi -- --nocapture
//!
//! With auth (enables balance/position/fill tests):
//!   OPENPX_LIVE_TESTS=1 \
//!   KALSHI_API_KEY_ID=... KALSHI_PRIVATE_KEY_PEM=... \
//!   cargo test -p openpx --test live kalshi -- --nocapture

use std::env;
use std::time::Duration;

use openpx::{ExchangeInner, WebSocketInner};
use px_core::{OrderbookRequest, PriceHistoryInterval, PriceHistoryRequest, TradesRequest};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Skip the test unless OPENPX_LIVE_TESTS=1.
/// Automatically loads `.env` from the workspace root so contributors
/// only need to set credentials there.
fn require_live() -> bool {
    // Load .env once (no-op if already loaded or file missing)
    let _ = dotenvy::dotenv();
    env::var("OPENPX_LIVE_TESTS").is_ok_and(|v| v == "1")
}

/// Build config JSON from env vars for a given exchange.
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
        "opinion" => &[
            ("OPINION_API_KEY", "api_key"),
            ("OPINION_PRIVATE_KEY", "private_key"),
            ("OPINION_MULTI_SIG_ADDR", "multi_sig_addr"),
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

/// Build an exchange from env vars. Returns None if required vars missing.
fn make_exchange(id: &str) -> Option<ExchangeInner> {
    ExchangeInner::new(id, make_exchange_config(id)).ok()
}

/// Returns true if the exchange has auth credentials configured.
fn is_authenticated(exchange: &ExchangeInner) -> bool {
    let info = exchange.describe();
    info.has_create_order
}

/// Helper: returns true if error message indicates auth failure.
fn is_auth_error(e: &px_core::error::OpenPxError) -> bool {
    let msg = format!("{e:?}");
    msg.contains("uthent") || msg.contains("Unauthorized") || msg.contains("403")
}

/// Helper: returns true if error indicates API unreachable.
fn is_network_error(e: &px_core::error::OpenPxError) -> bool {
    let msg = format!("{e:?}");
    msg.contains("http error") || msg.contains("timed out") || msg.contains("connection")
}

/// Helper: returns true if error indicates not-supported.
fn is_not_supported(e: &px_core::error::OpenPxError) -> bool {
    format!("{e:?}").contains("NotSupported")
}

/// Helper: returns true if error indicates invalid input.
fn is_invalid_input(e: &px_core::error::OpenPxError) -> bool {
    format!("{e:?}").contains("InvalidInput") || format!("{e:?}").contains("invalid input")
}

/// Helper: returns true if error indicates a config or SDK initialization failure.
fn is_config_error(e: &px_core::error::OpenPxError) -> bool {
    let msg = format!("{e:?}");
    msg.contains("config error")
        || msg.contains("Validation")
        || msg.contains("private key required")
}

/// Helper: returns true if error indicates rate limiting.
fn is_rate_limited(e: &px_core::error::OpenPxError) -> bool {
    let msg = format!("{e:?}");
    msg.contains("rate limit") || msg.contains("429") || msg.contains("RateLimited")
}

/// Derive the wallet/owner address for an exchange from env vars.
/// Returns None if the exchange doesn't have address-based activity or no address is configured.
fn owner_address_for(id: &str) -> Option<String> {
    match id {
        "polymarket" => env::var("POLYMARKET_FUNDER")
            .or_else(|_| env::var("POLYMARKET_WALLET_ADDRESS"))
            .ok(),
        "opinion" => env::var("OPINION_MULTI_SIG_ADDR").ok(),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Macros for per-exchange test generation
// ---------------------------------------------------------------------------

macro_rules! exchange_tests {
    ($exchange_id:ident) => {
        mod $exchange_id {
            use super::*;

            fn get_exchange() -> Option<ExchangeInner> {
                if !require_live() {
                    return None;
                }
                make_exchange(stringify!($exchange_id))
            }

            // ==================================================================
            // Unauthenticated — Exchange metadata
            // ==================================================================

            #[tokio::test]
            async fn describe() {
                let Some(ex) = get_exchange() else { return };
                let info = ex.describe();
                assert_eq!(info.id, stringify!($exchange_id));
                assert!(!info.name.is_empty());
                assert!(info.has_fetch_markets);
                // All exchanges should support single market fetch
                assert!(
                    info.has_fetch_markets,
                    "all exchanges must support fetch_markets"
                );
            }

            #[tokio::test]
            async fn describe_capabilities_consistent() {
                let Some(ex) = get_exchange() else { return };
                let info = ex.describe();
                // If an exchange has create_order, it must also have cancel_order
                if info.has_create_order {
                    assert!(
                        info.has_cancel_order,
                        "exchange with create_order must support cancel_order"
                    );
                }
                // If an exchange has websocket, it should report so
                // (just verify the field exists and is a bool — no panic)
                let _ = info.has_websocket;
            }

            // ==================================================================
            // Unauthenticated — Market data
            // ==================================================================

            #[tokio::test]
            async fn fetch_markets() {
                let Some(ex) = get_exchange() else { return };
                let result = ex.fetch_markets(&Default::default()).await;
                let (markets, cursor) = match result {
                    Ok(r) => r,
                    Err(e) if is_auth_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_markets: requires auth",
                            stringify!($exchange_id)
                        );
                        return;
                    }
                    Err(e) if is_network_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_markets: API unreachable: {e}",
                            stringify!($exchange_id)
                        );
                        return;
                    }
                    Err(e) => panic!("fetch_markets failed: {e:?}"),
                };
                eprintln!(
                    "\n=== {}: {} markets, cursor={:?} ===",
                    stringify!($exchange_id),
                    markets.len(),
                    cursor,
                );
                for m in markets.iter().take(3) {
                    eprintln!(
                        "  {} | {:50} | status={:?} | group={:?} | vol={:.0}",
                        m.id, m.title, m.status, m.group_id, m.volume
                    );
                }
                if markets.len() > 3 {
                    eprintln!("  ... and {} more", markets.len() - 3);
                }
                assert!(!markets.is_empty(), "expected at least 1 market");
                for m in &markets {
                    assert!(!m.id.is_empty(), "market id should not be empty");
                    assert!(!m.title.is_empty(), "market title should not be empty");
                    assert!(!m.outcomes.is_empty(), "market should have outcomes");
                }
            }

            #[tokio::test]
            async fn fetch_markets_field_invariants() {
                let Some(ex) = get_exchange() else { return };
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };

                for m in &markets {
                    // openpx_id must be {exchange}:{native_id}
                    let expected_openpx = format!("{}:{}", stringify!($exchange_id), m.id);
                    assert_eq!(
                        m.openpx_id, expected_openpx,
                        "openpx_id format mismatch for market {}",
                        m.id
                    );

                    // exchange field must match
                    assert_eq!(
                        m.exchange,
                        stringify!($exchange_id),
                        "exchange field mismatch for market {}",
                        m.id
                    );

                    // outcomes should be non-empty
                    assert!(
                        !m.outcomes.is_empty(),
                        "market {} should have outcomes",
                        m.id
                    );

                    // binary markets should have exactly 2 outcomes
                    if m.outcomes.len() == 2 {
                        assert!(m.is_binary(), "2-outcome market should report is_binary()");
                    }

                    // volume should be non-negative
                    assert!(
                        m.volume >= 0.0,
                        "market {} volume should be non-negative, got {}",
                        m.id,
                        m.volume
                    );

                    // outcome_prices values should be in [0, 1] range
                    for (outcome, price) in &m.outcome_prices {
                        assert!(
                            *price >= 0.0 && *price <= 1.0,
                            "market {} outcome '{}' price {} out of [0,1] range",
                            m.id,
                            outcome,
                            price
                        );
                    }

                    // tick_size, if present, should be positive and small
                    if let Some(tick) = m.tick_size {
                        assert!(
                            tick > 0.0 && tick <= 0.1,
                            "market {} tick_size {} out of expected range",
                            m.id,
                            tick
                        );
                    }
                }
            }

            #[tokio::test]
            async fn fetch_market_single() {
                let Some(ex) = get_exchange() else { return };
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) => m,
                    Err(_) => return, // covered by fetch_markets test
                };
                if markets.is_empty() {
                    return;
                }
                match ex.fetch_market(&markets[0].id).await {
                    Ok(single) => {
                        assert_eq!(single.id, markets[0].id);
                    }
                    Err(e) if is_rate_limited(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_market_single: rate limited",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_market failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_market_consistency() {
                let Some(ex) = get_exchange() else { return };
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };
                // Pick a market from the list and fetch individually
                let target = &markets[0];
                let single = match ex.fetch_market(&target.id).await {
                    Ok(m) => m,
                    Err(_) => return,
                };

                // Core identity fields must match
                assert_eq!(single.id, target.id, "id mismatch");
                assert_eq!(single.openpx_id, target.openpx_id, "openpx_id mismatch");
                assert_eq!(single.exchange, target.exchange, "exchange mismatch");
                assert_eq!(single.title, target.title, "title mismatch");
                assert_eq!(single.outcomes, target.outcomes, "outcomes mismatch");
                assert_eq!(
                    single.market_type, target.market_type,
                    "market_type mismatch"
                );
            }

            #[tokio::test]
            async fn fetch_market_invalid_id() {
                let Some(ex) = get_exchange() else { return };
                let result = ex.fetch_market("__nonexistent_market_id_12345__").await;
                // Should return an error, not panic
                assert!(
                    result.is_err(),
                    "fetch_market with invalid id should return error"
                );
            }

            // ==================================================================
            // Unauthenticated — Orderbook
            // ==================================================================

            #[tokio::test]
            async fn fetch_orderbook() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_orderbook {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };

                // Prefer a binary market (no outcome required); fall back to first market
                // with an explicit outcome for non-binary markets.
                let (market_id, outcome) = if let Some(m) = markets.iter().find(|m| m.is_binary()) {
                    (m.id.clone(), None)
                } else {
                    let m = &markets[0];
                    (m.id.clone(), m.outcomes.first().cloned())
                };

                match ex
                    .fetch_orderbook(OrderbookRequest {
                        market_id,
                        outcome,
                        token_id: None,
                    })
                    .await
                {
                    Ok(book) => {
                        let _ = &book.bids;
                        let _ = &book.asks;
                    }
                    Err(e) if is_auth_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_orderbook: requires auth",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if format!("{e:?}").contains("NotFound") => {
                        eprintln!(
                            "SKIP {}/fetch_orderbook: market not found on CLOB",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if is_invalid_input(&e) => {
                        eprintln!("SKIP {}/fetch_orderbook: {e}", stringify!($exchange_id));
                    }
                    Err(e) if is_rate_limited(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_orderbook: rate limited",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_orderbook failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_orderbook_structure() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_orderbook {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };

                // Try a few markets to find one with orderbook data
                for market in markets.iter().take(5) {
                    let book = match ex
                        .fetch_orderbook(OrderbookRequest {
                            market_id: market.id.clone(),
                            outcome: None,
                            token_id: None,
                        })
                        .await
                    {
                        Ok(b) if b.has_data() => b,
                        _ => continue,
                    };

                    // Bids should be sorted descending (highest first)
                    for window in book.bids.windows(2) {
                        assert!(
                            window[0].price >= window[1].price,
                            "bids should be sorted descending: {} >= {}",
                            window[0].price,
                            window[1].price
                        );
                    }

                    // Asks should be sorted ascending (lowest first)
                    for window in book.asks.windows(2) {
                        assert!(
                            window[0].price <= window[1].price,
                            "asks should be sorted ascending: {} <= {}",
                            window[0].price,
                            window[1].price
                        );
                    }

                    // All prices should be in [0, 1] range
                    for level in book.bids.iter().chain(book.asks.iter()) {
                        let price = level.price.to_f64();
                        assert!(
                            (0.0..=1.0).contains(&price),
                            "orderbook price {} out of [0,1] range",
                            price
                        );
                        assert!(level.size > 0.0, "orderbook level size should be positive");
                    }

                    // Best bid should be <= best ask (no crossed book)
                    if let (Some(best_bid), Some(best_ask)) = (book.best_bid(), book.best_ask()) {
                        assert!(
                            best_bid <= best_ask,
                            "crossed book: best_bid {} > best_ask {}",
                            best_bid,
                            best_ask
                        );
                    }

                    // Spread should be non-negative
                    if let Some(spread) = book.spread() {
                        assert!(
                            spread >= 0.0,
                            "spread should be non-negative, got {}",
                            spread
                        );
                    }

                    // Mid price should be between bid and ask
                    if let (Some(best_bid), Some(mid), Some(best_ask)) =
                        (book.best_bid(), book.mid_price(), book.best_ask())
                    {
                        assert!(
                            mid >= best_bid && mid <= best_ask,
                            "mid {} not between bid {} and ask {}",
                            mid,
                            best_bid,
                            best_ask
                        );
                    }

                    // Found a good book, test passed
                    return;
                }

                eprintln!(
                    "WARN: {}/fetch_orderbook_structure: no market with orderbook data found",
                    stringify!($exchange_id)
                );
            }

            // ==================================================================
            // Unauthenticated — Historical data
            // ==================================================================

            #[tokio::test]
            async fn fetch_price_history() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_price_history {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };
                // Try multiple markets — first one may be newly created with no history
                let mut found_candles = false;
                for market in &markets {
                    match ex
                        .fetch_price_history(PriceHistoryRequest {
                            market_id: market.id.clone(),
                            interval: PriceHistoryInterval::OneDay,
                            outcome: None,
                            token_id: None,
                            condition_id: None,
                            start_ts: None,
                            end_ts: None,
                        })
                        .await
                    {
                        Ok(candles) if !candles.is_empty() => {
                            for c in &candles {
                                assert!(c.high >= c.low, "high should >= low");
                            }
                            found_candles = true;
                            break;
                        }
                        Ok(_) => continue, // empty, try next market
                        Err(e) if format!("{e:?}").contains("token_id") => {
                            eprintln!(
                                "SKIP {}/fetch_price_history: requires token_id",
                                stringify!($exchange_id)
                            );
                            return;
                        }
                        Err(_) => continue,
                    }
                }
                if !found_candles {
                    eprintln!(
                        "WARN: {}/fetch_price_history: no candles from any of {} markets",
                        stringify!($exchange_id),
                        markets.len()
                    );
                }
            }

            #[tokio::test]
            async fn fetch_price_history_candle_invariants() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_price_history {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };

                for market in markets.iter().take(10) {
                    let candles = match ex
                        .fetch_price_history(PriceHistoryRequest {
                            market_id: market.id.clone(),
                            interval: PriceHistoryInterval::OneDay,
                            outcome: None,
                            token_id: None,
                            condition_id: None,
                            start_ts: None,
                            end_ts: None,
                        })
                        .await
                    {
                        Ok(c) if !c.is_empty() => c,
                        _ => continue,
                    };

                    for c in &candles {
                        // OHLC invariants
                        assert!(c.high >= c.low, "high {} < low {}", c.high, c.low);
                        assert!(c.high >= c.open, "high {} < open {}", c.high, c.open);
                        assert!(c.high >= c.close, "high {} < close {}", c.high, c.close);
                        assert!(c.low <= c.open, "low {} > open {}", c.low, c.open);
                        assert!(c.low <= c.close, "low {} > close {}", c.low, c.close);

                        // Prices should be in [0, 1] range for prediction markets
                        assert!(
                            c.open >= 0.0 && c.open <= 1.0,
                            "open {} out of [0,1]",
                            c.open
                        );
                        assert!(
                            c.close >= 0.0 && c.close <= 1.0,
                            "close {} out of [0,1]",
                            c.close
                        );

                        // Volume should be non-negative
                        assert!(
                            c.volume >= 0.0,
                            "candle volume should be non-negative, got {}",
                            c.volume
                        );

                        // Open interest, if present, should be non-negative
                        if let Some(oi) = c.open_interest {
                            assert!(
                                oi >= 0.0,
                                "open_interest should be non-negative, got {}",
                                oi
                            );
                        }
                    }

                    // Candles should be in chronological order
                    for window in candles.windows(2) {
                        assert!(
                            window[0].timestamp <= window[1].timestamp,
                            "candles should be chronological: {} > {}",
                            window[0].timestamp,
                            window[1].timestamp
                        );
                    }

                    // Found valid candles, test passed
                    return;
                }

                eprintln!(
                    "WARN: {}/fetch_price_history_candle_invariants: no candles found",
                    stringify!($exchange_id)
                );
            }

            #[tokio::test]
            async fn fetch_trades() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_trades {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };
                match ex
                    .fetch_trades(TradesRequest {
                        market_id: markets[0].id.clone(),
                        limit: Some(10),
                        ..Default::default()
                    })
                    .await
                {
                    Ok((trades, _cursor)) => {
                        for t in &trades {
                            assert!(t.price >= 0.0 && t.price <= 1.0, "price should be 0..1");
                            assert!(t.size > 0.0, "size should be positive");
                        }
                    }
                    Err(e) if format!("{e:?}").contains("token_id") => {
                        eprintln!(
                            "SKIP {}/fetch_trades: requires token_id",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_trades failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_trades_with_limit() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_trades {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };

                // Request a small limit and verify we don't exceed it
                let limit = 5;
                match ex
                    .fetch_trades(TradesRequest {
                        market_id: markets[0].id.clone(),
                        limit: Some(limit),
                        ..Default::default()
                    })
                    .await
                {
                    Ok((trades, _cursor)) => {
                        assert!(
                            trades.len() <= limit,
                            "got {} trades but limit was {}",
                            trades.len(),
                            limit
                        );
                    }
                    Err(_) => {} // Not all markets have trades
                }
            }

            #[tokio::test]
            async fn fetch_trades_pagination() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_trades {
                    return;
                }
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };

                // Fetch first page
                let (page1, cursor) = match ex
                    .fetch_trades(TradesRequest {
                        market_id: markets[0].id.clone(),
                        limit: Some(5),
                        ..Default::default()
                    })
                    .await
                {
                    Ok(r) => r,
                    Err(_) => return,
                };

                if page1.is_empty() {
                    return;
                }

                // If we got a cursor, fetch the next page
                if let Some(cursor) = cursor {
                    match ex
                        .fetch_trades(TradesRequest {
                            market_id: markets[0].id.clone(),
                            limit: Some(5),
                            cursor: Some(cursor),
                            ..Default::default()
                        })
                        .await
                    {
                        Ok((page2, _)) => {
                            // Pages should not overlap (if both non-empty)
                            if !page2.is_empty() && page1[0].id.is_some() && page2[0].id.is_some() {
                                assert_ne!(
                                    page1[0].id, page2[0].id,
                                    "pagination returned same first trade"
                                );
                            }
                        }
                        Err(_) => {} // Cursor may have expired
                    }
                }
            }

            // ==================================================================
            // Authenticated — Read-only account data
            // ==================================================================

            #[tokio::test]
            async fn fetch_balance() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_balance {
                    return;
                }
                match ex.fetch_balance().await {
                    Ok(balance) => {
                        assert!(
                            !balance.is_empty(),
                            "authenticated account should have at least one balance entry"
                        );
                        for (_asset, amount) in &balance {
                            assert!(*amount >= 0.0, "balance should be non-negative");
                        }
                    }
                    Err(e) if is_config_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_balance: config/init error: {e}",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if is_auth_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_balance: auth failed",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_balance failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_balance_raw() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) {
                    return;
                }
                match ex.fetch_balance_raw().await {
                    Ok(raw) => {
                        // Raw balance should be a valid JSON value
                        assert!(!raw.is_null(), "raw balance should not be null");
                    }
                    Err(e) if is_not_supported(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_balance_raw: not supported",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if is_config_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_balance_raw: config/init error: {e}",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_balance_raw failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_balance_consistency() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_balance {
                    return;
                }

                // Fetch balance twice — values should be consistent (same account)
                let balance1 = match ex.fetch_balance().await {
                    Ok(b) => b,
                    Err(_) => return,
                };
                let balance2 = match ex.fetch_balance().await {
                    Ok(b) => b,
                    Err(_) => return,
                };

                // Same keys should be present
                assert_eq!(
                    balance1.keys().collect::<std::collections::HashSet<_>>(),
                    balance2.keys().collect::<std::collections::HashSet<_>>(),
                    "balance keys should be consistent across calls"
                );
            }

            #[tokio::test]
            async fn refresh_balance() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_refresh_balance {
                    return;
                }
                match ex.refresh_balance().await {
                    Ok(()) => {}
                    Err(e) if is_config_error(&e) => {
                        eprintln!(
                            "SKIP {}/refresh_balance: config/init error: {e}",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("refresh_balance failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_positions() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_positions {
                    return;
                }
                let positions = ex
                    .fetch_positions(None)
                    .await
                    .expect("fetch_positions failed");
                for p in &positions {
                    assert!(
                        !p.market_id.is_empty(),
                        "position market_id should not be empty"
                    );
                    assert!(
                        !p.outcome.is_empty(),
                        "position outcome should not be empty"
                    );
                    assert!(
                        p.size >= 0.0,
                        "position size should be non-negative, got {}",
                        p.size
                    );
                    assert!(
                        p.average_price >= 0.0 && p.average_price <= 1.0,
                        "position average_price {} out of [0,1] range",
                        p.average_price
                    );
                    assert!(
                        p.current_price >= 0.0 && p.current_price <= 1.0,
                        "position current_price {} out of [0,1] range",
                        p.current_price
                    );
                }
            }

            #[tokio::test]
            async fn fetch_positions_computed_fields() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_positions {
                    return;
                }
                let positions = match ex.fetch_positions(None).await {
                    Ok(p) if !p.is_empty() => p,
                    _ => return,
                };

                for p in &positions {
                    // Verify computed fields are consistent
                    let expected_cost = p.size * p.average_price;
                    let expected_value = p.size * p.current_price;
                    let expected_pnl = expected_value - expected_cost;

                    assert!(
                        (p.cost_basis() - expected_cost).abs() < 1e-10,
                        "cost_basis() mismatch"
                    );
                    assert!(
                        (p.current_value() - expected_value).abs() < 1e-10,
                        "current_value() mismatch"
                    );
                    assert!(
                        (p.unrealized_pnl() - expected_pnl).abs() < 1e-10,
                        "unrealized_pnl() mismatch"
                    );
                }
            }

            #[tokio::test]
            async fn fetch_open_orders() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) {
                    return;
                }
                match ex.fetch_open_orders(None).await {
                    Ok(orders) => {
                        for o in &orders {
                            assert!(!o.id.is_empty(), "order id should not be empty");
                            assert!(
                                !o.market_id.is_empty(),
                                "order market_id should not be empty"
                            );
                            assert!(
                                o.price >= 0.0 && o.price <= 1.0,
                                "order price {} out of [0,1] range",
                                o.price
                            );
                            assert!(o.size > 0.0, "order size should be positive");
                            assert!(
                                o.is_active(),
                                "open order should have active status, got {:?}",
                                o.status
                            );
                        }
                    }
                    Err(e) if is_auth_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_open_orders: auth failed",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if is_config_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_open_orders: config/init error: {e}",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_open_orders failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_fills() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_fills {
                    return;
                }
                let fills = ex
                    .fetch_fills(None, Some(5))
                    .await
                    .expect("fetch_fills failed");
                for f in &fills {
                    assert!(!f.fill_id.is_empty(), "fill_id should not be empty");
                    assert!(
                        f.price >= 0.0 && f.price <= 1.0,
                        "fill price should be 0..1"
                    );
                    assert!(f.size > 0.0, "fill size should be positive");
                    assert!(f.fee >= 0.0, "fill fee should be non-negative");
                    assert!(!f.order_id.is_empty(), "fill order_id should not be empty");
                    assert!(
                        !f.market_id.is_empty(),
                        "fill market_id should not be empty"
                    );
                    assert!(!f.outcome.is_empty(), "fill outcome should not be empty");
                }
            }

            #[tokio::test]
            async fn fetch_fills_with_limit() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_fills {
                    return;
                }
                let limit = 3;
                let fills = match ex.fetch_fills(None, Some(limit)).await {
                    Ok(f) => f,
                    Err(_) => return,
                };
                assert!(
                    fills.len() <= limit,
                    "got {} fills but limit was {}",
                    fills.len(),
                    limit
                );
            }

            #[tokio::test]
            async fn fetch_user_activity() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_user_activity {
                    return;
                }
                let address = match owner_address_for(stringify!($exchange_id)) {
                    Some(a) => a,
                    None => {
                        eprintln!(
                            "SKIP {}/fetch_user_activity: no wallet address configured",
                            stringify!($exchange_id)
                        );
                        return;
                    }
                };
                match ex
                    .fetch_user_activity(px_core::FetchUserActivityParams {
                        address,
                        limit: None,
                    })
                    .await
                {
                    Ok(activity) => {
                        assert!(!activity.is_null(), "user activity should not be null");
                    }
                    Err(e) if is_not_supported(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_user_activity: not supported",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if is_auth_error(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_user_activity: auth failed",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) if is_invalid_input(&e) => {
                        eprintln!(
                            "SKIP {}/fetch_user_activity: invalid input: {e}",
                            stringify!($exchange_id)
                        );
                    }
                    Err(e) => panic!("fetch_user_activity failed: {e:?}"),
                }
            }

            // ==================================================================
            // WebSocket — connect, subscribe, receive at least one update
            // ==================================================================

            #[tokio::test]
            async fn websocket_orderbook_stream() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_websocket {
                    return;
                }

                // All exchanges except polymarket require auth for WebSocket
                if !is_authenticated(&ex) && stringify!($exchange_id) != "polymarket" {
                    eprintln!("SKIP {}/websocket: requires auth", stringify!($exchange_id));
                    return;
                }

                // Get a market to subscribe to
                let markets = match ex.fetch_markets(&Default::default()).await {
                    Ok((m, _)) if !m.is_empty() => m,
                    _ => return,
                };
                let market_id = &markets[0].id;

                use futures::StreamExt;
                use px_core::OrderBookWebSocket;

                // Install rustls crypto provider (needed for TLS connections in tests)
                let _ = rustls::crypto::ring::default_provider().install_default();

                let config = make_exchange_config(stringify!($exchange_id));
                let mut ws = match WebSocketInner::new(stringify!($exchange_id), config) {
                    Ok(ws) => ws,
                    Err(e) => {
                        eprintln!("SKIP {}/websocket: {e}", stringify!($exchange_id));
                        return;
                    }
                };

                if let Err(e) = ws.connect().await {
                    eprintln!(
                        "SKIP {}/websocket: connect failed: {e}",
                        stringify!($exchange_id)
                    );
                    return;
                }
                ws.subscribe(market_id).await.expect("ws subscribe failed");

                let mut stream = ws
                    .orderbook_stream(market_id)
                    .await
                    .expect("orderbook_stream failed");

                // Wait for at least one update within 15 seconds
                let result = tokio::time::timeout(Duration::from_secs(15), stream.next()).await;

                ws.disconnect().await.expect("ws disconnect failed");

                match result {
                    Ok(Some(Ok(update))) => match update {
                        px_core::OrderbookUpdate::Snapshot(book) => {
                            assert!(
                                !book.bids.is_empty() || !book.asks.is_empty(),
                                "snapshot should have some levels"
                            );
                        }
                        px_core::OrderbookUpdate::Delta { changes, .. } => {
                            assert!(!changes.is_empty(), "delta should have at least one change");
                        }
                    },
                    Ok(Some(Err(e))) => panic!("ws stream error: {e:?}"),
                    Ok(None) => panic!("ws stream ended unexpectedly"),
                    Err(_) => {
                        eprintln!(
                            "WARN: no orderbook update within 15s for {} market {}",
                            stringify!($exchange_id),
                            market_id
                        );
                    }
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Generate tests for each exchange
// ---------------------------------------------------------------------------

exchange_tests!(kalshi);
exchange_tests!(polymarket);
exchange_tests!(opinion);
