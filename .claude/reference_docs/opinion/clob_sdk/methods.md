# Methods

Complete reference for all methods available in the `Client` class.

### Overview

The `Client` class provides a unified interface for interacting with OPINION prediction markets. Methods are organized into these categories:

* **Market Data** - Query markets, prices, and orderbooks
* **Trading Operations** - Place and manage orders
* **User Data** - Access balances, positions, and trades
* **Smart Contract Operations** - Blockchain interactions (split, merge, redeem)

### Response Format

All API methods return responses with this structure:

```python
response = client.get_markets()

# Check success
if response.errno == 0:
    # Success - access data
    data = response.result.data  # For single objects
    # or
    items = response.result.list  # For arrays
else:
    # Error - check error message
    print(f"Error {response.errno}: {response.errmsg}")
```

**Response fields:**

* `errno` - Error code (`0` = success, non-zero = error)
* `errmsg` - Error message string
* `result` - Contains `data` (single object) or `list` (array of objects)

### Market Data Methods

#### get\_markets()

Get a paginated list of prediction markets.

**Signature:**

```python
def get_markets(
    topic_type: Optional[TopicType] = None,
    page: int = 1,
    limit: int = 20,
    status: Optional[TopicStatusFilter] = None
) -> Any
```

**Parameters:**

| Name         | Type                | Required | Default | Description                                                           |
| ------------ | ------------------- | -------- | ------- | --------------------------------------------------------------------- |
| `topic_type` | `TopicType`         | No       | `None`  | Filter by market type (`TopicType.BINARY` or `TopicType.CATEGORICAL`) |
| `page`       | `int`               | No       | `1`     | Page number (≥ 1)                                                     |
| `limit`      | `int`               | No       | `20`    | Items per page (1-20)                                                 |
| `status`     | `TopicStatusFilter` | No       | `None`  | Filter by status (`ACTIVATED`, `RESOLVED`, or `ALL`)                  |

**Returns:** API response with `result.list` containing market objects

**Example:**

```python
from opinion_clob_sdk.model import TopicType, TopicStatusFilter

# Get all active binary markets
response = client.get_markets(
    topic_type=TopicType.BINARY,
    status=TopicStatusFilter.ACTIVATED,
    page=1,
    limit=10
)

if response.errno == 0:
    markets = response.result.list
    for market in markets:
        print(f"{market.market_id}: {market.market_title}")
```

**Raises:**

* `InvalidParamError` - If page < 1 or limit not in range \[1, 20]

***

#### get\_market()

Get detailed information about a specific market.

**Signature:**

```python
def get_market(market_id: int, use_cache: bool = True) -> Any
```

**Parameters:**

| Name        | Type   | Required | Default | Description                             |
| ----------- | ------ | -------- | ------- | --------------------------------------- |
| `market_id` | `int`  | Yes      | -       | Market ID to query                      |
| `use_cache` | `bool` | No       | `True`  | Whether to use cached data if available |

**Returns:** API response with `result.data` containing market details

**Example:**

```python
response = client.get_market(market_id=123, use_cache=True)

if response.errno == 0:
    market = response.result.data
    print(f"Title: {market.market_title}")
    print(f"Status: {market.status}")
    print(f"Condition ID: {market.condition_id}")
    print(f"Quote Token: {market.quote_token}")
```

**Caching:**

* Cache duration controlled by `market_cache_ttl` (default: 300 seconds)
* Set `use_cache=False` to force fresh data
* Set `market_cache_ttl=0` in Client constructor to disable caching

**Raises:**

* `InvalidParamError` - If market\_id is missing or invalid
* `OpenApiError` - If API request fails

***

#### get\_categorical\_market()

Get detailed information about a categorical market (multi-outcome).

**Signature:**

```python
def get_categorical_market(market_id: int) -> Any
```

**Parameters:**

| Name        | Type  | Required | Description           |
| ----------- | ----- | -------- | --------------------- |
| `market_id` | `int` | Yes      | Categorical market ID |

**Returns:** API response with categorical market data

**Example:**

