# Precision

## Precision and Amount Handling

### Token Decimal Systems

#### Overview

The Opinion CLOB SDK interacts with token standards employing distinct decimal precision schemes. Precision handling accuracy is critical for preventing calculation errors and fund loss.

#### USDT (Collateral Token)

**Decimal Specification:** 18 (ERC-20 standard)

**Conversion Implementation:**

```python
# Human-readable: 100 USDT
usdt_amount = 100

# Contract representation 
amount_micro_usdt = usdt_amount * 10**18  

# Reverse transformation
usdt_amount = amount_micro_usdt / 10**18  # 100.0 USDT
```

**Conversion Table:**

| Human-Readable | Exponential Notation  |
| -------------- | --------------------- |
| 1 USDT         | 1 × 10^18             |
| 10 USDT        | 10 × 10^18            |
| 0.5 USDT       | 0.5 × 10^18           |
| 100.50 USDT    | 100.5 × 10^18         |
| 0.000001 USDT  | 10^-18 (minimum unit) |

#### Outcome Tokens (YES/NO)

**Decimal Specification:** 18 (ERC-1155 standard, Wei base unit)

**Conversion Implementation:**

```python
# Human-readable: 50 YES tokens
token_amount = 50

# Contract representation (Wei)
amount_wei = token_amount * 10**18  # 50_000_000_000_000_000_000 Wei

# Reverse transformation
token_amount = amount_wei / 10**18  # 50.0 tokens
```

**Conversion Table:**

| Human-Readable           | Wei (Contract)                   | Exponential Notation       |
| ------------------------ | -------------------------------- | -------------------------- |
| 1 YES                    | 1\_000\_000\_000\_000\_000\_000  | 1 × 10^18                  |
| 10 YES                   | 10\_000\_000\_000\_000\_000\_000 | 10 × 10^18                 |
| 0.1 YES                  | 100\_000\_000\_000\_000\_000     | 0.1 × 10^18                |
| 0.000000000000000001 YES | 1                                | 10^-18 (Wei, minimum unit) |

### Price Representation

#### Format Specification

Prices encode implied probability as decimal strings, representing the USDT cost per outcome token (normalized to $1.00 payout).

**Type Constraints:**

* Data Type: `str`
* Value Range: `[0.01, 0.99]` (1% to 99% implied probability)
* Precision: Maximum 4 decimal places (0.0001 tick size)

**Price Interpretation Table:**

| Price String | Implied Probability | Cost per Share | Payout (if correct) | Max Profit         |
| ------------ | ------------------- | -------------- | ------------------- | ------------------ |
| `"0.01"`     | 1%                  | $0.01          | $1.00               | $0.99 (9900% ROI)  |
| `"0.50"`     | 50%                 | $0.50          | $1.00               | $0.50 (100% ROI)   |
| `"0.652"`    | 65.2%               | $0.652         | $1.00               | $0.348 (53.3% ROI) |
| `"0.99"`     | 99%                 | $0.99          | $1.00               | $0.01 (1.01% ROI)  |

### Order Amount Specifications

#### Quote Token Amount (makerAmountInQuoteToken)

Specifies the USDT amount to spend (BUY orders) or receive (SELL orders).

**BUY Order Calculation:**

```python
price = "0.60"
maker_amount_quote = 100  # Spend 100 USDT

# Tokens received calculation
price_float = float(price)
tokens_received = maker_amount_quote / price_float  # 166.67 YES tokens
```

**SELL Order Calculation:**

```python
price = "0.75"
maker_amount_quote = 50  # Receive 50 USDT

# Tokens sold calculation
price_float = float(price)
tokens_sold = maker_amount_quote / price_float  # 66.67 YES tokens
```

**Implementation:**

```python
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import LIMIT_ORDER

order = PlaceOrderDataInput(
    marketId=813,
    tokenId='____',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.60',
    makerAmountInQuoteToken=100  # USDT amount (decimal, not Wei)
)
```

**Note:** The SDK handles conversion to Wei internally. Provide amounts in human-readable decimal format.

#### Base Token Amount (makerAmountInBaseToken)

Specifies the exact number of outcome tokens to buy or sell.

**BUY Order Calculation:**

```python
price = "0.60"
maker_amount_base = 200  # Buy 200 YES tokens

# USDT cost calculation
price_float = float(price)
usdt_cost = maker_amount_base * price_float  # 120 USDT
```

**SELL Order Calculation:**

```python
price = "0.75"
maker_amount_base = 100  # Sell 100 YES tokens

# USDT received calculation
price_float = float(price)
usdt_received = maker_amount_base * price_float  # 75 USDT
```

**Implementation:**

```python
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='_____',
    side=OrderSide.SELL,
    orderType=LIMIT_ORDER,
    price='0.75',
    makerAmountInBaseToken=100  # Token amount (decimal, not Wei)
)
```

