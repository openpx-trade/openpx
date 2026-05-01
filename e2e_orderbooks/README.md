# e2e_orderbooks

End-to-end coverage for the unified orderbook surface across every input
variation, every exchange, and every consumer SDK.

## Surfaces tested

| Surface     | Driver                                     | Cases | Result        |
|-------------|--------------------------------------------|-------|---------------|
| Rust SDK    | `engine/sdk/tests/e2e_orderbooks.rs`       | 27    | ✓ 27 / 0 fail |
| CLI         | `cli/run.sh` → `target/release/openpx`     | 21    | ✓ 21 / 0 fail |
| Python SDK  | `python/test_orderbooks.py`                | 25    | ✓ 25 / 0 fail |
| Node SDK    | `typescript/test_orderbooks.mjs`           | 25    | ✓ 25 / 0 fail |
| **Total**   |                                            | **98**| **✓ 98 / 0**  |

Per-run artifacts live in `results/`:

- `results/rust.log` — full `cargo test -- --nocapture` output
- `results/cli.log`  — every CLI invocation, the JSON it emitted, and PASS/FAIL
- `results/python.json` — structured per-case summary
- `results/typescript.json` — structured per-case summary

## Methods covered

The unified orderbook surface has five entry points; every one of them is
exercised on both Kalshi and Polymarket from every SDK:

- `fetch_orderbook(asset_id) -> Orderbook`
- `fetch_orderbooks_batch(asset_ids) -> Vec<Orderbook>`
- `fetch_orderbook_stats(asset_id) -> OrderbookStats`
- `fetch_orderbook_impact(asset_id, size) -> OrderbookImpact`
- `fetch_orderbook_microstructure(asset_id) -> OrderbookMicrostructure`

## Input variations

Each suite walks through the same input matrix:

- valid `asset_id` (single)
- multi-asset_id batch (3 assets)
- empty `asset_ids` list (batch returns `[]`)
- above-cap `asset_ids` list — Kalshi 101 entries → `InvalidOrder`
- nonexistent `asset_id` — Kalshi returns empty book, Polymarket 404 (both honour the unified contract: caller never sees a populated book)
- malformed `asset_id` — empty book or error
- impact size: small (full fill), large (partial), zero (`InvalidInput`), negative (`InvalidInput`)
- cross-exchange unification — same JSON keys, same invariants, same downstream pipeline

## Numeric invariants enforced everywhere

- bids sorted descending, asks sorted ascending
- every level has `0 < price < 1` and `size > 0`
- no crossed book — `best_bid ≤ best_ask`
- `stats.bid_depth` / `ask_depth` re-sum the raw book exactly
- `stats.mid` ∈ `[best_bid, best_ask]`
- `stats.spread_bps ≥ 0`, `stats.imbalance ∈ [-1, 1]`
- `weighted_mid ∈ [best_bid, best_ask]`
- `impact.buy_avg_price ≥ best_ask`, `impact.sell_avg_price ≤ best_bid`
- `0 ≤ buy_fill_pct, sell_fill_pct ≤ 100`
- oversized impact reports a partial fill on at least one side
- `microstructure.depth_buckets` are monotonic non-decreasing 10 → 50 → 100 bps
- `microstructure.level_count` matches raw book lengths
- `microstructure.max_gap_bps ≥ 0`

## Running

```bash
# Rust
OPENPX_LIVE_TESTS=1 cargo test -p openpx --test e2e_orderbooks -- --test-threads=2

# CLI (requires release binary at target/release/openpx)
cargo build -p px-cli --release
bash e2e_orderbooks/cli/run.sh

# Python (requires `just python` once to build the SDK)
sdks/python/.venv/bin/python e2e_orderbooks/python/test_orderbooks.py

# TypeScript (requires `just node` once to build the addon)
node e2e_orderbooks/typescript/test_orderbooks.mjs
```

Credentials read from `.env` at repo root (`KALSHI_API_KEY_ID`,
`KALSHI_PRIVATE_KEY_PATH`, `POLYMARKET_PRIVATE_KEY`, `POLYMARKET_FUNDER`).

## Bugs found and fixed

This pass surfaced and fixed three real defects:

1. **CLI surface gap** — only `fetch-orderbook` was wired; the four other
   orderbook methods (`fetch-orderbooks-batch`, `fetch-orderbook-stats`,
   `fetch-orderbook-impact`, `fetch-orderbook-microstructure`) were
   missing. Added in `engine/cli/src/main.rs`.

2. **Python wrapper drift** — `sdks/python/python/openpx/exchange.py` had a
   stale `fetch_orderbook(market_ticker, outcome, token_id)` signature from
   before the `asset_id` migration, and was missing the four other
   orderbook methods entirely. Realigned to the current native bindings.

3. **Missing native binding** — neither the PyO3 nor the NAPI layer exposed
   `fetch_orderbooks_batch`. Added to both
   (`sdks/python/src/exchange.rs`, `sdks/typescript/src/exchange.rs`),
   regenerated `index.d.ts`, and verified end-to-end.
