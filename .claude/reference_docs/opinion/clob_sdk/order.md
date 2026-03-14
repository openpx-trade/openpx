# Order

### Overview

The Opinion CLOB implements two order execution types (Market, Limit) and two position directions (Buy, Sell). These primitives enable participation in binary prediction markets through standardized order mechanics.

### Order Sides

#### BUY (Long Position)

**Definition:** Acquisition of outcome tokens representing a prediction that the specified event will occur.

**Example - Binary Market:** "Will BTC reach $100k in 2025?"

```python
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide

# Buy YES tokens (betting it will happen)
side = OrderSide.BUY
```

**Payoff Structure:**

| Position             | Purchase Cost | Resolution | Settlement | Net P\&L         |
| -------------------- | ------------- | ---------- | ---------- | ---------------- |
| Long 100 YES @ $0.60 | $60           | YES        | $100       | +$40 (66.7% ROI) |
| Long 100 YES @ $0.60 | $60           | NO         | $0         | -$60 (100% loss) |

**Risk Parameters:**

* **Maximum Loss:** Premium paid (position cost)
* **Maximum Gain:** $1.00 per share minus premium
* **Breakeven:** Event resolution matches position direction

#### SELL (Short Position or Position Exit)

**Definition:** Transfer of outcome tokens, either closing an existing long position or establishing a synthetic short position.

**Two Use Cases:**

**Use Case 1: Close Position (Take Profit/Loss)**

```python
# You previously bought 100 YES @ $0.50
# Now YES price is $0.75
# Sell to lock in profit

from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide

side = OrderSide.SELL  # Sell your YES tokens
```

**P\&L Calculation:**

* Entry: 100 YES @ $0.50 = $50 cost basis
* Exit: 100 YES @ $0.75 = $75 proceeds
* **Realized P\&L: +$25 (50% return)**

**Use Case 2: Synthetic Short (Advanced)**

**Strategy:** Split collateral into outcome token pairs, sell overpriced outcome, retain opposite side.

```python
# Thesis: YES tokens overpriced at $0.80 (implied 80% probability)
# Strategy: Create position exposed to NO outcome

# Step 1: Convert 100 USDT → 100 YES + 100 NO (via splitPosition)
client.split(market_id=123, amount=100_000000)  # 100 USDT

# Step 2: Sell YES tokens
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import LIMIT_ORDER

order = PlaceOrderDataInput(
    marketId=123,
    tokenId='token_yes',
    side=OrderSide.SELL,
    orderType=LIMIT_ORDER,
    price='0.80',
    makerAmountInBaseToken=100  # Sell 100 YES
)
client.place_order(order)

# Position Analysis:
# - Received: $80 USDT (from YES sale)
# - Holdings: 100 NO tokens
# - Net cost: $100 - $80 = $20
#
# Payoff scenarios:
# - If NO resolves: 100 NO → $100 USDT, P&L = $100 - $20 = +$80 (400% ROI)
# - If YES resolves: 100 NO → $0, P&L = $0 - $20 = -$20 (100% loss)
```

### Order Types

#### Market Orders

**Definition:** Orders executing immediately at the best available counterparty price, prioritizing fill certainty over price control.

**Execution Characteristics:**

* Fill guarantee (subject to liquidity availability)
* Immediate execution (latency: 200-500ms)
* Price discovery via orderbook matching
* Slippage exposure in thin markets

**Use Cases:**

* Urgent position entry/exit requirements
* Markets with deep liquidity (tight spread)
* Price movement urgency exceeds execution cost sensitivity
* Closing positions under time constraints

**Syntax:**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import MARKET_ORDER

# Market order: buy with specified USDT allocation
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.BUY,
    orderType=MARKET_ORDER,
    price='0',  # Ignored for market orders (accepts market price)
    makerAmountInQuoteToken=50  # Allocate 50 USDT for purchase
)

result = client.place_order(order)
```

**Order Matching Mechanism:**

1. Order transmitted to CLOB matching engine
2. Matching engine iterates best available limit orders
3. Fills sequentially until USDT allocation exhausted or orderbook cleared
4. Outcome tokens credited to multi\_sig\_addr
5. Execution report returned (average fill price, total quantity)

**Slippage Example:**

```
Orderbook:
  Sell: 10 YES @ $0.60
  Sell: 20 YES @ $0.61
  Sell: 50 YES @ $0.62

You place: Market BUY for $50 USDT

Execution:
  - Buy 10 YES @ $0.60 = $6.00
  - Buy 20 YES @ $0.61 = $12.20
  - Buy 51.6 YES @ $0.62 = $31.80

