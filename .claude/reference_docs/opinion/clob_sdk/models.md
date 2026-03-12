# Models

## Data Models

Reference for all data models and enums used in the Opinion CLOB SDK.

### Enums

#### TopicType

Defines the type of prediction market. **Topic** is conceptional equivalent to **Market.**

**Module:** `opinion_clob_sdk.model`

```python
from opinion_clob_sdk.model import TopicType

class TopicType(Enum):
    BINARY = 0        # Two-outcome markets (YES/NO)
    CATEGORICAL = 1   # Multi-outcome markets (Option A/B/C/...)
```

**Usage:**

```python
# Filter for binary markets only
markets = client.get_markets(topic_type=TopicType.BINARY)

# Filter for categorical markets
markets = client.get_markets(topic_type=TopicType.CATEGORICAL)
```

***

#### TopicStatus

Market lifecycle status codes.

**Module:** `opinion_clob_sdk.model`

```python
from opinion_clob_sdk.model import TopicStatus

class TopicStatus(Enum):
    CREATED = 1    # Market created but not yet active
    ACTIVATED = 2  # Market is live and accepting trades
    RESOLVING = 3  # Market ended, awaiting resolution
    RESOLVED = 4   # Market resolved with outcome
```

**Usage:**

```python
market = client.get_market(123)
status = market.result.data.status

if status == TopicStatus.ACTIVATED.value:
    print("Market is live for trading")
elif status == TopicStatus.RESOLVED.value:
    print("Market resolved, can redeem winnings")
```

***

#### TopicStatusFilter

Filter values for querying markets by status.

**Module:** `opinion_clob_sdk.model`

```python
from opinion_clob_sdk.model import TopicStatusFilter

class TopicStatusFilter(Enum):
    ALL = None           # All markets regardless of status
    ACTIVATED = "activated"  # Only active markets
    RESOLVED = "resolved"    # Only resolved markets
```

**Usage:**

```python
# Get only active markets
markets = client.get_markets(status=TopicStatusFilter.ACTIVATED)

# Get only resolved markets
markets = client.get_markets(status=TopicStatusFilter.RESOLVED)

# Get all markets
markets = client.get_markets(status=TopicStatusFilter.ALL)
```

***

#### OrderSide

Trade direction for orders.

**Module:** `opinion_clob_sdk.chain.py_order_utils.model.sides`

```python
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide

class OrderSide(IntEnum):
    BUY = 0   # Buy outcome tokens
    SELL = 1  # Sell outcome tokens
```

**Usage:**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput

# Place buy order
buy_order = PlaceOrderDataInput(
    marketId=123,
    tokenId="token_yes",
    side=OrderSide.BUY,  # Buy YES tokens
    # ...
)

# Place sell order
sell_order = PlaceOrderDataInput(
    marketId=123,
    tokenId="token_yes",
    side=OrderSide.SELL,  # Sell YES tokens
    # ...
)
```

***

#### Order Types

Constants for order type selection.

**Module:** `opinion_clob_sdk.chain.py_order_utils.model.order_type`

```python
from opinion_clob_sdk.chain.py_order_utils.model.order_type import (
    MARKET_ORDER,
    LIMIT_ORDER
)

MARKET_ORDER = 1  # Execute immediately at best available price
LIMIT_ORDER = 2   # Execute at specified price or better
```

**Usage:**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order_type import MARKET_ORDER, LIMIT_ORDER

# Market order - executes immediately
market_order = PlaceOrderDataInput(
    orderType=MARKET_ORDER,
    price="0",  # Price ignored for market orders
    # ...
)

# Limit order - waits for specified price
limit_order = PlaceOrderDataInput(
    orderType=LIMIT_ORDER,
    price="0.55",  # Execute at $0.55 or better
    # ...
)
```

***

### Data Classes

#### PlaceOrderDataInput

Input data for placing an order.

**Module:** `opinion_clob_sdk.chain.py_order_utils.model.order`

```python
@dataclass
class PlaceOrderDataInput:
    marketId: int
    tokenId: str
    side: int  # OrderSide.BUY or OrderSide.SELL
    orderType: int  # MARKET_ORDER or LIMIT_ORDER
    price: str
    makerAmountInQuoteToken: str = None  # Amount in USDT (optional)
    makerAmountInBaseToken: str = None   # Amount in YES/NO tokens (optional)
```

**Fields:**

| Field                     | Type  | Required | Description                                           |
| ------------------------- | ----- | -------- | ----------------------------------------------------- |
| `marketId`                | `int` | Yes      | Market ID to trade on                                 |
| `tokenId`                 | `str` | Yes      | Token ID (e.g., "token\_yes")                         |
| `side`                    | `int` | Yes      | `OrderSide.BUY` (0) or `OrderSide.SELL` (1)           |
| `orderType`               | `int` | Yes      | `MARKET_ORDER` (1) or `LIMIT_ORDER` (2)               |
| `price`                   | `str` | Yes      | Price as string (e.g., "0.55"), "0" for market orders |
| `makerAmountInQuoteToken` | `str` | No\*     | Amount in quote token (e.g., "100" for 100 USDT)      |
| `makerAmountInBaseToken`  | `str` | No\*     | Amount in base token (e.g., "50" for 50 YES tokens)   |

