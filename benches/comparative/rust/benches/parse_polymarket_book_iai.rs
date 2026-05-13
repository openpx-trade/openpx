//! OpenPX vs `polymarket_client_sdk_v2` — CPU-instruction (cachegrind)
//! and heap-allocation (DHAT) comparison on the same Polymarket
//! orderbook fixture used by `parse_polymarket_book.rs`.
//!
//! Sibling to the criterion bench: same fixture, same two contestants,
//! different instruments. Criterion measures wall-clock. This harness
//! runs under valgrind and produces deterministic
//!
//!  - **CPU instructions** (`Ir`) — cachegrind's executed-instruction
//!    count. Hardware-agnostic, <1% variance. Mirrors Codspeed's
//!    `--measurement-mode simulation`.
//!  - **Heap allocations** (`total_bytes`) — DHAT's cumulative bytes
//!    allocated on the hot path. Mirrors Codspeed's `--measurement-mode
//!    memory`.
//!
//! Both summaries land in `target/iai/.../summary.json`; the README
//! render script reads them to fill the CPU + memory rows of the Rust
//! comparison table.
//!
//! Linux-only — Valgrind/DHAT don't run on macOS. The
//! `required-features = ["iai"]` gate in `Cargo.toml` keeps this bench
//! out of local mac `cargo bench` runs.

use iai_callgrind::{
    black_box, library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig, Tool,
    ValgrindTool,
};
use polymarket_client_sdk_v2::clob::types::response::OrderBookSummaryResponse;
use serde::Deserialize;
use std::path::PathBuf;

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

#[library_benchmark]
fn openpx() -> (usize, usize) {
    let bytes = fixture_bytes();
    let s = std::str::from_utf8(black_box(&bytes)).unwrap();
    let frame = px_core::decode_frame::<OpenPxBook>(s).expect("decode");
    match frame {
        px_core::WsFrame::Single(book) => black_box((book.bids.len(), book.asks.len())),
        px_core::WsFrame::Array(_) => unreachable!("single object fixture"),
    }
}

#[library_benchmark]
fn polymarket_sdk() -> (usize, usize) {
    let bytes = fixture_bytes();
    let book: OrderBookSummaryResponse = serde_json::from_slice(black_box(&bytes)).expect("decode");
    black_box((book.bids.len(), book.asks.len()))
}

library_benchmark_group!(
    name = parse_polymarket_book;
    config = LibraryBenchmarkConfig::default()
        .tool(Tool::new(ValgrindTool::DHAT));
    benchmarks = openpx, polymarket_sdk
);

main!(library_benchmark_groups = parse_polymarket_book);
