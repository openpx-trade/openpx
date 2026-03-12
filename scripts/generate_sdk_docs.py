#!/usr/bin/env python3
"""Generate SDK documentation from openpx.schema.json.

Reads the JSON Schema and produces mdBook-compatible markdown pages showing
Rust, Python, and TypeScript representations of every exported type.

Usage:
    python3 scripts/generate_sdk_docs.py

Outputs:
    docs/src/reference/models.md   — full type reference (all languages)
    docs/src/rust/api.md           — Rust-only reference
    docs/src/python/api.md         — Python-only reference
    docs/src/typescript/api.md     — TypeScript-only reference
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
SCHEMA_PATH = ROOT / "schema" / "openpx.schema.json"
DOCS_SRC = ROOT / "docs" / "src"

# JSON Schema type → language type mappings
RUST_TYPES = {
    "string": "String",
    "number": "f64",
    "integer": "i64",
    "boolean": "bool",
}

PYTHON_TYPES = {
    "string": "str",
    "number": "float",
    "integer": "int",
    "boolean": "bool",
}

TS_TYPES = {
    "string": "string",
    "number": "number",
    "integer": "number",
    "boolean": "boolean",
}


def load_schema() -> dict[str, Any]:
    with open(SCHEMA_PATH) as f:
        return json.load(f)


def ref_name(ref: str) -> str:
    """Extract type name from $ref like '#/definitions/MarketStatus'."""
    return ref.rsplit("/", 1)[-1]


def resolve_type(prop: dict[str, Any], type_map: dict[str, str], lang: str) -> str:
    """Map a JSON Schema property to a language-specific type string."""
    # $ref
    if "$ref" in prop:
        return ref_name(prop["$ref"])

    # anyOf / oneOf with null (Option<T>)
    for key in ("anyOf", "oneOf"):
        if key in prop:
            variants = prop[key]
            non_null = [v for v in variants if v.get("type") != "null"]
            has_null = any(v.get("type") == "null" for v in variants)
            if len(non_null) == 1:
                inner = resolve_type(non_null[0], type_map, lang)
                if has_null:
                    if lang == "rust":
                        return f"Option<{inner}>"
                    elif lang == "python":
                        return f"Optional[{inner}]"
                    else:
                        return f"{inner} | null"
                return inner
            # Multiple non-null variants (enum)
            parts = [resolve_type(v, type_map, lang) for v in non_null]
            return " | ".join(parts)

    # nullable array type like ["string", "null"]
    schema_type = prop.get("type")
    if isinstance(schema_type, list):
        non_null = [t for t in schema_type if t != "null"]
        has_null = "null" in schema_type
        if len(non_null) == 1:
            base = _resolve_scalar(non_null[0], prop, type_map, lang)
            if has_null:
                if lang == "rust":
                    return f"Option<{base}>"
                elif lang == "python":
                    return f"Optional[{base}]"
                else:
                    return f"{base} | null"
            return base
        return " | ".join(type_map.get(t, t) for t in non_null)

    # array
    if schema_type == "array":
        items = prop.get("items", {})
        inner = resolve_type(items, type_map, lang)
        if lang == "rust":
            return f"Vec<{inner}>"
        elif lang == "python":
            return f"list[{inner}]"
        else:
            return f"{inner}[]"

    # object with additionalProperties (HashMap)
    if schema_type == "object":
        add_props = prop.get("additionalProperties")
        if add_props:
            val = resolve_type(add_props, type_map, lang)
            if lang == "rust":
                return f"HashMap<String, {val}>"
            elif lang == "python":
                return f"dict[str, {val}]"
            else:
                return f"Record<string, {val}>"
        if lang == "rust":
            return "serde_json::Value"
        elif lang == "python":
            return "Any"
        else:
            return "Record<string, unknown>"

    # simple scalar with format
    if schema_type in type_map:
        return _resolve_scalar(schema_type, prop, type_map, lang)

    # No type at all — this is serde_json::Value / any
    if not schema_type:
        if lang == "rust":
            return "serde_json::Value"
        elif lang == "python":
            return "Any"
        else:
            return "unknown"

    return type_map.get(schema_type, schema_type)


def _resolve_scalar(scalar_type: str, prop: dict[str, Any], type_map: dict[str, str], lang: str) -> str:
    """Resolve a scalar type, handling format annotations like date-time."""
    fmt = prop.get("format", "")

    # date-time → DateTime<Utc> / datetime / string
    if scalar_type == "string" and fmt == "date-time":
        if lang == "rust":
            return "DateTime<Utc>"
        elif lang == "python":
            return "datetime"
        else:
            return "string"  # ISO 8601 string in TS

    # uint64 → u64
    if scalar_type == "integer" and fmt == "uint64":
        if lang == "rust":
            return "u64"
        elif lang == "python":
            return "int"
        else:
            return "number"

    return type_map.get(scalar_type, scalar_type)


def is_enum_schema(defn: dict[str, Any]) -> bool:
    """Check if this definition is a string enum."""
    return "enum" in defn or "oneOf" in defn


def get_enum_variants(defn: dict[str, Any]) -> list[str]:
    """Extract enum variant names."""
    if "enum" in defn:
        return defn["enum"]
    if "oneOf" in defn:
        # Tagged enum (like ActivityEvent)
        variants = []
        for variant in defn["oneOf"]:
            if "properties" in variant:
                variants.extend(variant["properties"].keys())
            elif "const" in variant:
                variants.append(variant["const"])
        return variants
    return []


def get_fields(defn: dict[str, Any]) -> list[tuple[str, dict[str, Any], bool]]:
    """Return (field_name, field_schema, is_required) tuples."""
    props = defn.get("properties", {})
    required = set(defn.get("required", []))
    return [(name, schema, name in required) for name, schema in sorted(props.items())]


def categorize_types(definitions: dict[str, Any]) -> dict[str, list[str]]:
    """Group type names into categories."""
    categories: dict[str, list[str]] = {
        "Market Data": [],
        "Orders & Trading": [],
        "Account & Positions": [],
        "Orderbook": [],
        "Trades & History": [],
        "WebSocket & Streaming": [],
        "Configuration & Requests": [],
    }

    market_types = {"Market", "UnifiedMarket", "OutcomeToken", "MarketStatus"}
    order_types = {"Order", "OrderType", "OrderSide", "OrderStatus", "LiquidityRole", "Fill"}
    account_types = {"Position", "Nav", "PositionBreakdown", "DeltaInfo"}
    orderbook_types = {"Orderbook", "OrderbookSnapshot", "PriceLevel", "PriceLevelChange", "PriceLevelSide"}
    trade_types = {"MarketTrade", "Candlestick", "PriceHistoryInterval"}
    ws_types = {"ActivityEvent", "ActivityTrade", "ActivityFill"}

    for name in sorted(definitions.keys()):
        if name in market_types:
            categories["Market Data"].append(name)
        elif name in order_types:
            categories["Orders & Trading"].append(name)
        elif name in account_types:
            categories["Account & Positions"].append(name)
        elif name in orderbook_types:
            categories["Orderbook"].append(name)
        elif name in trade_types:
            categories["Trades & History"].append(name)
        elif name in ws_types:
            categories["WebSocket & Streaming"].append(name)
        else:
            categories["Configuration & Requests"].append(name)

    return {k: v for k, v in categories.items() if v}


def generate_rust_block(name: str, defn: dict[str, Any]) -> str:
    """Generate Rust struct/enum code block."""
    if is_enum_schema(defn):
        variants = get_enum_variants(defn)
        if not variants:
            return f"pub enum {name} {{}}"
        lines = [f"pub enum {name} {{"]
        for v in variants:
            # Convert snake_case to PascalCase for Rust
            pascal = "".join(word.capitalize() for word in v.replace("-", "_").split("_"))
            lines.append(f"    {pascal},")
        lines.append("}")
        return "\n".join(lines)

    fields = get_fields(defn)
    if not fields:
        return f"pub struct {name} {{}}"

    lines = [f"pub struct {name} {{"]
    for fname, fschema, required in fields:
        rtype = resolve_type(fschema, RUST_TYPES, "rust")
        if not required and not rtype.startswith("Option<"):
            rtype = f"Option<{rtype}>"
        lines.append(f"    pub {fname}: {rtype},")
    lines.append("}")
    return "\n".join(lines)


def generate_python_block(name: str, defn: dict[str, Any]) -> str:
    """Generate Python Pydantic model / enum code block."""
    if is_enum_schema(defn):
        variants = get_enum_variants(defn)
        if not variants:
            return f"class {name}(str, Enum):\n    pass"
        lines = [f"class {name}(str, Enum):"]
        for v in variants:
            lines.append(f'    {v.upper()} = "{v}"')
        return "\n".join(lines)

    fields = get_fields(defn)
    if not fields:
        return f"class {name}(BaseModel):\n    pass"

    lines = [f"class {name}(BaseModel):"]
    for fname, fschema, required in fields:
        ptype = resolve_type(fschema, PYTHON_TYPES, "python")
        if not required and not ptype.startswith("Optional["):
            ptype = f"Optional[{ptype}]"
            lines.append(f"    {fname}: {ptype} = None")
        else:
            lines.append(f"    {fname}: {ptype}")
    return "\n".join(lines)


def generate_ts_block(name: str, defn: dict[str, Any]) -> str:
    """Generate TypeScript interface / type alias."""
    if is_enum_schema(defn):
        variants = get_enum_variants(defn)
        if not variants:
            return f"type {name} = never;"
        parts = " | ".join(f'"{v}"' for v in variants)
        return f"type {name} = {parts};"

    fields = get_fields(defn)
    if not fields:
        return f"interface {name} {{}}"

    lines = [f"interface {name} {{"]
    for fname, fschema, required in fields:
        tstype = resolve_type(fschema, TS_TYPES, "typescript")
        opt = "" if required else "?"
        lines.append(f"  {fname}{opt}: {tstype};")
    lines.append("}")
    return "\n".join(lines)


def generate_field_table(defn: dict[str, Any]) -> str:
    """Generate a markdown table of fields."""
    fields = get_fields(defn)
    if not fields:
        return "*No fields*\n"

    lines = [
        "| Field | Type | Required | Description |",
        "|-------|------|----------|-------------|",
    ]
    for fname, fschema, required in fields:
        json_type = resolve_type(fschema, TS_TYPES, "typescript")
        req = "Yes" if required else "No"
        desc = fschema.get("description", "")
        lines.append(f"| `{fname}` | `{json_type}` | {req} | {desc} |")
    return "\n".join(lines)


def write_file(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    print(f"  wrote {path.relative_to(ROOT)}")


def generate_models_reference(definitions: dict[str, Any]) -> str:
    """Generate the full models reference page (all languages)."""
    categories = categorize_types(definitions)
    lines = ["# Type Reference", ""]
    lines.append("All types auto-generated from Rust source via `schema/openpx.schema.json`.")
    lines.append("Run `just docs` to regenerate.\n")

    for category, type_names in categories.items():
        lines.append(f"## {category}\n")

        for name in type_names:
            defn = definitions[name]
            lines.append(f"### {name}\n")

            if is_enum_schema(defn):
                variants = get_enum_variants(defn)
                lines.append(f"Enum with variants: {', '.join(f'`{v}`' for v in variants)}\n")
            else:
                lines.append(generate_field_table(defn))
                lines.append("")

            # Side-by-side code blocks
            lines.append("<details>")
            lines.append(f"<summary>Rust / Python / TypeScript definitions</summary>\n")

            lines.append("**Rust**")
            lines.append("```rust")
            lines.append(generate_rust_block(name, defn))
            lines.append("```\n")

            lines.append("**Python**")
            lines.append("```python")
            lines.append(generate_python_block(name, defn))
            lines.append("```\n")

            lines.append("**TypeScript**")
            lines.append("```typescript")
            lines.append(generate_ts_block(name, defn))
            lines.append("```\n")

            lines.append("</details>\n")
            lines.append("---\n")

    return "\n".join(lines)


def generate_lang_reference(definitions: dict[str, Any], lang: str) -> str:
    """Generate a single-language API reference page."""
    categories = categorize_types(definitions)
    lang_names = {"rust": "Rust", "python": "Python", "typescript": "TypeScript"}
    lang_display = lang_names[lang]
    ext = {"rust": "rust", "python": "python", "typescript": "typescript"}[lang]

    generators = {
        "rust": generate_rust_block,
        "python": generate_python_block,
        "typescript": generate_ts_block,
    }
    gen = generators[lang]

    lines = [f"# {lang_display} API Reference", ""]
    lines.append(f"All {lang_display} types auto-generated from `schema/openpx.schema.json`.\n")

    for category, type_names in categories.items():
        lines.append(f"## {category}\n")
        for name in type_names:
            defn = definitions[name]
            lines.append(f"### {name}\n")
            lines.append(f"```{ext}")
            lines.append(gen(name, defn))
            lines.append("```\n")

    return "\n".join(lines)


def generate_summary(categories: dict[str, list[str]]) -> str:
    """Generate SUMMARY.md for mdBook."""
    return """# Summary

