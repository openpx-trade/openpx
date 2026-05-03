# e2e_tests/websockets

End-to-end coverage for the unified WebSocket surface across both exchanges
and every consumer SDK. Companion to `e2e_tests/orderbooks/` and
`e2e_tests/trading/`.

## Surfaces tested

| Surface     | Driver                                         |
|-------------|------------------------------------------------|
| Rust SDK    | `rust/test_websockets.rs`                      |
| CLI         | `cli/run.sh` → `target/release/openpx ws ...`  |
| Python SDK  | `python/test_websockets.py`                    |
| Node SDK    | `typescript/test_websockets.mjs`               |

Per-run artifacts land in `results/`.

## Channels covered

The unified WebSocket surface multiplexes three channels through a single
`WsUpdate` stream + parallel `SessionEvent` stream:

- **orderbook** — `WsUpdate::Snapshot` (initial), `Delta` (incremental),
  `Clear` (invalidation).
- **trades** — `WsUpdate::Trade { trade: ActivityTrade }`. Public tape on
  Kalshi, `last_trade_price` on Polymarket.
- **fills** — `WsUpdate::Fill { fill: ActivityFill }`. Authenticated user
  channel on both exchanges.

Subscribe/receive payload mappings are documented in:

- `docs/schemas/mappings/orderbook-stream.mdx`
- `docs/schemas/mappings/trades-stream.mdx`
- `docs/schemas/mappings/fills-stream.mdx`

(All auto-generated from `schema/mappings/*.yaml` by `tools/render_mappings.py`.)

## Input variations exercised

The Rust matrix is the canonical contract; Python/TS/CLI mirror a subset.

| # | Variation                              | Channel    | Auth   |
|---|----------------------------------------|------------|--------|
| 1 | Single market subscribe → Snapshot+Δ   | orderbook  | mixed  |
| 2 | Trades arrive on the public tape       | trades     | mixed  |
| 3 | Multi-market subscribe per-market seq  | orderbook  | mixed  |
| 4 | Subscribe → unsubscribe → re-subscribe | orderbook  | mixed  |
| 5 | Connect-then-disconnect is clean       | (none)     | mixed  |
| 6 | Bad market_id surfaces session error   | orderbook  | mixed  |
| 7 | `updates()` take-once semantics        | (any)      | mixed  |
| 8 | Polymarket public-only (no auth)       | orderbook  | none   |
| 9 | Kalshi rejects construction without creds | (any)   | n/a    |
| 10| `register_outcomes` populates `outcome` on Trade | trades | optional |
| 11| Fills opt-in (off by default)          | fills      | both   |

The fills variant requires placing a real order; default e2e never reaches
it. Set `OPENPX_LIVE_WS_FILLS=1` to enable.

## Running

```bash
# Rust (canonical matrix)
just e2e-rust websockets
# (or: OPENPX_LIVE_TESTS=1 cargo test -p px-e2e-tests --test test_websockets -- --test-threads=1)

# CLI (requires release binary at target/release/openpx)
cargo build -p px-cli --release
just e2e-cli websockets

# Python (requires `just python` once to build the SDK)
just e2e-python websockets

# TypeScript (requires `just node` once to build the addon)
just e2e-typescript websockets

# All four in sequence
just e2e-websockets
```

Credentials read from `.env` at repo root (`KALSHI_API_KEY_ID`,
`KALSHI_PRIVATE_KEY_PATH`, `POLYMARKET_PRIVATE_KEY`, `POLYMARKET_FUNDER`,
optionally `POLYMARKET_API_KEY`/`SECRET`/`PASSPHRASE`).