```python
response = client.get_categorical_market(market_id=456)

if response.errno == 0:
    market = response.result.data
    print(f"Options: {market.options}")  # Multiple outcomes
```

***

#### get\_quote\_tokens()

Get list of supported quote tokens (collateral currencies).

**Signature:**

```python
def get_quote_tokens(use_cache: bool = True) -> Any
```

**Parameters:**

| Name        | Type   | Required | Default | Description                |
| ----------- | ------ | -------- | ------- | -------------------------- |
| `use_cache` | `bool` | No       | `True`  | Whether to use cached data |

**Returns:** API response with `result.list` containing quote token objects

**Example:**

```python
response = client.get_quote_tokens()

if response.errno == 0:
    tokens = response.result.list
    for token in tokens:
        print(f"Token: {token.quote_token_address}")
        print(f"Decimals: {token.decimal}")
        print(f"Exchange: {token.ctf_exchange_address}")
```

**Caching:**

* Default TTL: 3600 seconds (1 hour)
* Controlled by `quote_tokens_cache_ttl` parameter

***

#### get\_orderbook()

Get orderbook (bids and asks) for a specific token.

**Signature:**

```python
def get_orderbook(token_id: str) -> Any
```

**Parameters:**

| Name       | Type  | Required | Description                                 |
| ---------- | ----- | -------- | ------------------------------------------- |
| `token_id` | `str` | Yes      | Token ID (e.g., "token\_yes", "token\_123") |

**Returns:** API response with orderbook data

**Example:**

```python
response = client.get_orderbook(token_id="token_yes")

if response.errno == 0:
    book = response.result.data
    print("Bids (buy orders):")
    for bid in book.bids[:5]:  # Top 5
        print(f"  Price: {bid.price}, Size: {bid.size}")

    print("Asks (sell orders):")
    for ask in book.asks[:5]:
        print(f"  Price: {ask.price}, Size: {ask.size}")
```

**Raises:**

* `InvalidParamError` - If token\_id is missing
* `OpenApiError` - If API request fails

***

#### get\_latest\_price()

Get the current/latest price for a token.

**Signature:**

```python
def get_latest_price(token_id: str) -> Any
```

**Parameters:**

| Name       | Type  | Required | Description |
| ---------- | ----- | -------- | ----------- |
| `token_id` | `str` | Yes      | Token ID    |

**Returns:** API response with latest price data

**Example:**

```python
response = client.get_latest_price(token_id="token_yes")

if response.errno == 0:
    price_data = response.result.data
    print(f"Latest price: {price_data.price}")
    print(f"Timestamp: {price_data.timestamp}")
```

***

#### get\_price\_history()

Get historical price data (candlestick/OHLCV) for a token.

**Signature:**

```python
def get_price_history(
    token_id: str,
    interval: str = "1h",
    start_at: Optional[int] = None,
    end_at: Optional[int] = None
) -> Any
```

**Parameters:**

| Name       | Type  | Required | Default | Description                                  |
| ---------- | ----- | -------- | ------- | -------------------------------------------- |
| `token_id` | `str` | Yes      | -       | Token ID                                     |
| `interval` | `str` | No       | `"1h"`  | Time interval: `1m`, `1h`, `1d`, `1w`, `max` |
| `start_at` | `int` | No       | `None`  | Start timestamp (Unix seconds)               |
| `end_at`   | `int` | No       | `None`  | End timestamp (Unix seconds)                 |

**Returns:** API response with price history data

**Example:**

```python
import time

# Get last 24 hours of hourly data
end_time = int(time.time())
start_time = end_time - (24 * 3600)  # 24 hours ago

response = client.get_price_history(
    token_id="token_yes",
    interval="1h",
    start_at=start_time,
    end_at=end_time
)

if response.errno == 0:
    candles = response.result.data
    for candle in candles:
        print(f"Time: {candle.timestamp}, Price: {candle.close}")
```

***

#### get\_fee\_rates()

Get trading fee rates for a token.

**Signature:**

```python
def get_fee_rates(token_id: str) -> Any
```

**Parameters:**