[Introduction](./introduction.md)

# Getting Started

- [Installation](./installation.md)
- [Quick Start](./quickstart.md)

# SDK Reference

- [Rust](./rust/README.md)
  - [API Reference](./rust/api.md)
- [Python](./python/README.md)
  - [API Reference](./python/api.md)
- [TypeScript](./typescript/README.md)
  - [API Reference](./typescript/api.md)

# Reference

- [All Types](./reference/models.md)
- [Exchanges](./reference/exchanges.md)
- [Error Handling](./reference/errors.md)
"""


def generate_introduction() -> str:
    return """# OpenPX

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
"""


def generate_installation() -> str:
    return """# Installation

## Rust

Add OpenPX crates to your `Cargo.toml`:

```toml
[dependencies]
px-core = "0.1"

# Individual exchanges
px-exchange-kalshi = "0.1"
px-exchange-polymarket = "0.1"
px-exchange-opinion = "0.1"
px-exchange-limitless = "0.1"
px-exchange-predictfun = "0.1"

# Or use the unified SDK facade
px-sdk = "0.1"
```

## Python

```bash
pip install openpx
```

Requires Python >= 3.9. The package includes a native Rust extension compiled
with PyO3 — no Rust toolchain needed on the user's machine.

## TypeScript / Node.js

