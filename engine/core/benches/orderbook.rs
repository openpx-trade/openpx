use criterion::{black_box, criterion_group, criterion_main, Criterion};
use px_core::{
    insert_ask, insert_bid, sort_asks, sort_bids, FixedPrice, Orderbook, PriceLevel,
    PriceLevelChange, PriceLevelSide,
};
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
        hash: None,
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

    c.bench_function("orderbook_spread", |b| b.iter(|| black_box(book.spread())));
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
                    price: FixedPrice::from_f64(0.50 + i as f64 * 0.01),
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
                    price: FixedPrice::from_f64(0.50 + i as f64 * 0.01),
                    size: 100.0,
                });
            }
            black_box(changes)
        })
    });
}

fn bench_insert_vs_push_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_vs_push_sort");

    for depth in [10, 50, 200] {
        // Benchmark: push + sort (old approach)
        group.bench_function(format!("push_sort_bid_depth_{depth}"), |b| {
            b.iter_batched(
                || {
                    (0..depth)
                        .map(|i| PriceLevel::new(0.50 - (i as f64 * 0.01), 100.0))
                        .collect::<Vec<_>>()
                },
                |mut levels| {
                    levels.push(PriceLevel::new(0.455, 50.0));
                    sort_bids(&mut levels);
                    black_box(levels)
                },
                criterion::BatchSize::SmallInput,
            )
        });

        // Benchmark: sorted insertion (new approach)
        group.bench_function(format!("insert_bid_depth_{depth}"), |b| {
            b.iter_batched(
                || {
                    (0..depth)
                        .map(|i| PriceLevel::new(0.50 - (i as f64 * 0.01), 100.0))
                        .collect::<Vec<_>>()
                },
                |mut levels| {
                    insert_bid(&mut levels, PriceLevel::new(0.455, 50.0));
                    black_box(levels)
                },
                criterion::BatchSize::SmallInput,
            )
        });

        // Ask side
        group.bench_function(format!("push_sort_ask_depth_{depth}"), |b| {
            b.iter_batched(
                || {
                    (0..depth)
                        .map(|i| PriceLevel::new(0.51 + (i as f64 * 0.01), 100.0))
                        .collect::<Vec<_>>()
                },
                |mut levels| {
                    levels.push(PriceLevel::new(0.555, 50.0));
                    sort_asks(&mut levels);
                    black_box(levels)
                },
                criterion::BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("insert_ask_depth_{depth}"), |b| {
            b.iter_batched(
                || {
                    (0..depth)
                        .map(|i| PriceLevel::new(0.51 + (i as f64 * 0.01), 100.0))
                        .collect::<Vec<_>>()
                },
                |mut levels| {
                    insert_ask(&mut levels, PriceLevel::new(0.555, 50.0));
                    black_box(levels)
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_best_bid_ask,
    bench_sort,
    bench_changevec_alloc,
    bench_insert_vs_push_sort
);
criterion_main!(benches);
