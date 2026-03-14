//! Live integration tests against real exchange APIs.
//!
//! These tests hit production endpoints — they are **not** run in CI.
//!
//! ## Running
//!
//! Unauthenticated (market data only):
//!   OPENPX_LIVE_TESTS=1 cargo test -p px-sdk --test live -- --nocapture
//!
//! Single exchange:
//!   OPENPX_LIVE_TESTS=1 cargo test -p px-sdk --test live kalshi -- --nocapture
//!
//! With auth (enables balance/position/fill tests):
//!   OPENPX_LIVE_TESTS=1 \
//!   KALSHI_API_KEY_ID=... KALSHI_PRIVATE_KEY_PEM=... \
//!   cargo test -p px-sdk --test live kalshi -- --nocapture

use std::env;
use std::time::Duration;

use px_core::{
    FetchMarketsParams, OrderbookRequest, PriceHistoryInterval, PriceHistoryRequest, TradesRequest,
};
use px_sdk::{ExchangeInner, WebSocketInner};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Skip the test unless OPENPX_LIVE_TESTS=1.
fn require_live() -> bool {
    env::var("OPENPX_LIVE_TESTS").is_ok_and(|v| v == "1")
}

/// Build config JSON from env vars for a given exchange.
fn make_exchange_config(id: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    let vars: &[(&str, &str)] = match id {
        "kalshi" => &[
            ("KALSHI_API_KEY_ID", "api_key_id"),
            ("KALSHI_PRIVATE_KEY_PEM", "private_key_pem"),
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

            // ------------------------------------------------------------------
            // Unauthenticated — market data
            // ------------------------------------------------------------------

            #[tokio::test]
            async fn describe() {
                let Some(ex) = get_exchange() else { return };
                let info = ex.describe();
                assert_eq!(info.id, stringify!($exchange_id));
                assert!(!info.name.is_empty());
                assert!(info.has_fetch_markets);
            }

            #[tokio::test]
            async fn fetch_markets() {
                let Some(ex) = get_exchange() else { return };
                let result = ex
                    .fetch_markets(Some(FetchMarketsParams {
                        limit: Some(5),
                        cursor: None,
                    }))
                    .await;
                let markets = match result {
                    Ok(m) => m,
                    Err(e) if format!("{e:?}").contains("uthent") => {
                        eprintln!(
                            "SKIP {}/fetch_markets: requires auth",
                            stringify!($exchange_id)
                        );
                        return;
                    }
                    Err(e) if format!("{e:?}").contains("http error") => {
                        eprintln!(
                            "SKIP {}/fetch_markets: API unreachable: {e}",
                            stringify!($exchange_id)
                        );
                        return;
                    }
                    Err(e) => panic!("fetch_markets failed: {e:?}"),
                };
                assert!(!markets.is_empty(), "expected at least 1 market");
                for m in &markets {
                    assert!(!m.id.is_empty(), "market id should not be empty");
                    assert!(
                        !m.question.is_empty(),
                        "market question should not be empty"
                    );
                    assert!(!m.outcomes.is_empty(), "market should have outcomes");
                }
            }

            #[tokio::test]
            async fn fetch_market_single() {
                let Some(ex) = get_exchange() else { return };
                let markets = match ex
                    .fetch_markets(Some(FetchMarketsParams {
                        limit: Some(1),
                        cursor: None,
                    }))
                    .await
                {
                    Ok(m) => m,
                    Err(_) => return, // covered by fetch_markets test
                };
                if markets.is_empty() {
                    return;
                }
                let single = ex
                    .fetch_market(&markets[0].id)
                    .await
                    .expect("fetch_market failed");
                assert_eq!(single.id, markets[0].id);
            }

            #[tokio::test]
            async fn fetch_orderbook() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_orderbook {
                    return;
                }
                // Some exchanges require auth for orderbook
                let markets = match ex
                    .fetch_markets(Some(FetchMarketsParams {
                        limit: Some(1),
                        cursor: None,
                    }))
                    .await
                {
                    Ok(m) if !m.is_empty() => m,
                    _ => return,
                };
                let market_id = &markets[0].id;

                match ex
                    .fetch_orderbook(OrderbookRequest {
                        market_id: market_id.clone(),
                        outcome: None,
                        token_id: None,
                    })
                    .await
                {
                    Ok(book) => {
                        let _ = &book.bids;
                        let _ = &book.asks;
                    }
                    Err(e) if format!("{e:?}").contains("uthent") => {
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
                    Err(e) => panic!("fetch_orderbook failed: {e:?}"),
                }
            }

            #[tokio::test]
            async fn fetch_all_unified_markets() {
                let Some(ex) = get_exchange() else { return };
                // This paginates all markets — give it a timeout so it doesn't hang
                let result =
                    tokio::time::timeout(Duration::from_secs(60), ex.fetch_all_unified_markets())
                        .await;
                match result {
                    Ok(Ok(markets)) => {
                        assert!(!markets.is_empty(), "expected at least 1 unified market");
                        let sample = &markets[..markets.len().min(5)];
                        for m in sample {
                            assert!(!m.openpx_id.is_empty());
                            assert!(!m.title.is_empty());
                            assert_eq!(m.exchange, stringify!($exchange_id));
                        }
                    }
                    Ok(Err(e)) if format!("{e:?}").contains("uthent") => {
                        eprintln!(
                            "SKIP {}/fetch_all_unified_markets: requires auth",
                            stringify!($exchange_id)
                        );
                    }
                    Ok(Err(e)) if format!("{e:?}").contains("http error") => {
                        eprintln!(
                            "SKIP {}/fetch_all_unified_markets: API unreachable",
                            stringify!($exchange_id)
                        );
                    }
                    Ok(Err(e)) => panic!("fetch_all_unified_markets failed: {e:?}"),
                    Err(_) => {
                        eprintln!(
                            "WARN: {}/fetch_all_unified_markets timed out after 60s",
                            stringify!($exchange_id)
                        );
                    }
                }
            }

            // ------------------------------------------------------------------
            // Historical data (unauthenticated, exchange-dependent)
            // ------------------------------------------------------------------

            #[tokio::test]
            async fn fetch_price_history() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_price_history {
                    return;
                }
                let markets = match ex
                    .fetch_markets(Some(FetchMarketsParams {
                        limit: Some(3),
                        cursor: None,
                    }))
                    .await
                {
                    Ok(m) if !m.is_empty() => m,
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
            async fn fetch_trades() {
                let Some(ex) = get_exchange() else { return };
                if !ex.describe().has_fetch_trades {
                    return;
                }
                let markets = match ex
                    .fetch_markets(Some(FetchMarketsParams {
                        limit: Some(1),
                        cursor: None,
                    }))
                    .await
                {
                    Ok(m) if !m.is_empty() => m,
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

            // ------------------------------------------------------------------
            // Authenticated — read-only account data
            // ------------------------------------------------------------------

            #[tokio::test]
            async fn fetch_balance() {
                let Some(ex) = get_exchange() else { return };
                if !is_authenticated(&ex) || !ex.describe().has_fetch_balance {
                    return;
                }
                let balance = ex.fetch_balance().await.expect("fetch_balance failed");
                assert!(
                    !balance.is_empty(),
                    "authenticated account should have at least one balance entry"
                );
                for (_asset, amount) in &balance {
                    assert!(*amount >= 0.0, "balance should be non-negative");
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
                }
            }

            // ------------------------------------------------------------------
            // WebSocket — connect, subscribe, receive at least one update
            // ------------------------------------------------------------------

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
                let markets = match ex
                    .fetch_markets(Some(FetchMarketsParams {
                        limit: Some(3),
                        cursor: None,
                    }))
                    .await
                {
                    Ok(m) if !m.is_empty() => m,
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