### Smart Contract Amount Specifications

#### Split Operation

Converts USDT collateral into outcome token pairs (YES + NO).

```python
# Human amounts
usdt_to_split = 100  # 100 USDT

# Convert to Wei (USDT uses 6 decimals)
amount_in_wei = usdt_to_split * 10**18  

# Execute split
tx_hash = client.split(
    market_id=813,
    amount=amount_in_wei
)

# Result:
# - Deduct: 100 USDT (100_000_000_000_000_000_000 Wei)
# - Credit: 100 YES (100_000_000_000_000_000_000 Wei, 18 decimals)
# - Credit: 100 NO  (100_000_000_000_000_000_000 Wei, 18 decimals)
```

**Decimal Conversion:**

```python
def usdt_to_wei(usdt_amount: float) -> int:
    """Convert USDT amount to Wei representation."""
    return int(usdt_amount * 10**18)

def wei_to_usdt(wei_amount: int) -> float:
    """Convert Wei representation to USDT amount."""
    return wei_amount / 10**18

# Usage
wei = usdt_to_wei(100.50)  # 100_500_000_000_000_000_000
usdt = wei_to_usdt(100_500_000_000_000_000_000)  # 100.5
```

#### Merge Operation

Converts outcome token pairs (YES + NO) back into USDT collateral.

```python
# Human amounts
token_pairs_to_merge = 50  # 50 YES + 50 NO pairs

# Convert to Wei (outcome tokens use 18 decimals)
amount_in_wei = token_pairs_to_merge * 10**18  # 50_000_000_000_000_000_000

# Execute merge
tx_hash = client.merge(
    market_id=123,
    amount=amount_in_wei
)

# Result:
# - Deduct: 50 YES (50_000_000_000_000_000_000 Wei)
# - Deduct: 50 NO  (50_000_000_000_000_000_000 Wei)
# - Credit: 50 USDT (50_000_000_000_000_000_000 Wei, 18 decimals)
```

**Decimal Conversion:**

```python
def tokens_to_wei(token_amount: float) -> int:
    """Convert outcome token amount to Wei representation."""
    return int(token_amount * 10**18)

def wei_to_tokens(wei_amount: int) -> float:
    """Convert Wei representation to outcome token amount."""
    return wei_amount / 10**18

# Usage
wei = tokens_to_wei(50.5)  # 50_500_000_000_000_000_000
tokens = wei_to_tokens(50_500_000_000_000_000_000)  # 50.5
```

#### Redeem Operation

Claims winnings from resolved markets.

```python
# Redeem automatically converts all winning tokens to USDT
# No amount parameter required - redeems entire position

tx_hash = client.redeem(market_id=813)

# If market resolved YES and you hold 100 YES tokens:
# - Deduct: 100 YES (100_000_000_000_000_000_000 Wei)
# - Credit: 100 USDT (100_000_000_000_000_000_000 Wei)
```

### Floating Point Precision Issues

#### Problem: IEEE 754 Rounding Errors

Python's `float` type implements IEEE 754 binary floating-point arithmetic, which cannot precisely represent all decimal fractions.

**Problematic Pattern:**

```python
# ❌ Incorrect: Binary floating-point accumulates rounding errors
price = 0.65
amount = 100
result = price * amount  # May yield 64.99999999999999 (15-17 sig figs)

# Verification
result == 65.0  # False on some systems
```

**Correct Pattern:**

```python
# ✅ Correct: Decimal type provides exact decimal arithmetic
from decimal import Decimal, ROUND_DOWN

price = Decimal('0.65')
amount = Decimal('100')
result = price * amount  # Exactly Decimal('65.00')

# Verification
result == Decimal('65.00')  # True (exact equality)
```

#### Best Practices for Precision

```python
from decimal import Decimal, ROUND_DOWN

def calculate_tokens_from_usdt(usdt_amount: str, price: str) -> str:
    """
    Calculate token amount from USDT budget and price.

    Args:
        usdt_amount: USDT amount as string (e.g., "100.50")
        price: Price as string (e.g., "0.65")

    Returns:
        Token amount as string with appropriate precision
    """
    usdt_decimal = Decimal(usdt_amount)
    price_decimal = Decimal(price)
    tokens = usdt_decimal / price_decimal
    return str(tokens.quantize(Decimal('0.01'), rounding=ROUND_DOWN))

# Usage
tokens = calculate_tokens_from_usdt("100", "0.65")  # "153.84"
```

### Amount Formatting

#### Display Formatting

```python
def format_usdt(amount: float) -> str:
    """Format USDT amount for display."""
    return f"${amount:,.2f}"

def format_tokens(amount: float) -> str:
    """Format token amount for display."""
    return f"{amount:,.4f}"

def format_price(price: str) -> str:
    """Format price for display."""
    return f"{float(price):.4f} USDT"

# Usage
print(format_usdt(1234.56))      # "$1,234.56"
print(format_tokens(1234.5678))  # "1,234.5678"
print(format_price("0.6525"))    # "0.6525 USDT"
```

