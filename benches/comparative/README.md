# Comparative Benchmarks

Head-to-head measurements of OpenPX against the **exchanges' own
official native SDKs** for every prediction market it unifies, on
**real, unauthenticated** orderbook data from highly-liquid 5/15-min
BTC markets.

Third-party clients are deliberately excluded — users choose between
OpenPX and the SDK their exchange actually publishes, so that's the
only comparison that matters.

Numbers are measured on every push to `main` by
[`.github/workflows/bench.yml`](../../.github/workflows/bench.yml) and
land directly on the [CodSpeed dashboard](https://codspeed.io/openpx-trade/openpx).
The **Performance** block at the top of the root [`README.md`](../../README.md)
is regenerated from CodSpeed **per release** by the agent-driven
[`/refresh-bench-readme`](../../.claude/commands/refresh-bench-readme.md)
command — there is no in-CI rendering and no local JSON.

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
2. The script fetches that market's full orderbook from the public
   REST endpoint and writes the raw bytes to
   `benches/comparative/fixtures/`.
3. Bench runs read those bytes — same input every iteration, real
   exchange shape, no credentials required to reproduce.
4. CI re-runs step 1 on every push to `main` and commits the refreshed
   fixture back so it's never a stale snapshot.

For Python and TypeScript the bench also makes **live unauthenticated
HTTP calls** end-to-end (still no credentials), which is what end
users actually experience.

## CodSpeed instrumentation

Every harness is wrapped in
[`CodSpeedHQ/action@v3`](https://github.com/CodSpeedHQ/action) so all
three modes feed the [CodSpeed dashboard](https://codspeed.io/openpx-trade/openpx)
with PR-level regression alerts on every push to `main`:

| Mode          | Languages | What it captures |
|---------------|-----------|------------------|
| **CPU sim**   | Rust      | Cachegrind instruction counts. Hardware-agnostic, <1% variance. Best for regression detection. |
| **Memory**    | Rust      | Heap allocations / peak usage via eBPF. Locks in the zero-alloc design. |
| **Walltime**  | Rust + Python + TS | Real wall-clock time on stable CodSpeed runners — what users feel. |

CPU simulation and memory are Linux-only (Valgrind / eBPF); they run
under CodSpeed's `codspeed-macro` runner. Walltime runs everywhere
including local laptops via `cargo bench` / `pytest` / `node bench.mjs`.

The dashboard surfaces all three Rust modes plus walltime for the
Python and TypeScript SDK harnesses, with full per-PR deltas.

## Refreshing the README block

The README's `<!-- BENCH:START -->` block is regenerated **per release**,
not per push. The flow:

1. release-please opens a release PR.
2. A maintainer checks out that branch and runs `/refresh-bench-readme`
   in a Claude Code session.
3. The agent calls the CodSpeed MCP server, pulls the latest `main` run,
   maps bench URIs to README cells, and rewrites the block.
4. The maintainer commits the README change onto the release-please
   branch before merging.

This is agent-driven because CodSpeed has no public REST API — only
the OAuth-gated MCP server (`mcp.codspeed.io`), which CI can't use
but an authenticated session can. See
[`.claude/commands/refresh-bench-readme.md`](../../.claude/commands/refresh-bench-readme.md)
for the exact mapping table and rendering rules.

## Running locally

From the repo root:

```bash
# Refresh fixtures + run all three suites (vanilla harnesses for
# walltime; CodSpeed instruments only engage under `cargo codspeed run`
# / `pytest --codspeed` / CodSpeed-wrapped node).
just bench-compare

# Or each suite individually:
python3 tools/capture_bench_fixtures.py
cargo bench -p px-bench-comparative
pytest benches/comparative/python/bench_polymarket.py
pytest benches/comparative/python/bench_kalshi.py
node benches/comparative/typescript/bench_polymarket.mjs
node benches/comparative/typescript/bench_kalshi.mjs

# Run under CodSpeed instruments (requires `cargo install cargo-codspeed`
# and a CODSPEED_TOKEN; results upload to codspeed.io):
cargo codspeed build -p px-bench-comparative
cargo codspeed run -p px-bench-comparative --measurement-mode simulation
cargo codspeed run -p px-bench-comparative --measurement-mode memory
cargo codspeed run -p px-bench-comparative --measurement-mode walltime
pytest --codspeed benches/comparative/python/
```

## Methodology details

- **Hardware (CI)**: `codspeed-macro` (CodSpeed's stable bench runner)
  for the deterministic Rust instruments; same runner for the live
  Python / TS calls. Local laptop numbers will differ; only relative
  speedups carry across.
- **Iterations**: criterion / codspeed-criterion-compat defaults
  (warm-up + 100-sample steady state) for Rust; pytest-codspeed
  auto-rounds for Python; tinybench 2 s budget per task for
  TypeScript.
- **Variance**: each row reports the mean. Live-network rows carry
  whatever variance the upstream API is feeling that minute; the
  Rust fixture row is deterministic to ±1% under CPU simulation.
- **Inputs**: orderbooks captured from live 5-min Polymarket and
  15-min Kalshi BTC markets — both libraries see the exact same byte
  buffer.
