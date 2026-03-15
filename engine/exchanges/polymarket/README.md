# px-exchange-polymarket

[![Crates.io](https://img.shields.io/crates/v/px-exchange-polymarket.svg)](https://crates.io/crates/px-exchange-polymarket)
[![Documentation](https://docs.rs/px-exchange-polymarket/badge.svg)](https://docs.rs/px-exchange-polymarket)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[Polymarket](https://polymarket.com) exchange implementation for OpenPX.

## Overview

This crate provides a complete Polymarket integration including:

- **REST API**: Fetch markets, create/cancel orders, manage positions
- **WebSocket**: Real-time orderbook streaming
- **CLOB Client**: Direct access to Polymarket's Central Limit Order Book
- **Authentication**: Ethereum wallet signing for trading

## Installation

```toml
[dependencies]
px-exchange-polymarket = "0.1"
```

## Quick Start

```rust
use px_core::Exchange;
use px_exchange_polymarket::{Polymarket, PolymarketConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Public API (no auth required)
    let exchange = Polymarket::with_default_config()?;
    
    // Fetch markets
    let markets = exchange.fetch_markets().await?;
    for market in markets.iter().take(5) {
        println!("{}: {:?}", market.question, market.prices);
    }
    
    Ok(())
}
```

## Authentication

For trading operations, you need to provide your Ethereum private key:

```rust
use px_exchange_polymarket::{Polymarket, PolymarketConfig};

let config = PolymarketConfig::new()
    .with_private_key("0x...")
    .with_funder("0x...");

let exchange = Polymarket::new(config)?;
exchange.init_trading().await?;

// Now you can create orders, cancel orders, etc.
```

## WebSocket Streaming

```rust
use px_exchange_polymarket::PolymarketWebSocket;
use px_core::WebSocketClient;

let ws = PolymarketWebSocket::new();
let mut stream = ws.subscribe_orderbook("token_id").await?;

while let Some(orderbook) = stream.next().await {
    println!("Bids: {:?}, Asks: {:?}", orderbook.bids, orderbook.asks);
}
```

## Features

| Feature | Status |
|---------|--------|
| Fetch markets | ✅ |
| Fetch orderbook | ✅ |
| Create orders | ✅ |
| Cancel orders | ✅ |
| Fetch positions | ✅ |
| Fetch balance | ✅ |
| WebSocket orderbook | ✅ |

\
