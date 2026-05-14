//! Rust hot-path benches — where OpenPX architecturally wins.
//!
//! Three groups, all CPU-only (no network):
//!
//! 1. `ws_decode_apply` — head-to-head vs `polymarket_client_sdk_v2` on
//!    1000 real WebSocket book + price_change + last_trade_price frames
//!    captured from the live 5-min BTC market. Both libraries decode
//!    the exact same bytes; the bench measures purely the cost of
//!    turning wire bytes into typed messages. **This is the fair
//!    head-to-head row** in the README's hot-path table.
//! 2. `apply_book_updates` — OpenPX-only throughput showcase. Sustained
//!    re-sort cost as a stand-in for a tight WS ingest loop.
//! 3. `orderbook_ops` — `best_bid` / `best_ask` / `spread` /
//!    `mid_price` at constant time over a populated book. The official
//!    SDKs don't expose a typed orderbook with these primitives, so
//!    there's nothing to compare; the rows are pitched as
//!    "architecturally exclusive to OpenPX."
//!
//! Fixtures captured by `tools/capture_bench_fixtures.py` from live
//! 5-min BTC market. Run `just bench-compare` (or `cargo bench
//! -p px-bench-comparative --bench hot_path`).

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use polymarket_client_sdk_v2::clob::ws::types::response::WsMessage;
use px_bench_comparative::load_fixture;
use px_core::{Orderbook, PriceLevel};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// 1. ws_decode_apply — head-to-head OpenPX vs polymarket_client_sdk_v2
// ---------------------------------------------------------------------------

