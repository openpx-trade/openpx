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

**Real-World API Performance (with network I/O)** — `fetch_orderbook`

End-to-end performance against live Polymarket and Kalshi orderbook endpoints, including network latency, JSON parsing, and decompression:

| Lang | Exchange | OpenPX | Official SDK | Speedup |
|---|---|---:|---|---:|
| Rust | Polymarket | **171.37 ms ± 33.44 ms** | polymarket_client_sdk_v2 173.57 ms ± 27.39 ms | **1.01× faster** |
| Rust | Kalshi | **123.05 ms ± 53.31 ms** | _no official Rust SDK_ | — |
| Python | Polymarket | **274.08 ms ± 30.76 ms** | py-clob-client 289.54 ms ± 38.36 ms | **1.06× faster** |
| Python | Kalshi | **219.57 ms ± 50.64 ms** | kalshi-python 222.69 ms ± 44.33 ms | **1.01× faster** |
| TypeScript | Polymarket | **269.74 ms ± 26.01 ms** | @polymarket/clob-client 267.75 ms ± 16.90 ms | 0.99× |
| TypeScript | Kalshi | **217.25 ms ± 41.97 ms** | @kalshi/typescript-sdk _(not installed)_ | — |

**Performance vs official SDKs:**

- **on par** with `polymarket_client_sdk_v2` (Rust · Polymarket, ratio 1.01×)
- **1.06× faster** than `py-clob-client` (Python · Polymarket)
- **on par** with `kalshi-python` (Python · Kalshi, ratio 1.01×)
- **on par** with `@polymarket/clob-client` (TypeScript · Polymarket, ratio 0.99×)

**Benchmark Methodology:** All benchmarks run side-by-side on the same machine, same network, same time using 20 iterations, 100 ms delay between requests against the public `/book` (Polymarket) and `/markets/{ticker}/orderbook` (Kalshi) endpoints. Best performance achieved with HTTP keep-alive enabled. See [`benches/comparative/`](benches/comparative/README.md) for the full implementation.

**WebSocket Support (real-time orderbook streams)**

OpenPX gives you `exchange.websocket().orderbook(asset_id)` returning typed orderbook deltas — same shape across both exchanges. The official SDKs leave WebSocket handling to the user:

| Lang | Exchange | OpenPX | Official SDK | Speedup |
|---|---|---:|---|---:|
| Rust | Polymarket | **760.71 µs** | polymarket_client_sdk_v2 1.18 ms | **1.55× faster** |
| Rust | Kalshi | ✅ Typed deltas | _no official Rust SDK_ | — |
| Python | Polymarket | ✅ Typed deltas (via FFI) | `py-clob-client` _doesn't ship WS_ | — |
| Python | Kalshi | ✅ Typed deltas (via FFI) | `kalshi-python` _doesn't ship WS_ | — |
| TypeScript | Polymarket | ✅ Typed deltas (via FFI) | `@polymarket/clob-client` _doesn't ship WS_ | — |
| TypeScript | Kalshi | ✅ Typed deltas (via FFI) | `@kalshi/typescript-sdk` _doesn't ship WS_ | — |

**WebSocket vs official SDKs:**

- **Only client** that ships typed WebSocket support across Polymarket *and* Kalshi in all three languages.
- Rust hot path decodes + applies 999 captured Polymarket book frames **1.55× faster** than `polymarket_client_sdk_v2`'s WS decoder.
- Reconnect + resync, auth, and orderbook state are first-class — the official SDKs leave all of that to you.

<sub>Last updated: 2026-05-14 · Reproduce: `just bench-compare`</sub>
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
