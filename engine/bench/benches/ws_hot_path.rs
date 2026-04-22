//! Measures the Polymarket book-update hot path: decode WS JSON + apply to
//! Orderbook. Several variants track different decoding strategies:
//!
//! - `legacy` — historical pre-optimisation path (serde_json to typed
//!   struct, f64 parse per level, push+sort insert).
//! - `optimized` — the current production path (`px_core::decode_frame` +
//!   `px_core::parse_level` + partition_point insert).
//! - `simd_parse_only` / `simd_serde_apply` / `simd_decode_apply` — SIMD
//!   parse ceilings, for comparing against BorrowedValue-walk approaches.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use px_core::{insert_ask, insert_bid, Orderbook, PriceLevel, WsFrame};
use serde::Deserialize;
use simd_json::prelude::*;

#[derive(Debug, Deserialize)]
struct BookWire {
    #[allow(dead_code)]
    event_type: Option<String>,
    #[allow(dead_code)]
    asset_id: Option<String>,
    #[allow(dead_code)]
    market: Option<String>,
    #[allow(dead_code)]
    hash: Option<String>,
    bids: Option<Vec<LvlWire>>,
    asks: Option<Vec<LvlWire>>,
}

#[derive(Debug, Deserialize)]
struct LvlWire {
    price: String,
    size: String,
}

