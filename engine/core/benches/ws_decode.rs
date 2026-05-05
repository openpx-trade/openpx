//! `decode_frame` parse-time bench across realistic frame sizes.
//!
//! `decode_frame` is the entry point for every WS message. With the simd-json
//! feature enabled it dispatches between serde_json (small frames) and
//! simd-json (≥ SIMD_CROSSOVER_BYTES). This bench measures parse latency at
//! four sizes representative of the live workload — recent simd-json upgrades
//! and crossover tweaks are validated here.
//!
//! Frames are shaped like Polymarket / Kalshi book payloads (top-level array
//! of objects with `event`, `seq`, `price`, `size`, optional `levels` array)
//! so the parser exercises both the array-vs-single dispatch and nested
//! structure walk it sees in production.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use px_core::{decode_frame, decode_value, WsFrame};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct Level {
    #[serde(rename = "p")]
    _price: f64,
    #[serde(rename = "s")]
    _size: f64,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct BookMsg {
    event: String,
    seq: u64,
    market: String,
    asset: String,
    price: Option<f64>,
    size: Option<f64>,
    #[serde(default)]
    levels: Vec<Level>,
}

/// Build one BookMsg-shaped JSON object with `n` price levels.
/// Approx sizes: 0 lvls ≈ 90B, 8 ≈ 280B, 32 ≈ 850B, 200 ≈ 4.7KB.
fn make_book_msg(n: usize, seq: u64) -> String {
    let mut out = String::with_capacity(64 + n * 16);
    out.push_str(r#"{"event":"book","seq":"#);
    out.push_str(&seq.to_string());
    out.push_str(r#","market":"KXBTCD-25APR1517","asset":"yes","levels":["#);
    for i in 0..n {
        if i > 0 {
            out.push(',');
        }
        let p = 0.5 + (i as f64) * 0.001;
        let s = 100.0 + (i as f64) * 1.5;
        out.push_str(&format!(r#"{{"p":{p:.4},"s":{s:.2}}}"#));
    }
    out.push_str("]}");
    out
}

/// Wrap `count` BookMsg objects in a top-level JSON array (matches the
/// batched-deltas frame shape both exchanges emit).
fn make_array_frame(count: usize, levels_per_msg: usize) -> String {
    let mut out = String::with_capacity(count * (64 + levels_per_msg * 16) + 8);
    out.push('[');
    for i in 0..count {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&make_book_msg(levels_per_msg, i as u64));
    }
    out.push(']');
    out
}

fn bench_decode_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("ws_decode_single");
    // Sizes calibrated to the live workload:
    //  -  0 lvls (~90B)   — tiny price-change / sub ack
    //  -  8 lvls (~280B)  — small delta, below SIMD crossover (512B)
    //  - 32 lvls (~850B)  — mid delta, above crossover
    //  - 200 lvls (~4.7KB) — deep book snapshot
    for levels in [0usize, 8, 32, 200] {
        let payload = make_book_msg(levels, 1);
        let bytes = payload.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("levels", levels),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let frame = decode_frame::<BookMsg>(black_box(payload.as_str()))
                        .expect("decode failed");
                    match frame {
                        WsFrame::Single(m) => black_box(m.seq),
                        WsFrame::Array(_) => unreachable!("expected Single"),
                    }
                })
            },
        );
    }
    group.finish();
}

fn bench_decode_array(c: &mut Criterion) {
    let mut group = c.benchmark_group("ws_decode_array");
    // Mid-volume batched frames: each has 8 levels (~280B per msg).
    // 4 msgs ≈ 1.2KB, 16 msgs ≈ 4.7KB, 64 msgs ≈ 18KB.
    for count in [4usize, 16, 64] {
        let payload = make_array_frame(count, 8);
        let bytes = payload.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("msgs", count), &payload, |b, payload| {
            b.iter(|| {
                let frame =
                    decode_frame::<BookMsg>(black_box(payload.as_str())).expect("decode failed");
                match frame {
                    WsFrame::Array(items) => black_box(items.len()),
                    WsFrame::Single(_) => unreachable!("expected Array"),
                }
            })
        });
    }
    group.finish();
}

fn bench_decode_value(c: &mut Criterion) {
    // `decode_value` is the loosely-typed counterpart used by Kalshi to
    // dispatch on a top-level field before the typed parse. Track its
    // latency at the same sizes.
    let mut group = c.benchmark_group("ws_decode_value");
    for levels in [0usize, 8, 32, 200] {
        let payload = make_book_msg(levels, 1);
        let bytes = payload.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("levels", levels),
            &payload,
            |b, payload| {
                b.iter(|| {
                    let v = decode_value(black_box(payload.as_str())).expect("decode failed");
                    black_box(v.get("seq").and_then(|s| s.as_u64()))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    ws_decode_benches,
    bench_decode_single,
    bench_decode_array,
    bench_decode_value
);
criterion_main!(ws_decode_benches);
