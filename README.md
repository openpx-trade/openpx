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
## Performance

OpenPX vs the official native SDKs, head-to-head against live 5/15-min BTC markets. Real bytes, no credentials, refreshes on every push to `main`.

[![CodSpeed](https://img.shields.io/endpoint?url=https%3A%2F%2Fcodspeed.io%2Fbadge.json)](https://codspeed.io/openpx-trade/openpx) — Rust benches also tracked under Codspeed CPU-simulation and memory-allocation instruments. Click the badge for the full per-metric history and PR-level regression alerts.

### Rust core

_Kalshi has no upstream Rust SDK — OpenPX is the only Rust client that supports it._

| Operation | OpenPX | polymarket_client_sdk_v2 | Speedup |
|---|---:|---:|---:|
| Parse Polymarket book (5-min BTC fixture) | 4.38 µs | 5.84 µs | **1.33×** |

### Python SDK

| Operation | OpenPX | py-clob-client | Speedup |
|---|---:|---:|---:|
| Polymarket fetch_orderbook (5-min BTC, live) | 276.21 ms | 362.99 ms | **1.31×** |

| Operation | OpenPX | kalshi-python | Speedup |
|---|---:|---:|---:|
| Kalshi fetch_orderbook (15-min BTC, live) | 363.96 ms | 366.15 ms | **1.01×** |

### TypeScript SDK

| Operation | OpenPX | @polymarket/clob-client | Speedup |
|---|---:|---:|---:|
| Polymarket fetch_orderbook (5-min BTC, live) | 261.89 ms | 278.14 ms | **1.06×** |

| Operation | OpenPX | kalshi-typescript | Speedup |
|---|---:|---:|---:|
| Kalshi fetch_orderbook (15-min BTC, live) | 370.13 ms | 385.13 ms | **1.04×** |

<sub>Last updated: 2026-05-05 · Methodology: [benches/comparative/README.md](benches/comparative/README.md) · Raw data: [benches/comparative/results/](benches/comparative/results/)</sub>
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