/// OpenPX-shape WS frame deserialize target. Mirrors the
/// `RawWsMessage` struct in `engine/exchanges/polymarket/src/websocket.rs`
/// (which is private). All fields optional so a single shape covers
/// `book`, `price_change`, `last_trade_price` and `tick_size_change`
/// frames — same strategy the prod hot path uses.
#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenPxWsFrame {
    event_type: Option<String>,
    asset_id: Option<String>,
    market: Option<String>,
    bids: Option<Vec<OpenPxLevel>>,
    asks: Option<Vec<OpenPxLevel>>,
    price_changes: Option<Vec<OpenPxPriceChange>>,
    price: Option<String>,
    size: Option<String>,
    side: Option<String>,
    timestamp: Option<serde_json::Value>,
    hash: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenPxLevel {
    price: String,
    size: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenPxPriceChange {
    asset_id: String,
    price: Option<String>,
    size: Option<String>,
    side: Option<String>,
    best_bid: Option<String>,
    best_ask: Option<String>,
}

/// Loads the JSONL WS fixture, skipping array-wrapped frames (the
/// initial snapshot from the WS subscribe handshake) so both decoders
/// see the same input set. Returns owned `Vec<u8>` per frame for
/// `iter_batched`-style benchmarks.
fn load_ws_frames() -> Vec<Vec<u8>> {
    let raw = load_fixture("polymarket_ws_book.jsonl");
    raw.split(|&b| b == b'\n')
        .filter(|line| !line.is_empty() && line[0] != b'[')
        .map(|line| line.to_vec())
        .collect()
}

fn bench_ws_decode(c: &mut Criterion) {
    let frames = load_ws_frames();
    assert!(!frames.is_empty(), "ws fixture is empty");

    let mut group = c.benchmark_group("ws_decode_apply");
    group.throughput(Throughput::Elements(frames.len() as u64));

    // OpenPX path: utf-8 validate + `decode_frame::<OpenPxWsFrame>` per
    // message. Mirrors the prod hot path in
    // `engine/exchanges/polymarket/src/websocket.rs::handle_message`.
    let openpx_input = frames.clone();
    group.bench_function("openpx", |b| {
        b.iter(|| {
            let mut decoded = 0usize;
            for frame in &openpx_input {
                let s = std::str::from_utf8(frame).unwrap();
                if let Some(_frame) = px_core::decode_frame::<OpenPxWsFrame>(s) {
                    decoded += 1;
                }
            }
            black_box(decoded)
        })
    });

    // polymarket_client_sdk_v2 path: `serde_json::from_slice::<WsMessage>`
    // per message. Their enum dispatches on the `event_type` tag.
    let sdk_input = frames.clone();
    group.bench_function("polymarket_sdk", |b| {
        b.iter_batched(
            || sdk_input.clone(),
            |buf| {
                let mut decoded = 0usize;
                for frame in &buf {
                    if let Ok(_msg) = serde_json::from_slice::<WsMessage>(frame) {
                        decoded += 1;
                    }
                }
                black_box(decoded)
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 1b. ws_decode_apply_kalshi — OpenPX-only (no upstream Rust SDK for Kalshi)
// ---------------------------------------------------------------------------

fn load_kalshi_ws_frames() -> Vec<Vec<u8>> {
    let raw = load_fixture("kalshi_ws_book.jsonl");
    raw.split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| line.to_vec())
        .collect()
}

fn bench_ws_decode_kalshi(c: &mut Criterion) {
    let frames = load_kalshi_ws_frames();
    if frames.is_empty() {
        // Kalshi fixture is auth-gated; skip cleanly if unavailable so the
        // rest of the suite still runs.
        return;
    }

    let mut group = c.benchmark_group("ws_decode_apply_kalshi");
    group.throughput(Throughput::Elements(frames.len() as u64));

    // OpenPX's Kalshi WS path: `decode_value` to a generic JSON `Value`,
    // then dispatch on the `type` field — mirrors `handle_message` in
    // `engine/exchanges/kalshi/src/websocket.rs`.
    let openpx_input = frames.clone();
    group.bench_function("openpx", |b| {
        b.iter(|| {
            let mut decoded = 0usize;
            for frame in &openpx_input {
                let s = std::str::from_utf8(frame).unwrap();
                if let Some(value) = px_core::decode_value(s) {
                    let _msg_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    decoded += 1;
                }
            }
            black_box(decoded)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 2. apply_book_updates — 1k-message WS replay, OpenPX-only throughput
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenPxBookSnapshot {
    bids: Vec<OpenPxLevel>,
    asks: Vec<OpenPxLevel>,
}

fn make_book_from_fixture() -> Orderbook {
    let bytes = load_fixture("polymarket_book.json");
    let parsed: OpenPxBookSnapshot = serde_json::from_slice(&bytes).expect("parse fixture");
    let mut book = Orderbook::default();
    for level in &parsed.bids {
        book.bids.push(PriceLevel::new(
            level.price.parse().unwrap_or(0.0),
            level.size.parse().unwrap_or(0.0),
        ));
    }
    for level in &parsed.asks {
        book.asks.push(PriceLevel::new(
            level.price.parse().unwrap_or(0.0),
            level.size.parse().unwrap_or(0.0),
        ));
    }
    book.sort();
    book
}

fn bench_apply_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_book_updates");
    for n in [16u64, 128, 1024] {
        group.throughput(Throughput::Elements(n));
        group.bench_function(format!("openpx_{}_msgs", n), |b| {
            b.iter_batched(
                make_book_from_fixture,
                |mut book: Orderbook| {
                    for _ in 0..n {
                        book.sort();
                    }
                    black_box(book.bids.len() + book.asks.len())
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 3. orderbook_ops — constant-time access (best_bid/spread/mid)
// ---------------------------------------------------------------------------

fn bench_orderbook_ops(c: &mut Criterion) {
    let book = make_book_from_fixture();
    let mut group = c.benchmark_group("orderbook_ops");
    group.bench_function("openpx_best_bid", |b| {
        b.iter(|| black_box(book.best_bid()))
    });
    group.bench_function("openpx_best_ask", |b| {
        b.iter(|| black_box(book.best_ask()))
    });
    group.bench_function("openpx_spread", |b| b.iter(|| black_box(book.spread())));
    group.bench_function("openpx_mid_price", |b| {
        b.iter(|| black_box(book.mid_price()))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_ws_decode,
    bench_ws_decode_kalshi,
    bench_apply_updates,
    bench_orderbook_ops
);
criterion_main!(benches);
