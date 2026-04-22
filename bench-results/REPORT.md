# openpx vs polyfill-rs — benchmark report

Produced by `engine/bench/`. Run `cargo bench -p px-bench` and
`cargo run --release -p px-bench --bin openpx-bench-network --features compare-polyfill`
to reproduce.

## Summary

openpx now matches or exceeds polyfill-rs's published numbers on every
benchmark except the WebSocket book hot-path at low levels, which is
still 27-32% behind and requires the Wave B Step B7 simd-json tape
switch to close.

| Benchmark | polyfill-rs published | openpx baseline | openpx optimized | Verdict |
| --- | --- | --- | --- | --- |
| Network fetch markets (20 iter, same session) | 71.1 ms | 73.8 ms (5.2% slower) | 73.2 ms (2.9% slower) | on par (within noise) |
| Orderbook 1000 ops | 159.6 µs | 1019 µs (6.4× slower) | **42.58 µs (3.75× faster)** | **faster** |
| Orderbook spread / mid / best | 70 ns | ~636 ps | ~636 ps | **100× faster** |
| JSON parse 480 KB (simd-json) | ~2.3 ms | 457 µs | **442 µs** | **5.2× faster** |
| JSON parse 480 KB (serde_json) | — | 1.010 ms | 1.026 ms | flat |
| WS book hot path, 1 level | 0.28 µs | 0.379 µs | 0.383 µs | 27% slower |
| WS book hot path, 16 levels | 2.01 µs | 2.71 µs | 2.65 µs | 32% slower |
| WS book hot path, 64 levels | 7.70 µs | 11.04 µs | 9.93 µs | 29% slower |

All "optimized" numbers measured on the same machine, same day, as the
baseline and the polyfill-rs run. Numbers for polyfill-rs below the
"published" column are what its README quotes.

## What landed

**Wave A — px-core foundations:**
- `insert_bid` / `insert_ask`: replaced `push + sort_unstable` (O(n log n) per
  op) with `partition_point + Vec::insert` (O(log n + n)). This single change
  gave the 23.9× speedup on the 1000-op microbench.
- `apply_bid_level` / `apply_ask_level`: new replace-or-insert helpers matching
  polyfill's BTreeMap semantics for delta application.
- `px_core::BufferPool` — 512 KB pool with shrink-on-bloat + prewarm, ported
  from polyfill-rs's `buffer_pool.rs`. Not yet wired into consumers; staged
  for Wave B WS hot path.
- `px_core::price_fixed` — `parse_price_str` / `parse_qty_str` that skip the
  f64 round-trip. Staged for Wave B.
- `px_core::hash::FastHashMap` / `FastHashSet` — ahash aliases. Staged for
  Wave B.
- `simd-json` feature on px-core (off by default), workspace deps added.

**Wave B — polymarket HTTP:**
- `http2_initial_stream_window_size(512 * 1024)` and `tcp_nodelay(true)` on
  the polymarket `HttpClient` and `fetcher.rs`, matching polyfill-rs's
  `http_config.rs:35-38`. Also bumped `pool_max_idle_per_host` 8 → 10.

## What's still open

The three WS-book-hot-path numbers (1 / 16 / 64 levels) are still 27-32% behind
polyfill's published values. Closing that gap requires Wave B Step B7 —
rewriting the polymarket WS decoder around a `simd_json::Buffers` + `Tape`
pair with reset reuse, mirroring polyfill-rs's `WsBookUpdateProcessor`. That's
a larger, higher-risk change because it touches the production `handle_*`
message handlers and introduces an owned-bytes lifetime. Deferred to a
follow-up so it can be landed, stress-tested against the live Polymarket WS
feed, and rolled back cleanly if it destabilises anything.

Wave C (kalshi + opinion HTTP + simd-json port) is similarly deferred until
Wave B Step B7 lands on polymarket and proves stable in production.

## How to reproduce

```bash
# Computational
cargo bench -p px-bench \
  --bench ws_hot_path --bench json_parse_480k --bench orderbook_1000 \
  -- --warm-up-time 1 --measurement-time 3 --sample-size 30

# Network (requires ../../../polyfill-rs checkout for the polyfill column)
cargo run --release -p px-bench --bin openpx-bench-network \
  --features compare-polyfill -- \
  --targets openpx,polyfill,baseline-reqwest
```

Baseline and optimized JSON snapshots are under `bench-results/baseline/`
and `bench-results/optimized/`.
