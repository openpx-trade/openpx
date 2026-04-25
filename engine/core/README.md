# px-core

[![Crates.io](https://img.shields.io/crates/v/px-core.svg)](https://crates.io/crates/px-core)
[![Documentation](https://docs.rs/px-core/badge.svg)](https://docs.rs/px-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Core traits, models, and errors for the OpenPX prediction market SDK.

## Overview

`px-core` provides the foundational components for building prediction market integrations:

- **Exchange trait**: Unified async interface for all prediction market operations
- **Models**: `Market`, `Order`, `Position`, `Orderbook`, and more
- **WebSocket trait**: Real-time orderbook and trade streaming
- **Strategy trait**: Framework for building trading strategies
- **Error types**: Comprehensive error hierarchy (`OpenPxError`)

## Installation

```toml
[dependencies]
px-core = "0.1"
```

## Usage

This crate is typically used as a dependency by exchange implementations (`px-exchange-polymarket`, `px-exchange-kalshi`).

```rust
use px_core::{Exchange, Market, Order, OrderSide, OpenPxError};

// The Exchange trait defines the unified API
pub trait Exchange: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    async fn fetch_markets(&self) -> Result<Vec<Market>, OpenPxError>;
    async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError>;
    async fn create_order(&self, ...) -> Result<Order, OpenPxError>;
    // ... more methods
}
```

## Models

| Model | Description |
|-------|-------------|
| `Market` | Prediction market with question, outcomes, prices, volume |
| `Order` | Order with price, size, status, timestamps |
| `Position` | Position with size, average price, current price |
| `Orderbook` | Orderbook with bids and asks |
| `Trade` | Executed trade information |

## Features

- **Async-first**: Built on `tokio` for high-performance async operations
- **Type-safe**: Leverage Rust's type system for compile-time safety
- **Serde support**: All models are serializable/deserializable


