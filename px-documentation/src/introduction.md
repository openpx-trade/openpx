# OpenPX

OpenPX is an open-source, CCXT-style unified SDK for prediction markets.
Users bring their own exchange credentials and trade directly through a single interface.

## Supported Exchanges

| Exchange | Market Data | Trading | WebSocket |
|----------|------------|---------|-----------|
| Kalshi | Yes | Yes | Yes |
| Polymarket | Yes | Yes | Yes |
| Opinion | Yes | Yes | Yes |
| Limitless | Yes | Yes | Yes |
| Predict.fun | Yes | Yes | No |

## Architecture

```
Rust types (#[derive(JsonSchema)])
        |
        v
px-schema binary → schema/openpx.schema.json
        |                    |
        v                    v
datamodel-codegen      json-schema-to-typescript
        |                    |
        v                    v
Python SDK             TypeScript SDK
(Pydantic v2)          (TS interfaces)
        |                    |
        v                    v
PyO3 native ext        NAPI-RS native addon
```

All contributions go to **Rust only** — language SDKs are automatically
regenerated from Rust types via `just sync-all`.