\* Must provide exactly ONE of `makerAmountInQuoteToken` or `makerAmountInBaseToken`

**Amount Selection Rules:**

**For BUY orders:**

* ✅ `makerAmountInQuoteToken` - Common (specify how much USDT to spend)
* ✅ `makerAmountInBaseToken` - Specify how many tokens to buy
* ❌ Both - Invalid

**For SELL orders:**

* ✅ `makerAmountInBaseToken` - Common (specify how many tokens to sell)
* ✅ `makerAmountInQuoteToken` - Specify how much USDT to receive
* ❌ Both - Invalid

**Examples:**

**Buy 100 USDT worth at $0.55:**

```python
order = PlaceOrderDataInput(
    marketId=123,
    tokenId="token_yes",
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price="0.55",
    makerAmountInQuoteToken="100"  # Spend 100 USDT
)
```

**Sell 50 YES tokens at market price:**

```python
order = PlaceOrderDataInput(
    marketId=123,
    tokenId="token_yes",
    side=OrderSide.SELL,
    orderType=MARKET_ORDER,
    price="0",
    makerAmountInBaseToken="50"  # Sell 50 tokens
)
```

***

#### OrderData

Internal order data structure (used by OrderBuilder).

**Module:** `opinion_clob_sdk.chain.py_order_utils.model.order`

```python
@dataclass
class OrderData:
    maker: str              # Maker address (multi-sig wallet)
    taker: str              # Taker address (ZERO_ADDRESS for public orders)
    tokenId: str            # Token ID
    makerAmount: str        # Maker amount in wei
    takerAmount: str        # Taker amount in wei
    side: int               # OrderSide
    feeRateBps: str         # Fee rate in basis points
    nonce: str              # Nonce (default "0")
    signer: str             # Signer address
    expiration: str         # Expiration timestamp (default "0" = no expiration)
    signatureType: int      # Signature type (POLY_GNOSIS_SAFE)
```

**Note:** This is an internal structure. Users should use `PlaceOrderDataInput` instead.

***

#### OrderDataInput

Simplified order input (internal use).

**Module:** `opinion_clob_sdk.chain.py_order_utils.model.order`

```python
@dataclass
class OrderDataInput:
    marketId: int
    tokenId: str
    makerAmount: str  # Already calculated amount
    price: str
    side: int
    orderType: int
```

**Note:** This is used internally by `_place_order()`. Users should use `PlaceOrderDataInput`.

***

### Response Models

#### API Response Structure

All API methods return responses with this standard structure:

```python
class APIResponse:
    errno: int        # Error code (0 = success)
    errmsg: str       # Error message
    result: Result    # Result data
```

#### Result Types

**For single objects:**

```python
class Result:
    data: Any  # Single object (market, order, etc.)
```

**For lists/arrays:**

```python
class Result:
    list: List[Any]  # Array of objects
    total: int       # Total count (for pagination)
```

**Example Usage:**

```python
# Single object response
market_response = client.get_market(123)
if market_response.errno == 0:
    market = market_response.result.data  # Access via .data

# List response
markets_response = client.get_markets()
if markets_response.errno == 0:
    markets = markets_response.result.list  # Access via .list
    total = markets_response.result.total
```

***

### Market Data Models

#### Market Object

Returned by `get_market()` and `get_markets()`.

**Key Fields:**

| Field           | Type  | Description                           |
| --------------- | ----- | ------------------------------------- |
| `marketId`      | `int` | Market ID                             |
| `marketTitle`   | `str` | Market question/title                 |
| `status`        | `int` | Market status (see TopicStatus)       |
| `marketType`    | `int` | Market type (0=binary, 1=categorical) |
| `conditionId`   | `str` | Blockchain condition ID (hex string)  |
| `quoteToken`    | `str` | Quote token address (e.g., USDT)      |
| `chainId`       | `str` | Blockchain chain ID                   |
| `volume`        | `str` | Trading volume                        |
| `yesTokenId`    | `str` | Token ID of Yes side                  |
| `noTokenId`     | `str` | Token ID of No side                   |
| `resultTokenId` | `str` | Token ID of Winning side              |
| `yesLabel`      | `str` | Token Label of Yes side               |
| `noLabel`       | `str` | Token Label of No side                |
| `rules`         | `str` | Market Resolution Criteria            |
| `cutoffAt`      | `int` | The latest date to resolve the market |
| `resolvedAt`    | `int` | The date that market resolved         |

**Example:**

```python
market = client.get_market(123).result.data

print(f"ID: {market.topic_id}")
print(f"Title: {market.topic_title}")
print(f"Status: {market.status}")  # 2 = ACTIVATED
print(f"Type: {market.topic_type}")  # 0 = BINARY
print(f"Condition: {market.condition_id}")
```

***

#### Quote Token Object

Returned by `get_quote_tokens()`.

**Key Fields:**