```bash
npm install @openpx/sdk
```

Requires Node.js >= 18. The package includes a native Rust addon compiled
with NAPI-RS.
"""


def generate_quickstart() -> str:
    return """# Quick Start

## Fetch Markets

### Rust

```rust
use px_sdk::ExchangeInner;
use serde_json::json;

#[tokio::main]
async fn main() {
    let exchange = ExchangeInner::new("kalshi", json!({})).unwrap();
    let markets = exchange.fetch_markets(None).await.unwrap();
    for market in &markets[..5] {
        println!("{}: {}", market.id, market.question);
    }
}
```

### Python

```python
from openpx import Exchange

exchange = Exchange("kalshi")
markets = exchange.fetch_markets(limit=5)
for market in markets:
    print(f"{market.id}: {market.question}")
```

### TypeScript

```typescript
import { Exchange } from "@openpx/sdk";

const exchange = new Exchange("kalshi", {});
const markets = await exchange.fetchMarkets(5);
for (const market of markets) {
  console.log(`${market.id}: ${market.question}`);
}
```

## Create an Order

### Rust

```rust
use px_sdk::ExchangeInner;
use px_core::OrderSide;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let exchange = ExchangeInner::new("kalshi", json!({
        "api_key_id": "your-key",
        "private_key_pem": "your-pem"
    })).unwrap();

    let order = exchange.create_order(
        "MARKET-ID",
        "Yes",
        OrderSide::Buy,
        0.65,
        10.0,
        HashMap::new(),
    ).await.unwrap();

    println!("Order {}: {:?}", order.id, order.status);
}
```