Total: 81.6 YES for $50.00
Average price: $0.613 per YES
```

#### Limit Orders

**Definition:** Orders that only execute at your specified price or better.

**Characteristics:**

* ✅ **Price control** (you set maximum/minimum)
* ✅ **No slippage** (always your price or better)
* ❌ **May not fill** (if price never reached)
* ❌ **Delayed execution** (passive waiting)

**When to Use:**

* You want a specific price
* No urgency to execute
* Market making strategies
* Large orders (avoid slippage)

**Syntax:**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import LIMIT_ORDER

# Limit buy - only buy if price drops to $0.55 or lower
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.55',
    makerAmountInQuoteToken=100  # Willing to spend $100 USDT
)

result = client.place_order(order)
```

**How Limit Orders Execute:**

1. SDK sends order to CLOB
2. CLOB adds order to orderbook
3. Order waits for counterparty
4. Fills when matching order arrives (or immediate if crosses spread)
5. You receive tokens (partial fills possible)

**Order Matching Example:**

```
Orderbook before your order:
  Buy:  20 YES @ $0.58
  Buy:  30 YES @ $0.57
  Sell: 40 YES @ $0.60
  Sell: 50 YES @ $0.61

You place: Limit BUY 100 YES @ $0.59

Orderbook after:
  Buy:  100 YES @ $0.59  ← Your order (waiting)
  Buy:  20 YES @ $0.58
  Buy:  30 YES @ $0.57
  Sell: 40 YES @ $0.60
  Sell: 50 YES @ $0.61

Later, someone places: Limit SELL 60 YES @ $0.59

Result: You buy 60 YES @ $0.59, your order now shows 40 YES remaining
```

### Price Mechanics

#### Price Range

**Valid Prices:** `0.01` to `0.99`

**Interpretation:**

* `0.01` = 1% probability = 1¢ per $1 share
* `0.50` = 50% probability = 50¢ per $1 share
* `0.99` = 99% probability = 99¢ per $1 share

**Invalid Prices:**

```python
price='0.00'   # ❌ Too low
price='1.00'   # ❌ Too high
price='1.05'   # ❌ Greater than 1.00
```

#### Price Precision

Prices are strings with up to 4 decimal places:

```python
price='0.5'      # Valid: 50%
price='0.511'   # Valid: 51.10%
price='0.55555'  # Invalid: too many decimals
```

#### Bid-Ask Spread

The difference between best buy and sell prices.

```
Orderbook:
  Best Buy:  $0.58  ← Highest bid
  Best Sell: $0.62  ← Lowest ask

Spread = $0.62 - $0.58 = $0.04 (4¢)
```

**Spread Implications:**

| Spread | Market Condition | Strategy                 |
| ------ | ---------------- | ------------------------ |
| $0.01  | Tight (liquid)   | Market orders OK         |
| $0.05  | Moderate         | Limit orders recommended |
| $0.10+ | Wide (illiquid)  | Limit orders essential   |

### Amount Specifications

#### Quote Token Amount (USDT)

Specify how much USDT to spend (BUY) or receive (SELL).

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput

# Buy YES tokens by spending $50 USDT
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.60',
    makerAmountInQuoteToken=50  # $50 USDT
)

# Calculation:
# Tokens received = $50 / $0.60 = 83.33 YES tokens
```

**When to Use:**

* You have a fixed budget (e.g., "spend $100")
* Dollar-cost averaging
* Portfolio allocation (e.g., "allocate 10% of portfolio")

#### Base Token Amount (Outcome Tokens)

Specify exact number of outcome tokens to buy/sell.

```python
# Sell exactly 100 YES tokens
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.SELL,
    orderType=LIMIT_ORDER,
    price='0.75',
    makerAmountInBaseToken=100  # 100 YES tokens
)

# Calculation:
# USDT received = 100 × $0.75 = $75 USDT
```

**When to Use:**

* Closing a specific position (e.g., "sell all my 100 YES tokens")
* Rebalancing to exact token counts
* Arbitrage strategies

#### Conversion Between Amounts

```python
# Given: price and one amount type, calculate the other

price = 0.60
quote_amount = 50  # USDT

# Calculate base tokens
base_tokens = quote_amount / price  # 83.33 YES

# Reverse calculation
quote_amount = base_tokens * price  # $50 USDT
```

### Order Examples

#### Example 1: Simple Market Buy

```python
from opinion_clob_sdk import Client
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import MARKET_ORDER

client = Client(...)

# "Buy YES tokens with $100 USDT immediately"
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.BUY,
    orderType=MARKET_ORDER,
    price='0',  # Ignored for market orders
    makerAmountInQuoteToken=100
)