| Name       | Type  | Required | Description |
| ---------- | ----- | -------- | ----------- |
| `token_id` | `str` | Yes      | Token ID    |

**Returns:** API response with fee rate data

**Example:**

```python
response = client.get_fee_rates(token_id="token_yes")

if response.errno == 0:
    fees = response.result.data
    print(f"Maker fee: {fees.maker_fee}")
    print(f"Taker fee: {fees.taker_fee}")
```

***

### Trading Operations

#### place\_order()

Place a market or limit order.

**Signature:**

```python
def place_order(
    data: PlaceOrderDataInput,
    check_approval: bool = False
) -> Any
```

**Parameters:**

| Name             | Type                  | Required | Description                                         |
| ---------------- | --------------------- | -------- | --------------------------------------------------- |
| `data`           | `PlaceOrderDataInput` | Yes      | Order parameters (see below)                        |
| `check_approval` | `bool`                | No       | Whether to check and enable trading approvals first |

**PlaceOrderDataInput fields:**

| Field                     | Type             | Required | Description                                                |
| ------------------------- | ---------------- | -------- | ---------------------------------------------------------- |
| `marketId`                | `int`            | Yes      | Market ID                                                  |
| `tokenId`                 | `str`            | Yes      | Token ID to trade                                          |
| `side`                    | `OrderSide`      | Yes      | `OrderSide.BUY` or `OrderSide.SELL`                        |
| `orderType`               | `int`            | Yes      | `MARKET_ORDER` (1) or `LIMIT_ORDER` (2)                    |
| `price`                   | `str`            | Yes\*    | Price string (required for limit orders, `"0"` for market) |
| `makerAmountInQuoteToken` | `int` or `float` | No\*\*   | Amount in quote token (e.g., 100 for 100 USDT)             |
| `makerAmountInBaseToken`  | `int` or `float` | No\*\*   | Amount in base token (e.g., 50 for 50 YES tokens)          |

\* Price is required for limit orders, set to `"0"` for market orders \*\* Must provide exactly ONE of `makerAmountInQuoteToken` or `makerAmountInBaseToken`

**Returns:** API response with order result

**Examples:**

**Limit Buy Order (using quote token):**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import LIMIT_ORDER

order = PlaceOrderDataInput(
    marketId=123,
    tokenId="token_yes",
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price="0.55",  # Buy at $0.55 or better
    makerAmountInQuoteToken=100  # Spend 100 USDT (int or float)
)

result = client.place_order(order, check_approval=True)
if result.errno == 0:
    print(f"Order placed: {result.result.data.order_id}")
```

**Market Sell Order (using base token):**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order_type import MARKET_ORDER

order = PlaceOrderDataInput(
    marketId=123,
    tokenId="token_yes",
    side=OrderSide.SELL,
    orderType=MARKET_ORDER,
    price="0",  # Market orders don't need price
    makerAmountInBaseToken=50  # Sell 50 YES tokens (int or float)
)

result = client.place_order(order)
```

**Raises:**

* `InvalidParamError` - If parameters are invalid or missing
* `OpenApiError` - If API request fails or chain\_id mismatch

***

#### place\_orders\_batch()

Place multiple orders in a single batch operation.

**Signature:**

```python
def place_orders_batch(
    orders: List[PlaceOrderDataInput],
    check_approval: bool = False
) -> List[Any]
```

**Parameters:**

| Name             | Type                        | Required | Description                                          |
| ---------------- | --------------------------- | -------- | ---------------------------------------------------- |
| `orders`         | `List[PlaceOrderDataInput]` | Yes      | A list containing the order details.                 |
| `check_approval` | `bool`                      | No       | Determines if approvals are verified for all orders. |

**Returns:** List of results with `success`, `result`, and `error` fields for each order

**Example:**

```python
orders = [
    PlaceOrderDataInput(marketId=123, tokenId="token_yes", side=OrderSide.BUY, ...),
    PlaceOrderDataInput(marketId=124, tokenId="token_no", side=OrderSide.SELL, ...),
]

results = client.place_orders_batch(orders, check_approval=True)

for i, result in enumerate(results):
    if result['success']:
        print(f"Order {i}: Success - {result['result']}")
    else:
        print(f"Order {i}: Failed - {result['error']}")
```