### Python

```python
from openpx import Exchange

exchange = Exchange("kalshi", {
    "api_key_id": "your-key",
    "private_key_pem": "your-pem",
})

order = exchange.create_order(
    market_id="MARKET-ID",
    outcome="Yes",
    side="buy",
    price=0.65,
    size=10.0,
)
print(f"Order {order.id}: {order.status}")
```

### TypeScript

```typescript
import { Exchange } from "@openpx/sdk";

const exchange = new Exchange("kalshi", {
  api_key_id: "your-key",
  private_key_pem: "your-pem",
});

const order = await exchange.createOrder(
  "MARKET-ID", "Yes", "buy", 0.65, 10.0
);
console.log(`Order ${order.id}: ${order.status}`);
```

## Fetch Orderbook

### Rust

```rust
let book = exchange.fetch_orderbook(OrderbookRequest {
    market_id: "MARKET-ID".into(),
    ..Default::default()
}).await.unwrap();
println!("Best bid: {:?}, Best ask: {:?}", book.best_bid(), book.best_ask());
```

### Python

```python
book = exchange.fetch_orderbook("MARKET-ID")
print(f"Best bid: {book.bids[0].price}, Best ask: {book.asks[0].price}")
```

### TypeScript

```typescript
const book = await exchange.fetchOrderbook("MARKET-ID");
console.log(`Best bid: ${book.bids[0].price}, Best ask: ${book.asks[0].price}`);
```
"""


def generate_rust_readme() -> str:
    return """# Rust SDK

