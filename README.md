<div align="center">

# OpenPX

**The unified, open-source prediction market SDK.**
**Rust engine. Python & TypeScript SDKs. Built for speed.**

Trade across every major prediction market through one interface — with a core engine
built in Rust for the latency demands of high-frequency trading.

[![CI](https://github.com/openpx-ai/openpx/actions/workflows/ci.yml/badge.svg)](https://github.com/openpx-ai/openpx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org)
[![GitHub Stars](https://img.shields.io/github/stars/openpx-ai/openpx?style=social)](https://github.com/openpx-ai/openpx/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/openpx-ai/openpx?style=social)](https://github.com/openpx-ai/openpx/network/members)

[![GitHub Issues](https://img.shields.io/github/issues/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/pulls)
[![GitHub Last Commit](https://img.shields.io/github/last-commit/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/commits/main)
[![GitHub Contributors](https://img.shields.io/github/contributors/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/graphs/contributors)

---

### Supported Exchanges

<a href="https://polymarket.com"><img src="https://img.logo.dev/polymarket.com?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="60" height="60" alt="Polymarket" /></a>&nbsp;&nbsp;&nbsp;&nbsp;
<a href="https://kalshi.com"><img src="https://img.logo.dev/kalshi.com?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="60" height="60" alt="Kalshi" /></a>&nbsp;&nbsp;&nbsp;&nbsp;
<a href="https://limitless.exchange"><img src="https://img.logo.dev/limitless.exchange?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="60" height="60" alt="Limitless" /></a>&nbsp;&nbsp;&nbsp;&nbsp;
<a href="https://opinion.trade"><img src="https://img.logo.dev/opinion.trade?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="60" height="60" alt="Opinion" /></a>&nbsp;&nbsp;&nbsp;&nbsp;
<a href="https://predict.fun"><img src="https://img.logo.dev/predict.fun?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="60" height="60" alt="Predict.fun" /></a>

**[Polymarket](https://polymarket.com)** &middot; **[Kalshi](https://kalshi.com)** &middot; **[Limitless](https://limitless.exchange)** &middot; **[Opinion](https://opinion.trade)** &middot; **[Predict.fun](https://predict.fun)**

</div>

---

## Why OpenPX?

OpenPX is the **CCXT of prediction markets** — a single, unified interface to every major exchange. The entire engine is written in **Rust** and designed for **high-frequency trading**: zero-copy deserialization, lock-free structures, and sub-millisecond latency on hot paths.

Use it natively in Rust, or through first-class **Python** (PyO3) and **TypeScript** (NAPI-RS) SDKs that call directly into the same Rust core — no REST wrappers, no performance penalty.

| | |
|---|---|
| **Rust Core** | Zero-alloc hot paths, async I/O with Tokio, LTO-optimized release builds |
| **Python SDK** | PyO3 FFI bindings with auto-generated Pydantic models — full type safety |
| **TypeScript SDK** | NAPI-RS native bindings with auto-generated TypeScript types |
| **Unified Interface** | One `Exchange` trait across all markets — fetch, trade, stream, done |
| **WebSocket Streams** | Real-time orderbook & trade feeds with automatic reconnection |
| **Docker Ready** | `docker compose up` for 24/7 automated trading with built-in dashboard |

## Exchange Support Matrix

| Exchange | Markets | Trading | Orderbook | WebSocket |
|----------|---------|---------|-----------|-----------|
| [Polymarket](https://polymarket.com) | Yes | Yes | Yes | Yes |
| [Kalshi](https://kalshi.com) | Yes | Yes | Yes | Yes |
| [Limitless](https://limitless.exchange) | Yes | Yes | No | Yes |
| [Opinion](https://opinion.trade) | Yes | Yes | No | No |
| [Predict.fun](https://predict.fun) | Yes | Yes | No | No |

## Quick Start

### Rust

```toml
[dependencies]
px-sdk = "0.1.4"
```

```rust
use px_core::{Exchange, FetchMarketsParams};
use px_exchange_kalshi::{Kalshi, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = KalshiConfig::new()
        .with_api_key("your-api-key-id")
        .with_private_key_path("/path/to/key.pem");

    let exchange = Kalshi::new(config)?;

    let markets = exchange.fetch_markets(None).await?;
    for market in &markets {
        println!("{}: {}", market.id, market.question);
    }

    Ok(())
}
```

### Python

```bash
pip install openpx
```

```python
from openpx import Exchange

exchange = Exchange("kalshi", {
    "api_key_id": "your-api-key-id",
    "private_key_pem": "your-private-key"
})

markets = exchange.fetch_markets(limit=10)
for market in markets:
    print(f"{market.id}: {market.question}")

# Place an order
order = exchange.create_order(
    market_id="KXBTC-25MAR14",
    outcome="Yes",
    side="buy",
    price=0.65,
    size=10
)
```

### TypeScript

```bash
npm install @openpx/sdk
```

```typescript
const { Exchange } = require("@openpx/sdk");

const exchange = new Exchange("kalshi", {
  apiKeyId: "your-api-key-id",
  privateKeyPem: "your-private-key"
});

const markets = await exchange.fetchMarkets({ limit: 10 });
markets.forEach(m => console.log(`${m.id}: ${m.question}`));
```

## Architecture

```
openpx/
├── engine/                       # Rust core — powers everything
│   ├── core/                     # Core types, traits, timing, error handling
│   │   ├── src/exchange/         # Exchange trait + factory + rate limiting
│   │   ├── src/models/           # Market, Order, Position, Orderbook
│   │   ├── src/websocket/        # WebSocket infrastructure + reconnection
│   │   ├── src/timing.rs         # timed! macro + TimingGuard
│   │   └── src/error.rs          # OpenPxError hierarchy
│   ├── exchanges/                # Exchange implementations
│   │   ├── kalshi/               # Kalshi (RSA/JWT auth)
│   │   ├── polymarket/           # Polymarket (CLOB + onchain)
│   │   ├── opinion/              # Opinion
│   │   ├── limitless/            # Limitless
│   │   └── predictfun/           # Predict.fun
│   ├── sdk/                      # Unified facade (enum dispatch)
│   └── schema/                   # JSON Schema export binary
├── sdks/                         # Language SDKs (call into Rust via FFI)
│   ├── python/                   # PyO3 + auto-generated Pydantic models
│   └── typescript/               # NAPI-RS + auto-generated TS types
├── dashboard/                    # Trading terminal (Axum + vanilla JS)
├── docs/                         # Starlight documentation site
├── schema/                       # openpx.schema.json (generated artifact)
├── scripts/                      # Build & codegen scripts
├── docker-compose.yml            # 24/7 trading deployment
└── justfile                      # Single-command SDK sync
```

## The Exchange Trait

All exchanges implement the unified `Exchange` trait:

```rust
#[async_trait]
pub trait Exchange: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    async fn fetch_markets(&self, params: Option<FetchMarketsParams>) -> Result<Vec<Market>, OpenPxError>;
    async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError>;
    async fn create_order(&self, market_id: &str, outcome: &str, side: OrderSide, price: f64, size: f64, params: HashMap<String, String>) -> Result<Order, OpenPxError>;
    async fn cancel_order(&self, order_id: &str, market_id: Option<&str>) -> Result<Order, OpenPxError>;
    async fn fetch_order(&self, order_id: &str, market_id: Option<&str>) -> Result<Order, OpenPxError>;
    async fn fetch_open_orders(&self, params: Option<FetchOrdersParams>) -> Result<Vec<Order>, OpenPxError>;
    async fn fetch_positions(&self, market_id: Option<&str>) -> Result<Vec<Position>, OpenPxError>;
    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError>;
    async fn fetch_orderbook(&self, req: OrderbookRequest) -> Result<Orderbook, OpenPxError>;
    // ... and more
}
```

## Configuration

Copy `.env.example` to `.env` and add your exchange credentials:

```bash
cp .env.example .env
```

Each exchange requires its own API credentials. See the respective exchange documentation:
- [Polymarket Docs](https://docs.polymarket.com/developers/)
- [Kalshi Docs](https://docs.kalshi.com/)
- [Opinion Docs](https://docs.opinion.trade/developer-guide/opinion-open-api)
- [Limitless Docs](https://api.limitless.exchange/api-v1)
- [Predict.fun Docs](https://dev.predict.fun/)

## Docker

Run the trading dashboard 24/7:

```bash
docker compose up -d
```

The dashboard is available at `http://localhost:3000` with multi-exchange portfolio management, live orderbook data, and order execution.

## Development

```bash
# Check workspace
cargo check --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Sync SDKs + docs from Rust types
just sync-all
```

## Star History

<a href="https://star-history.com/#openpx-ai/openpx&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=openpx-ai/openpx&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=openpx-ai/openpx&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=openpx-ai/openpx&type=Date" />
 </picture>
</a>

## Contributors

<a href="https://github.com/openpx-ai/openpx/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=openpx-ai/openpx" />
</a>

## Community

- [GitHub Issues](https://github.com/openpx-ai/openpx/issues) — Bug reports & feature requests
- [GitHub Discussions](https://github.com/openpx-ai/openpx/discussions) — Questions & general chat
- [Contributing Guide](CONTRIBUTING.md) — How to contribute
- [Code of Conduct](CODE_OF_CONDUCT.md) — Community standards
- [Security Policy](SECURITY.md) — Reporting vulnerabilities
- [Changelog](CHANGELOG.md) — Release history

## License

MIT
