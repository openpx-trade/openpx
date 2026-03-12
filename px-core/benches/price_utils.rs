use criterion::{black_box, criterion_group, criterion_main, Criterion};
use px_core::{clamp_price, is_valid_price, mid_price, round_to_tick_size, spread_bps};

fn bench_round_to_tick(c: &mut Criterion) {
    c.bench_function("round_to_tick_cent", |b| {
        b.iter(|| black_box(round_to_tick_size(0.5678, 0.01)))
    });

    c.bench_function("round_to_tick_kalshi", |b| {
        // Kalshi uses 1-cent ticks (0.01)
        b.iter(|| black_box(round_to_tick_size(0.4321, 0.01)))
    });

    c.bench_function("round_to_tick_fine", |b| {
        // Polymarket uses 0.001 ticks
        b.iter(|| black_box(round_to_tick_size(0.56789, 0.001)))
    });
}

fn bench_is_valid_price(c: &mut Criterion) {
    c.bench_function("is_valid_price_aligned", |b| {
        b.iter(|| black_box(is_valid_price(0.56, 0.01)))
    });

    c.bench_function("is_valid_price_misaligned", |b| {
        b.iter(|| black_box(is_valid_price(0.5678, 0.01)))
    });
}

fn bench_clamp_price(c: &mut Criterion) {
    c.bench_function("clamp_price_in_bounds", |b| {
        b.iter(|| black_box(clamp_price(0.50, 0.01, 0.99, 0.01)))
    });

    c.bench_function("clamp_price_out_of_bounds", |b| {
        b.iter(|| black_box(clamp_price(1.50, 0.01, 0.99, 0.01)))
    });
}

fn bench_mid_price(c: &mut Criterion) {
    c.bench_function("mid_price_both", |b| {
        b.iter(|| black_box(mid_price(Some(0.49), Some(0.51))))
    });

    c.bench_function("mid_price_one_side", |b| {
        b.iter(|| black_box(mid_price(Some(0.49), None)))
    });
}

fn bench_spread_bps(c: &mut Criterion) {
    c.bench_function("spread_bps_normal", |b| {
        b.iter(|| black_box(spread_bps(Some(0.49), Some(0.51))))
    });
}

criterion_group!(
    benches,
    bench_round_to_tick,
    bench_is_valid_price,
    bench_clamp_price,
    bench_mid_price,
    bench_spread_bps
);
criterion_main!(benches);
