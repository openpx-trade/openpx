# Comparative Benchmarks

Head-to-head WebSocket measurements of OpenPX vs Polymarket's official
Rust client (`polymarket_client_sdk_v2`) on real, captured 5-min BTC
book frames.

REST `fetch_orderbook` is intentionally **not** benched — it's
network-bound, every client lands within a few percent of every other
client, and the table hides more than it reveals. The numbers users
should care about live on the WebSocket hot path: decode + apply,
sustained throughput, and the constant-time orderbook primitives.

The criterion output feeds the `<!-- BENCH:START -->` block at the top
of the root [`README.md`](../../README.md). All measurement is
**local** — no CodSpeed, no shared-tenant CI runners, no remote
dashboards. Run the suite once on a stable laptop before each release;
the renderer splices the new numbers into the README block.

## What we measure

1. **`ws_decode_apply`** — side-by-side vs `polymarket_client_sdk_v2`
   on 999 real WebSocket frames (book + price_change +
   last_trade_price) captured from a live 5-min BTC market. Same bytes,
   same op — purely measures decode cost. This is the head-to-head row
   in the README's Real-World table.
2. **`apply_book_updates`** — OpenPX-only sustained throughput at 16 /
   128 / 1024 updates per round. Measures the zero-allocation apply
   pipeline.
3. **`orderbook_ops`** — constant-time `best_bid` / `best_ask` /
   `spread` / `mid_price` queries on a populated book. Primitives the
   official SDKs don't expose at all.

## What we don't bench (and why)

- **REST endpoints** (`fetch_markets`, `fetch_orderbook`,
  `fetch_trades`) — dominated by network RTT. The Rust vs Python/TS
  gap on a single REST call is single-digit milliseconds out of ~200
  ms; the OpenPX vs SDK gap inside the same language is near zero. Not
  a useful story for the README.
- **Python / TypeScript WebSocket** — none of the official Python or
  TypeScript SDKs ship a WebSocket client at all, so there's nothing
  to head-to-head against. The OpenPX Python and TS SDKs use the Rust
  WebSocket engine via FFI, so the Rust hot-path numbers above
  apply to all three languages.

## How real data flows in

1. `tools/capture_bench_fixtures.py` calls
   `Exchange.next_active_market_in_series("btc-up-or-down-5m")` — the
   same SeriesRoller primitive end users subscribe through — to
   resolve the currently-live 5-min BTC market.
2. The script fetches that market's full orderbook from
   `clob.polymarket.com/book` and writes the raw bytes to
   `benches/comparative/fixtures/polymarket_book.json`. The
   `apply_book_updates` and `orderbook_ops` benches use this snapshot
   to seed an `Orderbook` for measurement.
3. The script then subscribes to Polymarket's market WS and records the
   next ~1000 book + price_change + last_trade_price frames as JSONL
   into `polymarket_ws_book.jsonl`. The `ws_decode_apply` head-to-head
   replays these bytes deterministically — no network, no jitter, both
   libraries see the exact same input.

## Running the suite

```bash
# Capture fresh fixtures, run the benches, splice the README block
just bench-compare

# Or each step individually
python3 tools/capture_bench_fixtures.py
cargo bench -p px-bench-comparative --bench hot_path
python3 tools/render_bench_readme.py
```

## Methodology details

- **Hardware**: whatever you run it on. Local laptop is fine, just be
  consistent across runs. Commit the criterion JSON alongside the
  README update so the numbers are reproducible.
- **Iterations**: criterion defaults (warm-up + ~100 samples) on a
  steady-state loop.
- **Variance**: every cell in the README's headline table reports
  mean ± stddev pulled from `target/criterion/<group>/<id>/new/estimates.json`.
- **Inputs**: both libraries decode the exact same captured byte
  buffer per frame.
