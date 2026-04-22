//! Parses a captured 480KB /simplified-markets response. Compares the three
//! parsers openpx could use: serde_json::Value (current baseline),
//! simd_json::to_borrowed_value (polyfill-rs's choice), and serde_json's
//! DeserializeOwned into a typed struct.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use px_bench::fixtures;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SimplifiedMarket {
    #[allow(dead_code)]
    condition_id: Option<String>,
    #[allow(dead_code)]
    question_id: Option<String>,
    #[allow(dead_code)]
    tokens: Option<Vec<TokenWire>>,
    #[allow(dead_code)]
    rewards: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TokenWire {
    #[allow(dead_code)]
    token_id: Option<String>,
    #[allow(dead_code)]
    outcome: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SimplifiedMarketsResp {
    #[allow(dead_code)]
    data: Vec<SimplifiedMarket>,
    #[allow(dead_code)]
    next_cursor: Option<String>,
}

fn load_payload() -> Vec<u8> {
    let path = fixtures::fixtures_dir().join("simplified_markets_480k.json");
    if path.exists() {
        return std::fs::read(&path).expect("read fixture");
    }
    eprintln!(
        "  fixtures/simplified_markets_480k.json missing -> synthesizing ~480KB fallback.\n  \
         Capture the real fixture with:\n  \
           curl -s 'https://clob.polymarket.com/simplified-markets?next_cursor=MA==' \\\n  \
             > engine/bench/fixtures/simplified_markets_480k.json"
    );
    synth_480k().into_bytes()
}

fn synth_480k() -> String {
    let mut entries: Vec<String> = Vec::with_capacity(800);
    for i in 0..800 {
        let cond = format!("0x{:064x}", i);
        let tok_yes = format!("{:077}", i * 2);
        let tok_no = format!("{:077}", i * 2 + 1);
        entries.push(format!(
            r#"{{"condition_id":"{cond}","question_id":"{cond}","market_slug":"synthetic-market-{i}","tokens":[{{"token_id":"{tok_yes}","outcome":"Yes","price":0.5,"winner":false}},{{"token_id":"{tok_no}","outcome":"No","price":0.5,"winner":false}}],"rewards":{{"rates":[{{"asset_address":"0x0000000000000000000000000000000000000000","rewards_daily_rate":0}}],"min_size":0,"max_spread":0}},"closed":false,"active":true,"minimum_tick_size":"0.01","minimum_order_size":"5"}}"#,
        ));
    }
    format!(r#"{{"data":[{}],"next_cursor":"LTE="}}"#, entries.join(","))
}

fn benchmarks(c: &mut Criterion) {
    let payload = load_payload();
    let mut group = c.benchmark_group("json_parse_480k");
    group.throughput(Throughput::Bytes(payload.len() as u64));

    group.bench_function("serde_json_value", |b| {
        b.iter_batched(
            || payload.clone(),
            |bytes| {
                let v: serde_json::Value = serde_json::from_slice(&bytes).expect("parse");
                black_box(v);
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("serde_json_typed", |b| {
        b.iter_batched(
            || payload.clone(),
            |bytes| {
                let v: SimplifiedMarketsResp = serde_json::from_slice(&bytes).expect("parse");
                black_box(v);
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("simd_json_borrowed_value", |b| {
        b.iter_batched(
            || payload.clone(),
            |mut bytes| {
                let v: simd_json::BorrowedValue =
                    simd_json::to_borrowed_value(&mut bytes).expect("parse");
                black_box(v);
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