***

#### cancel\_order()

Cancel a single order by order ID.

**Signature:**

```python
def cancel_order(order_id: str) -> Any
```

**Parameters:**

| Name       | Type  | Required | Description        |
| ---------- | ----- | -------- | ------------------ |
| `order_id` | `str` | Yes      | Order ID to cancel |

**Returns:** API response for cancellation

**Example:**

```python
result = client.cancel_order(order_id="order_123")

if result.errno == 0:
    print("Order cancelled successfully")
```

***

#### cancel\_orders\_batch()

Cancel multiple orders in a batch.

**Signature:**

```python
def cancel_orders_batch(order_ids: List[str]) -> List[Any]
```

**Parameters:**

| Name        | Type        | Required | Description                 |
| ----------- | ----------- | -------- | --------------------------- |
| `order_ids` | `List[str]` | Yes      | List of order IDs to cancel |

**Returns:** List of cancellation results for each order

**Example:**

```python
order_ids = ["order_123", "order_456", "order_789"]
results = client.cancel_orders_batch(order_ids)

for i, result in enumerate(results):
    if result['success']:
        print(f"Cancelled: {order_ids[i]}")
    else:
        print(f"Failed: {order_ids[i]} - {result['error']}")
```

***

#### cancel\_all\_orders()

Cancel all open orders, optionally filtered by market and/or side.

**Signature:**

```python
def cancel_all_orders(
    market_id: Optional[int] = None,
    side: Optional[OrderSide] = None
) -> Dict[str, Any]
```

**Parameters:**

| Name        | Type        | Required | Description                                  |
| ----------- | ----------- | -------- | -------------------------------------------- |
| `market_id` | `int`       | No       | Filter by market ID (all markets if None)    |
| `side`      | `OrderSide` | No       | Filter by side (BUY/SELL, all sides if None) |

**Returns:** Dictionary with cancellation summary:

```python
{
    'total_orders': int,      # Total orders found matching filter
    'cancelled': int,         # Successfully cancelled count
    'failed': int,            # Failed cancellation count
    'results': List[dict]     # Detailed results for each order
}
```

**Example:**

```python
# Cancel all open orders across all markets
result = client.cancel_all_orders()
print(f"Cancelled {result['cancelled']} out of {result['total_orders']} orders")

# Cancel all BUY orders in market 123
result = client.cancel_all_orders(market_id=123, side=OrderSide.BUY)
print(f"Success: {result['cancelled']}, Failed: {result['failed']}")

# Cancel all orders in market 456 (both sides)
result = client.cancel_all_orders(market_id=456)
```

***

#### get\_my\_orders()

Get user's orders with optional filters.

**Signature:**

```python
def get_my_orders(
    market_id: int = 0,
    status: str = "",
    limit: int = 10,
    page: int = 1
) -> Any
```

**Parameters:**

| Name        | Type  | Required | Default | Description                                            |
| ----------- | ----- | -------- | ------- | ------------------------------------------------------ |
| `market_id` | `int` | No       | `0`     | Filter by market (0 = all markets)                     |
| `status`    | `str` | No       | `""`    | Filter by status (e.g., "open", "filled", "cancelled") |
| `limit`     | `int` | No       | `10`    | Items per page                                         |
| `page`      | `int` | No       | `1`     | Page number                                            |

**Returns:** API response with `result.list` containing orders

**Example:**

```python
# Get all open orders
response = client.get_my_orders(status="open", limit=50)

if response.errno == 0:
    orders = response.result.list
    for order in orders:
        print(f"Order {order.order_id}: {order.side} @ {order.price}")
```

***

#### get\_order\_by\_id()

Get details for a specific order by ID.

**Signature:**

```python
def get_order_by_id(order_id: str) -> Any
```

**Parameters:**

| Name       | Type  | Required | Description |
| ---------- | ----- | -------- | ----------- |
| `order_id` | `str` | Yes      | Order ID    |

