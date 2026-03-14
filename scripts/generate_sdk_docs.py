#!/usr/bin/env python3
"""Generate SDK documentation from openpx.schema.json.

Reads the JSON Schema and produces Starlight-compatible MDX pages showing
Rust, Python, and TypeScript representations of every exported type.

Usage:
    python3 scripts/generate_sdk_docs.py

Outputs:
    docs/src/content/docs/reference/models.mdx   — full type reference (tabbed)
    docs/src/content/docs/sdks/rust-api.mdx       — Rust-only reference
    docs/src/content/docs/sdks/python-api.mdx     — Python-only reference
    docs/src/content/docs/sdks/typescript-api.mdx — TypeScript-only reference
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
SCHEMA_PATH = ROOT / "schema" / "openpx.schema.json"
DOCS_SRC = ROOT / "docs" / "src" / "content" / "docs"

TABS_IMPORT = 'import { Tabs, TabItem } from "@astrojs/starlight/components";'

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


def escape_mdx(text: str) -> str:
    """Escape characters that MDX would interpret as JSX expressions."""
    return text.replace("{", "\\{").replace("}", "\\}")


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
        desc = escape_mdx(fschema.get("description", ""))
        lines.append(f"| `{fname}` | `{json_type}` | {req} | {desc} |")
    return "\n".join(lines)


def write_file(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    print(f"  wrote {path.relative_to(ROOT)}")


def tabs_block(rust_code: str, python_code: str, ts_code: str) -> list[str]:
    """Generate a <Tabs> block with Rust/Python/TypeScript code."""
    return [
        '<Tabs syncKey="lang">',
        '<TabItem label="Rust">',
        "",
        "```rust",
        rust_code,
        "```",
        "",
        "</TabItem>",
        '<TabItem label="Python">',
        "",
        "```python",
        python_code,
        "```",
        "",
        "</TabItem>",
        '<TabItem label="TypeScript">',
        "",
        "```typescript",
        ts_code,
        "```",
        "",
        "</TabItem>",
        "</Tabs>",
    ]


def generate_models_reference(definitions: dict[str, Any]) -> str:
    """Generate the full models reference page (all languages, tabbed)."""
    categories = categorize_types(definitions)
    lines = [
        "---",
        "title: Type Reference",
        "---",
        "",
        TABS_IMPORT,
        "",
        "All types auto-generated from Rust source via `schema/openpx.schema.json`.",
        "Run `just docs` to regenerate.",
        "",
    ]

    for category, type_names in categories.items():
        lines.append(f"## {category}")
        lines.append("")

        for name in type_names:
            defn = definitions[name]
            lines.append(f"### {name}")
            lines.append("")

            if is_enum_schema(defn):
                variants = get_enum_variants(defn)
                lines.append(f"Enum with variants: {', '.join(f'`{v}`' for v in variants)}")
                lines.append("")
            else:
                lines.append(generate_field_table(defn))
                lines.append("")

            lines.extend(tabs_block(
                generate_rust_block(name, defn),
                generate_python_block(name, defn),
                generate_ts_block(name, defn),
            ))
            lines.append("")
            lines.append("---")
            lines.append("")

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

    lines = [
        "---",
        f"title: {lang_display} API Reference",
        "---",
        "",
        f"All {lang_display} types auto-generated from `schema/openpx.schema.json`.",
        "",
    ]

    for category, type_names in categories.items():
        lines.append(f"## {category}")
        lines.append("")
        for name in type_names:
            defn = definitions[name]
            lines.append(f"### {name}")
            lines.append("")
            lines.append(f"```{ext}")
            lines.append(gen(name, defn))
            lines.append("```")
            lines.append("")

    return "\n".join(lines)


def generate_introduction() -> str:
    return """---
title: OpenPX
---

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
    return """---
title: Installation
---

import { Tabs, TabItem } from "@astrojs/starlight/components";

<Tabs syncKey="lang">
<TabItem label="Rust">

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

</TabItem>
<TabItem label="Python">

```bash
pip install openpx
```

Requires Python >= 3.9. The package includes a native Rust extension compiled
with PyO3 — no Rust toolchain needed on the user's machine.

</TabItem>
<TabItem label="TypeScript">

```bash
npm install @openpx/sdk
```

Requires Node.js >= 18. The package includes a native Rust addon compiled
with NAPI-RS.

</TabItem>
</Tabs>
"""


