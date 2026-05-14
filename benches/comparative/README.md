# Comparative Benchmarks

Head-to-head measurements of OpenPX vs the exchanges' own official
native SDKs on real, unauthenticated 5/15-min BTC markets.

Third-party clients are deliberately excluded — users choose between
OpenPX and the SDK their exchange actually publishes, so that's the
only comparison that matters.

The numbers feed the `<!-- BENCH:START -->` block at the top of the
root [`README.md`](../../README.md). All measurement is **local** — no
CodSpeed, no shared-tenant CI runners, no remote dashboards. Run the
suite once on a stable laptop before each release; the renderer
splices the new numbers into the README block.

## Three sections, three angles

1. **Rust hot path** — where OpenPX architecturally wins.
   - `ws_decode_apply`: side-by-side vs `polymarket_client_sdk_v2` on
     999 real WebSocket frames captured from a live 5-min BTC market.
     Same bytes, same op — purely measures decode cost.
   - `apply_book_updates`: OpenPX-only sustained throughput.
   - `orderbook_ops`: constant-time `best_bid`/`spread`/`mid_price` —
     primitives the official SDKs don't expose at all.
2. **REST `fetch_orderbook`** — the one operation where both libraries
   hit the same upstream endpoint and return the same shape. Anything
   else is apples-to-oranges (different endpoints, different payload
   sizes), see "What we don't compare" below.
3. **WebSocket** — none of the official Python or TypeScript SDKs
   ship WebSocket support, so we report (a) a feature matrix and
   (b) the cost of replicating it yourself in pure Python / TypeScript
   on the same captured frames.

## What we don't compare (and why)

- **`fetch_markets`** — OpenPX hits Polymarket's Gamma `/events/keyset`
  (5.9 MB payload, full event tree → typed unified Market) while
  py-clob hits CLOB `/sampling-markets` (2.4 MB, raw dicts). Different
  endpoints on different services. OpenPX processes ~1.66× more bytes
  per second but the wall-clock comparison is unfair because we're
  doing more work.
- **`fetch_trades`** — same shape problem: different endpoints,
  different trade-event semantics.
- **`fetch_market` (singular)** — not exposed on the OpenPX Python/TS
  SDKs yet.

If you want a benchmark for any of the above, open a PR — but it needs
to control for the endpoint disparity.

## What we compare

| Language    | Polymarket comparison target  | Kalshi comparison target |
|-------------|-------------------------------|--------------------------|
| Rust        | `polymarket_client_sdk_v2`    | _no upstream Rust SDK_   |
| Python      | `py-clob-client`              | `kalshi-python`          |
| TypeScript  | `@polymarket/clob-client`     | _no upstream TS SDK_     |

The Rust pin tracks the same canary `engine/exchanges/polymarket`
already depends on, so a comparison-target bump is impossible to land
without also touching the production crate.

## How real data flows in

1. `tools/capture_bench_fixtures.py` calls
   `Exchange.next_active_market_in_series(...)` — the same
   SeriesRoller primitive end users will subscribe through — to
   resolve the *currently-live* market in the revolving series.
   - Polymarket: `btc-up-or-down-5m` → live 5-minute BTC up/down market.
   - Kalshi:     `KXBTC15M`          → live 15-minute BTC up/down market.
2. The script fetches that market's full orderbook from the public
   REST endpoint and writes the raw bytes to
   `benches/comparative/fixtures/`, plus a small `.meta.json` with the
   asset_id / ticker / condition_id the benches read.
3. For the WebSocket hot path, the script then subscribes to
   Polymarket's market WS and records the next ~1000 book +
   price_change + last_trade_price frames as JSONL. Hot-path benches
   replay these bytes deterministically — no network, no jitter.
4. REST benches drive the asset_id/ticker live in tight loops (20
   iterations × 100 ms gap) — both libraries hit the same upstream URL
   at the same minute.

## Running the suite

```bash
# 1. Build the SDKs locally (if you haven't already)
just python-build
just node-build

# 2. Capture fresh fixtures (REST snapshots + 1000 WS frames)
python3 tools/capture_bench_fixtures.py

# 3. Run all suites + render the README block
just bench-compare
```

Or each suite individually:

```bash
# Rust hot path (criterion, plain — no CodSpeed)
cargo bench -p px-bench-comparative

# Python — fetch_orderbook + WS DIY
pytest benches/comparative/python/bench_polymarket.py --benchmark-only \
    --benchmark-json=benches/comparative/results/python_polymarket.json
pytest benches/comparative/python/bench_kalshi.py --benchmark-only \
    --benchmark-json=benches/comparative/results/python_kalshi.json
pytest benches/comparative/python/bench_ws_diy.py --benchmark-only \
    --benchmark-json=benches/comparative/results/python_ws_diy.json

# TypeScript — fetch_orderbook + WS DIY
cd benches/comparative/typescript && npm install --legacy-peer-deps
node bench_polymarket.mjs > ../results/typescript_polymarket.json
node bench_kalshi.mjs     > ../results/typescript_kalshi.json
node bench_ws_diy.mjs     > ../results/typescript_ws_diy.json

# Render the README block
python3 tools/render_bench_readme.py
```

## Methodology details

- **Hardware**: whatever you run it on. Local laptop is fine, just be
  consistent across runs. Commit the JSON alongside the README update
  so the numbers are reproducible.
- **Iterations**: criterion defaults for Rust (warm-up + 100-sample
  steady state); `pytest --benchmark-only` with `pedantic(rounds=20)`
  for Python; tinybench `iterations: 20` for TypeScript.
- **Gap**: 100 ms between live-network iterations — same as
  polyfill-rs's methodology. Smooths upstream API jitter without
  making the run absurdly slow.
- **Variance**: each row reports mean ± stddev. The ± matters; a row
  that's "1.07× faster ± 90 ms" isn't actually faster.
- **Inputs**: both libraries see the exact same bytes (hot-path) and
  hit the same upstream asset_id (REST).
