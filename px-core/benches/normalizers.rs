use criterion::{black_box, criterion_group, criterion_main, Criterion};
use px_core::{coerce_to_float, coerce_to_int, coerce_to_string, get_nested};
use serde_json::json;

fn bench_coerce_to_int(c: &mut Criterion) {
    let num = json!(42);
    let str_num = json!("12345");
    let float_num = json!(1.2345);
    let bool_val = json!(true);

    c.bench_function("coerce_int_from_number", |b| {
        b.iter(|| black_box(coerce_to_int(&num)))
    });

    c.bench_function("coerce_int_from_string", |b| {
        b.iter(|| black_box(coerce_to_int(&str_num)))
    });

    c.bench_function("coerce_int_from_float", |b| {
        b.iter(|| black_box(coerce_to_int(&float_num)))
    });

    c.bench_function("coerce_int_from_bool", |b| {
        b.iter(|| black_box(coerce_to_int(&bool_val)))
    });
}

fn bench_coerce_to_float(c: &mut Criterion) {
    let num = json!(98.7654);
    let str_num = json!("123.456");

    c.bench_function("coerce_float_from_number", |b| {
        b.iter(|| black_box(coerce_to_float(&num)))
    });

    c.bench_function("coerce_float_from_string", |b| {
        b.iter(|| black_box(coerce_to_float(&str_num)))
    });
}

fn bench_coerce_to_string(c: &mut Criterion) {
    let str_val = json!("hello world");
    let num_val = json!(42);

    c.bench_function("coerce_string_from_string", |b| {
        b.iter(|| black_box(coerce_to_string(&str_val)))
    });

    c.bench_function("coerce_string_from_number", |b| {
        b.iter(|| black_box(coerce_to_string(&num_val)))
    });
}

fn bench_nested_path(c: &mut Criterion) {
    let data = json!({
        "events": [
            {
                "id": "evt-123",
                "markets": [
                    {"ticker": "MARKET-A", "volume": 50000}
                ]
            }
        ],
        "metadata": {
            "source": "kalshi"
        }
    });

    c.bench_function("get_nested_shallow", |b| {
        b.iter(|| black_box(get_nested(&data, "metadata.source")))
    });

    c.bench_function("get_nested_deep", |b| {
        b.iter(|| black_box(get_nested(&data, "events.0.markets.0.ticker")))
    });

    c.bench_function("get_nested_miss", |b| {
        b.iter(|| black_box(get_nested(&data, "events.0.nonexistent.field")))
    });
}

criterion_group!(
    benches,
    bench_coerce_to_int,
    bench_coerce_to_float,
    bench_coerce_to_string,
    bench_nested_path
);
criterion_main!(benches);
