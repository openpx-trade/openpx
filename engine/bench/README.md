# px-bench — openpx vs official exchange client libraries

Internal, unpublished crate. Measures openpx's exchange implementations
(polymarket, kalshi, opinion) against the official client libraries
Polymarket publishes for each language — `polymarket-rs-client` (Rust,
crates.io) and `py-clob-client` (Python, PyPI) — and a stock `reqwest`
baseline as a sanity floor.

## Running

### Computational (Criterion, no network)

```bash
cargo bench -p px-bench --bench ws_hot_path \
                        --bench json_parse_480k \
                        --bench orderbook_1000
```

HTML reports land under `target/criterion/`.

### Network (real-world latency, 20 iterations, 100 ms gap)

```bash
# Default: openpx + baseline-reqwest + py-clob + polymarket-rs
cargo run --release -p px-bench --bin openpx-bench-network -- \
    --iterations 20 --delay-ms 100 --warmup 5

# Subset
cargo run --release -p px-bench --bin openpx-bench-network -- \
    --targets openpx,py-clob
```

Requirements:

- `py-clob`: `pip install py-clob-client`.
- `polymarket-rs`: no extra setup — the external crate at
  `external/polymarket_rs_bench/` is built by the harness on demand.

JSON reports land under `bench-results/`.

## Methodology

- 20 timed iterations per target, 100 ms delay between requests.
- 5 warmup requests (discarded) so DNS / TLS session / HTTP/2 connection
  pool are primed before measurement.
- All targets run in a single process invocation so machine, network, and
  time of day are held constant.
- 500 ms settle between targets so TLS session state doesn't bleed.
- Endpoint: `GET https://clob.polymarket.com/simplified-markets?next_cursor=MA==`
  (~480 KB JSON response).

## Why polymarket-rs-client lives outside the workspace

`polymarket-rs-client` 0.1 pins `pkcs8 = "=0.10.2"` through `alloy-signer-local`.
Our kalshi crate needs `pkcs8 = "^0.10"` (we pick 0.10.1). Cargo refuses to
resolve both in the same workspace. Two options existed: downgrade kalshi
or isolate polymarket-rs-client. We pick isolation — the standalone crate
at `engine/bench/external/polymarket_rs_bench/` has its own Cargo.lock and
is subprocess-invoked by `openpx-bench-network`.

## Fixtures

- `fixtures/simplified_markets_480k.json` — one-time `curl` capture of the
  `/simplified-markets` response. Commit and re-use.
- WS book fixtures are synthesised at bench startup.

## CI

Not run in CI. Network benches are too flaky for CI. The existing
`cargo bench -p px-core` job in `.github/workflows/ci.yml` continues to
act as a compilation-regression smoke test for `px-core` benches.
