# openpx vs official exchange clients — benchmark report

Produced by `engine/bench/`. Two practical end-to-end comparisons:

1. **REST — fetch markets**: openpx vs `polymarket-rs-client` (Rust) vs
   `py-clob-client` (Python) vs stock `reqwest` baseline.
2. **WebSocket — subscribe → first book snapshot**: openpx's integrated
   `PolymarketWebSocket` vs hand-rolled `tokio-tungstenite` (Rust) vs
   hand-rolled `websockets` (Python). Neither `polymarket-rs-client` nor
   `py-clob-client` ships WebSocket support — both are REST-only — so the
   fair alternative for a WS benchmark is what you'd write yourself.

## 1. REST — Fetch Markets

`GET https://clob.polymarket.com/simplified-markets?next_cursor=MA==` (~480 KB
response), 40 iterations, 100 ms spacing, 10 warmup discarded, all targets
in a single process so machine / network / time are held constant.

| Target                 | Mean         | Median       | Min      | Max       |
| ---                    | ---          | ---          | ---      | ---       |
| **openpx**             | **51.4 ms**  | **37.3 ms**  | 32.1 ms  | 138.3 ms  |
| polymarket-rs-client   | 50.3 ms      | 38.0 ms      | 30.1 ms  | 127.3 ms  |
| baseline-reqwest       | 74.4 ms      | 46.5 ms      | 31.0 ms  | 410.7 ms  |
| py-clob-client         | 71.6 ms      | 67.6 ms      | 57.6 ms  | 106.4 ms  |

Median comparisons (noise-robust):

- **openpx vs polymarket-rs-client**: 37.3 vs 38.0 ms — **1.8% faster**.
  Effectively tied. Our HTTP tunings match polymarket-rs-client's own
  defaults.
- **openpx vs py-clob-client**: 37.3 vs 67.6 ms — **45% faster** (1.8× speedup).
- **openpx vs baseline-reqwest**: 37.3 vs 46.5 ms — **20% faster**.
  The `tuned_client_builder` (HTTP/2 512 KB stream window, `tcp_nodelay`,
  pool sizing, keep-alive) is paying off vs stock `reqwest::Client::new()`.

## 2. WebSocket — Subscribe to first book snapshot

Cold-start metric: every iteration opens a fresh WS connection,
subscribes to one asset, waits for the first `book` event, closes. 30
iterations, 500 ms spacing, 5 warmup discarded. A liquid + open market is
fetched once from `gamma-api.polymarket.com/markets?active=true&closed=false`
and every iteration / target subscribes to the same asset_id.

| Target                 | Mean         | Median       | Min       | Max       |
| ---                    | ---          | ---          | ---       | ---       |
| **openpx**             | **974.1 ms** | **942.0 ms** | 756.4 ms  | 1.51 s    |
| baseline-tungstenite   | 1.02 s       | 965.0 ms     | 728.6 ms  | 1.45 s    |
| python-ws              | 1.12 s       | 1.08 s       | 764.4 ms  | 1.77 s    |

Median comparisons:

- **openpx vs baseline-tungstenite**: 942 vs 965 ms — **openpx 2.4% faster**.
  The integrated WS (state machine, dispatcher, reconnect, watchdog) does
  NOT impose measurable overhead over a hand-written tungstenite loop —
  we're actually slightly faster, likely because our SIMD-gated JSON decode
  beats manual `serde_json` on the first book payload.
- **openpx vs python-ws**: 942 vs 1080 ms — **openpx 13% faster**.

Network latency (TLS handshake + WS upgrade + server push of first book)
dominates the measurement; the openpx library overhead is a small fraction
of total time. The takeaway is that **openpx gives you the full integrated
WS stack without giving up raw-tungstenite performance**.

## 3. Computational — cargo bench

Tracked for regression; neither official client publishes comparable
internal benchmarks.

