# Overview

## Opinion CLOB SDK

Welcome to the official documentation for the **Opinion CLOB SDK** - a Python library for interacting with Opinion Labs' prediction markets via the Central Limit Order Book (CLOB) API.

> 🔬 **Technical Preview**: Version 0.4.1 features BNB Chain support. While fully functional and tested, we recommend thorough testing before production use.
>
> To request SDK/API access, Please kindly fill out this [short application form ](https://docs.google.com/forms/d/1h7gp8UffZeXzYQ-lv4jcou9PoRNOqMAQhyW4IwZDnII).&#x20;
>
> *API Key can be used for Opinion OpenAPI, Opinion Websocket, and Opinion CLOB SDK*

### What is Opinion CLOB SDK?

The Opinion CLOB SDK provides a Python interface for building applications on top of Opinion prediction market infrastructure. It enables developers to:

* **Query market data** - Access real-time market information, prices, and orderbooks
* **Execute trades** - Place market and limit orders with EIP712 signing
* **Manage positions** - Track balances, positions, and trading history
* **Interact with smart contracts** - Split, merge, and redeem tokens on BNB Chain blockchain

### Key Features

#### Production-Ready

* **Type-safe** - Full type hints and Pythonic naming conventions
* **Well-tested** - test suite with 95%+ coverage
* **Reliable** - Built on industry-standard libraries (Web3.py, eth-account)
* **Documented** - Extensive documentation with examples

#### Performance Optimized

* **Smart caching** - Configurable TTL for market data and quote tokens
* **Batch operations** - Place or cancel multiple orders efficiently
* **Gas optimization** - Minimal on-chain transactions

#### Secure by Design

* **EIP712 signing** - Industry-standard typed data signatures
* **Multi-sig support** - Gnosis Safe integration for institutional users
* **Private key safety** - Keys never leave your environment

#### Blockchain Support

* **BNB Chain Mainnet** (Chain ID: 56)

### Use Cases

#### Trading Applications

Build automated trading bots, market-making applications, or custom trading interfaces.

```python
from opinion_clob_sdk import Client
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide

client = Client(host='https://proxy.opinion.trade:8443', apikey='your_key', ...)

# Place a limit order
order = PlaceOrderDataInput(
    marketId=123,
    tokenId='token_yes',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.55',
    makerAmountInQuoteToken=100
)
result = client.place_order(order)
```

#### Market Analytics

Aggregate and analyze market data for research or monitoring dashboards.

```python
# Get all active markets
markets = client.get_markets(status=TopicStatusFilter.ACTIVATED, limit=100)

# Analyze orderbook depth
orderbook = client.get_orderbook(token_id='token_123')
print(f"Best bid: {orderbook.bids[0]['price']}")
print(f"Best ask: {orderbook.asks[0]['price']}")
```

#### Portfolio Management

Track positions and balances across multiple markets.

```python
# Get user positions
positions = client.get_my_positions(limit=50)

# Get balances
balances = client.get_my_balances()

# Get trade history
trades = client.get_my_trades(market_id=123)
```

### Architecture

The Opinion CLOB SDK is built with a modular architecture:

```
┌─────────────────────────────────────────────┐
│          Application Layer                  │
│         (Your Python Code)                  │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│         Opinion CLOB SDK                    │
│  ┌──────────────┐   ┌─────────────────┐     │
│  │ Client API   │   │ Contract Caller │     │
│  │ (REST)       │   │ (Blockchain)    │     │
│  └──────┬───────┘   └──────────┬──────┘     │
└─────────┼──────────────────────┼────────────┘
          │                      │
┌─────────▼──────────┐  ┌───────-▼───────────┐
│  Opinion API       │  │     Blockchain     │
│  (CLOB Exchange)   │  │  (Smart Contracts) │
└────────────────────┘  └────────────────────┘
```

### Quick Links

* [📦 Installation Guide](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/getting-started/installation)
* [⚡ Quick Start](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/getting-started/quick-start)
* [🧠 Core Concepts](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts)
* [📚 API Reference](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/api-references)
* [❓ FAQ](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/support/faq)

***

Ready to get started? Head to the Installation Guide to begin building with Opinion CLOB SDK!
