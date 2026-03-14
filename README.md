<div align="center">

# OpenPX

**Unified, open-source prediction market SDK.**

One interface to trade across Polymarket and Kalshi.
Rust engine with Python & TypeScript SDKs.

[![CI](https://github.com/openpx-ai/openpx/actions/workflows/ci.yml/badge.svg)](https://github.com/openpx-ai/openpx/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.91%2B-orange.svg)](https://www.rust-lang.org)

<a href="https://polymarket.com"><img src="https://img.logo.dev/polymarket.com?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="48" height="48" alt="Polymarket" /></a>&nbsp;&nbsp;
<a href="https://kalshi.com"><img src="https://img.logo.dev/kalshi.com?token=pk_JRbMGCbBRUOkIXIn2SFJNw&size=60&retina=true" width="48" height="48" alt="Kalshi" /></a>

</div>

---

## Getting Started

### Prerequisites

- [Rust 1.91+](https://rustup.rs/)
- API credentials for at least one exchange (see [Exchange Credentials](#exchange-credentials))

### 1. Clone and configure

```bash
git clone https://github.com/openpx-ai/openpx.git
cd openpx
cp .env.example .env
# Edit .env with your exchange API keys
```

### 2. Build and run

```bash
# Build everything
cargo build --workspace

# Run the trading dashboard
cargo run -p px-dashboard

# Dashboard is now at http://localhost:3000
```

### 3. Or use Docker

```bash
docker compose up -d
# Dashboard at http://localhost:3000
```

## Usage

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

## Exchange Credentials

Edit `.env` with your keys. Each exchange is optional — only configure what you need.

| Exchange | Required Keys | Docs |
|----------|--------------|------|
| Polymarket | `POLYMARKET_PRIVATE_KEY` | [docs](https://docs.polymarket.com/developers/) |
| Kalshi | `KALSHI_API_KEY_ID`, `KALSHI_PRIVATE_KEY_PATH` | [docs](https://docs.kalshi.com/) |

See `.env.example` for the full list of optional fields.

## Exchange Support Matrix

| Exchange | Markets | Trading | Orderbook | WebSocket |
|----------|---------|---------|-----------|-----------|
| Polymarket | Yes | Yes | Yes | Yes |
| Kalshi | Yes | Yes | Yes | Yes |

## Development

```bash
cargo check --workspace     # Type check
cargo test --workspace      # Run tests
cargo clippy --workspace -- -D warnings  # Lint
cargo fmt --all             # Format

just sync-all               # Regenerate Python/TS SDKs + docs from Rust types
just dashboard              # Run dashboard in debug mode
just docs-serve             # Serve docs locally
```

## Project Structure

```
engine/               Rust core
  core/               Types, traits, error handling
  exchanges/          Exchange implementations (kalshi, polymarket, etc.)
  sdk/                Unified facade (enum dispatch)
sdks/
  python/             PyO3 bindings
  typescript/         NAPI-RS bindings
dashboard/            Trading terminal (Axum server + JS frontend)
docs/                 Starlight docs site
```

## Community

- [Issues](https://github.com/openpx-ai/openpx/issues) — Bugs & feature requests
- [Discussions](https://github.com/openpx-ai/openpx/discussions) — Questions & chat
- [Contributing](CONTRIBUTING.md)

## License

MIT