| Field                | Type  | Description                        |
| -------------------- | ----- | ---------------------------------- |
| `quoteTokenAddress`  | `str` | Token contract address             |
| `decimal`            | `int` | Token decimals (e.g., 18 for USDT) |
| `ctfExchangeAddress` | `str` | CTF exchange contract address      |
| `chainId`            | `int` | Blockchain chain ID                |
| `quoteTokenName`     | `str` | Token name (e.g., "USDT")          |
| `symbol`             | `str` | Token symbol                       |

**Example:**

```python
tokens = client.get_quote_tokens().result.list

for token in tokens:
    print(f"{token.symbol}: {token.quote_token_address}")
    print(f"  Decimals: {token.decimal}")
    print(f"  Exchange: {token.ctf_exchange_address}")
```

***

#### Orderbook Object

Returned by `get_orderbook()`.

**Structure:**

```python
{
    "bids": [  # Buy orders
        {"price": "0.55", "amount": "100", ...},
        {"price": "0.54", "amount": "200", ...},
    ],
    "asks": [  # Sell orders
        {"price": "0.56", "amount": "150", ...},
        {"price": "0.57", "amount": "250", ...},
    ]
}
```

**Example:**

```python
book = client.get_orderbook("token_yes").result.data

# Best bid (highest buy price)
best_bid = book.bids[0] if book.bids else None
print(f"Best bid: ${best_bid['price']} x {best_bid['amount']}")

# Best ask (lowest sell price)
best_ask = book.asks[0] if book.asks else None
print(f"Best ask: ${best_ask['price']} x {best_ask['amount']}")

# Spread
if best_bid and best_ask:
    spread = float(best_ask['price']) - float(best_bid['price'])
    print(f"Spread: ${spread:.4f}")
```

***

### Constants

#### Signature Types

**Module:** `opinion_clob_sdk.chain.py_order_utils.model.signatures`

```python
EOA = 0               # Externally Owned Account (regular wallet)
POLY_PROXY = 1        # Polymarket proxy
POLY_GNOSIS_SAFE = 2  # Gnosis Safe (used by Opinion SDK)
```

**Usage:** Orders are signed with `POLY_GNOSIS_SAFE` signature type by default.

***

#### Address Constants

**Module:** `opinion_clob_sdk.chain.py_order_utils.constants`

```python
ZERO_ADDRESS = "0x0000000000000000000000000000000000000000"
ZX = "0x"  # Hex prefix
```

**Usage:**

* `ZERO_ADDRESS` is used for `taker` field in public orders (anyone can fill)

***

#### Chain IDs

**Module:** `opinion_clob_sdk.sdk`

```python
CHAIN_ID_BNBCHAIN_MAINNET = 56
SUPPORTED_CHAIN_IDS = [56]  # BNB Chain mainnet
```

**Usage:**

```python
# Mainnet
client = Client(chain_id=56, ...)
```

***

#### Decimals

**Module:** `opinion_clob_sdk.sdk`

```python
MAX_DECIMALS = 18  # Maximum token decimals (ERC20 standard)
```

**Common Decimals:**

* USDT: 18 decimals
* BNB: 18 decimals
* Outcome tokens: Usually match quote token decimals

***

### Helper Functions

#### safe\_amount\_to\_wei()

Convert human-readable amount to wei units.

**Module:** `opinion_clob_sdk.sdk`

**Signature:**

```python
def safe_amount_to_wei(amount: float, decimals: int) -> int
```

**Parameters:**

* `amount` - Human-readable amount (e.g., `1.5`)
* `decimals` - Token decimals (e.g., `18` for USDT)

**Returns:** Integer amount in wei units

**Example:**

```python
from opinion_clob_sdk.sdk import safe_amount_to_wei

# Convert 10.5 USDT to wei (18 decimals)
amount_wei = safe_amount_to_wei(10.5, 18)
print(amount_wei)  # 105000000000000000000

# Convert 1 BNB to wei (18 decimals)
amount_wei = safe_amount_to_wei(1.0, 18)
print(amount_wei)  # 100000000000000000000
```

***

#### calculate\_order\_amounts()

Calculate maker and taker amounts for limit orders.

**Module:** `opinion_clob_sdk.chain.py_order_utils.utils`

**Signature:**

```python
def calculate_order_amounts(
    price: float,
    maker_amount: int,
    side: int,
    decimals: int
) -> Tuple[int, int]
```

**Parameters:**

* `price` - Order price (e.g., `0.55`)
* `maker_amount` - Maker amount in wei
* `side` - `OrderSide.BUY` or `OrderSide.SELL`
* `decimals` - Token decimals

**Returns:** Tuple of `(recalculated_maker_amount, taker_amount)`

**Example:**

```python
from opinion_clob_sdk.chain.py_order_utils.utils import calculate_order_amounts
from opinion_clob_sdk.chain.py_order_utils.model.sides import BUY

maker_amount = 100000000000000000000  # 100 USDT (18 decimals)
price = 0.55
side = BUY
decimals = 18

maker, taker = calculate_order_amounts(price, maker_amount, side, decimals)
print(f"Maker: {maker}, Taker: {taker}")
```

***

### Next Steps

* [**Methods**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/api-references/methods): Full API Reference
