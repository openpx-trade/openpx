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
│   │   ├── src/models/       # Market, Order, Position, Orderbook, Trade
│   │   ├── src/events.rs     # Canonical event ID handling
│   │   ├── src/timing.rs     # timed! macro + TimingGuard
│   │   ├── src/error.rs      # OpenPxError hierarchy + define_exchange_error! macro
│   │   ├── src/utils/        # Utility helpers (price conversion)
│   │   └── src/websocket/    # WebSocket traits
│   ├── exchanges/            # Exchange implementations
│   │   ├── kalshi/           # auth, config, error, exchange, fetcher, normalize, websocket
│   │   └── polymarket/       # auth, config, error, exchange, fetcher, websocket, approvals, clob, ctf, signer, relayer, swap, client, diagnostics
│   ├── sdk/                  # Unified facade (enum dispatch)
│   ├── cli/                  # CLI tool for testing APIs & WebSocket streams
│   └── schema/               # JSON Schema export binary
├── sdks/                     # Language SDKs
│   ├── python/               # PyO3 + auto-generated Pydantic models
│   └── typescript/           # NAPI-RS + auto-generated TS types
├── docs/                     # Mintlify documentation site (docs.json + MDX)
├── schema/                   # openpx.schema.json (generated artifact)
└── justfile                  # Single-command SDK sync
```

## Key Files Reference

| Purpose | File | Notes |
|---------|------|-------|
| Exchange trait | `engine/core/src/exchange/traits.rs` | All exchanges implement this |
| Exchange manifest base | `engine/core/src/exchange/manifest.rs` | ExchangeManifest struct, PaginationConfig, FieldMapping, Transform |
| Exchange manifests | `engine/core/src/exchange/manifests/` | Per-exchange configs (kalshi.rs, polymarket.rs) |
| Timing macros | `engine/core/src/timing.rs` | `timed!` macro + metric name constants |
| Error types | `engine/core/src/error.rs` | `OpenPxError` hierarchy + `define_exchange_error!` macro |
| Event IDs | `engine/core/src/events.rs` | Canonical cross-exchange event grouping |
| WebSocket traits | `engine/core/src/websocket/traits.rs` | WebSocket connection interface |
| Kalshi exchange | `engine/exchanges/kalshi/src/exchange.rs` | Reference implementation |
| Market model | `engine/core/src/models/market.rs` | Unified Market type (single struct for all exchanges) |

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

## How to extend the manifest

The unified schema mapping is OpenPX's core business value. The contract is:
**every JSON key read in `engine/exchanges/<id>/src/exchange.rs` must be declared
in `engine/core/src/exchange/manifests/<id>.rs::field_mappings`** (as a
`source_paths` entry, possibly with fallbacks), OR explicitly listed in
`maintenance/manifest-allowlists/<id>.txt` with a one-line justification.
This is enforced mechanically by `maintenance/tests/manifest_coverage.rs`
(wired into Cargo via `engine/core/Cargo.toml::[[test]]`).

When you read a new JSON key in `exchange.rs`:
1. If it maps to a `Market` model field: add a `FieldMapping` entry in `manifests/<id>.rs`.
2. If it's part of order/fill/position/orderbook parsing or a wrapper field
   (e.g. `data`, `error`, `code`): add it to `maintenance/manifest-allowlists/<id>.txt` with a comment.
3. Run `cargo test -p px-core --test manifest_coverage` locally before
   pushing — CI gates this on every PR.

Hardcoded contract addresses in `engine/exchanges/polymarket/src/{swap,approvals,clob,ctf,relayer,signer}.rs`
are similarly gated by `maintenance/tests/contracts_test.rs` (wired into Cargo
via `engine/exchanges/polymarket/Cargo.toml::[[test]]`) against the snapshot at
`maintenance/snapshots/polymarket-contracts.snapshot.json`. Never bypass that test —
a wrong contract address can move user funds.

## Autonomous maintenance

This repo is maintained primarily by Claude Code agents. **Everything related to
the agent system lives under `maintenance/`** (runbooks, scripts, policy docs,
data files, allowlists). The agent definitions themselves live at
`.claude/agents/` (Claude Code's required location) and reference the
`maintenance/` tree for everything else. See `.claude/agents/README.md` for the
index, or the plan at
`/Users/mppathiyal/.claude/plans/just-so-i-can-rustling-planet.md` for the full
design. Every PR opened by an agent requires explicit human approval — there is
no auto-merge. CODEOWNERS at `.github/CODEOWNERS` and the policy at
`maintenance/policy/REVIEW_POLICY.md` define what each surface allows.

**The bot's sync invariant:** every PR opened by an agent must complete
`maintenance/runbooks/pr-preflight.md` first. That runbook enforces the rule
that the Rust core, Python SDK, TypeScript SDK, and Mintlify docs all stay in
sync on every edit, and that both SDKs actually build and import cleanly. The
CI gates `SDK Sync Check`, `Python SDK Build`, and `Node.js SDK Build`
mechanically backstop the same rule — they are required by branch protection,
so a partial sync cannot land regardless of what any agent claims in a PR
body.

**The bot's scope:** the agent system fires once a day at 00:00 UTC. Its
single job is to keep OpenPX in sync with the upstream Kalshi and Polymarket
changelogs — propose new overlapping features (`parity-analyst` files an
issue), implement breaking exchange-specific features (the relevant
maintainer opens a PR), refresh the lock at the end. To catch up on a
quiet period, use `just backfill YYYY-MM-DD`; the orchestrator walks every
`<Update>` block on or after that date and classifies them with the same
rules as the daily cycle.
