# openpx vs official exchange clients — benchmark report

Produced by `engine/bench/`. Compares openpx's polymarket implementation
against the two official Polymarket client libraries (`py-clob-client` for
Python, `polymarket-rs-client` for Rust) plus an untuned `reqwest` baseline
as a sanity floor.

## Network — Fetch Markets

`GET https://clob.polymarket.com/simplified-markets?next_cursor=MA==` (~480 KB
response), 40 iterations, 100 ms spacing, 10 warmup requests discarded, all
targets in a single process so machine / network / time are held constant.

| Target                 | Mean         | Median    | Min       | Max       | n  |
| ---                    | ---          | ---       | ---       | ---       | -- |
| **openpx**             | **51.4 ms**  | **37.3 ms** | 32.1 ms | 138.3 ms  | 40 |
| polymarket-rs-client   | 50.3 ms      | 38.0 ms   | 30.1 ms   | 127.3 ms  | 40 |
| baseline-reqwest       | 74.4 ms      | 46.5 ms   | 31.0 ms   | 410.7 ms  | 40 |
| py-clob-client         | 71.6 ms      | 67.6 ms   | 57.6 ms   | 106.4 ms  | 40 |

Comparisons (median-based, less noise-sensitive than mean):

- **openpx vs polymarket-rs-client**: 37.3 vs 38.0 ms → openpx **1.8% faster**.
  Effectively tied; the HTTP tunings give us parity with polymarket-rs-client's
  out-of-the-box defaults.
- **openpx vs py-clob-client**: 37.3 vs 67.6 ms → openpx **45% faster**
  (1.8× speedup).
- **openpx vs baseline-reqwest**: 37.3 vs 46.5 ms → openpx **20% faster**.
  The `tuned_client_builder` tunings (HTTP/2 512 KB stream window,
  `tcp_nodelay`, pool sizing, keep-alive) are paying off vs stock
  `reqwest::Client::new()`.

## Computational — cargo bench

| Benchmark                        | openpx optimized | Notes                                     |
| ---                              | ---              | ---                                       |
| Orderbook 1000 ops               | **43.5 µs**      | `partition_point + Vec::insert` beats push+sort |
| Orderbook spread / mid / best    | ~636 ps          | `FixedPrice` integer math                 |
| JSON parse 480 KB (simd-json)    | **443 µs**       | simd-json borrowed-value path             |
| JSON parse 480 KB (serde_json)   | 1.010 ms         | serde_json for reference                  |
| WS book hot path, 1 lvl          | 387 ns           | small frame → serde_json (SIMD skipped)   |
| WS book hot path, 16 lvls        | 2.535 µs         | SIMD active (≥ 512 B)                     |
| WS book hot path, 64 lvls        | 8.530 µs         | SIMD active                               |

`polymarket-rs-client` / `py-clob-client` don't publish comparable
computational benches — the above numbers are tracked for regression, not
comparison.

## What landed to get here

### Shared infrastructure in `px-core`

Every exchange (Polymarket, Kalshi, Opinion, and any future addition) pulls
tunings from one place. Adding a new exchange is one `features = ["http",
"simd-json"]` on the `px-core` dep plus a couple of helper calls.

- `px_core::http::tuned_client_builder()` — pre-tuned
  `reqwest::ClientBuilder` (HTTP/2 512 KB stream window, `tcp_nodelay`,
  keep-alive, pooled connections, no proxy).
- `px_core::decode_frame<T>` / `decode_value` — single-pass JSON parse with
  size-gated simd-json switch (512 B threshold). Small frames (`<512 B` —
  price-change deltas, pings, acks) stay on `serde_json::from_str` because
  SIMD startup exceeds the tokenizer speedup; large frames (book snapshots,
  trade batches) switch to `simd_json::serde::from_slice`.
- `px_core::parse_level(price_str, size_str)` — one-call string →
  `PriceLevel` helper. Integer-tick parse, skips the f64 round-trip.
- `px_core::BufferPool` — 512 KB pool with shrink-on-bloat and prewarm.
- `px_core::hash::FastHashMap` / `FastHashSet` — ahash aliases.
- `px_core::TapeScratch` (simd-json feature) — reusable simd-json buffers
  for future zero-alloc decoders.

### Core optimisations

- `insert_bid` / `insert_ask`: `partition_point + Vec::insert`
  (O(log n + n)) replaces `push + sort_unstable` (O(n log n)) — gave a
  ~23× speedup on the 1000-op microbench.
- `apply_bid_level` / `apply_ask_level`: new replace-or-insert helpers
  with sorted-associative-map semantics for delta application.

### Adoption on every exchange

- **polymarket** — `client.rs` + `fetcher.rs` use `tuned_client_builder`;
  `websocket.rs` uses `decode_frame` + `parse_level` in the book /
  price-change handlers. `handle_single_message` takes `RawWsMessage` by
  value (no `serde_json::Value` intermediate, no `value.clone()`).
- **kalshi** — `exchange.rs` + `fetcher.rs` use `tuned_client_builder`;
  `websocket.rs::parse_levels` uses `parse_level`; `handle_message` uses
  `decode_value`.
- **opinion** — `exchange.rs` uses `tuned_client_builder`;
  `websocket.rs::handle_message_at` uses `decode_value`.

## Reproducing

```bash
# Computational
cargo bench -p px-bench \
  --bench ws_hot_path --bench json_parse_480k --bench orderbook_1000 \
  -- --warm-up-time 1 --measurement-time 3 --sample-size 30

# Network (openpx + polymarket-rs-client + py-clob-client + baseline)
pip install py-clob-client  # one-time
cargo run --release -p px-bench --bin openpx-bench-network -- \
  --iterations 40 --delay-ms 100 --warmup 10
```

`polymarket-rs-client` is built on demand from the standalone crate at
`engine/bench/external/polymarket_rs_bench/` (it has its own `Cargo.lock`
because its `pkcs8` pin conflicts with our kalshi crate's alloy stack).

JSON snapshots under `bench-results/baseline/` and `bench-results/optimized/`.

## Still open

- `TapeScratch` BorrowedValue decoder wired into production WS handlers.
  Would eliminate the per-message `Vec<u8>` copy in the SIMD path. Staged
  in `px-core` but not yet adopted; deferred until we've run a few
  production cycles on the current decode path.
- EIP-712 domain + typehash caching in polymarket `exchange.rs` — would
  cut ~50-100 µs off `sign_request_us` per order. No bench in `px-bench`
  yet; add when benchmarking the order-submission path.
- Kalshi and Opinion official client comparisons. The benchmark harness
  has the structure to drop them in as `--targets kalshi-py,opinion-py`
  once we script the equivalent Python probes.
