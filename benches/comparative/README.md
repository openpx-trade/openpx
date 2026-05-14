# Comparative Benchmarks

Head-to-head measurements of OpenPX vs the **exchanges' own official
native SDKs** on real, unauthenticated orderbook data from highly-liquid
5/15-min BTC markets.

Third-party clients are deliberately excluded — users choose between
OpenPX and the SDK their exchange actually publishes, so that's the
only comparison that matters.

The numbers feed the `<!-- BENCH:START -->` block at the top of the
root [`README.md`](../../README.md). All measurement is **local** — no
shared-tenant CI runners, no CodSpeed, no remote dashboards. Run the
suite once on a stable laptop before each release; the renderer
splices the new numbers into the README block.

## Two tables, two angles

Modeled on [polyfill-rs](https://github.com/floor-licker/polyfill-rs)'s
methodology:

1. **Computational / hot path** (Rust criterion, pure CPU): same byte
   buffer in, same op. This is where OpenPX architecturally wins —
   single-pass typed decoding, sorted-vec orderbook with O(1)
   best_bid/spread/mid, zero-allocation hot paths.
2. **End-to-end REST API** (Python + TypeScript, live HTTP): every
   unauthenticated `Exchange`-trait method, 20 iterations × 100 ms gap,
   same machine, same minute. What users actually feel.

Authenticated methods (`create_order`, `cancel_order`, `fetch_positions`,
`fetch_balance`, `fetch_fills`) are deferred to a follow-up local-mock
harness — they require credentials and a stable server to be a fair
benchmark.

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

The bench fixtures aren't synthetic:

1. `tools/capture_bench_fixtures.py` calls
   `Exchange.next_active_market_in_series(...)` — the same SeriesRoller
   primitive end users will subscribe through — to resolve the
   *currently-live* market in the revolving series.
   - Polymarket: `btc-up-or-down-5m` → live 5-minute BTC up/down market.
   - Kalshi:     `KXBTC15M`          → live 15-minute BTC up/down market.
2. The script fetches that market's full orderbook from the public
   REST endpoint and writes the raw bytes to
   `benches/comparative/fixtures/`, plus a small `.meta.json` with the
   asset_id / ticker / condition_id the benches use.
3. Hot-path benches read those bytes; REST benches drive the same
   asset_id through both libraries' live HTTP paths.

## Running the suite

From the repo root:

```bash
# 1. Build the SDKs locally (if you haven't already)
just python-build
just node-build

# 2. Capture fresh fixtures from live BTC markets
python3 tools/capture_bench_fixtures.py

# 3. Run all suites + render the README block
just bench-compare
```

Or each suite individually:

```bash
# Rust hot path
cargo bench -p px-bench-comparative

# Python — Polymarket + Kalshi
pytest benches/comparative/python/bench_polymarket.py \
    --benchmark-only \
    --benchmark-json=benches/comparative/results/python_polymarket.json
pytest benches/comparative/python/bench_kalshi.py \
    --benchmark-only \
    --benchmark-json=benches/comparative/results/python_kalshi.json

# TypeScript — Polymarket + Kalshi
cd benches/comparative/typescript && npm install
node bench_polymarket.mjs > ../results/typescript_polymarket.json
node bench_kalshi.mjs > ../results/typescript_kalshi.json

# Render the README block
python3 tools/render_bench_readme.py
```

## Methodology details

- **Hardware**: whatever you run it on. Local laptop is fine, just be
  consistent across runs. Commit the JSON alongside the README update
  so the numbers are reproducible.
- **Iterations**: criterion defaults (warm-up + 100-sample steady
  state) for Rust; `pytest --benchmark-only` with `pedantic(rounds=20)`
  for Python; tinybench `iterations: 20` for TypeScript.
- **Gap**: 100 ms between live-network iterations — same as polyfill.
  Smooths upstream API jitter without making the run absurdly slow.
- **Variance**: each row reports mean ± stddev. The ± matters; a row
  that's "1.07× faster ± 90 ms" isn't actually faster.
- **Inputs**: orderbooks captured from live 5-min Polymarket and
  15-min Kalshi BTC markets — both libraries see the exact same byte
  buffer (hot-path) and hit the same upstream asset_id (REST).
