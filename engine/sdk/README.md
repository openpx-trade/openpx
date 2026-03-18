# openpx

[![Crates.io](https://img.shields.io/crates/v/openpx.svg)](https://crates.io/crates/openpx)
[![Documentation](https://docs.rs/openpx/badge.svg)](https://docs.rs/openpx)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Unified Rust SDK for prediction markets — Kalshi, Polymarket, Opinion and more.

## Installation

```toml
[dependencies]
openpx = "0.1"
```

## Quick Start

```rust
use openpx::{Exchange, ExchangeInner};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to Kalshi (public API)
    let exchange = ExchangeInner::new("kalshi", json!({}))?;
    let (markets, _) = exchange.fetch_markets(&Default::default()).await?;

    for market in markets.iter().take(5) {
        println!("{}: {:?}", market.question, market.prices);
    }

    Ok(())
}
```

## Supported Exchanges

- **Kalshi** — US-regulated event contracts
- **Polymarket** — Crypto-native prediction markets
- **Opinion** — Opinion exchange markets

## Features

Everything you need in a single crate:

- Unified `Exchange` trait across all prediction markets
- Core types: `Market`, `Order`, `Position`, `Orderbook`, `Trade`
- Enum dispatch (no vtable overhead)
- Real-time WebSocket streaming
- Crypto price feeds (`CryptoPriceWebSocket`)
- Sports score feeds (`SportsWebSocket`)