result = client.place_order(order)
```

#### Example 2: Limit Buy at Specific Price

```python
# "Buy 50 YES tokens, but only if price drops to $0.45 or lower"
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.45',
    makerAmountInBaseToken=50
)

result = client.place_order(order)
print(f"Order placed, waiting for $0.45 or better")
```

#### Example 3: Take Profit Sell

```python
# You own 200 YES, current price is $0.80, you want to sell
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.SELL,
    orderType=MARKET_ORDER,
    price='0',
    makerAmountInBaseToken=200  # Sell all 200 YES
)

result = client.place_order(order)
# You receive ~$160 USDT (200 × $0.80)
```

#### Example 4: Limit Sell (Ask)

```python
# "Sell 100 YES tokens, but only at $0.85 or higher"
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='84286908393008806294032747949016601113812276485362312899677525031544985576186',
    side=OrderSide.SELL,
    orderType=LIMIT_ORDER,
    price='0.85',
    makerAmountInBaseToken=100
)

result = client.place_order(order)
print(f"Order on book at $0.85")
```

### Order Lifecycle

#### 1. Order Creation

```python
result = client.place_order(order)
```

#### 2. Order States

| State         | Description                                            | Next Actions       |
| ------------- | ------------------------------------------------------ | ------------------ |
| **Pending**   | Waiting in orderbook                                   | Cancel or wait     |
| **Filled**    | Fully executed                                         | View trade history |
| **Cancelled** | Manually cancelled / Cancelled by system default rules | None               |
| **Expired**   | Time limit reached                                     | Place new order    |

#### 3. Checking Order Status

```python
# Get all your orders for a market
orders = client.get_my_orders(market_id=813, limit=50)

for order in orders['result']['data']:
    print(f"Order {order['orderId']}: {order['status']}")
```

#### 4. Cancelling Orders

```python
# Cancel single order
client.cancel_order(orderId='________')

# Cancel all orders for a market
cancelled = client.cancel_all_orders(market_id=813)
print(f"Cancelled {len(cancelled['result'])} orders")
```

### Best Practices

#### 1. Check Orderbook Before Trading

```python
# Always check current prices before placing orders
orderbook = client.get_orderbook(token_id='84286908393008806294032747949016601113812276485362312899677525031544985576186')

best_bid = orderbook['result']['bids'][0]['price'] if orderbook['result']['bids'] else None
best_ask = orderbook['result']['asks'][0]['price'] if orderbook['result']['asks'] else None

print(f"Best bid: {best_bid}, Best ask: {best_ask}")

# Place limit order between bid and ask for better fill chances
if best_bid and best_ask:
    mid_price = (float(best_bid) + float(best_ask)) / 2
    # Place buy slightly above bid, sell slightly below ask
```

#### 2. Use Limit Orders for Large Sizes

```python
# ❌ Bad: Large market order causes slippage
order = PlaceOrderDataInput(
    side=OrderSide.BUY,
    orderType=MARKET_ORDER,
    makerAmountInQuoteToken=10000  # $10,000 - will move market!
)

# ✅ Good: Break into smaller limit orders
for i in range(10):
    order = PlaceOrderDataInput(
        side=OrderSide.BUY,
        orderType=LIMIT_ORDER,
        price=f'{0.60 + i * 0.001}',  # Incrementing prices
        makerAmountInQuoteToken=1000  # $1,000 each
    )
    client.place_order(order)
```

#### 3. Price Validation

```python
def validate_price(price: str) -> bool:
    try:
        p = float(price)
        return 0.01 <= p <= 0.99
    except ValueError:
        return False

price = '0.75'
if validate_price(price):
    order = PlaceOrderDataInput(price=price, ...)
else:
    raise ValueError("Invalid price")
```

### Common Mistakes

#### Mistake 1: Wrong Amount Type

```python
# ❌ Using quote amount for SELL orders often confusing
order = PlaceOrderDataInput(
    side=OrderSide.SELL,
    makerAmountInQuoteToken=50  # "Sell $50 worth" - hard to calculate
)

# ✅ Better: Specify exact tokens to sell
order = PlaceOrderDataInput(
    side=OrderSide.SELL,
    makerAmountInBaseToken=100  # "Sell 100 tokens" - clear
)
```

#### Mistake 2: Forgetting Price for Limit Orders

```python
# ❌ Missing price
order = PlaceOrderDataInput(
    orderType=LIMIT_ORDER,
    # price missing!
    makerAmountInQuoteToken=50
)

# ✅ Always specify price for limit orders
order = PlaceOrderDataInput(
    orderType=LIMIT_ORDER,
    price='0.65',
    makerAmountInQuoteToken=50
)
```

### Next Steps

* **Gas Operations** - Understanding when you need BNB
* **API Reference - Trading** - Full method documentation
