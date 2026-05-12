# Comparative Benchmarks

Head-to-head measurements of OpenPX against the **exchanges' own
official native SDKs** for every prediction market it unifies, on
**real, unauthenticated** orderbook data from highly-liquid 5/15-min
BTC markets.

Third-party clients are deliberately excluded — users choose between
OpenPX and the SDK their exchange actually publishes, so that's the
only comparison that matters.

Numbers feed the **Performance** block at the top of the root
[`README.md`](../../README.md) and refresh on every push to `main`
via [`.github/workflows/bench.yml`](../../.github/workflows/bench.yml).

## What's measured

| Language    | Polymarket comparison target  | Kalshi comparison target |
|-------------|-------------------------------|--------------------------|
| Rust        | `polymarket_client_sdk_v2`    | _no upstream Rust SDK_   |
| Python      | `py-clob-client`              | `kalshi-python`          |
| TypeScript  | `@polymarket/clob-client`     | `kalshi-typescript-sdk`  |

The Rust pin tracks the same canary `engine/exchanges/polymarket`
already depends on, so a comparison-target bump is impossible to land
without also touching the production crate.

## How real data flows in

The bench fixtures aren't synthetic. The capture pipeline is:

1. `tools/capture_bench_fixtures.py` calls
   `Exchange.next_active_market_in_series(...)` — the same
   SeriesRoller primitive end users will subscribe through — to
   resolve the *currently-live* market in the revolving series.
   - Polymarket: `btc-up-or-down-5m` → live 5-minute BTC up/down market.
   - Kalshi: `KXBTC15M` → live 15-minute BTC up/down market.
2. The script then fetches that market's full orderbook from the
   public REST endpoint and writes the raw bytes to
   `benches/comparative/fixtures/`.
3. Bench runs read those bytes — same input every iteration, real
   exchange shape, no credentials required to reproduce.
4. CI re-runs step 1 on every push to `main` so the committed
   fixture is always fresh — never a stale snapshot.

For Python and TypeScript the bench also makes **live unauthenticated
HTTP calls** end-to-end (still no credentials), which is what end
users actually experience.

## Codspeed instrumentation

Every harness is wrapped in
[`CodSpeedHQ/action@v3`](https://github.com/CodSpeedHQ/action) so all
three modes feed the [Codspeed dashboard](https://codspeed.io/openpx-trade/openpx)
with PR-level regression alerts on every push to `main`:

| Mode          | Languages | What it captures |
|---------------|-----------|------------------|
| **CPU sim**   | Rust      | Cachegrind instruction counts. Hardware-agnostic, <1% variance. Best for regression detection. |
| **Memory**    | Rust      | Heap allocations / peak usage via eBPF. Locks in the zero-alloc design. |
| **Walltime**  | Rust + Python + TS | Real wall-clock time on stable Codspeed runners — what users feel. |

CPU simulation and memory are Linux-only (Valgrind / eBPF); they run
under Codspeed's `codspeed-macro` runner. Walltime runs everywhere
including local laptops via `cargo bench` / `pytest` / `node bench.mjs`.

The README block surfaces all three Rust modes plus walltime for the
SDK harnesses:

- **Rust walltime** — `cargo bench` (criterion local JSON)
- **Rust CPU instructions** — `cargo bench --features iai` runs
  `parse_polymarket_book_iai.rs` under valgrind/cachegrind; the
  `Ir` cost lands in `target/iai/.../summary.json`.
- **Rust heap allocations** — same iai-callgrind run, with DHAT
  attached as a second valgrind tool; `total_bytes` lands in the same
  summary file.
- **Python / TS walltime** — pytest-benchmark / tinybench local JSON.

The Codspeed dashboard surfaces the same three Rust modes plus per-PR
deltas. Numbers should agree to within a few percent between the iai
local JSON and Codspeed's simulation/memory reports — both use the
same valgrind tools.

### Why the workflow runs each harness twice

The dashboard pass (`codspeed run -m walltime` / `pytest --codspeed` /
Codspeed-wrapped node) feeds Codspeed's collector, which captures
measurements via its own protocol and *does not* emit local JSON. A
second non-instrumented pass (`cargo bench` / `pytest --benchmark-only`
/ raw `node`) writes the local JSON files the README render reads.
The cost is ~25% extra CI time; the win is one pipeline producing
both a live dashboard and an in-repo table.

## Running locally

From the repo root:

```bash
# Refresh fixtures + run all three suites + rewrite the README block.
just bench-compare

# Or each suite individually:
python3 tools/capture_bench_fixtures.py
cargo bench -p px-bench-comparative
pytest benches/comparative/python/bench_polymarket.py \
    --benchmark-only \
    --benchmark-json=benches/comparative/results/python_polymarket.json
pytest benches/comparative/python/bench_kalshi.py \
    --benchmark-only \
    --benchmark-json=benches/comparative/results/python_kalshi.json
node benches/comparative/typescript/bench_polymarket.mjs \
    > benches/comparative/results/typescript_polymarket.json
node benches/comparative/typescript/bench_kalshi.mjs \
    > benches/comparative/results/typescript_kalshi.json
python3 tools/render_bench_readme.py

# Run under Codspeed instruments (requires `cargo install cargo-codspeed`
# and a CODSPEED_TOKEN; results upload to codspeed.io):
cargo codspeed build -p px-bench-comparative
cargo codspeed run -p px-bench-comparative --measurement-mode simulation
cargo codspeed run -p px-bench-comparative --measurement-mode memory
cargo codspeed run -p px-bench-comparative --measurement-mode walltime
pytest --codspeed benches/comparative/python/

# Or produce the Rust CPU-instruction + heap-allocation summaries that
# feed the README (Linux only — requires valgrind + iai-callgrind-runner):
cargo install iai-callgrind-runner@0.14
cargo bench -p px-bench-comparative --features iai --bench parse_polymarket_book_iai
```

## Methodology details

- **Hardware (CI)**: `codspeed-macro` (CodSpeed's stable bench runner)
  for the deterministic Rust instruments; same runner for the live
  Python / TS calls. Local laptop numbers will differ; only relative
  speedups carry across.
- **Iterations**: criterion / codspeed-criterion-compat defaults
  (warm-up + 100-sample steady state) for Rust; pytest-benchmark
  auto-rounds for Python; tinybench 2 s budget per task for
  TypeScript.
- **Variance**: each row reports the mean. Live-network rows carry
  whatever variance the upstream API is feeling that minute; the
  Rust fixture row is deterministic to ±1% under CPU simulation.
- **Inputs**: orderbooks captured from live 5-min Polymarket and
  15-min Kalshi BTC markets — both libraries see the exact same byte
  buffer.