fn synth_book_fixture(levels: usize) -> String {
    let asset = "11015470973684010567109736840105671097368401056710973684010567109";
    let market = "0x1a2b3c4d5e6f7890abcdef1234567890abcdef12";
    let mut bids = Vec::with_capacity(levels);
    let mut asks = Vec::with_capacity(levels);
    for i in 0..levels {
        let bid_px = 0.5000 - (i as f64) * 0.0001;
        let ask_px = 0.5001 + (i as f64) * 0.0001;
        let size = 100.0 + (i as f64);
        bids.push(format!(r#"{{"price":"{bid_px:.4}","size":"{size:.2}"}}"#));
        asks.push(format!(r#"{{"price":"{ask_px:.4}","size":"{size:.2}"}}"#));
    }
    format!(
        r#"{{"event_type":"book","asset_id":"{asset}","market":"{market}","timestamp":"1704067200","hash":"abc123","bids":[{}],"asks":[{}]}}"#,
        bids.join(","),
        asks.join(",")
    )
}

fn preseed_book(levels: usize) -> Orderbook {
    let mut ob = Orderbook {
        market_id: String::new(),
        asset_id: String::from("asset"),
        bids: Vec::with_capacity(levels),
        asks: Vec::with_capacity(levels),
        last_update_id: None,
        timestamp: None,
        hash: None,
    };
    for i in 0..levels {
        ob.bids
            .push(PriceLevel::new(0.5000 - i as f64 * 0.0001, 100.0));
        ob.asks
            .push(PriceLevel::new(0.5001 + i as f64 * 0.0001, 100.0));
    }
    ob
}

fn bench_variants(c: &mut Criterion, levels: usize) {
    let template = synth_book_fixture(levels);
    let mut group = c.benchmark_group("ws_book_hot_path");
    group.throughput(Throughput::Bytes(template.len() as u64));

    // Legacy path — what openpx ran before this perf work: serde_json to
    // typed struct, f64 parse per price/size, push+sort insert.
    group.bench_function(format!("legacy_{levels}lvl"), |b| {
        let mut ob = preseed_book(levels);
        b.iter_batched(
            || template.clone(),
            |frame| {
                let parsed: BookWire = serde_json::from_str(&frame).expect("parse");
                ob.bids.clear();
                ob.asks.clear();
                if let Some(bids) = parsed.bids {
                    for l in &bids {
                        let p = l.price.parse::<f64>().unwrap_or(0.0);
                        let s = l.size.parse::<f64>().unwrap_or(0.0);
                        if p > 0.0 && s > 0.0 {
                            insert_bid(&mut ob.bids, PriceLevel::new(p, s));
                        }
                    }
                }
                if let Some(asks) = parsed.asks {
                    for l in &asks {
                        let p = l.price.parse::<f64>().unwrap_or(0.0);
                        let s = l.size.parse::<f64>().unwrap_or(0.0);
                        if p > 0.0 && s > 0.0 {
                            insert_ask(&mut ob.asks, PriceLevel::new(p, s));
                        }
                    }
                }
                black_box(&ob);
            },
            BatchSize::SmallInput,
        );
    });

    // Production path: decode_frame on &str (size-gated — simd-json for
    // large frames, serde_json for small) + parse_level + partition_point
    // insert. This is exactly what polymarket's handle_message_at runs.
    group.bench_function(format!("optimized_{levels}lvl"), |b| {
        let mut ob = preseed_book(levels);
        b.iter_batched(
            || template.clone(),
            |frame| {
                match px_core::decode_frame::<BookWire>(&frame) {
                    Some(WsFrame::Single(parsed)) => {
                        ob.bids.clear();
                        ob.asks.clear();
                        if let Some(bids) = parsed.bids {
                            for l in &bids {
                                if let Some(level) = px_core::parse_level(&l.price, &l.size) {
                                    insert_bid(&mut ob.bids, level);
                                }
                            }
                        }
                        if let Some(asks) = parsed.asks {
                            for l in &asks {
                                if let Some(level) = px_core::parse_level(&l.price, &l.size) {
                                    insert_ask(&mut ob.asks, level);
                                }
                            }
                        }
                    }
                    Some(WsFrame::Array(_)) => unreachable!("fixture is a single object"),
                    None => panic!("parse"),
                }
                black_box(&ob);
            },
            BatchSize::SmallInput,
        );
    });

    // simd-json parse-only ceiling — upper bound on what Step B7 can reach.
    group.bench_function(format!("simd_parse_only_{levels}lvl"), |b| {
        b.iter_batched(
            || template.clone().into_bytes(),
            |mut bytes| {
                let v: simd_json::BorrowedValue =
                    simd_json::to_borrowed_value(&mut bytes).expect("simd parse");
                black_box(v);
            },
            BatchSize::SmallInput,
        );
    });

    // simd-json + serde adapter: parses into typed BookWire via simd-json's
    // SIMD tokenizer. Allocates String fields (same as serde_json) but the
    // parse itself is ~2x faster. Easy drop-in for production.
    group.bench_function(format!("simd_serde_apply_{levels}lvl"), |b| {
        let mut ob = preseed_book(levels);
        b.iter_batched(
            || template.clone().into_bytes(),
            |mut bytes| {
                let parsed: BookWire =
                    simd_json::serde::from_slice(&mut bytes).expect("simd serde parse");
                ob.bids.clear();
                ob.asks.clear();
                if let Some(bids) = parsed.bids {
                    for l in &bids {
                        if let Some(level) = px_core::parse_level(&l.price, &l.size) {
                            insert_bid(&mut ob.bids, level);
                        }
                    }
                }
                if let Some(asks) = parsed.asks {
                    for l in &asks {
                        if let Some(level) = px_core::parse_level(&l.price, &l.size) {
                            insert_ask(&mut ob.asks, level);
                        }
                    }
                }
                black_box(&ob);
            },
            BatchSize::SmallInput,
        );
    });

    // Full simd-json borrowed decode + apply — the ceiling. Parses via
    // BorrowedValue (zero string alloc for price/size — they point into the
    // input bytes), extracts bids/asks with parse_level, applies to book.
    group.bench_function(format!("simd_decode_apply_{levels}lvl"), |b| {
        let mut ob = preseed_book(levels);
        let mut scratch = px_core::ws_decoder::TapeScratch::new();
        b.iter_batched(
            || template.clone().into_bytes(),
            |mut bytes| {
                let v = scratch.parse_value(&mut bytes).expect("simd parse");
                ob.bids.clear();
                ob.asks.clear();
                if let Some(obj) = v.as_object() {
                    if let Some(bids_v) = obj.get("bids").and_then(|v| v.as_array()) {
                        for lvl in bids_v {
                            if let Some(lvl_obj) = lvl.as_object() {
                                let p = lvl_obj.get("price").and_then(|v| v.as_str()).unwrap_or("");
                                let s = lvl_obj.get("size").and_then(|v| v.as_str()).unwrap_or("");
                                if let Some(level) = px_core::parse_level(p, s) {
                                    insert_bid(&mut ob.bids, level);
                                }
                            }
                        }
                    }
                    if let Some(asks_v) = obj.get("asks").and_then(|v| v.as_array()) {
                        for lvl in asks_v {
                            if let Some(lvl_obj) = lvl.as_object() {
                                let p = lvl_obj.get("price").and_then(|v| v.as_str()).unwrap_or("");
                                let s = lvl_obj.get("size").and_then(|v| v.as_str()).unwrap_or("");
                                if let Some(level) = px_core::parse_level(p, s) {
                                    insert_ask(&mut ob.asks, level);
                                }
                            }
                        }
                    }
                }
                black_box(&ob);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    for levels in [1usize, 16, 64] {
        bench_variants(c, levels);
    }
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
