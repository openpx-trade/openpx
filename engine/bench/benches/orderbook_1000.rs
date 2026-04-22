//! Order book throughput: apply 1000 price-level changes and exercise
//! best-bid / best-ask / spread / mid in a tight loop.
//!
//! Mirrors polyfill-rs's published numbers:
//!   Order Book Updates (1000 ops): 159.6 µs ± 32 µs
//!   Spread/Mid calc:                70 ns  ± 77 ns

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use px_core::{insert_ask, insert_bid, Orderbook, PriceLevel};

const LEVELS: usize = 100;

fn empty_book() -> Orderbook {
    Orderbook {
        market_id: String::new(),
        asset_id: String::from("asset"),
        bids: Vec::with_capacity(LEVELS),
        asks: Vec::with_capacity(LEVELS),
        last_update_id: None,
        timestamp: None,
        hash: None,
    }
}

fn seeded_book() -> Orderbook {
    let mut ob = empty_book();
    for i in 0..LEVELS {
        ob.bids
            .push(PriceLevel::new(0.5000 - i as f64 * 0.0001, 100.0));
        ob.asks
            .push(PriceLevel::new(0.5001 + i as f64 * 0.0001, 100.0));
    }
    ob.sort();
    ob
}

fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook");
    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_ops", |b| {
        b.iter_batched(
            empty_book,
            |mut ob| {
                for i in 0..500u32 {
                    let px = 0.4000 + (i % 100) as f64 * 0.0001;
                    insert_bid(&mut ob.bids, PriceLevel::new(px, 10.0));
                }
                for i in 0..500u32 {
                    let px = 0.5001 + (i % 100) as f64 * 0.0001;
                    insert_ask(&mut ob.asks, PriceLevel::new(px, 10.0));
                }
                black_box(&ob);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();

    let ob = seeded_book();
    let mut group = c.benchmark_group("orderbook_calc");
    group.bench_function("best_bid", |b| b.iter(|| black_box(ob.best_bid())));
    group.bench_function("best_ask", |b| b.iter(|| black_box(ob.best_ask())));
    group.bench_function("mid_price", |b| b.iter(|| black_box(ob.mid_price())));
    group.bench_function("spread", |b| b.iter(|| black_box(ob.spread())));
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