def generate_quickstart() -> str:
    return """---
title: Quick Start
---

import { Tabs, TabItem } from "@astrojs/starlight/components";

## Fetch Markets

<Tabs syncKey="lang">
<TabItem label="Rust">

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

</TabItem>
<TabItem label="Python">

```python
from openpx import Exchange

exchange = Exchange("kalshi")
markets = exchange.fetch_markets(limit=5)
for market in markets:
    print(f"{market.id}: {market.question}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
import { Exchange } from "@openpx/sdk";

const exchange = new Exchange("kalshi", {});
const markets = await exchange.fetchMarkets(5);
for (const market of markets) {
  console.log(`${market.id}: ${market.question}`);
}
```

</TabItem>
</Tabs>

## Create an Order

<Tabs syncKey="lang">
<TabItem label="Rust">

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

</TabItem>
<TabItem label="Python">

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

</TabItem>
<TabItem label="TypeScript">

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

</TabItem>
</Tabs>

## Fetch Orderbook

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let book = exchange.fetch_orderbook(OrderbookRequest {
    market_id: "MARKET-ID".into(),
    ..Default::default()
}).await.unwrap();
println!("Best bid: {:?}, Best ask: {:?}", book.best_bid(), book.best_ask());
```

</TabItem>
<TabItem label="Python">

```python
book = exchange.fetch_orderbook("MARKET-ID")
print(f"Best bid: {book.bids[0].price}, Best ask: {book.asks[0].price}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const book = await exchange.fetchOrderbook("MARKET-ID");
console.log(`Best bid: ${book.bids[0].price}, Best ask: ${book.asks[0].price}`);
```

</TabItem>
</Tabs>
"""


def generate_rust_readme() -> str:
    return """---
title: Rust SDK
---

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
[API Reference](/sdks/rust-api/) for all available types.
"""


def generate_python_readme() -> str:
    return """---
title: Python SDK
---

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

See the [API Reference](/sdks/python-api/) for all available types.
"""


def generate_typescript_readme() -> str:
    return """---
title: TypeScript SDK
---

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

See the [API Reference](/sdks/typescript-api/) for all available types.
"""


def generate_exchanges_reference() -> str:
    return """---
title: Exchanges
---

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


def generate_api_guide() -> str:
    return """---
title: API Methods
---

import { Tabs, TabItem } from "@astrojs/starlight/components";

Every exchange in OpenPX implements the same interface. Pick your language
once — all examples on this page switch together.

## Constructor

Create an exchange instance. Pass the exchange ID and an optional config
object with credentials.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `exchange_id` | `string` | Yes | Exchange identifier (`"kalshi"`, `"polymarket"`, `"opinion"`, `"limitless"`, `"predictfun"`) |
| `config` | `object` | No | Credentials object — omit for unauthenticated (market data only) access |

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_sdk::ExchangeInner;
use serde_json::json;

// Unauthenticated (market data only)
let exchange = ExchangeInner::new("kalshi", json!({})).unwrap();

// Authenticated (trading)
let exchange = ExchangeInner::new("kalshi", json!({
    "api_key_id": "your-key",
    "private_key_pem": "your-pem"
})).unwrap();
```

</TabItem>
<TabItem label="Python">

```python
from openpx import Exchange

# Unauthenticated (market data only)
exchange = Exchange("kalshi")

# Authenticated (trading)
exchange = Exchange("kalshi", {
    "api_key_id": "your-key",
    "private_key_pem": "your-pem",
})
```

</TabItem>
<TabItem label="TypeScript">

```typescript
import { Exchange } from "@openpx/sdk";

// Unauthenticated (market data only)
const exchange = new Exchange("kalshi", {});

// Authenticated (trading)
const exchange = new Exchange("kalshi", {
  api_key_id: "your-key",
  private_key_pem: "your-pem",
});
```

</TabItem>
</Tabs>

---

## Exchange Info

### id / name

Get the exchange identifier and human-readable name.

No parameters.

**Returns:** `string`

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let id: &str = exchange.id();     // "kalshi"
let name: &str = exchange.name(); // "Kalshi"
```

</TabItem>
<TabItem label="Python">

```python
exchange.id    # "kalshi"
exchange.name  # "Kalshi"
```

</TabItem>
<TabItem label="TypeScript">

```typescript
exchange.id;   // "kalshi"
exchange.name; // "Kalshi"
```

</TabItem>
</Tabs>

### describe

Returns capability flags for this exchange — which methods are supported.

No parameters.

