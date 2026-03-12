# OpenPX

A unified, open-source Rust SDK for prediction markets. Trade across multiple prediction market exchanges through a single, consistent interface.

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

## License

MIT
