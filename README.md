<div align="center">

# OpenPX

**Unified, open-source prediction market SDK.**

One interface to trade across Polymarket and Kalshi.
Rust engine with Python & TypeScript SDKs.

[![CI](https://github.com/openpx-trade/openpx/actions/workflows/ci.yml/badge.svg)](https://github.com/openpx-trade/openpx/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/openpx.svg)](https://crates.io/crates/openpx)
[![PyPI](https://img.shields.io/pypi/v/openpx.svg)](https://pypi.org/project/openpx/)
[![npm](https://img.shields.io/npm/v/@openpx/sdk.svg)](https://www.npmjs.com/package/@openpx/sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

<br/>

<a href="https://polymarket.com"><img src="assets/logos/polymarket.png" width="56" height="56" alt="Polymarket" style="border-radius:12px" /></a>&nbsp;&nbsp;&nbsp;
<a href="https://kalshi.com"><img src="assets/logos/kalshi.png" width="56" height="56" alt="Kalshi" style="border-radius:12px" /></a>

</div>

---

<!-- BENCH:START -->
## Performance Comparison

**Real-World WebSocket Performance (live captured frames)**

End-to-end decode + apply over 999 real Polymarket book frames captured from a live 5-min BTC market — measures the cost of turning wire bytes into typed orderbook updates, the operation that dominates an HFT loop once you're subscribed:

| Operation | OpenPX | polymarket_client_sdk_v2 |
|---|---|---|
| **Decode + apply 999 WS book frames** | **760.71 µs ± 8.03 µs** | 1.18 ms ± 13.86 µs |

**Performance vs `polymarket_client_sdk_v2`:**

- **55.3% faster**
- **9.9% more consistent** (lower coefficient of variation)
- **Only client** that ships typed WebSocket support across Polymarket *and* Kalshi in all three languages (Rust + Python + TypeScript); the official Python and TypeScript SDKs don't ship WebSocket at all.

**Benchmark Methodology:** All benchmarks run side-by-side on the same machine using criterion, decoding the same captured JSONL frames byte-for-byte. Both libraries deserialize identical inputs into their respective typed message shapes; the ratio reflects pure decoder + orderbook-apply overhead with no network jitter. See [`benches/comparative/rust/benches/hot_path.rs`](benches/comparative/rust/benches/hot_path.rs) for the complete implementation.

**Computational Performance (pure CPU, no I/O)**

| Operation | Performance | Notes |
|---|---|---|
| **WS decode + apply (999 frames)** | 760.71 µs | ~761 ns / frame, ~1.3M frames/sec, zero-allocation |
| **`Orderbook::best_bid`** | 0.63 ns | ~1.6B ops/sec, sorted-vec O(1) |
| **`Orderbook::spread`** | 0.62 ns | ~1.6B ops/sec, branchless |
| **`Orderbook::mid_price`** | 0.63 ns | ~1.6B ops/sec, branchless |

Run the WS hot-path benchmark locally with `cargo bench -p px-bench-comparative --bench hot_path`.

**Key Performance Optimizations:**

The 55.3% WebSocket speedup comes from a single-shape `decode_frame` fast path (one `serde::Deserialize` target covers `book`, `price_change`, `last_trade_price`, and `tick_size_change` — no tagged-enum dispatch), a sorted-`Vec` orderbook that keeps both sides in contiguous, cache-friendly arrays, and a zero-allocation apply pipeline that reuses level buffers instead of churning the heap.

**Memory Architecture**

The `Orderbook` stores price levels in two sorted `Vec<PriceLevel>` arrays — no `BTreeMap` pointer chasing, no per-update heap churn. Hot-path queries (`best_bid`, `best_ask`, `spread`, `mid_price`) are constant-time accesses on the head element of each `Vec`, which the compiler reduces to a load + sub. Sub-nanosecond figures above reflect this: there's nothing to do but read two `f64`s.

**Architectural Principles**

Wire bytes deserialize once into typed structs at the WebSocket ingress; every downstream consumer reads typed fields with no re-parsing. The unified `Exchange` trait dispatches via match + UFCS (no `&dyn Exchange` vtable indirection), so each exchange's `fetch_orderbook` / `ws orderbook` monomorphizes and inlines into the call site. Errors flow through `define_exchange_error!` macros that map per-exchange variants into the unified `ExchangeError` hierarchy — strict types at the boundary, trust internal code internally.

<sub>Last updated: 2026-05-14 · Methodology: [benches/comparative/README.md](benches/comparative/README.md) · Reproduce: `just bench-compare`</sub>
<!-- BENCH:END -->

## Quick Start

### Install

```bash
# Rust — add to Cargo.toml
openpx = "0.1"

# Python
pip install openpx

# TypeScript
npm install @openpx/sdk
```

### Fetch Markets

```rust
use openpx::ExchangeInner;
use serde_json::json;

#[tokio::main]
async fn main() {
    let exchange = ExchangeInner::new("kalshi", json!({})).unwrap();
    let (markets, _) = exchange.fetch_markets(&Default::default()).await.unwrap();
    for m in &markets[..5] {
        println!("{}: {}", m.id, m.title);
    }
}
```

```python
from openpx import Exchange

exchange = Exchange("kalshi")
markets = exchange.fetch_markets()
for m in markets:
    print(f"{m.id}: {m.title}")
```

```typescript
import { Exchange } from "@openpx/sdk";

const exchange = new Exchange("kalshi", {});
const markets = await exchange.fetchMarkets();
markets.forEach(m => console.log(`${m.id}: ${m.title}`));
```

```bash
# CLI
openpx kalshi fetch-markets --limit 5
```

### Place an Order

```rust
let order = exchange.create_order(
    "KXBTC-25MAR14", "Yes", OrderSide::Buy, 0.65, 10.0, HashMap::new(),
).await?;
```

```python
order = exchange.create_order("KXBTC-25MAR14", outcome="Yes", side="buy", price=0.65, size=10.0)
```

```typescript
const order = await exchange.createOrder("KXBTC-25MAR14", "Yes", "buy", 0.65, 10.0);
```

## Unified API

Every exchange exposes the same interface — switch exchanges by changing one string.

| Method | Description |
|--------|-------------|
| `fetch_markets` | List markets with pagination |
| `fetch_market` | Get a single market by ID |
| `fetch_orderbook` | L2 orderbook (bids/asks) |
| `fetch_trades` | Recent public trades |
| `create_order` | Place a limit order |
| `cancel_order` | Cancel an open order |
| `fetch_positions` | Current portfolio positions |
| `fetch_balance` | Account balance |
| `fetch_fills` | Trade execution history |
| `ws orderbook` | Real-time orderbook stream |
| `ws activity` | Real-time trade & fill stream |

## Exchange Support

| Feature | <img src="assets/logos/polymarket.png" width="20" height="20" /> Polymarket | <img src="assets/logos/kalshi.png" width="20" height="20" /> Kalshi |
|---------|:---:|:---:|
| Markets | :white_check_mark: | :white_check_mark: |
| Trading | :white_check_mark: | :white_check_mark: |
| Orderbook | :white_check_mark: | :white_check_mark: |
| Trades | :white_check_mark: | :white_check_mark: |
| Positions | :white_check_mark: | :white_check_mark: |
| Balance | :white_check_mark: | :white_check_mark: |
| Fills | :white_check_mark: | :white_check_mark: |
| WebSocket | :white_check_mark: | :white_check_mark: |

## Exchange Credentials

Each exchange is optional — only configure what you need.

| Exchange | Required Keys | Docs |
|----------|--------------|------|
| <img src="assets/logos/polymarket.png" width="16" height="16" /> Polymarket | `POLYMARKET_PRIVATE_KEY` | [docs](https://docs.polymarket.com/developers/) |
| <img src="assets/logos/kalshi.png" width="16" height="16" /> Kalshi | `KALSHI_API_KEY_ID`, `KALSHI_PRIVATE_KEY_PEM` | [docs](https://docs.kalshi.com/) |

Set them as environment variables or in a `.env` file (auto-loaded by the CLI).

## CLI

```bash
cargo install --path engine/cli

# Market data (no auth needed)
openpx kalshi fetch-markets
openpx polymarket fetch-market "0x1234..."
openpx kalshi fetch-orderbook KXBTC-25MAR14

# WebSocket streams
openpx kalshi ws-orderbook KXBTC-25MAR14
openpx polymarket ws-activity "0x1234..."

# Sports & crypto (no auth needed)
openpx sports --league nba --live-only
openpx crypto --symbols btcusdt,ethusdt

# Pipe to jq
openpx kalshi fetch-markets --limit 1 | jq '.markets[0].title'
```

## Project Structure

```
engine/
  core/               Core types, Exchange trait, error handling
  exchanges/          Exchange implementations (kalshi, polymarket)
  sdk/                Unified facade (enum dispatch)
  cli/                CLI tool
  sports/             Sports WebSocket (Polymarket live scores)
  crypto/             Crypto price WebSocket (Binance + Chainlink)
sdks/
  python/             PyO3 bindings + Pydantic models
  typescript/         NAPI-RS bindings + TS types
docs/                 Mintlify documentation site
```

## Development

```bash
cargo check --workspace                    # Type check
cargo test --workspace                     # Run tests
cargo clippy --workspace -- -D warnings    # Lint
cargo fmt --all                            # Format
just sync-all                              # Regenerate Python/TS SDKs from Rust types
```

## Star History

<div align="center">

[![Star History Chart](https://api.star-history.com/svg?repos=openpx-trade/openpx&type=Date)](https://star-history.com/#openpx-trade/openpx&Date)

</div>

## Community

- [Documentation](https://openpx.dev) — Full API reference, guides, and tutorials
- [LLM-ready docs](https://openpx.dev/llms.md) — All docs in one copy-pasteable markdown file
- [Issues](https://github.com/openpx-trade/openpx/issues) — Bugs & feature requests
- [Discussions](https://github.com/openpx-trade/openpx/discussions) — Questions & chat

## License

MIT
