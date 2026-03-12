# Contributing to OpenPX

Thank you for your interest in contributing to OpenPX. This guide will help you get started.

## Development Setup

### Prerequisites

- Rust 1.75+ (`rustup update stable`)
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

## Adding a New Exchange

1. Create a new crate: `px-exchange-{name}/`
2. Implement the `Exchange` trait from `px-core`
3. Add exchange-specific config, error types, and auth
4. Add the crate to the workspace `members` in the root `Cargo.toml`
5. Add tests in `tests/exchange_tests.rs`
6. Add an example in `examples/fetch_markets.rs`
7. Update the root `README.md` feature matrix

Use `px-exchange-kalshi` as a reference implementation.

## Pull Request Process

1. Fork the repo and create a feature branch from `main`
2. Make your changes with clear, focused commits
3. Ensure all checks pass: `cargo fmt`, `cargo clippy`, `cargo test`
4. Open a PR against `main` with:
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
