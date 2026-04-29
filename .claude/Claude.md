# Claude Project Profile: OpenPX

## What OpenPX Is

OpenPX is a **unified API for prediction markets** — one `Exchange` trait, one set of models, one shape that hides Kalshi's and Polymarket's per-exchange quirks behind a single interface. Users bring their own credentials and trade through the unified surface. CCXT, but for prediction markets, in Rust.

The codebase is **lean, stable, HFT-oriented, and elegantly designed**. Every change is judged against those four properties.

- **Lean** — no file, dependency, or abstraction lands unless it pulls its weight. Three similar lines beat a premature abstraction.
- **Stable** — the unified surface is a contract. Breaking it without a migration plan is not acceptable.
- **HFT-oriented** — every nanosecond counts. Zero-alloc hot paths, lock-free where possible, mechanical sympathy over OOP layering.
- **Elegantly designed** — naming reads like prose; shapes are obvious; one concept lives in one place.

## The Unification Mandate

The single most important property of this codebase is that **the same operation looks identical across every exchange**. If a user fetches markets, places an order, or subscribes to a stream, they should not be able to tell which exchange they're talking to from the call site alone — only from the data flowing back.

Before writing or accepting any code, ask:

1. **Is this shape already in the unified API?** If yes, conform to it. If no, design the shape first, then implement both exchanges against it.
2. **Does both Kalshi *and* Polymarket need this surface?** If only one does, it does not belong on the unified trait — push it to an exchange-specific extension or rethink whether the user actually needs it.
3. **Does the field naming reflect the unified vocabulary, not the exchange's vocabulary?** Kalshi calls something `ticker`, Polymarket calls it `condition_id` — the unified model picks one name (or a more general one) and both implementations map into it. Never leak per-exchange terminology.
4. **Does the error map cleanly into `ExchangeError`?** Per-exchange error variants stay private; only the unified hierarchy crosses the boundary.

When you encounter divergence on `main`, treat it as a bug, not a feature. Reconcile.

## Reference Discipline (HARD RULE)

**Any time you are asked to add, change, or unify an endpoint, model, error, field, or stream — you must first look it up in the upstream reference docs for both exchanges.** No exceptions, no "I remember how it works."

Examples of requests that trigger doc lookup:
- "ensure the markets endpoints are unified correctly" → fetch markets endpoints from both exchanges' OpenAPI specs, compare fields, design the unified shape, then implement.
- "add a fetch_volume endpoint" → search both exchanges' docs for `volume`, find every endpoint/field that exposes it, then design the unified call.
- "fix order cancellation on Polymarket" → fetch the Polymarket CLOB cancel spec, fetch Kalshi's cancel spec to confirm the unified contract still holds, then change code.
- "add a new field to the orderbook" → fetch both exchanges' orderbook channel specs, see what they actually publish, then update the model.

The lookup workflow:

1. **Search the keyword against the reference URLs below** using `WebFetch` (Tier 1 specs first, then prose pages).
2. **Diff the two exchanges' shapes** — what does each call it, what fields exist, what's optional, what's typed how.
3. **Check the changelogs** for any recent breaking changes that touch the keyword.
4. **Then design the unified shape** that maps both into one model.
5. **Then write code.**

If you skip step 1, you will produce drift. Drift is the one thing this codebase cannot tolerate.

## Reference URLs

All upstream doc URLs (Kalshi + Polymarket, Tier 1 specs/changelogs/discovery + Tier 2 prose) live in a single source of truth: **`.claude/references.md`**. Read that file before any unification work — do not hardcode URLs from memory.

The `/unify` skill (`.claude/commands/unify.md`) reads from the same file.

## Workspace Structure

```
openpx/
├── engine/                   # Rust core — powers everything
│   ├── core/                 # Unified types, traits, timing, errors
│   │   ├── src/exchange/     # Exchange trait, manifests, normalizers, rate limiting
│   │   ├── src/models/       # Market, Order, Position, Orderbook, Trade
│   │   ├── src/events.rs     # Canonical event ID handling
│   │   ├── src/timing.rs     # timed! macro + TimingGuard
│   │   ├── src/error.rs      # OpenPxError hierarchy + define_exchange_error! macro
│   │   ├── src/utils/        # Utility helpers (price conversion)
│   │   └── src/websocket/    # WebSocket traits
│   ├── exchanges/
│   │   ├── kalshi/           # auth, config, error, exchange, fetcher, normalize, websocket
│   │   └── polymarket/       # auth, config, error, exchange, fetcher, websocket, approvals, clob, ctf, signer, relayer, swap, client, diagnostics
│   ├── sdk/                  # Unified facade (enum dispatch)
│   ├── cli/                  # CLI for testing APIs & WebSocket streams
│   └── schema/               # JSON Schema export binary
├── sdks/
│   ├── python/               # PyO3 + auto-generated Pydantic models
│   └── typescript/           # NAPI-RS + auto-generated TS types
├── docs/                     # Mintlify documentation site (docs.json + MDX)
├── schema/                   # openpx.schema.json (generated artifact)
└── justfile                  # Single-command SDK sync
```

## Key Files

| Purpose | File |
|---------|------|
| Exchange trait (the unified contract) | `engine/core/src/exchange/traits.rs` |
| Exchange manifest base | `engine/core/src/exchange/manifest.rs` |
| Per-exchange manifests | `engine/core/src/exchange/manifests/{kalshi,polymarket}.rs` |
| Timing macros | `engine/core/src/timing.rs` |
| Error types | `engine/core/src/error.rs` |
| Event IDs | `engine/core/src/events.rs` |
| WebSocket traits | `engine/core/src/websocket/traits.rs` |
| Reference exchange impl | `engine/exchanges/kalshi/src/exchange.rs` |
| Unified Market model | `engine/core/src/models/market.rs` |

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

Each exchange defines its error type using `define_exchange_error!`, then maps to the unified `ExchangeError`. Per-exchange variants stay private; only the unified hierarchy crosses the trait boundary.

```rust
use px_core::define_exchange_error;

define_exchange_error!(KalshiError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),
});

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
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo fmt --all
cargo build --release --workspace
```

## Best Practices

- **Code style:** lean, minimal, organized. Easy to read at a glance.
- **Comments:** default to none. Only write one when the *why* is non-obvious.
- **Testing:** every exchange implementation has tests; every unified-trait method has a contract test that runs against both exchanges.
- **Allocations:** avoid in hot paths. Reuse buffers, prefer stack, pool when needed.
- **Concurrency:** lock-free where possible; never a global lock on a hot path.
- **Validation:** strict types, boundary checks at the edge — internal code trusts internal code.
