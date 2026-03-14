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
│   │   ├── src/exchange/     # Exchange trait + factory + rate limiting
│   │   ├── src/models/       # Market, Order, Position, Orderbook
│   │   ├── src/timing.rs     # timed! macro + TimingGuard
│   │   └── src/error.rs      # OpenPxError hierarchy
│   ├── exchanges/            # Exchange implementations
│   │   ├── kalshi/           # src/exchange.rs, config.rs, error.rs
│   │   ├── polymarket/
│   │   └── opinion/
│   ├── sdk/                  # Unified facade (enum dispatch)
│   └── schema/               # JSON Schema export binary
├── sdks/                     # Language SDKs
│   ├── python/               # PyO3 + auto-generated Pydantic models
│   └── typescript/           # NAPI-RS + auto-generated TS types
├── docs/                     # mdBook documentation (auto-generated)
├── schema/                   # openpx.schema.json (generated artifact)
├── scripts/                  # Build & codegen scripts
└── justfile                  # Single-command SDK sync
```

## Key Files Reference

| Purpose | File | Notes |
|---------|------|-------|
| Exchange trait | `engine/core/src/exchange/traits.rs` | All exchanges implement this |
| Timing macros | `engine/core/src/timing.rs` | `timed!` macro for metrics |
| Error types | `engine/core/src/error.rs` | `OpenPxError` hierarchy |
| Kalshi exchange | `engine/exchanges/kalshi/src/exchange.rs` | Reference implementation |
| Market model | `engine/core/src/models/market.rs` | Market, UnifiedMarket types |

## Common Patterns

### Timing Instrumentation

```rust
use px_core::timed;

let result = timed!(
    "openpx.exchange.http_send_us",
    "exchange" => self.id(),
    "operation" => "create_order";
    client.post(&url).send().await
);
```

### Error Handling

```rust
use px_core::{OpenPxError, ExchangeError};

// Map exchange-specific errors
impl From<KalshiError> for ExchangeError {
    fn from(err: KalshiError) -> Self {
        match err {
            KalshiError::AuthFailed(msg) => Self::Authentication(msg),
            _ => Self::Api(err.to_string()),
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