**Returns:** [`ExchangeInfo`](/reference/models/#exchangeinfo)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let info = exchange.describe();
println!("Has WebSocket: {}", info.has_websocket);
println!("Has price history: {}", info.has_fetch_price_history);
```

</TabItem>
<TabItem label="Python">

```python
info = exchange.describe()
print(f"Has WebSocket: {info['has_websocket']}")
print(f"Has price history: {info['has_fetch_price_history']}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const info = exchange.describe();
console.log(`Has WebSocket: ${info.has_websocket}`);
console.log(`Has price history: ${info.has_fetch_price_history}`);
```

</TabItem>
</Tabs>

---

## Market Data

### fetch_markets

Fetch a paginated list of markets. All parameters are optional — call with no
arguments to use exchange defaults.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | `int` | No | Max markets to return per page |
| `cursor` | `string` | No | Pagination cursor from a previous response |

**Returns:** `list[Market]` — see [`Market`](/reference/models/#market)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::FetchMarketsParams;

// All markets (default pagination)
let markets = exchange.fetch_markets(None).await?;

// With pagination
let markets = exchange.fetch_markets(Some(FetchMarketsParams {
    limit: Some(10),
    cursor: None,
})).await?;

for m in &markets {
    println!("[{}] {} — ${:.2}", m.id, m.question, m.prices["Yes"]);
}
```

</TabItem>
<TabItem label="Python">

```python
# All markets (default pagination)
markets = exchange.fetch_markets()

# With limit
markets = exchange.fetch_markets(limit=10)

for m in markets:
    print(f"[{m.id}] {m.question}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
// All markets (default pagination)
const markets = await exchange.fetchMarkets();

// With limit
const markets = await exchange.fetchMarkets(10);

for (const m of markets) {
  console.log(`[${m.id}] ${m.question}`);
}
```

</TabItem>
</Tabs>

### fetch_market

Fetch a single market by its exchange-native ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Exchange-native market ID |

**Returns:** [`Market`](/reference/models/#market)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let market = exchange.fetch_market("KXBTC-25MAR14").await?;
println!("{}: {}", market.id, market.question);
```

</TabItem>
<TabItem label="Python">

```python
market = exchange.fetch_market("KXBTC-25MAR14")
print(f"{market.id}: {market.question}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const market = await exchange.fetchMarket("KXBTC-25MAR14");
console.log(`${market.id}: ${market.question}`);
```

</TabItem>
</Tabs>

### fetch_all_unified_markets

Fetch all markets with normalized fields across exchanges. Handles pagination
internally — returns the complete set.

No parameters.

**Returns:** `list[UnifiedMarket]` — see [`UnifiedMarket`](/reference/models/#unifiedmarket)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let markets = exchange.fetch_all_unified_markets().await?;
for m in &markets {
    println!("[{}] {} — {}", m.openpx_id, m.title, m.exchange);
}
```

</TabItem>
<TabItem label="Python">

```python
markets = exchange.fetch_all_unified_markets()
for m in markets:
    print(f"[{m['openpx_id']}] {m['title']} — {m['exchange']}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const markets = await exchange.fetchAllUnifiedMarkets();
for (const m of markets) {
  console.log(`[${m.openpx_id}] ${m.title} — ${m.exchange}`);
}
```

</TabItem>
</Tabs>

### fetch_event_markets

Fetch all markets belonging to an event or group. Falls back to scanning all
markets if the exchange has no native group endpoint.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `group_id` | `string` | **Yes** | Exchange event or group ID |

**Returns:** `list[UnifiedMarket]` — see [`UnifiedMarket`](/reference/models/#unifiedmarket)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let markets = exchange.fetch_event_markets("group-abc").await?;
for m in &markets {
    println!("[{}] {}", m.openpx_id, m.title);
}
```

</TabItem>
<TabItem label="Python">

```python
markets = exchange.fetch_event_markets("group-abc")
for m in markets:
    print(f"[{m['openpx_id']}] {m['title']}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const markets = await exchange.fetchEventMarkets("group-abc");
for (const m of markets) {
  console.log(`[${m.openpx_id}] ${m.title}`);
}
```

</TabItem>
</Tabs>

---

## Trading

### create_order

Place a limit order on a market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to trade |
| `outcome` | `string` | **Yes** | Outcome to buy/sell (e.g. `"Yes"`, `"No"`) |
| `side` | `OrderSide` | **Yes** | `Buy` or `Sell` |
| `price` | `float` | **Yes** | Limit price (0.0 – 1.0) |
| `size` | `float` | **Yes** | Number of contracts |
| `params` | `map[string, string]` | No | Exchange-specific parameters (e.g. order type, time-in-force) |

**Returns:** [`Order`](/reference/models/#order)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::OrderSide;
use std::collections::HashMap;

let order = exchange.create_order(
    "KXBTC-25MAR14",
    "Yes",
    OrderSide::Buy,
    0.65,
    10.0,
    HashMap::new(),
).await?;

println!("Order {}: {:?}", order.id, order.status);
```

</TabItem>
<TabItem label="Python">

```python
order = exchange.create_order(
    market_id="KXBTC-25MAR14",
    outcome="Yes",
    side="buy",
    price=0.65,
    size=10.0,
)
print(f"Order {order.id}: {order.status}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const order = await exchange.createOrder(
  "KXBTC-25MAR14", "Yes", "buy", 0.65, 10.0
);
console.log(`Order ${order.id}: ${order.status}`);
```

</TabItem>
</Tabs>

### cancel_order

Cancel an open order.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | `string` | **Yes** | Order to cancel |
| `market_id` | `string` | No | Market ID — required by some exchanges for faster lookup |

**Returns:** [`Order`](/reference/models/#order)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let cancelled = exchange.cancel_order("order-123", None).await?;
println!("Cancelled: {:?}", cancelled.status);
```

</TabItem>
<TabItem label="Python">

```python
cancelled = exchange.cancel_order("order-123")
print(f"Cancelled: {cancelled.status}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const cancelled = await exchange.cancelOrder("order-123");
console.log(`Cancelled: ${cancelled.status}`);
```

</TabItem>
</Tabs>

### fetch_order

Fetch a single order by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | `string` | **Yes** | Order ID |
| `market_id` | `string` | No | Market ID — required by some exchanges for faster lookup |

**Returns:** [`Order`](/reference/models/#order)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let order = exchange.fetch_order("order-123", None).await?;
println!("Status: {:?}, Filled: {}", order.status, order.filled);
```

</TabItem>
<TabItem label="Python">

```python
order = exchange.fetch_order("order-123")
print(f"Status: {order.status}, Filled: {order.filled}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const order = await exchange.fetchOrder("order-123");
console.log(`Status: ${order.status}, Filled: ${order.filled}`);
```

</TabItem>
</Tabs>

### fetch_open_orders

Fetch all open orders, optionally filtered by market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | No | Filter to a specific market |

**Returns:** `list[Order]` — see [`Order`](/reference/models/#order)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::FetchOrdersParams;

// All open orders
let orders = exchange.fetch_open_orders(None).await?;

// For a specific market
let orders = exchange.fetch_open_orders(Some(FetchOrdersParams {
    market_id: Some("KXBTC-25MAR14".into()),
})).await?;

for o in &orders {
    println!("{}: {} @ {:.2}", o.id, o.side, o.price);
}
```

</TabItem>
<TabItem label="Python">

```python
# All open orders
orders = exchange.fetch_open_orders()

# For a specific market
orders = exchange.fetch_open_orders(market_id="KXBTC-25MAR14")

for o in orders:
    print(f"{o.id}: {o.side} @ {o.price:.2f}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
// All open orders
const orders = await exchange.fetchOpenOrders();

// For a specific market
const orders = await exchange.fetchOpenOrders("KXBTC-25MAR14");

for (const o of orders) {
  console.log(`${o.id}: ${o.side} @ ${o.price.toFixed(2)}`);
}
```

</TabItem>
</Tabs>

---

## Portfolio

### fetch_positions

Fetch current positions, optionally filtered by market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | No | Filter to a specific market |

**Returns:** `list[Position]` — see [`Position`](/reference/models/#position)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let positions = exchange.fetch_positions(None).await?;
for p in &positions {
    println!("{} {} — size: {}, avg: {:.2}",
        p.market_id, p.outcome, p.size, p.average_price);
}
```

</TabItem>
<TabItem label="Python">

```python
positions = exchange.fetch_positions()
for p in positions:
    print(f"{p.market_id} {p.outcome} — size: {p.size}, avg: {p.average_price:.2f}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const positions = await exchange.fetchPositions();
for (const p of positions) {
  console.log(`${p.market_id} ${p.outcome} — size: ${p.size}, avg: ${p.average_price.toFixed(2)}`);
}
```

</TabItem>
</Tabs>

### fetch_balance

Fetch account balance.

No parameters.

**Returns:** `map[string, float]` — asset name to balance (e.g. `{"USDC": 1250.00}`)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let balance = exchange.fetch_balance().await?;
for (asset, amount) in &balance {
    println!("{}: ${:.2}", asset, amount);
}
```

</TabItem>
<TabItem label="Python">

```python
balance = exchange.fetch_balance()
for asset, amount in balance.items():
    print(f"{asset}: ${amount:.2f}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const balance = await exchange.fetchBalance();
for (const [asset, amount] of Object.entries(balance)) {
  console.log(`${asset}: $${amount.toFixed(2)}`);
}
```

</TabItem>
</Tabs>

### fetch_fills

Fetch trade execution history (fills).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | No | Filter to a specific market |
| `limit` | `int` | No | Max fills to return |

**Returns:** `list[Fill]` — see [`Fill`](/reference/models/#fill)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
let fills = exchange.fetch_fills(None, Some(20)).await?;
for f in &fills {
    println!("{}: {} {} @ {:.2} x {}",
        f.fill_id, f.side, f.outcome, f.price, f.size);
}
```

</TabItem>
<TabItem label="Python">

```python
fills = exchange.fetch_fills(limit=20)
for f in fills:
    print(f"{f.fill_id}: {f.side} {f.outcome} @ {f.price:.2f} x {f.size}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const fills = await exchange.fetchFills(undefined, 20);
for (const f of fills) {
  console.log(`${f.fill_id}: ${f.side} ${f.outcome} @ ${f.price.toFixed(2)} x ${f.size}`);
}
```

</TabItem>
</Tabs>

---

## Orderbook

### fetch_orderbook

Fetch the L2 orderbook for a market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market ID |
| `outcome` | `string` | No | Filter by outcome (e.g. `"Yes"`) |
| `token_id` | `string` | No | Filter by CTF token ID (Polymarket) |

**Returns:** [`Orderbook`](/reference/models/#orderbook)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::OrderbookRequest;

let book = exchange.fetch_orderbook(OrderbookRequest {
    market_id: "KXBTC-25MAR14".into(),
    outcome: None,
    token_id: None,
}).await?;

println!("Best bid: {:?}", book.best_bid());
println!("Best ask: {:?}", book.best_ask());
println!("Spread: {:?}", book.spread());

for level in &book.bids[..5.min(book.bids.len())] {
    println!("  BID {:.2} x {}", level.price, level.size);
}
```

</TabItem>
<TabItem label="Python">

```python
book = exchange.fetch_orderbook("KXBTC-25MAR14")

print(f"Best bid: {book['bids'][0]['price']}")
print(f"Best ask: {book['asks'][0]['price']}")

for level in book["bids"][:5]:
    print(f"  BID {level['price']:.2f} x {level['size']}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const book = await exchange.fetchOrderbook("KXBTC-25MAR14");

console.log(`Best bid: ${book.bids[0].price}`);
console.log(`Best ask: ${book.asks[0].price}`);

for (const level of book.bids.slice(0, 5)) {
  console.log(`  BID ${level.price.toFixed(2)} x ${level.size}`);
}
```

</TabItem>
</Tabs>

### fetch_orderbook_history

Fetch historical orderbook snapshots. Not all exchanges support this.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market ID |
| `token_id` | `string` | No | Token ID |
| `start_ts` | `int` | No | Start time (Unix seconds, inclusive) |
| `end_ts` | `int` | No | End time (Unix seconds, inclusive) |
| `limit` | `int` | No | Max snapshots per page |
| `cursor` | `string` | No | Pagination cursor from a previous response |

**Returns:** `(list[OrderbookSnapshot], cursor?)` — see [`OrderbookSnapshot`](/reference/models/#orderbooksnapshot).
The cursor is `null` when there are no more pages.

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::OrderbookHistoryRequest;

let (snapshots, next_cursor) = exchange.fetch_orderbook_history(
    OrderbookHistoryRequest {
        market_id: "KXBTC-25MAR14".into(),
        limit: Some(10),
        ..Default::default()
    }
).await?;

for snap in &snapshots {
    println!("{}: {} bids, {} asks",
        snap.timestamp, snap.bids.len(), snap.asks.len());
}
```

</TabItem>
<TabItem label="Python">

```python
# Not yet exposed in Python SDK
```

</TabItem>
<TabItem label="TypeScript">

```typescript
// Not yet exposed in TypeScript SDK
```

</TabItem>
</Tabs>

---

## Price History & Trades

### fetch_price_history

Fetch OHLCV candlestick data.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market ID |
| `interval` | `PriceHistoryInterval` | **Yes** | Candle interval: `1m`, `1h`, `6h`, `1d`, `1w`, or `max` |
| `outcome` | `string` | No | Filter by outcome name |
| `token_id` | `string` | No | Filter by token ID |
| `condition_id` | `string` | No | CTF condition ID (Polymarket) |
| `start_ts` | `int` | No | Start time (Unix seconds, inclusive) |
| `end_ts` | `int` | No | End time (Unix seconds, inclusive) |

**Returns:** `list[Candlestick]` — see [`Candlestick`](/reference/models/#candlestick)

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::{PriceHistoryRequest, PriceHistoryInterval};

let candles = exchange.fetch_price_history(PriceHistoryRequest {
    market_id: "KXBTC-25MAR14".into(),
    interval: PriceHistoryInterval::OneHour,
    start_ts: None,
    end_ts: None,
    outcome: None,
    token_id: None,
    condition_id: None,
}).await?;

for c in &candles {
    println!("{}: O={:.2} H={:.2} L={:.2} C={:.2} V={}",
        c.timestamp, c.open, c.high, c.low, c.close, c.volume);
}
```

</TabItem>
<TabItem label="Python">

```python
# Not yet exposed in Python SDK — use Rust directly
```

</TabItem>
<TabItem label="TypeScript">

```typescript
// Not yet exposed in TypeScript SDK — use Rust directly
```

</TabItem>
</Tabs>

### fetch_trades

Fetch recent trades for a market. Returns trades and an optional cursor for
pagination.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market ID |
| `market_ref` | `string` | No | Alternate market reference (e.g. Polymarket `conditionId`) |
| `outcome` | `string` | No | Filter by outcome name |
| `token_id` | `string` | No | Filter by token ID |
| `start_ts` | `int` | No | Start time (Unix seconds, inclusive) |
| `end_ts` | `int` | No | End time (Unix seconds, inclusive) |
| `limit` | `int` | No | Max trades per page |
| `cursor` | `string` | No | Pagination cursor from a previous response |

**Returns:** `(list[MarketTrade], cursor?)` — see [`MarketTrade`](/reference/models/#markettrade).
The cursor is `null` when there are no more pages.

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::TradesRequest;

let (trades, next_cursor) = exchange.fetch_trades(TradesRequest {
    market_id: "KXBTC-25MAR14".into(),
    limit: Some(50),
    ..Default::default()
}).await?;

for t in &trades {
    println!("{}: {:.2} x {} ({})",
        t.timestamp, t.price, t.size, t.source_channel);
}
```

</TabItem>
<TabItem label="Python">

```python
# Not yet exposed in Python SDK — use Rust directly
```

</TabItem>
<TabItem label="TypeScript">

```typescript
// Not yet exposed in TypeScript SDK — use Rust directly
```

</TabItem>
</Tabs>

---

## WebSocket

Real-time streaming via WebSocket for orderbook updates, trades, and fills.

See the [WebSocket Streaming](/guides/websocket/) guide for full documentation.

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::OrderBookWebSocket;
use futures::StreamExt;

let mut ws = exchange.websocket().unwrap();

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.orderbook_stream("KXBTC-25MAR14").await?;
while let Some(update) = stream.next().await {
    match update? {
        OrderbookUpdate::Snapshot(book) => {
            println!("Snapshot: {} bids, {} asks", book.bids.len(), book.asks.len());
        }
        OrderbookUpdate::Delta { changes, .. } => {
            for c in &changes {
                println!("  {:?} {:.2} x {}", c.side, c.price, c.size);
            }
        }
    }
}
```

</TabItem>
<TabItem label="Python">

```python
ws = exchange.websocket()
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for update in ws.orderbook_stream("KXBTC-25MAR14"):
    if update["type"] == "Snapshot":
        book = update["Snapshot"]
        print(f"Snapshot: {len(book['bids'])} bids, {len(book['asks'])} asks")
    elif update["type"] == "Delta":
        for c in update["Delta"]["changes"]:
            print(f"  {c['side']} {c['price']:.2f} x {c['size']}")

ws.disconnect()
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const ws = exchange.websocket();
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onOrderbookUpdate("KXBTC-25MAR14", (err, update) => {
  if (err) { console.error(err); return; }
  if (update.type === "Snapshot") {
    console.log(`Snapshot: ${update.Snapshot.bids.length} bids`);
  } else {
    for (const c of update.Delta.changes) {
      console.log(`  ${c.side} ${c.price} x ${c.size}`);
    }
  }
});
```

</TabItem>
</Tabs>
"""


def generate_websocket_guide() -> str:
    return """---
title: WebSocket Streaming
---

import { Tabs, TabItem } from "@astrojs/starlight/components";

OpenPX provides real-time streaming via WebSocket for orderbook updates,
trades, and fill events across all supported exchanges.

## Exchange Support

| Exchange | Orderbook | Trades | Fills | Protocol |
|----------|-----------|--------|-------|----------|
| Kalshi | Yes | Yes | Yes | Native WS |
| Polymarket | Yes | Yes | Yes | Native WS (dual connection) |
| Opinion | Yes | — | — | Native WS |
| Limitless | Yes | — | — | Socket.IO |
| Predict.fun | — | — | — | Not supported |

## Connection Lifecycle

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::OrderBookWebSocket;

// 1. Get WebSocket handle from exchange
let mut ws = exchange.websocket().unwrap();

// 2. Connect
ws.connect().await?;

// 3. Subscribe to markets
ws.subscribe("KXBTC-25MAR14").await?;
ws.subscribe("KXETH-25MAR14").await?;

// 4. Stream data (see sections below)
// ...

// 5. Unsubscribe when done
ws.unsubscribe("KXBTC-25MAR14").await?;

// 6. Disconnect
ws.disconnect().await?;
```

</TabItem>
<TabItem label="Python">

```python
from openpx import Exchange

exchange = Exchange("kalshi", {
    "api_key_id": "your-key",
    "private_key_pem": "your-pem",
})
ws = exchange.websocket()

# 2. Connect
ws.connect()

# 3. Subscribe to markets
ws.subscribe("KXBTC-25MAR14")
ws.subscribe("KXETH-25MAR14")

# 4. Stream data (see sections below)
# ...

# 5. Unsubscribe when done
ws.unsubscribe("KXBTC-25MAR14")

# 6. Disconnect
ws.disconnect()
```

</TabItem>
<TabItem label="TypeScript">

```typescript
import { Exchange } from "@openpx/sdk";

const exchange = new Exchange("kalshi", {
  api_key_id: "your-key",
  private_key_pem: "your-pem",
});
const ws = exchange.websocket();

// 2. Connect
await ws.connect();

// 3. Subscribe to markets
await ws.subscribe("KXBTC-25MAR14");
await ws.subscribe("KXETH-25MAR14");

// 4. Stream data (see sections below)
// ...

// 5. Unsubscribe when done
await ws.unsubscribe("KXBTC-25MAR14");

// 6. Disconnect
await ws.disconnect();
```

</TabItem>
</Tabs>

## Method Reference

### connect

Open the WebSocket connection. Must be called before subscribing or streaming.

No parameters.

**Returns:** `void` — throws on connection failure.

### disconnect

Close the WebSocket connection and clean up resources.

No parameters.

**Returns:** `void`

### subscribe

Subscribe to a market to begin receiving updates. You can subscribe to
multiple markets on the same connection.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to subscribe to |

**Returns:** `void` — throws if the market ID is invalid or the connection is not open.

### unsubscribe

Stop receiving updates for a market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to unsubscribe from |

**Returns:** `void`

### state

Check the connection state. Uses lock-free atomics for zero-cost reads.

No parameters.

**Returns:** `WebSocketState` — one of `Disconnected`, `Connecting`, `Connected`, `Reconnecting`, or `Closed`.

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::WebSocketState;

match ws.state() {
    WebSocketState::Disconnected => println!("Not connected"),
    WebSocketState::Connecting => println!("Connecting..."),
    WebSocketState::Connected => println!("Ready"),
    WebSocketState::Reconnecting => println!("Reconnecting..."),
    WebSocketState::Closed => println!("Closed"),
}
```

</TabItem>
<TabItem label="Python">

```python
state = ws.state  # "Connected", "Disconnected", etc.
print(f"WebSocket state: {state}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const state = ws.state; // "Connected", "Disconnected", etc.
console.log(`WebSocket state: ${state}`);
```

</TabItem>
</Tabs>

### orderbook_stream

Open a stream of real-time orderbook updates for a subscribed market. The
first message is always a full `Snapshot`, followed by incremental `Delta`
updates. You must call `subscribe` before opening a stream.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to stream (must be already subscribed) |

**Returns:** `Stream[OrderbookUpdate]` — yields `Snapshot` or `Delta` events.

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::{OrderBookWebSocket, OrderbookUpdate};
use futures::StreamExt;

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.orderbook_stream("KXBTC-25MAR14").await?;

while let Some(update) = stream.next().await {
    match update? {
        OrderbookUpdate::Snapshot(book) => {
            println!("Full snapshot:");
            println!("  Best bid: {:?}", book.bids.first());
            println!("  Best ask: {:?}", book.asks.first());
            println!("  {} bids, {} asks", book.bids.len(), book.asks.len());
        }
        OrderbookUpdate::Delta { changes, timestamp } => {
            for change in &changes {
                let action = if change.size == 0.0 { "REMOVE" } else { "UPDATE" };
                println!("  {} {:?} {:.2} x {}",
                    action, change.side, change.price, change.size);
            }
        }
    }
}
```

</TabItem>
<TabItem label="Python">

```python
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for update in ws.orderbook_stream("KXBTC-25MAR14"):
    if update["type"] == "Snapshot":
        book = update["Snapshot"]
        print(f"Full snapshot:")
        print(f"  Best bid: {book['bids'][0]}")
        print(f"  Best ask: {book['asks'][0]}")
        print(f"  {len(book['bids'])} bids, {len(book['asks'])} asks")
    elif update["type"] == "Delta":
        delta = update["Delta"]
        for change in delta["changes"]:
            action = "REMOVE" if change["size"] == 0 else "UPDATE"
            print(f"  {action} {change['side']} {change['price']:.2f} x {change['size']}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onOrderbookUpdate("KXBTC-25MAR14", (err, update) => {
  if (err) { console.error(err); return; }
  if (update.type === "Snapshot") {
    const book = update.Snapshot;
    console.log(`Full snapshot:`);
    console.log(`  Best bid: ${JSON.stringify(book.bids[0])}`);
    console.log(`  Best ask: ${JSON.stringify(book.asks[0])}`);
    console.log(`  ${book.bids.length} bids, ${book.asks.length} asks`);
  } else if (update.type === "Delta") {
    for (const change of update.Delta.changes) {
      const action = change.size === 0 ? "REMOVE" : "UPDATE";
      console.log(`  ${action} ${change.side} ${change.price} x ${change.size}`);
    }
  }
});
```

</TabItem>
</Tabs>

#### Update Types

| Type | Description |
|------|-------------|
| **Snapshot** | Full orderbook state. Sent on first subscribe and after reconnection. Contains complete `bids` and `asks` arrays. |
| **Delta** | Incremental change. Each change has `side` (Bid/Ask), `price`, and `size`. A `size` of `0` means remove that price level. |

See the [Type Reference](/reference/models/#orderbook) for the full `Orderbook`,
`PriceLevel`, and `PriceLevelChange` type definitions.

### activity_stream

Open a stream of real-time trade and fill events for a subscribed market.
Trades are public market activity; fills are your personal order executions.
You must call `subscribe` before opening a stream.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to stream (must be already subscribed) |

**Returns:** `Stream[ActivityEvent]` — yields `Trade` or `Fill` events.

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::{OrderBookWebSocket, ActivityEvent};
use futures::StreamExt;

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.activity_stream("KXBTC-25MAR14").await?;

while let Some(event) = stream.next().await {
    match event? {
        ActivityEvent::Trade(trade) => {
            println!("TRADE: {} x {} @ {:.2} [{}]",
                trade.outcome.unwrap_or_default(),
                trade.size, trade.price,
                trade.source_channel);
        }
        ActivityEvent::Fill(fill) => {
            println!("FILL: {} x {} @ {:.2} ({})",
                fill.outcome.unwrap_or_default(),
                fill.size, fill.price,
                fill.liquidity_role
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_default());
        }
    }
}
```

</TabItem>
<TabItem label="Python">

```python
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for event in ws.activity_stream("KXBTC-25MAR14"):
    if "Trade" in event:
        t = event["Trade"]
        print(f"TRADE: {t.get('outcome', '')} x {t['size']} @ {t['price']:.2f}")
    elif "Fill" in event:
        f = event["Fill"]
        print(f"FILL: {f.get('outcome', '')} x {f['size']} @ {f['price']:.2f}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onActivityUpdate("KXBTC-25MAR14", (err, event) => {
  if (err) { console.error(err); return; }
  if (event.Trade) {
    const t = event.Trade;
    console.log(`TRADE: ${t.outcome} x ${t.size} @ ${t.price}`);
  } else if (event.Fill) {
    const f = event.Fill;
    console.log(`FILL: ${f.outcome} x ${f.size} @ ${f.price}`);
  }
});
```

</TabItem>
</Tabs>

#### Event Types

See [`ActivityTrade`](/reference/models/#activitytrade) and
[`ActivityFill`](/reference/models/#activityfill) for field details.

| Event | Description | Exchanges |
|-------|-------------|-----------|
| **Trade** | Public market trade. Includes price, size, aggressor side, and outcome. | Kalshi, Polymarket |
| **Fill** | Your order was filled. Includes fill ID, order ID, liquidity role (maker/taker), and fee info. | Kalshi, Polymarket |

## Auto-Reconnect

WebSocket connections automatically reconnect on failure with exponential
backoff. No user intervention required.

| Setting | Value |
|---------|-------|
| Ping interval | 20 seconds |
| Initial reconnect delay | 3 seconds |
| Max reconnect delay | 60 seconds |
| Max reconnect attempts | 10 |

After reconnecting, subscriptions are automatically restored and a fresh
orderbook snapshot is sent.

## Error Handling

<Tabs syncKey="lang">
<TabItem label="Rust">

```rust
use px_core::WebSocketError;

match ws.connect().await {
    Ok(()) => println!("Connected"),
    Err(WebSocketError::Connection(msg)) => {
        // Retryable — auto-reconnect will handle this
        eprintln!("Connection failed: {msg}");
    }
    Err(WebSocketError::Subscription(msg)) => {
        // Bad market ID or unauthorized
        eprintln!("Subscription failed: {msg}");
    }
    Err(WebSocketError::Protocol(msg)) => {
        eprintln!("Protocol error: {msg}");
    }
    Err(WebSocketError::Closed) => {
        eprintln!("Connection was closed");
    }
}
```

</TabItem>
<TabItem label="Python">

```python
from openpx import Exchange, OpenPxError

exchange = Exchange("kalshi", {"api_key_id": "...", "private_key_pem": "..."})
ws = exchange.websocket()

try:
    ws.connect()
except OpenPxError as e:
    print(f"WebSocket error: {e}")
```

</TabItem>
<TabItem label="TypeScript">

```typescript
const ws = exchange.websocket();

try {
  await ws.connect();
} catch (e) {
  console.error(`WebSocket error: ${e.message}`);
}
```

</TabItem>
</Tabs>

## Performance Notes

The WebSocket implementation is optimized for low-latency trading:

- **Lock-free state reads** — `WebSocketState` uses atomic operations, no mutex contention
- **Stack-allocated deltas** — up to 4 price level changes per update use `SmallVec` (no heap allocation)
- **Broadcast channels** — 16K-slot capacity prevents slow consumers from blocking producers
- **Cached orderbooks** — full book state is maintained per-market so reconnects only need a snapshot diff
"""


def generate_errors_reference() -> str:
    return """---
title: Error Handling
---

import { Tabs, TabItem } from "@astrojs/starlight/components";

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

<Tabs syncKey="lang">
<TabItem label="Rust">

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

</TabItem>
<TabItem label="Python">

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

</TabItem>
<TabItem label="TypeScript">

```typescript
try {
  await exchange.fetchBalance();
} catch (e) {
  console.error(e.message);
}
```

</TabItem>
</Tabs>
"""


def main():
    if not SCHEMA_PATH.exists():
        print(f"ERROR: {SCHEMA_PATH} not found. Run 'just schema' first.", file=sys.stderr)
        sys.exit(1)

    schema = load_schema()
    definitions = schema.get("definitions", {})
    categories = categorize_types(definitions)

    print(f"Generating SDK docs from {len(definitions)} type definitions...")

    # Auto-generated pages (from schema)
    write_file(DOCS_SRC / "reference" / "models.mdx", generate_models_reference(definitions))
    write_file(DOCS_SRC / "sdks" / "rust-api.mdx", generate_lang_reference(definitions, "rust"))
    write_file(DOCS_SRC / "sdks" / "python-api.mdx", generate_lang_reference(definitions, "python"))
    write_file(DOCS_SRC / "sdks" / "typescript-api.mdx", generate_lang_reference(definitions, "typescript"))

    # Static pages (always overwritten — edit the script, not the output)
    static_pages = {
        DOCS_SRC / "index.mdx": generate_introduction(),
        DOCS_SRC / "getting-started" / "installation.mdx": generate_installation(),
        DOCS_SRC / "getting-started" / "quickstart.mdx": generate_quickstart(),
        DOCS_SRC / "guides" / "api.mdx": generate_api_guide(),
        DOCS_SRC / "guides" / "websocket.mdx": generate_websocket_guide(),
        DOCS_SRC / "sdks" / "rust.mdx": generate_rust_readme(),
        DOCS_SRC / "sdks" / "python.mdx": generate_python_readme(),
        DOCS_SRC / "sdks" / "typescript.mdx": generate_typescript_readme(),
        DOCS_SRC / "reference" / "exchanges.mdx": generate_exchanges_reference(),
        DOCS_SRC / "reference" / "errors.mdx": generate_errors_reference(),
    }

    for path, content in static_pages.items():
        write_file(path, content)

    print(f"Done. {len(definitions)} types across {len(categories)} categories.")
    print("Run 'cd docs && npm run dev' to preview.")


if __name__ == "__main__":
    main()