### Common Precision Errors

#### Error 1: Incorrect Decimal Places

```python
# ❌ Incorrect: Using 6 decimals for USDT
usdt_wei = 100 * 10*6  # Wrong!
client.split(market_id=813, amount=usdt_wei)  # Will behave unexpectedly

# ✅ Correct: Using 18 decimals for USDT
usdt_wei = 100 * 10**18  # Correct
client.split(market_id=813, amount=usdt_wei)
```

#### Error 2: Float to Wei Conversion

```python
# ❌ Problematic: Direct float multiplication
amount = 100.5
wei = int(amount * 10**18)  # May have rounding errors

# ✅ Better: Use Decimal for precision
from decimal import Decimal
amount = Decimal('100.5')
wei = int(amount * 10**18)
```

#### Error 3: Price Outside Valid Range

```python
# ❌ Invalid prices
order = PlaceOrderDataInput(price='0.00', ...)  # Below minimum
order = PlaceOrderDataInput(price='1.00', ...)  # Above maximum
order = PlaceOrderDataInput(price='1.50', ...)  # Far out of range

# ✅ Valid prices
order = PlaceOrderDataInput(price='0.01', ...)  # Minimum
order = PlaceOrderDataInput(price='0.65', ...)  # Typical
order = PlaceOrderDataInput(price='0.99', ...)  # Maximum
```

### Position Size Calculations

#### Total Position Value

```python
def calculate_position_value(token_amount: float, current_price: str) -> float:
    """
    Calculate current market value of position.

    Args:
        token_amount: Number of outcome tokens held
        current_price: Current market price

    Returns:
        Position value in USDT
    """
    from decimal import Decimal
    tokens = Decimal(str(token_amount))
    price = Decimal(current_price)
    return float(tokens * price)

# Usage
position_value = calculate_position_value(100, "0.75")  # 75.0 USDT
```

#### Profit and Loss Calculation

```python
def calculate_pnl(
    buy_amount: float,
    buy_price: str,
    sell_amount: float,
    sell_price: str
) -> dict:
    """
    Calculate profit/loss for a completed trade.

    Args:
        buy_amount: Tokens purchased
        buy_price: Purchase price
        sell_amount: Tokens sold
        sell_price: Sale price

    Returns:
        Dictionary with PnL metrics
    """
    from decimal import Decimal

    buy_cost = Decimal(str(buy_amount)) * Decimal(buy_price)
    sell_proceeds = Decimal(str(sell_amount)) * Decimal(sell_price)
    pnl = sell_proceeds - buy_cost
    pnl_percent = (pnl / buy_cost * 100) if buy_cost > 0 else Decimal('0')

    return {
        'buy_cost': float(buy_cost),
        'sell_proceeds': float(sell_proceeds),
        'pnl': float(pnl),
        'pnl_percent': float(pnl_percent)
    }

# Usage
pnl = calculate_pnl(100, "0.60", 100, "0.75")
# {'buy_cost': 60.0, 'sell_proceeds': 75.0, 'pnl': 15.0, 'pnl_percent': 25.0}
```

#### Break-even Analysis

```python
def calculate_breakeven_price(
    buy_amount: float,
    buy_price: str,
    fee_rate: float = 0.02(taker 2%, maker 0%)
) -> str:
    """
    Calculate price needed to break even after fees.

    Args:
        buy_amount: Tokens purchased
        buy_price: Purchase price
        fee_rate: Trading fee rate (default 2%)

    Returns:
        Break-even price as string
    """
    from decimal import Decimal, ROUND_UP

    cost = Decimal(str(buy_amount)) * Decimal(buy_price)
    fees = cost * Decimal(str(fee_rate))
    total_cost = cost + fees
    breakeven = total_cost / Decimal(str(buy_amount))

    return str(breakeven.quantize(Decimal('0.0001'), rounding=ROUND_UP))

# Usage
breakeven = calculate_breakeven_price(100, "0.60", 0.02)  # "0.6120"
```

### API Response Amount Parsing

#### Parse Balance Response

```python
def parse_balance_response(balance_data: dict) -> dict:
    """
    Parse balance API response to human-readable amounts.

    Args:
        balance_data: Raw balance data from API

    Returns:
        Parsed balance information
    """
    balances = {}
    for item in balance_data.get('result', []):
        token_name = item['quoteTokenName']
        amount_str = item['available']

        # Determine decimal places based on token type
        if token_name == 'USDT':
            amount = float(amount_str) / 10**18

        balances[token_name] = amount

    return balances

# Usage
response = client.get_my_balances()
balances = parse_balance_response(response)
print(f"USDT: {balances.get('USDT', 0):.2f}")
```

### Next Steps

* **API Reference - Models** - Data type specifications
