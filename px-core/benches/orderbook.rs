use criterion::{black_box, criterion_group, criterion_main, Criterion};
use px_core::{Orderbook, PriceLevel, PriceLevelChange, PriceLevelSide};
use smallvec::SmallVec;

fn make_orderbook(depth: usize) -> Orderbook {
    let bids: Vec<PriceLevel> = (0..depth)
        .map(|i| PriceLevel::new(0.50 - (i as f64 * 0.01), 100.0 + i as f64))
        .collect();
    let asks: Vec<PriceLevel> = (0..depth)
        .map(|i| PriceLevel::new(0.51 + (i as f64 * 0.01), 100.0 + i as f64))
        .collect();
    Orderbook {
        market_id: "bench-market".into(),
        asset_id: "bench-asset".into(),
        bids,
        asks,
        last_update_id: None,
        timestamp: None,
    }
}

fn bench_best_bid_ask(c: &mut Criterion) {
    let book = make_orderbook(50);

    c.bench_function("orderbook_best_bid", |b| {
        b.iter(|| black_box(book.best_bid()))
    });

    c.bench_function("orderbook_best_ask", |b| {
        b.iter(|| black_box(book.best_ask()))
    });

    c.bench_function("orderbook_mid_price", |b| {
        b.iter(|| black_box(book.mid_price()))
    });

    c.bench_function("orderbook_spread", |b| {
        b.iter(|| black_box(book.spread()))
    });
}

fn bench_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_sort");

    for depth in [10, 50, 200] {
        group.bench_function(format!("depth_{depth}"), |b| {
            b.iter_batched(
                || make_orderbook(depth),
                |mut book| {
                    book.sort();
                    black_box(book)
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn bench_changevec_alloc(c: &mut Criterion) {
    c.bench_function("changevec_stack_4", |b| {
        b.iter(|| {
            let mut changes: SmallVec<[PriceLevelChange; 4]> = SmallVec::new();
            for i in 0..4 {
                changes.push(PriceLevelChange {
                    side: PriceLevelSide::Bid,
                    price: 0.50 + i as f64 * 0.01,
                    size: 100.0,
                });
            }
            black_box(changes)
        })
    });

    c.bench_function("changevec_heap_8", |b| {
        b.iter(|| {
            let mut changes: SmallVec<[PriceLevelChange; 4]> = SmallVec::new();
            for i in 0..8 {
                changes.push(PriceLevelChange {
                    side: PriceLevelSide::Ask,
                    price: 0.50 + i as f64 * 0.01,
                    size: 100.0,
                });
            }
            black_box(changes)
        })
    });
}

criterion_group!(benches, bench_best_bid_ask, bench_sort, bench_changevec_alloc);
criterion_main!(benches);
