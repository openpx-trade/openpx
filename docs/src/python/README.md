# Python SDK

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
