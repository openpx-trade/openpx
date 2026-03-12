# Contributing to OpenPX

Thank you for your interest in contributing to OpenPX. This guide will help you get started.

## Development Setup

### Prerequisites

- Rust 1.91+ (`rustup update stable`)
- Python 3.9+ (for SDK model generation)
- Node.js 18+ (for TypeScript type generation)
- [just](https://github.com/casey/just) (`cargo install just` or `brew install just`)
- Git

### Getting Started

```bash
git clone https://github.com/openpx/openpx.git
cd openpx
cargo check --workspace
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Single crate
cargo test -p px-exchange-kalshi
```

### Linting and Formatting

All PRs must pass these checks:

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

Format your code before committing:

```bash
cargo fmt --all
```

## Code Style

- **Lean and minimal.** No unnecessary abstractions, no premature optimization of code structure.
- **Performance is non-negotiable.** Every allocation in a hot path needs justification. Prefer stack allocation, avoid unnecessary cloning.
- **Zero-alloc hot paths.** Minimize heap allocations in order submission, orderbook processing, and WebSocket handling.
- **No unnecessary dependencies.** Every new crate dependency must be justified in the PR description.

## Multi-Language SDK Pipeline

OpenPX ships Rust, Python, and TypeScript SDKs. **All contributions go to Rust only** — the Python and TypeScript SDKs are automatically regenerated.

### How It Works

```
Rust types (engine/core)
    → engine/schema binary → schema/openpx.schema.json
        → datamodel-codegen     → sdks/python/_models.py (Pydantic v2)
        → json-schema-to-typescript → sdks/typescript/types/models.d.ts
        → generate_sdk_docs.py  → docs/src/ (mdBook)
```

### Syncing SDKs and Docs

After modifying any Rust types in `engine/core`, run:

```bash
just sync-all
```

This single command:
1. Exports the JSON Schema from Rust types
2. Regenerates Python Pydantic models
3. Regenerates TypeScript type definitions
4. Regenerates SDK documentation

### Available Just Recipes

| Command | What it does |
|---------|-------------|
| `just sync-all` | Full sync: schema + Python + TypeScript + docs |
| `just schema` | Export `schema/openpx.schema.json` from Rust types |
| `just python-models` | Regenerate Python Pydantic models from schema |
| `just node-models` | Regenerate TypeScript types from schema |
| `just docs` | Regenerate SDK documentation from schema |
| `just docs-serve` | Generate docs and open local preview |
| `just docs-build` | Generate docs and build static HTML |
| `just check-sync` | Verify generated files are up to date (used in CI) |

### Viewing Documentation Locally

```bash
# Install mdbook (one-time)
cargo install mdbook

# Generate and preview docs
just docs-serve
```

## Adding a New Exchange

1. Create a new crate: `engine/exchanges/{name}/`
2. Implement the `Exchange` trait from `px-core`
3. Add exchange-specific config, error types, and auth
4. Add the crate to the workspace `members` in the root `Cargo.toml`
5. Add the enum variant to `engine/sdk/src/lib.rs` (the `ExchangeInner` enum + `new()` match arm)
6. Add tests
7. Run `just sync-all` (only needed if core model types changed)
8. Commit everything

Use `engine/exchanges/kalshi` as a reference implementation.

**Contributors never need to touch Python or TypeScript code.**

## Adding a New Model Type

If you add a new struct or enum to `engine/core/src/models/`:

1. Add `#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]` to the type
2. Add a `schema_for!()` call in `engine/schema/src/main.rs`
3. Run `just sync-all` to regenerate all SDKs and docs
4. Commit the regenerated files alongside your Rust changes

## Workspace Structure

```
openpx/
├── engine/                   # Rust core — powers everything
│   ├── core/                 # Core types, Exchange trait, errors
│   ├── exchanges/            # Exchange implementations (Rust only)
│   │   ├── kalshi/
│   │   ├── polymarket/
│   │   ├── opinion/
│   │   ├── limitless/
│   │   └── predictfun/
│   ├── sdk/                  # Unified facade — enum dispatch over all exchanges
│   └── schema/               # Binary: exports JSON Schema from Rust types
├── sdks/                     # Language SDKs
│   ├── python/               # PyO3 bindings + auto-generated Pydantic models
│   └── typescript/           # NAPI-RS bindings + auto-generated TS types
├── docs/                     # mdBook docs (auto-generated from schema)
├── schema/                   # openpx.schema.json (checked into git)
├── scripts/                  # Doc generation scripts
└── justfile                  # Single-command SDK sync
```

## Pull Request Process

1. Fork the repo and create a feature branch from `main`
2. Make your changes with clear, focused commits
3. Ensure all checks pass: `cargo fmt`, `cargo clippy`, `cargo test`
4. If you changed model types: run `just sync-all` and commit the regenerated files
5. Open a PR against `main` with:
   - A clear description of what changed and why
   - Any breaking changes called out explicitly
   - Test coverage for new functionality

## What We Look For in Reviews

- **Correctness:** Does the code handle edge cases? Are error paths covered?
- **Performance:** No unnecessary allocations, clones, or blocking in async contexts
- **Simplicity:** Is this the simplest solution that works? Can anything be removed?
- **Test coverage:** New exchange methods should have corresponding tests

## Reporting Issues

- Use GitHub Issues for bugs and feature requests
- Include reproduction steps, expected behavior, and actual behavior
- For security vulnerabilities, see [SECURITY.md](SECURITY.md)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