The Rust SDK is the source of truth for OpenPX. All types, traits, and
exchange implementations are written in Rust.

## Crate Structure

| Crate | Description |
|-------|-------------|
| `px-core` | Core types, Exchange trait, error handling, timing |
| `px-sdk` | Unified facade — enum dispatch over all exchanges |
| `px-exchange-kalshi` | Kalshi exchange implementation |
| `px-exchange-polymarket` | Polymarket exchange implementation |
| `px-exchange-opinion` | Opinion exchange implementation |
| `px-exchange-limitless` | Limitless exchange implementation |
| `px-exchange-predictfun` | Predict.fun exchange implementation |

## Usage

```rust
use px_sdk::ExchangeInner;
use px_core::{Exchange, FetchMarketsParams};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an exchange (unauthenticated — market data only)
    let exchange = ExchangeInner::new("polymarket", json!({}))?;

    // Fetch markets with pagination
    let markets = exchange.fetch_markets(Some(FetchMarketsParams {
        limit: Some(10),
        cursor: None,
    })).await?;

    for m in &markets {
        println!("[{}] {} — prices: {:?}", m.id, m.question, m.prices);
    }

    Ok(())
}
```

## Exchange Trait

Every exchange implements `px_core::Exchange`. See the
[API Reference](./api.md) for all available types.
"""


def generate_python_readme() -> str:
    return """# Python SDK

The Python SDK wraps the Rust engine via PyO3, giving you native performance
with Pydantic models for type safety and autocomplete.

## Installation

```bash
pip install openpx
```

## Usage

```python
from openpx import Exchange

# Unauthenticated (market data only)
exchange = Exchange("polymarket")
markets = exchange.fetch_markets(limit=10)
for m in markets:
    print(f"[{m.id}] {m.question}")

# Authenticated (trading)
exchange = Exchange("kalshi", {
    "api_key_id": "...",
    "private_key_pem": "...",
})
positions = exchange.fetch_positions()
balance = exchange.fetch_balance()
```

