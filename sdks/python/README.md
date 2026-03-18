# openpx

[![PyPI](https://img.shields.io/pypi/v/openpx.svg)](https://pypi.org/project/openpx/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Python SDK for OpenPX — a unified interface for prediction markets.

## Installation

```bash
pip install openpx
```

## Quick Start

```python
from openpx import Exchange

# Connect to Kalshi (public API)
exchange = Exchange("kalshi")
markets = exchange.fetch_markets()

for market in markets[:5]:
    print(f"{market['question']}: {market['prices']}")
```

## Supported Exchanges

- **Kalshi** — US-regulated event contracts
- **Polymarket** — Crypto-native prediction markets
- **Opinion** — Opinion exchange markets

## Requirements

- Python >= 3.9
- pydantic >= 2.0