| Benchmark                        | openpx optimized | Notes                                     |
| ---                              | ---              | ---                                       |
| Orderbook 1000 ops               | **43.5 µs**      | `partition_point + Vec::insert` beats push+sort |
| Orderbook spread / mid / best    | ~636 ps          | `FixedPrice` integer math                 |
| JSON parse 480 KB (simd-json)    | **443 µs**       | simd-json borrowed-value path             |
| JSON parse 480 KB (serde_json)   | 1.010 ms         | serde_json for reference                  |
| WS book hot path, 1 level        | 387 ns           | small frame → serde_json (SIMD skipped)   |
| WS book hot path, 16 levels      | 2.535 µs         | SIMD active (≥ 512 B)                     |
| WS book hot path, 64 levels      | 8.530 µs         | SIMD active                               |

## What landed

### Shared infrastructure in `px-core`

Every exchange (Polymarket, Kalshi, Opinion, any future addition) pulls
tunings from one place. Adding a new exchange is one
`features = ["http", "simd-json"]` on the `px-core` dep plus a couple of
helper calls.

- `px_core::http::tuned_client_builder()` — pre-tuned
  `reqwest::ClientBuilder` (HTTP/2 512 KB stream window, `tcp_nodelay`,
  keep-alive, pooled connections, no proxy).
- `px_core::decode_frame<T>` / `decode_value` — single-pass JSON parse with
  size-gated simd-json switch (512 B threshold). Small frames
  (price-change deltas, pings, acks) stay on `serde_json::from_str`; large
  frames (book snapshots, trade batches) switch to simd-json.
- `px_core::parse_level(price_str, size_str)` — one-call string →
  `PriceLevel` helper.
- `px_core::BufferPool`, `FastHashMap`, `TapeScratch` — staged for
  follow-up wins.
- `insert_bid` / `insert_ask`: `partition_point + Vec::insert`
  (O(log n + n)) replaces `push + sort_unstable` (O(n log n)) — 23×
  speedup on the 1000-op microbench.

### Adoption on every exchange

- **polymarket** — `client.rs` + `fetcher.rs` use `tuned_client_builder`;
  `websocket.rs` uses `decode_frame` + `parse_level`.
- **kalshi** — same HTTP builder; `parse_levels` uses `parse_level`;
  `handle_message` uses `decode_value`.
- **opinion** — same HTTP builder; `handle_message_at` uses `decode_value`.

## Reproducing

```bash
# Computational
cargo bench -p px-bench \
  --bench ws_hot_path --bench json_parse_480k --bench orderbook_1000 \
  -- --warm-up-time 1 --measurement-time 3 --sample-size 30

# REST (openpx + polymarket-rs + py-clob + baseline)
pip install py-clob-client  # one-time
cargo run --release -p px-bench --bin openpx-bench-network -- \
  --iterations 40 --delay-ms 100 --warmup 10

# WebSocket (openpx + baseline-tungstenite + python-ws)
pip install websockets  # one-time
cargo run --release -p px-bench --bin openpx-bench-ws -- \
  --iterations 30 --delay-ms 500 --warmup 5
```

`polymarket-rs-client` is built on demand from the standalone crate at
`engine/bench/external/polymarket_rs_bench/` (own `Cargo.lock` because
its `pkcs8` pin conflicts with our kalshi crate's alloy stack).

JSON snapshots under `bench-results/baseline/` and `bench-results/optimized/`.

## Still open

- `TapeScratch` BorrowedValue decoder wired into production WS handlers.
  Eliminates the per-message `Vec<u8>` copy in the SIMD path. Staged in
  `px-core` but not adopted yet.
- EIP-712 domain + typehash caching in polymarket `exchange.rs` — would
  cut ~50-100 µs off `sign_request_us` per order. No bench in `px-bench`
  yet; add when benchmarking the order-submission path.
- Kalshi and Opinion official client comparisons. The harness has the
  structure to drop them in as `--targets kalshi-py,opinion-py` once we
  script the equivalent Python probes.
