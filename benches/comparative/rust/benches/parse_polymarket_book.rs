//! OpenPX vs `polymarket_client_sdk_v2` — parse a real Polymarket
//! orderbook response.
//!
//! The fixture at `benches/comparative/fixtures/polymarket_book.json` is
//! captured live from the 5-min `btc-up-or-down-5m` series via
//! `tools/capture_bench_fixtures.py`. Both libraries see the exact same
//! bytes; the bench measures the cost of turning those bytes into a
//! typed orderbook structure.
//!
//! This bench runs under three Codspeed instruments via the
//! `codspeed-criterion-compat` harness:
//!
//! - **CPU simulation** (`codspeed run --mode simulation`) — instruction
//!   counts via cachegrind. Deterministic, hardware-agnostic.
//! - **Memory** (`codspeed run --mode memory`) — heap-allocation
//!   tracking via eBPF.
//! - **Walltime** (`codspeed run --mode walltime`) — wall-clock on
//!   stable hardware.
//!
//! Plain `cargo bench` falls through to vanilla criterion wall-clock
//! timings so the bench still works locally without the Codspeed CLI.
//!
//! Kalshi has no official Rust SDK, so there is no Kalshi row in this
//! file — the rendered README notes this explicitly.

use codspeed_criterion_compat::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput,
};
use polymarket_client_sdk_v2::clob::types::response::OrderBookSummaryResponse;
use serde::Deserialize;
use std::path::PathBuf;

/// OpenPX-shape minimal orderbook deserialization. Mirrors the
/// `bids`/`asks`/`timestamp`/`hash` fields used by `get_orderbook` in
/// `px-exchange-polymarket`. We pull this in here rather than relying
/// on the per-exchange crate's private parser so the bench stays
/// honest about what `decode_frame::<T>` is doing.
#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenPxBook {
    asset_id: String,
    market: String,
    timestamp: String,
    bids: Vec<OpenPxLevel>,
    asks: Vec<OpenPxLevel>,
    #[serde(default)]
    hash: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct OpenPxLevel {
    price: String,
    size: String,
}

fn fixture_bytes() -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/polymarket_book.json");
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "missing fixture {}: {e}\nRun `python3 tools/capture_bench_fixtures.py` first.",
            path.display()
        )
    })
}

fn bench_parse(c: &mut Criterion) {
    let bytes = fixture_bytes();
    let mut group = c.benchmark_group("parse_polymarket_book");
    group.throughput(Throughput::Bytes(bytes.len() as u64));

    // OpenPX: utf-8 validate + `decode_frame::<OpenPxBook>` — single-pass
    // typed deserialize, simd-json above the crossover threshold.
    let openpx_input = bytes.clone();
    group.bench_function("openpx", |b| {
        b.iter(|| {
            let s = std::str::from_utf8(black_box(openpx_input.as_slice())).unwrap();
            let frame = px_core::decode_frame::<OpenPxBook>(s).expect("decode");
            match frame {
                px_core::WsFrame::Single(book) => black_box((book.bids.len(), book.asks.len())),
                px_core::WsFrame::Array(_) => unreachable!("single object fixture"),
            }
        })
    });

    // polymarket_client_sdk_v2: typed deserialize via `serde_json` —
    // matches the SDK's own internal `crate::request<T>` path.
    group.bench_function("polymarket_sdk", |b| {
        b.iter_batched(
            || bytes.clone(),
            |buf| {
                let book: OrderBookSummaryResponse =
                    serde_json::from_slice(black_box(&buf)).expect("decode");
                black_box((book.bids.len(), book.asks.len()))
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