## How It Works

```
User calls exchange.fetch_markets(limit=5)
         |
exchange.py  (pure Python wrapper)
         |  calls _native.NativeExchange.fetch_markets()
         |
lib.rs  (PyO3 → Rust, returns Python dict via pythonize)
         |
exchange.py  receives list[dict]
         |  wraps: [Market(**d) for d in raw_dicts]
         |
User receives list[Market]  (Pydantic models with autocomplete)
```

## Error Handling

```python
from openpx import Exchange, OpenPxError, AuthenticationError

try:
    exchange = Exchange("kalshi", {"api_key_id": "bad"})
    exchange.fetch_balance()
except AuthenticationError as e:
    print(f"Auth failed: {e}")
except OpenPxError as e:
    print(f"Error: {e}")
```

See the [API Reference](./api.md) for all available types.
"""


def generate_typescript_readme() -> str:
    return """# TypeScript SDK

The TypeScript SDK wraps the Rust engine via NAPI-RS, giving you native
performance with full TypeScript type definitions.

## Installation

```bash
npm install @openpx/sdk
```

## Usage

```typescript
import { Exchange } from "@openpx/sdk";

// Unauthenticated (market data only)
const exchange = new Exchange("polymarket", {});
const markets = await exchange.fetchMarkets(10);
for (const m of markets) {
  console.log(`[${m.id}] ${m.question}`);
}

// Authenticated (trading)
const authed = new Exchange("kalshi", {
  api_key_id: "...",
  private_key_pem: "...",
});
const positions = await authed.fetchPositions();
const balance = await authed.fetchBalance();
```

## Error Handling

```typescript
import { Exchange } from "@openpx/sdk";

try {
  const exchange = new Exchange("kalshi", { api_key_id: "bad" });
  await exchange.fetchBalance();
} catch (e) {
  console.error(`Error: ${e.message}`);
}
```

See the [API Reference](./api.md) for all available types.
"""


def generate_exchanges_reference() -> str:
    return """# Exchanges

## Supported Exchanges

### Kalshi

