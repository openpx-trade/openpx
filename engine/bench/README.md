# px-bench — openpx vs polyfill-rs benchmark harness

Internal, unpublished crate. Compares openpx latency against
[polyfill-rs](https://github.com/floor-licker/polyfill-rs) and other
Polymarket clients using the same methodology polyfill-rs publishes.

## Running

**Computational (Criterion, CPU only, no network):**

```bash
cargo bench -p px-bench --bench ws_hot_path \
                        --bench json_parse_480k \
                        --bench orderbook_1000
```

HTML reports land under `target/criterion/`.

**Network (real-world latency, 20-iteration 100ms-delay run):**

```bash
# openpx + baseline-reqwest only (no sibling checkout needed)
cargo run --release -p px-bench --bin openpx-bench-network -- \
    --iterations 20 --delay-ms 100 --warmup 5

# Add polyfill-rs (requires ../../../polyfill-rs checkout)
cargo run --release -p px-bench --bin openpx-bench-network \
    --features compare-polyfill -- \
    --targets openpx,polyfill,baseline-reqwest

# Add py-clob-client column (pip install py-clob-client first)
cargo run --release -p px-bench --bin openpx-bench-network \
    --features compare-polyfill -- \
    --targets openpx,polyfill,baseline-reqwest,py-clob
```

JSON reports land under `bench-results/`.

## Methodology

Mirrors [polyfill-rs's `side_by_side_benchmark.rs`](https://github.com/floor-licker/polyfill-rs/blob/a63a170/examples/side_by_side_benchmark.rs):

- 20 timed iterations per target, 100 ms delay between requests.
- 5 warmup requests (discarded) so DNS / TLS session / HTTP/2 connection pool
  are primed before measurement.
- All targets run in a single process invocation so machine, network, and
  time of day are held constant.
- 500 ms settle between targets.

## Fixtures

- `fixtures/simplified_markets_480k.json` — one-time `curl` capture of the
  `/simplified-markets` response. Commit and re-use; re-capture only if
  Polymarket changes the schema.
- WS book fixtures are synthesized at bench startup so byte-identical inputs
  are produced every run.

## Not included

- `polymarket-rs-client` — its 0.1 release pins `pkcs8` in a way that
  conflicts with `alloy` in our kalshi crate. polyfill-rs already publishes
  a head-to-head number against it (README: 409 ms vs polyfill's 322 ms);
  if we beat polyfill, we transitively beat `polymarket-rs-client`.

## CI

Not run in CI. Network benches are too flaky for CI, and the sibling
`polyfill-rs` checkout isn't available to the CI runner.