**Returns:** API response with order details

**Example:**

```python
response = client.get_order_by_id(order_id="order_123")

if response.errno == 0:
    order = response.result.data
    print(f"Status: {order.status}")
    print(f"Filled: {order.filled_amount}/{order.maker_amount}")
```

***

### User Data Methods

#### get\_my\_balances()

Get user's token balances.

**Signature:**

```python
def get_my_balances() -> Any
```

**Returns:** API response with `result.data.balances` containing list of balance objects

**Example:**

```python
response = client.get_my_balances()

if response.errno == 0:
    balance_data = response.result.data
    balances = balance_data.balances  # List of quote token balances
    for balance in balances:
        print(f"Token: {balance.quote_token}")
        print(f"  Available: {balance.available_balance}")
        print(f"  Frozen: {balance.frozen_balance}")
        print(f"  Total: {balance.total_balance}")
```

***

#### get\_my\_positions()

Get user's open positions across markets.

**Signature:**

```python
def get_my_positions(
    market_id: int = 0,
    page: int = 1,
    limit: int = 10
) -> Any
```

**Parameters:**

| Name        | Type  | Required | Default | Description                |
| ----------- | ----- | -------- | ------- | -------------------------- |
| `market_id` | `int` | No       | `0`     | Filter by market (0 = all) |
| `page`      | `int` | No       | `1`     | Page number                |
| `limit`     | `int` | No       | `10`    | Items per page             |

**Returns:** API response with `result.list` containing positions

**Example:**

```python
response = client.get_my_positions(limit=50)

if response.errno == 0:
    positions = response.result.list
    for pos in positions:
        print(f"Market {pos.market_id}: {pos.market_title}")
        print(f"  Shares: {pos.shares_owned} ({pos.outcome_side_enum})")
        print(f"  Value: {pos.current_value_in_quote_token}")
        print(f"  P&L: {pos.unrealized_pnl} ({pos.unrealized_pnl_percent}%)")
```

***

#### get\_my\_trades()

Get user's trade history.

**Signature:**

```python
def get_my_trades(
    market_id: Optional[int] = None,
    page: int = 1,
    limit: int = 10
) -> Any
```

**Parameters:**

| Name        | Type  | Required | Default | Description      |
| ----------- | ----- | -------- | ------- | ---------------- |
| `market_id` | `int` | No       | `None`  | Filter by market |
| `page`      | `int` | No       | `1`     | Page number      |
| `limit`     | `int` | No       | `10`    | Items per page   |

**Returns:** API response with `result.list` containing trade history

**Example:**

```python
response = client.get_my_trades(market_id=123, limit=20)

if response.errno == 0:
    trades = response.result.list
    for trade in trades:
        print(f"{trade.created_at}: {trade.side} {trade.shares} shares @ {trade.price}")
        print(f"  Amount: {trade.amount}, Fee: {trade.fee}")
        print(f"  Status: {trade.status_enum}")
```

***

### Smart Contract Operations

These methods interact directly with the blockchain and **require gas (BNB)**.

#### enable\_trading()

Enable trading by approving quote tokens for the exchange contract. Must be called once before placing orders or doing split/merge/redeem operations.

**Signature:**

```python
def enable_trading() -> Tuple[Any, Any, Any]
```

**Returns:** Tuple of `(tx_hash, tx_receipt, contract_event)`

**Example:**

```python
tx_hash, receipt, event = client.enable_trading()
print(f"Trading enabled! TX: {tx_hash.hex()}")
```

**Notes:**

* Only needs to be called once (result is cached for `enable_trading_check_interval` seconds)
* Automatically called by `split()`, `merge()`, `redeem()` if `check_approval=True`

***

#### split()

Convert collateral tokens (e.g., USDT) into outcome tokens (e.g., YES + NO).

**Signature:**

```python
def split(
    market_id: int,
    amount: int,
    check_approval: bool = True
) -> Tuple[Any, Any, Any]
```

**Parameters:**