- **ID:** `kalshi`
- **Website:** [kalshi.com](https://kalshi.com)
- **API Docs:** [docs.kalshi.com](https://docs.kalshi.com)
- **Auth:** RSA key pair (`api_key_id` + `private_key_pem`)
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades, WebSocket

### Polymarket

- **ID:** `polymarket`
- **Website:** [polymarket.com](https://polymarket.com)
- **API Docs:** [docs.polymarket.com](https://docs.polymarket.com/developers/)
- **Auth:** Private key + optional CLOB API credentials
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades, WebSocket

### Opinion

- **ID:** `opinion`
- **Website:** [opinion.trade](https://opinion.trade)
- **API Docs:** [docs.opinion.trade](https://docs.opinion.trade/developer-guide/opinion-open-api)
- **Auth:** API key + private key + multi-sig address
- **Features:** Markets, Orders, Positions, Balance, Orderbook, WebSocket

### Limitless

- **ID:** `limitless`
- **Website:** [limitless.exchange](https://limitless.exchange)
- **API Docs:** [api.limitless.exchange](https://api.limitless.exchange/api-v1)
- **Auth:** Private key
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Trades, WebSocket

### Predict.fun

- **ID:** `predictfun`
- **Website:** [predict.fun](https://predict.fun)
- **API Docs:** [dev.predict.fun](https://dev.predict.fun/)
- **Auth:** API key + private key
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades

## Configuration

All exchanges accept a JSON config object. Pass exchange-specific fields:

```json
{
  "kalshi": {
    "api_key_id": "...",
    "private_key_pem": "...",
    "demo": false
  },
  "polymarket": {
    "private_key": "0x...",
    "funder": "0x...",
    "api_key": "...",
    "api_secret": "...",
    "api_passphrase": "..."
  },
  "opinion": {
    "api_key": "...",
    "private_key": "0x...",
    "multi_sig_addr": "0x..."
  },
  "limitless": {
    "private_key": "0x..."
  },
  "predictfun": {
    "api_key": "...",
    "private_key": "0x...",
    "testnet": false
  }
}
```
"""


def generate_errors_reference() -> str:
    return """# Error Handling

## Error Hierarchy

```
OpenPxError
├── Network
│   ├── Http(String)
│   ├── Timeout(u64)
│   └── Connection(String)
├── Exchange
│   ├── MarketNotFound(String)
│   ├── InvalidOrder(String)
│   ├── OrderRejected(String)
│   ├── InsufficientFunds(String)
│   ├── Authentication(String)
│   ├── NotSupported(String)
│   └── Api(String)
├── WebSocket
│   ├── Connection(String)
│   ├── Closed
│   ├── Protocol(String)
│   └── Subscription(String)
├── Signing
│   ├── InvalidKey
│   ├── SigningFailed(String)
│   └── Unsupported(String)
├── RateLimitExceeded
├── Serialization(Error)
├── Config(String)
├── InvalidInput(String)
└── Other(String)
```

## Language Mapping

### Rust

```rust
use px_core::{OpenPxError, ExchangeError};

match result {
    Err(OpenPxError::Exchange(ExchangeError::Authentication(msg))) => {
        eprintln!("Auth failed: {msg}");
    }
    Err(OpenPxError::Network(e)) => {
        eprintln!("Network error: {e}");
    }
    Err(e) => eprintln!("Error: {e}"),
    Ok(v) => { /* success */ }
}
```

### Python

```python
from openpx import Exchange, OpenPxError, AuthenticationError, NetworkError

try:
    exchange.fetch_balance()
except AuthenticationError as e:
    print(f"Auth failed: {e}")
except NetworkError as e:
    print(f"Network error: {e}")
except OpenPxError as e:
    print(f"Error: {e}")
```

### TypeScript

```typescript
try {
  await exchange.fetchBalance();
} catch (e) {
  console.error(e.message);
}
```
"""


def main():
    if not SCHEMA_PATH.exists():
        print(f"ERROR: {SCHEMA_PATH} not found. Run 'just schema' first.", file=sys.stderr)
        sys.exit(1)

    schema = load_schema()
    definitions = schema.get("definitions", {})
    categories = categorize_types(definitions)

    print(f"Generating SDK docs from {len(definitions)} type definitions...")

    # Auto-generated pages
    write_file(DOCS_SRC / "reference" / "models.md", generate_models_reference(definitions))
    write_file(DOCS_SRC / "rust" / "api.md", generate_lang_reference(definitions, "rust"))
    write_file(DOCS_SRC / "python" / "api.md", generate_lang_reference(definitions, "python"))
    write_file(DOCS_SRC / "typescript" / "api.md", generate_lang_reference(definitions, "typescript"))

    # Hand-written pages (only write if they don't exist, to allow manual edits)
    static_pages = {
        DOCS_SRC / "SUMMARY.md": generate_summary(categories),
        DOCS_SRC / "introduction.md": generate_introduction(),
        DOCS_SRC / "installation.md": generate_installation(),
        DOCS_SRC / "quickstart.md": generate_quickstart(),
        DOCS_SRC / "rust" / "README.md": generate_rust_readme(),
        DOCS_SRC / "python" / "README.md": generate_python_readme(),
        DOCS_SRC / "typescript" / "README.md": generate_typescript_readme(),
        DOCS_SRC / "reference" / "exchanges.md": generate_exchanges_reference(),
        DOCS_SRC / "reference" / "errors.md": generate_errors_reference(),
    }

    for path, content in static_pages.items():
        # Always overwrite — these are generated too. Users edit the script, not the output.
        write_file(path, content)

    print(f"Done. {len(definitions)} types across {len(categories)} categories.")
    print("Run 'cd docs && mdbook serve' to preview.")


if __name__ == "__main__":
    main()
