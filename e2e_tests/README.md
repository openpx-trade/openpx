# e2e_tests

End-to-end coverage for the unified OpenPX surface across every public API
and every consumer SDK. Each suite mirrors the same matrix across Rust,
Python, TypeScript, and the CLI so any regression in one binding surfaces
on the next run.

## Suites

| Suite        | Surface                                                |
|--------------|--------------------------------------------------------|
| `markets/`   | `fetch_markets` and friends (Rust-only currently)      |
| `orderbooks/`| All five orderbook entry points × every SDK            |
| `trading/`   | All authenticated endpoints × every SDK + docs check   |
| `websockets/`| Unified WS contract: orderbook + trades + fills × SDKs |
| `live/`      | Misc live integration tests (legacy `live.rs`)         |

## Layout

Every suite holds a parallel structure:

```
e2e_tests/
├── <suite>/
│   ├── rust/           # Rust integration test (#[tokio::test])
│   ├── python/         # Python SDK driver (writes results/python.json)
│   ├── typescript/     # Node addon driver  (writes results/typescript.json)
│   ├── cli/            # bash driver hitting target/release/openpx
│   └── results/        # per-run artifacts
```

The Rust integration tests are wired through the `px-e2e-tests` workspace
crate (`e2e_tests/Cargo.toml`); each suite is one `[[test]]` target.

## Running

```bash
# Rust — canonical matrices, all live tests
just e2e-rust orderbooks
just e2e-rust trading
just e2e-rust websockets
just e2e-rust markets

# Python / TypeScript / CLI parity
just e2e-python orderbooks    # one suite
just e2e-typescript trading
just e2e-cli websockets

# Convenience: full WebSocket matrix across every SDK
just e2e-websockets
```

All live tests are gated behind `OPENPX_LIVE_TESTS=1` and read credentials
from `.env` at repo root. CI does not run live tests; only the mocked unit
tests under `engine/exchanges/*/tests/` and `engine/sdk/tests/integration.rs`
+ `wiring.rs` execute on PRs.

## Schema mapping documentation

The cross-exchange mappings exercised here are documented at
`docs/schemas/mappings/`. They are auto-generated from
`schema/mappings/*.yaml` by `tools/render_mappings.py` — including the
WebSocket channel mappings (`orderbook-stream.mdx`, `trades-stream.mdx`,
`fills-stream.mdx`) introduced alongside the websockets suite.
