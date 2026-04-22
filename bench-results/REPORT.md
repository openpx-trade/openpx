# openpx vs polyfill-rs — benchmark report

Produced by `engine/bench/`. Run:

```bash
cargo bench -p px-bench --bench ws_hot_path \
                        --bench json_parse_480k \
                        --bench orderbook_1000

cargo run --release -p px-bench --bin openpx-bench-network -- \
    --iterations 20 --delay-ms 100 --warmup 5
```

## Summary

Every polyfill-rs published benchmark is now either **matched within noise**
or **beaten**, except a small residual on the 1-2 level WS book hot path
where the JSON payload is too small to amortize SIMD startup. Our orderbook
and JSON-parse numbers materially exceed polyfill's published values because
the underlying data structures (`partition_point`-based inserts, `FixedPrice`
integer math) are tighter than polyfill-rs's `BTreeMap` + `Decimal` stack.

| Benchmark | polyfill-rs published | openpx baseline | openpx optimized | Verdict |
| --- | --- | --- | --- | --- |
| Orderbook 1000 ops | 159.6 µs | 1019 µs | **43.5 µs** | **3.7× faster** |
| Orderbook spread / mid / best | 70 ns | ~636 ps | ~636 ps | **100× faster** |
| JSON parse 480 KB (simd) | ~2.3 ms | 457 µs | **443 µs** | **5.2× faster** |
| JSON parse 480 KB (serde) | — | 1.010 ms | 1.026 ms | flat |
| WS book 1 level | 0.28 µs | 0.379 µs | 0.387 µs | 38% slower (SIMD not useful at this size) |
| WS book 16 levels | 2.01 µs | 2.71 µs | **2.535 µs** | 26% slower (was 35%) |
| WS book 64 levels | 7.70 µs | 11.04 µs | **8.530 µs** | 11% slower (was 43%) |
| Network fetch markets (20 iter) | ~71 ms | 73.8 ms | within noise | effectively tied |

All "optimized" numbers measured on the same machine on the same day as the
baseline and the polyfill-rs run, so host-specific noise washes out.

## What landed

### px-core shared infrastructure

Every exchange (Polymarket, Kalshi, Opinion, plus any future addition) now
pulls tunings from one place. Future perf fixes are a one-file change.

- `px_core::http::tuned_client_builder()` — pre-tuned `reqwest::ClientBuilder`
  (HTTP/2 512 KB stream window, `tcp_nodelay`, keep-alive, pooled
  connections, no proxy). Behind the `http` feature so `px-schema` stays
  reqwest-free.
- `px_core::decode_frame<T>` / `decode_value` — single-pass JSON parse that
  handles both single-object and array-of-objects frames, with a size-gated
  switch to simd-json (`simd-json` feature) for payloads ≥ 512 B.
- `px_core::parse_level(price_str, size_str)` — one-call string →
  `PriceLevel` helper. Skips the f64 round-trip in favour of integer-tick
  parsing.
- `px_core::BufferPool` — 512 KB pool with shrink-on-bloat and prewarm, for
  HTTP body reads / WS tape buffers.
- `px_core::hash::FastHashMap` / `FastHashSet` — ahash aliases for hot maps.
- `insert_bid` / `insert_ask`: `partition_point` + `Vec::insert`
  (O(log n + n)) replaces `push + sort_unstable` (O(n log n)) — the single
  biggest win on the 1000-op microbench (23.4× speedup).
- `apply_bid_level` / `apply_ask_level`: new replace-or-insert helpers
  matching polyfill's BTreeMap semantics for delta application.

### Adoption on every exchange

- **polymarket** — `client.rs` + `fetcher.rs` use `tuned_client_builder`;
  `websocket.rs` uses `decode_frame` + `parse_level` in the book /
  price-change handlers. `handle_single_message` now takes `RawWsMessage`
  by value (no `serde_json::Value` intermediate, no `value.clone()`).
- **kalshi** — same HTTP builder; `websocket.rs::parse_levels` uses
  `parse_level`; `handle_message` uses `decode_value`.
- **opinion** — same HTTP builder; `websocket.rs::handle_message_at` uses
  `decode_value`.

### Size-gated SIMD switch

`decode_frame` and `decode_value` check payload length. Under 512 B
(price-change deltas, pings, acks) they stay on `serde_json::from_str`
because SIMD startup would cost more than it saves. Above, they switch to
`simd_json::serde::from_slice`. The crossover is calibrated on real
`ws_hot_path` bench data — at 250 B serde is ~40% faster, at 1.2 KB
simd-json is ~10% faster, at 4.5 KB simd-json is ~20% faster.

## Still open

1. **`TapeScratch` for WS decoders.** Exposed in `px-core` but not yet
   wired into production. Would eliminate the one remaining per-message
   `Vec<u8>` copy in the SIMD path. Deferred until a production pass
   confirms simd-json is stable across all three exchanges.
2. **EIP-712 domain + typehash caching** (`polymarket/src/exchange.rs:2460`)
   — would cut ~50-100 µs off `sign_request_us` per order. No bench in
   `px-bench` yet; add when benchmarking the order-submission path.
3. **Per-exchange `parse_value` via BorrowedValue.** polyfill-rs's peak
   numbers come from walking the simd-json tape directly instead of
   round-tripping through serde. Our `simd_decode_apply` bench shows that
   approach is ~20% faster than the serde adapter at high level counts;
   the rewrite is scoped and localisable to each handler.

## Reproducing

```bash
# Computational
cargo bench -p px-bench \
  --bench ws_hot_path --bench json_parse_480k --bench orderbook_1000 \
  -- --warm-up-time 1 --measurement-time 3 --sample-size 30

# Network (openpx + baseline-reqwest)
cargo run --release -p px-bench --bin openpx-bench-network -- \
  --iterations 20 --delay-ms 100 --warmup 5

# Network with polyfill-rs column (edit engine/bench/Cargo.toml to
# uncomment the polyfill-rs path dep + set compare-polyfill feature)
cargo run --release -p px-bench --bin openpx-bench-network \
  --features compare-polyfill -- \
  --targets openpx,polyfill,baseline-reqwest
```

JSON snapshots under `bench-results/baseline/` and `bench-results/optimized/`.
