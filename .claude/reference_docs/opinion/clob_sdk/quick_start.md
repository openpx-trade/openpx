# Quick Start

Get up and running with the Opinion CLOB SDK in minutes. This guide will walk you through your first integration.

### Prerequisites

Before starting, ensure you have:

1. **Python 3.9.10+** installed
2. **Opinion CLOB SDK** installed (Installation Guide)
3. **API credentials** from Opinion Labs:
   * API Key
   * Private Key (for signing orders)
   * Multi-sig wallet address (create on <https://app.opinion.trade>)
   * RPC URL (BNB Chain mainnet)

> **Need credentials?** Fill out this [short application form](https://docs.google.com/forms/d/1h7gp8UffZeXzYQ-lv4jcou9PoRNOqMAQhyW4IwZDnII) to get your API key.

### 5-Minute Quickstart

#### Step 1: Set Up Environment

Create a `.env` file in your project directory:

```bash
# .env file
API_KEY=your_api_key_here
RPC_URL=https://bsc-dataseed.binance.org
PRIVATE_KEY=0x1234567890abcdef...
MULTI_SIG_ADDRESS=0xYourWalletAddress...
HOST=https://proxy.opinion.trade:8443
CHAIN_ID=56
CONDITIONAL_TOKEN_ADDR=0xAD1a38cEc043e70E83a3eC30443dB285ED10D774
MULTISEND_ADDR=0x998739BFdAAdde7C933B942a68053933098f9EDa
```

#### Step 2: Initialize the Client

Create a new Python file (`my_first_app.py`):

```python
import os
from dotenv import load_dotenv
from opinion_clob_sdk import Client

# Load environment variables
load_dotenv()

# Initialize client
client = Client(
    host='https://proxy.opinion.trade:8443',
    apikey=os.getenv('API_KEY'),
    chain_id=56,  # BNB Chain mainnet
    rpc_url=os.getenv('RPC_URL'),
    private_key=os.getenv('PRIVATE_KEY'),
    multi_sig_addr=os.getenv('MULTI_SIG_ADDRESS'),
    conditional_tokens_addr=os.getenv('CONDITIONAL_TOKEN_ADDR'),
    multisend_addr=os.getenv('0x998739BFdAAdde7C933B942a68053933098f9EDa')
)

print("✓ Client initialized successfully!")
```

#### Step 3: Fetch Market Data

Add market data fetching:

```python
from opinion_clob_sdk.model import TopicStatusFilter

# Get all active markets
markets_response = client.get_markets(
    status=TopicStatusFilter.ACTIVATED,
    page=1,
    limit=10
)

# Parse the response
if markets_response.errno == 0:
    markets = markets_response.result.list
    print(f"\n✓ Found {len(markets)} active markets:")

    for market in markets[:3]:  # Show first 3
        print(f"  - Market #{market.market_id}: {market.market_title}")
        print(f"    Status: {market.status}")
        print()
else:
    print(f"Error: {markets_response.errmsg}")
```

#### Step 4: Get Market Details

```python
# Get details for a specific market
market_id = markets[0].topic_id  # Use first market from above

market_detail = client.get_market(market_id)
if market_detail.errno == 0:
    market = market_detail.result.data
    print(f"\n✓ Market Details for #{market_id}:")
    print(f"  Title: {market.market_title}")
    print(f"  Question ID: {market.question_id}")
    print(f"  Quote Token: {market.quote_token}")
    print(f"  Chain ID: {market.chain_id}")
```

#### Step 5: Check Orderbook

```python
# Assuming the market has a token (get from market.options for binary markets)
# For this example, we'll use a placeholder token_id
token_id = "your_token_id_here"  # Replace with actual token ID

try:
    orderbook = client.get_orderbook(token_id)
    if orderbook.errno == 0:
        book = orderbook.result.data
        print(f"\n✓ Orderbook for token {token_id}:")
        print(f"  Best Bid: {book.bids[0] if book.bids else 'No bids'}")
        print(f"  Best Ask: {book.asks[0] if book.asks else 'No asks'}")
except Exception as e:
    print(f"  (Skip if token_id not set: {e})")
```

#### Complete Example

Here's the complete `my_first_app.py`:

```python
import os
from dotenv import load_dotenv
from opinion_clob_sdk import Client
from opinion_clob_sdk.model import TopicStatusFilter

# Load environment variables
load_dotenv()

def main():
    # Initialize client
    client = Client(
        host='https://proxy.opinion.trade:8443',
        apikey=os.getenv('API_KEY'),
        chain_id=56,
        rpc_url=os.getenv('RPC_URL'),
        private_key=os.getenv('PRIVATE_KEY'),
        multi_sig_addr=os.getenv('MULTI_SIG_ADDRESS')
    )
    print("✓ Client initialized successfully!")

    # Get active markets
    markets_response = sdk.get_markets(
        status=TopicStatusFilter.ACTIVATED,
        limit=5
    )

    if markets_response.errno == 0:
        markets = markets_response.result.list
        print(f"\n✓ Found {len(markets)} active markets\n")

        # Display markets
        for i, market in enumerate(markets, 1):
            print(f"{i}. {market.market_title}")
            print(f"   Market ID: {market.market_id}")
            print()

        # Get details for first market
        if markets:
            first_market = markets[0]
            detail = sdk.get_categorical_market(market_id=first_market.market_id)

            if detail.errno == 0:
                m = detail.result.data
                print(f"✓ Details for '{m.market_title}':")
                print(f"  Status: {m.status}")
                print(f"  Question ID: {m.question_id}")
                print(f"  Quote Token: {m.quote_token}")
     else:
        print(f"Error fetching markets: {markets_response.errmsg}")

if __name__ == '__main__':
    main()
```

#### Run Your App

```bash
# Install python-dotenv if not already installed
pip install python-dotenv

# Run the script
python my_first_app.py
```

**Expected Output:**

```
✓ Client initialized successfully!

✓ Found 5 active markets

1. Will Bitcoin reach $100k by end of 2025?
   Market ID: 1

2. Will AI surpass human intelligence by 2030?
   Market ID: 2

...

✓ Details for 'Will Bitcoin reach $100k by end of 2025?':
  Status: 2
  Condition ID: 0xabc123...
  Quote Token: 0xdef456...
```

### Next Steps

Now that you've fetched market data, explore more advanced features:

#### Trading

Learn how to place orders:

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import LIMIT_ORDER

# Enable trading (required once before placing orders)
client.enable_trading()

# Place a buy order of "No" token
order_data = PlaceOrderDataInput(
    marketId=813,
    tokenId='33095770954068818933468604332582424490740136703838404213332258128147961949614',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.55',
    makerAmountInQuoteToken=10  # 10 USDT
)

result = client.place_order(order_data)
print(f"Order placed: {result}")
```

See Placing Orders for detailed examples.

#### Position Management

Track your positions:

```python
# Get balances
balances = client.get_my_balances()

# Get positions
positions = client.get_my_positions(limit=20)

# Get trade history
trades = client.get_my_trades(market_id=813)
```

See Managing Positions for more.

#### Smart Contract Operations

Interact with blockchain:

```python
# Split USDT into outcome tokens
tx_hash, receipt, event = client.split(
    market_id=813,
    amount=1000000000000000000  # 1 USDT (18 decimals for USDT)
)

# Merge outcome tokens back to USDT
tx_hash, receipt, event = client.merge(
    market_id=813,
    amount=1000000000000000000
)

# Redeem winnings after market resolves
tx_hash, receipt, event = client.redeem(market_id=813)
```

See Contract Operations for details.

### Common Patterns

#### Error Handling

Always check response status:

```python
response = client.get_markets()

if response.errno == 0:
    # Success
    markets = response.result.list
else:
    # Error
    print(f"Error {response.errno}: {response.errmsg}")
```

#### Using Try-Except

```python
from opinion_clob_sdk import InvalidParamError, OpenApiError

try:
    market = client.get_market(market_id=123)
except InvalidParamError as e:
    print(f"Invalid parameter: {e}")
except OpenApiError as e:
    print(f"API error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

#### Pagination

For large datasets:

```python
page = 1
all_markets = []

while True:
    response = client.get_markets(page=page, limit=100)
    if response.errno != 0:
        break

    markets = response.result.list
    all_markets.extend(markets)

    # Check if more pages exist
    if len(markets) < 100:
        break

    page += 1

print(f"Total markets: {len(all_markets)}")
```

### Configuration Tips

#### Cache Settings

Optimize performance with caching:

```python
client = Client(
    # ... other params ...
    market_cache_ttl=300,        # Cache markets for 5 minutes
    quote_tokens_cache_ttl=3600, # Cache quote tokens for 1 hour
    enable_trading_check_interval=3600  # Check trading status hourly
)
```

Set to `0` to disable caching:

```python
client = Client(
    # ... other params ...
    market_cache_ttl=0  # Disable market caching
)
```

#### Chain Selection

For production deployment, ensure you're using the correct configuration:

```python
client = Client(
    host='https://proxy.opinion.trade:8443',
    chain_id=56,  # BNB Chain mainnet
    rpc_url='https://bsc-dataseed.binance.org',  # BNB Chain RPC
    # ... other params ...
)
```

### Resources

* [**API Reference**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/api-references): All Supported Methods
* [**Configuration Guide**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/getting-started/configuration): Configuration
* [**Core Concepts**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts): Architecture
* [**Troubleshooting**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/support/troubleshooting): Common Issues

***

**Ready to build?** Explore the API Reference to see all available methods!