| Name             | Type   | Required | Description                                                             |
| ---------------- | ------ | -------- | ----------------------------------------------------------------------- |
| `market_id`      | `int`  | Yes      | Market ID                                                               |
| `amount`         | `int`  | Yes      | Amount in wei (e.g., 105000000000000000000 for 1 USDT with 18 decimals) |
| `check_approval` | `bool` | No       | Auto-call `enable_trading()` if needed                                  |

**Returns:** Tuple of `(tx_hash, tx_receipt, contract_event)`

**Example:**

```python
# Split 10 USDT (18 decimals) into YES + NO tokens
amount_wei = 10 * 10**18  # 10 USDT

tx_hash, receipt, event = client.split(
    market_id=123,
    amount=amount_wei,
    check_approval=True
)

print(f"Split complete! TX: {tx_hash.hex()}")
print(f"Gas used: {receipt.gasUsed}")
```

**Raises:**

* `InvalidParamError` - If market\_id or amount is invalid
* `OpenApiError` - If market is not in valid state or chain mismatch
* Blockchain errors - If insufficient balance or gas

***

#### merge()

Convert outcome tokens back into collateral tokens.

**Signature:**

```python
def merge(
    market_id: int,
    amount: int,
    check_approval: bool = True
) -> Tuple[Any, Any, Any]
```

**Parameters:**

| Name             | Type   | Required | Description                            |
| ---------------- | ------ | -------- | -------------------------------------- |
| `market_id`      | `int`  | Yes      | Market ID                              |
| `amount`         | `int`  | Yes      | Amount of outcome tokens in wei        |
| `check_approval` | `bool` | No       | Auto-call `enable_trading()` if needed |

**Returns:** Tuple of `(tx_hash, tx_receipt, contract_event)`

**Example:**

```python
# Merge 5 YES + 5 NO tokens back to 5 USDT
amount_wei = 5 * 10**18

tx_hash, receipt, event = client.merge(
    market_id=123,
    amount=amount_wei
)

print(f"Merge complete! TX: {tx_hash.hex()}")
```

***

#### redeem()

Claim winnings after a market is resolved. Redeems winning outcome tokens for collateral.

**Signature:**

```python
def redeem(
    market_id: int,
    check_approval: bool = True
) -> Tuple[Any, Any, Any]
```

**Parameters:**

| Name             | Type   | Required | Description                            |
| ---------------- | ------ | -------- | -------------------------------------- |
| `market_id`      | `int`  | Yes      | Resolved market ID                     |
| `check_approval` | `bool` | No       | Auto-call `enable_trading()` if needed |

**Returns:** Tuple of `(tx_hash, tx_receipt, contract_event)`

**Example:**

```python
# Redeem winnings from resolved market
tx_hash, receipt, event = client.redeem(market_id=123)

print(f"Winnings redeemed! TX: {tx_hash.hex()}")
```

**Raises:**

* `InvalidParamError` - If market\_id is invalid
* `OpenApiError` - If market is not resolved or chain mismatch
* `NoPositionsToRedeem` - If no winning positions to claim

***

### Error Handling

#### Exceptions

The SDK defines these custom exceptions:

```python
from opinion_clob_sdk import InvalidParamError, OpenApiError
from opinion_clob_sdk.chain.exception import (
    BalanceNotEnough,
    NoPositionsToRedeem,
    InsufficientGasBalance
)
```

| Exception                | Description                              |
| ------------------------ | ---------------------------------------- |
| `InvalidParamError`      | Invalid method parameters                |
| `OpenApiError`           | API communication or response errors     |
| `BalanceNotEnough`       | Insufficient token balance for operation |
| `NoPositionsToRedeem`    | No winning positions to redeem           |
| `InsufficientGasBalance` | Not enough BNB for gas fees              |

#### Example Error Handling

```python
try:
    result = client.place_order(order_data)
    if result.errno == 0:
        print("Success!")
    else:
        print(f"API Error: {result.errmsg}")

except InvalidParamError as e:
    print(f"Invalid parameter: {e}")
except OpenApiError as e:
    print(f"API error: {e}")
except BalanceNotEnough as e:
    print(f"Insufficient balance: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```
