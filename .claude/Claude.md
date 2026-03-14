# Claude Project Profile: OpenPX

## Overview

OpenPX is an open-source, CCXT-style unified SDK for prediction markets in Rust. Users bring their own exchange credentials and trade directly through the unified `Exchange` trait.

## Engineering Principles

**SPEED IS EVERYTHING.** Every nanosecond counts — in network I/O, internal calculations,
serialization, deserialization, matching, routing, everything. This is an HFT-oriented
library where latency is the difference between profit and loss for users. Never introduce
unnecessary overhead. Always ask: "is there a faster way to do this?"

- **Performance is Non-Negotiable:** Prioritize lowest possible latency and minimal memory footprint in every code path.
- **Mechanical Sympathy:** Favor data-oriented design over deep OOP abstractions.
- **Zero-Alloc Hot Paths:** Minimize or eliminate memory allocations in hot loops.
- **Modular Simplicity:** Keep the codebase lean and organized. Do not add dependencies or files unless absolutely necessary.

## Technical Standards
- **Concurrency:** Use lock-free structures where possible; avoid global locks that cause contention.
- **Memory:** Prefer stack allocation and object pooling over frequent heap allocations.
- **Validation:** Strict type safety and boundary checking are non-negotiable.

## Workspace Structure

```
openpx/
├── engine/                   # Rust core — powers everything
│   ├── core/                 # Core types, traits, timing, error handling
│   │   ├── src/exchange/     # Exchange trait, manifests, normalizers, rate limiting
│   │   ├── src/models/       # Market, Order, Position, Orderbook, Trade, RawMarket
│   │   ├── src/events.rs     # Canonical event ID handling
│   │   ├── src/timing.rs     # timed! macro + TimingGuard
│   │   ├── src/error.rs      # OpenPxError hierarchy + define_exchange_error! macro
│   │   ├── src/utils/        # Utility helpers (price conversion)
│   │   └── src/websocket/    # WebSocket traits
│   ├── exchanges/            # Exchange implementations
│   │   ├── kalshi/           # auth, config, error, exchange, fetcher, normalize, websocket
│   │   ├── polymarket/
│   │   └── opinion/
│   ├── sdk/                  # Unified facade (enum dispatch)
│   └── schema/               # JSON Schema export binary
├── sdks/                     # Language SDKs
│   ├── python/               # PyO3 + auto-generated Pydantic models
│   └── typescript/           # NAPI-RS + auto-generated TS types
├── docs/                     # Astro-based documentation site
├── schema/                   # openpx.schema.json (generated artifact)
├── scripts/                  # Build & codegen scripts
└── justfile                  # Single-command SDK sync
```

## Key Files Reference

| Purpose | File | Notes |
|---------|------|-------|
| Exchange trait | `engine/core/src/exchange/traits.rs` | All exchanges implement this |
| Exchange manifests | `engine/core/src/exchange/manifest.rs` | Connection + pagination config per exchange |
| Timing macros | `engine/core/src/timing.rs` | `timed!` macro + metric name constants |
| Error types | `engine/core/src/error.rs` | `OpenPxError` hierarchy + `define_exchange_error!` macro |
| Event IDs | `engine/core/src/events.rs` | Canonical cross-exchange event grouping |
| WebSocket traits | `engine/core/src/websocket/traits.rs` | WebSocket connection interface |
| Kalshi exchange | `engine/exchanges/kalshi/src/exchange.rs` | Reference implementation |
| Market model | `engine/core/src/models/market.rs` | Market, UnifiedMarket types |

## Common Patterns

### Timing Instrumentation

```rust
use px_core::timed;

let result = timed!(
    "openpx.exchange.http_request_us",
    "exchange" => self.id(),
    "operation" => "create_order";
    client.post(&url).send().await
);
```

### Error Handling

Each exchange defines its error type using the `define_exchange_error!` macro, then maps to `ExchangeError`:

```rust
use px_core::define_exchange_error;

// Define exchange-specific error (auto-includes Http, Api, RateLimited, AuthRequired, MarketNotFound)
define_exchange_error!(KalshiError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),
});

// Map exchange-specific errors to unified ExchangeError
impl From<KalshiError> for px_core::ExchangeError {
    fn from(err: KalshiError) -> Self {
        match err {
            KalshiError::MarketNotFound(id) => px_core::ExchangeError::MarketNotFound(id),
            KalshiError::AuthFailed(msg) => px_core::ExchangeError::Authentication(msg),
            KalshiError::InsufficientBalance(msg) => px_core::ExchangeError::InsufficientFunds(msg),
            other => px_core::ExchangeError::Api(other.to_string()),
        }
    }
}
```

## Common Commands

```bash
# Check workspace
cargo check --workspace

# Run clippy with warnings as errors
cargo clippy --workspace -- -D warnings

# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Build release
cargo build --release --workspace
```

## Best Practices
- **Code Style:** All code must be lean, minimal, and organized, so it's easy to read.
- **Testing:** All exchange implementations should have tests.

## API Reference

Always refer to the official documentation for each prediction market when implementing solutions:
- Polymarket: https://docs.polymarket.com/developers/
- Kalshi: https://docs.kalshi.com/
- Opinion: https://docs.opinion.trade/developer-guide/opinion-open-api
