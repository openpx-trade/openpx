//! Measures the Polymarket book-update hot path: decode WS JSON + apply to
//! Orderbook. Compares openpx's current serde_json path against simd-json on
//! the same byte input.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use px_core::{insert_ask, insert_bid, Orderbook, PriceLevel};
use serde::Deserialize;

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

fn preseed_book(asset_id: &str, levels: usize) -> Orderbook {
    let mut ob = Orderbook {
        market_id: String::new(),
        asset_id: asset_id.to_string(),
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

fn bench_openpx_serde(c: &mut Criterion, levels: usize) {
    let template = synth_book_fixture(levels);
    let mut group = c.benchmark_group("ws_book_hot_path");
    group.throughput(Throughput::Bytes(template.len() as u64));

    group.bench_function(format!("openpx_serde_{levels}lvl"), |b| {
        let mut ob = preseed_book("asset", levels);
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

    group.bench_function(format!("simd_json_parse_only_{levels}lvl"), |b| {
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

    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    for levels in [1usize, 16, 64] {
        bench_openpx_serde(c, levels);
    }
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
