<div align="center">

# OpenPX

**A unified, open-source Rust SDK for prediction markets.**
Trade across multiple prediction market exchanges through a single, consistent interface.

[![CI](https://github.com/openpx-ai/openpx/actions/workflows/ci.yml/badge.svg)](https://github.com/openpx-ai/openpx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/px-core.svg)](https://crates.io/crates/px-core)
[![Downloads](https://img.shields.io/crates/d/px-core.svg)](https://crates.io/crates/px-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![GitHub Stars](https://img.shields.io/github/stars/openpx-ai/openpx?style=social)](https://github.com/openpx-ai/openpx/stargazers)
[![GitHub Forks](https://img.shields.io/github/forks/openpx-ai/openpx?style=social)](https://github.com/openpx-ai/openpx/network/members)
[![GitHub Watchers](https://img.shields.io/github/watchers/openpx-ai/openpx?style=social)](https://github.com/openpx-ai/openpx/watchers)

[![GitHub Issues](https://img.shields.io/github/issues/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/pulls)
[![GitHub Last Commit](https://img.shields.io/github/last-commit/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/commits/main)
[![GitHub Contributors](https://img.shields.io/github/contributors/openpx-ai/openpx)](https://github.com/openpx-ai/openpx/graphs/contributors)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

</div>

---

## Supported Exchanges

| Exchange | Markets | Trading | Orderbook | WebSocket |
|----------|---------|---------|-----------|-----------|
| [Polymarket](https://polymarket.com) | Yes | Yes | Yes | Yes |
| [Kalshi](https://kalshi.com) | Yes | Yes | Yes | Yes |
| [Limitless](https://limitless.exchange) | Yes | Yes | No | Yes |
| [Opinion](https://opinion.trade) | Yes | Yes | No | No |
| [Predict.fun](https://predict.fun) | Yes | Yes | No | No |

## Architecture

```
openpx/
├── px-core/                  # Core types, traits, timing, error handling
│   ├── src/exchange/         # Exchange trait + factory + rate limiting
│   ├── src/models/           # Market, Order, Position, Orderbook
│   ├── src/timing.rs         # timed! macro + TimingGuard
│   └── src/error.rs          # OpenPxError hierarchy
├── px-exchange-polymarket/   # Polymarket implementation
├── px-exchange-kalshi/       # Kalshi implementation
├── px-exchange-limitless/    # Limitless implementation
├── px-exchange-opinion/      # Opinion implementation
├── px-exchange-predictfun/   # Predict.fun implementation
├── px-mcp/                   # MCP server (Node.js)
└── px-documentation/         # API documentation
```

## Quick Start

Add OpenPX to your `Cargo.toml`:

```toml
[dependencies]
px-core = "0.1.4"
px-exchange-kalshi = "0.1.4"  # or any exchange crate
```

### Example: Fetch Markets

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
