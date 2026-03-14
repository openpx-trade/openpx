# Gas Operations

## Gas vs Gas-Free Operations

### Overview

The Opinion CLOB SDK implements a hybrid execution model: off-chain order matching via CLOB infrastructure eliminates gas costs for order operations, while direct smart contract invocations require BNB native token for transaction fees.

### Gas-Free Operations

#### Off-Chain Order Book

The Central Limit Order Book functions as an off-chain matching engine. Orders are authenticated via EIP712 cryptographic signatures and submitted to the Opinion API. On-chain settlement is executed asynchronously by backend infrastructure, abstracting gas costs from end users.

**Supported Gas-Free Operations**

**Market Data Queries**

* `get_markets()` - Market metadata retrieval
* `get_market()` - Individual market details
* `get_categorical_market()` - Categorical market information
* `get_orderbook()` - Real-time order book snapshots
* `get_latest_price()` - Current market prices
* `get_price_history()` - Historical price data (OHLCV candles)
* `get_quote_tokens()` - Supported collateral currencies
* `get_fee_rates()` - Fee schedule information

**Order Management**

* `place_order()` - Order submission (market and limit)
* `cancel_order()` - Single order cancellation
* `cancel_all_orders()` - Batch order cancellation
* `get_my_orders()` - Active order retrieval
* `get_order_by_id()` - Order status queries

**Position Tracking**

* `get_my_balances()` - Token balance queries
* `get_my_positions()` - Position inventory
* `get_my_trades()` - Trade history

#### Technical Implementation

```python
from opinion_clob_sdk import Client
from opinion_clob_sdk.chain.py_order_utils.model.order import PlaceOrderDataInput
from opinion_clob_sdk.chain.py_order_utils.model.sides import OrderSide
from opinion_clob_sdk.chain.py_order_utils.model.order_type import LIMIT_ORDER

client = Client(
    host='https://proxy.opinion.trade:8443',
    apikey='your_api_key',
    chain_id=56,
    rpc_url='https://bsc-dataseed.binance.org',
    private_key='0x...',      # Signs orders (no gas required)
    multi_sig_addr='0x...'
)

# Gas-free order placement
order = PlaceOrderDataInput(
    marketId=813,
    tokenId='——————',
    side=OrderSide.BUY,
    orderType=LIMIT_ORDER,
    price='0.65',
    makerAmountInQuoteToken=100
)

result = client.place_order(order)  # Zero gas cost to user
```

#### EIP712 Signature Protocol

Orders employ typed structured data signatures conforming to EIP712 specification. Signatures provide cryptographic proof of authorization without blockchain state modification.

**Signature Generation Flow:**

1. Order parameters encoded per EIP712 TypedData standard
2. Digest computed: `keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)`
3. ECDSA signature generated via secp256k1 curve
4. Signature transmitted with order to API endpoint
5. Backend performs ecrecover validation (signer authenticity)
6. Matched orders batch-settled on-chain (gas paid by infrastructure)

**Pseudocode:**

```python
# EIP712 signature construction (simplified)
domain_separator = keccak256(
    encode(EIP712Domain_TYPEHASH, name, version, chainId, verifyingContract)
)
struct_hash = keccak256(encode(ORDER_TYPEHASH, order_params))
digest = keccak256(concat(0x1901, domain_separator, struct_hash))
signature = ecdsa_sign(digest, private_key)  # Returns (v, r, s)
```

### Gas-Required Operations

#### On-Chain Smart Contract Invocations

Direct blockchain transactions invoke smart contract methods and require gas payment in BNB native tokens. These operations modify on-chain state and are irreversible post-confirmation.

**Operations Requiring Gas**

**Token Approval (One-Time Setup)**

* `enable_trading()` - Grant ERC20/ERC1155 allowances to exchange contracts

**Position Operations**

* `split()` - Invoke `ConditionalTokens.splitPosition()` (USDT → outcome tokens)
* `merge()` - Invoke `ConditionalTokens.mergePositions()` (outcome tokens → USDT)
* `redeem()` - Invoke `ConditionalTokens.redeemPositions()` (claim winning payouts)

**State Query (RPC Call, No Gas)**

* `check_enable_trading()` - Read contract allowance state via `eth_call`

#### Gas Cost Analysis

**BNB Chain Network Parameters**

| Parameter          | Value     | Notes                                  |
| ------------------ | --------- | -------------------------------------- |
| Block Time         | \~1.5s    | Average block production interval      |
| Gas Price          | 0.05 Gwei | Minimum gas price (EIP-1559 base fee)  |
| Finality Threshold | 10 blocks | Recommended confirmation depth (\~15s) |

**Operation Gas Consumption Estimates**

| Operation                        | Gas Units | Cost @ 0.05 Gwei | USD Cost @ $600/BNB |
| -------------------------------- | --------- | ---------------- | ------------------- |
| `enable_trading()` (2 approvals) | \~100,000 | 0.000005 BNB     | $0.003              |
| `split()`                        | \~150,000 | 0.0000075 BNB    | $0.0045             |
| `merge()`                        | \~120,000 | 0.000006 BNB     | $0.0036             |
| `redeem()`                       | \~180,000 | 0.000009 BNB     | $0.0054             |

**Cost Formula:**

```
Transaction Fee = Gas Units × Gas Price (Gwei) × 10^-9 BNB
USD Cost = Transaction Fee × BNB/USD Price
```

**Variability Factors:**

* Network congestion (gas price auction)
* Contract state complexity (storage operations)
* Transaction data size (calldata cost)
* BNB market price volatility

#### Implementation Examples

**Enable Trading (Required Once)**

```python
from opinion_clob_sdk import Client

client = Client(...)

# Check current approval status
status = client.check_enable_trading()
print(f"USDT approved: {status['usdt_approved']}")
print(f"Conditional tokens approved: {status['conditional_tokens_approved']}")

# Grant approvals if needed
if not (status['usdt_approved'] and status['conditional_tokens_approved']):
    tx_hash = client.enable_trading()
    print(f"Approval transaction: {tx_hash}")
    # Wait for confirmation before trading
```

**Required Approvals:**

* **USDT Contract** → Exchange contract (for collateral deposits)
* **ConditionalTokens Contract** → Exchange contract (for outcome token trading)

**Split Position**

```python
# Convert 100 USDT into 100 YES + 100 NO tokens
amount_in_usdt = 100
amount_in_wei = amount_in_usdt * 10**18  # USDT has 18 decimals

tx_hash = client.split(
    market_id=813,
    amount=amount_in_wei
)

print(f"Split transaction: {tx_hash}")
# Result: +100 YES tokens, +100 NO tokens, -100 USDT
```

**Use Cases:**

* Creating outcome tokens for selling
* Market making strategies
* Arbitrage opportunities

**Merge Position**

```python
# Convert 50 YES + 50 NO back into 50 USDT
amount_to_merge = 50 * 10**18  # Outcome tokens use 18 decimals

tx_hash = client.merge(
    market_id=813,
    amount=amount_to_merge
)

print(f"Merge transaction: {tx_hash}")
# Result: -50 YES tokens, -50 NO tokens, +50 USDT
```

**Requirements:**

* Must hold equal amounts of both outcome tokens
* Amount specified in Wei (18 decimals)

**Redeem Winnings**

```python
# Claim winnings from resolved market
try:
    tx_hash = client.redeem(market_id=813)
    print(f"Redeem transaction: {tx_hash}")
except NoPositionsToRedeem:
    print("No winning positions to redeem")
```

**Redemption Logic:**

* Markets resolved to YES: 1 YES token → 1 USDT
* Markets resolved to NO: 1 NO token → 1 USDT
* Losing outcome tokens become worthless

### Gas Balance Requirements

#### Recommended BNB Holdings

Maintain sufficient BNB balance to execute gas-required operations without transaction failures.

**Allocation Guidelines:**

| Use Case       | Minimum BNB         | Rationale                                   |
| -------------- | ------------------- | ------------------------------------------- |
| Initial setup  | 0.001 BNB (\~$0.60) | Single `enable_trading()` call              |
| High-frequency | 0.1 BNB (\~$60.00)  | Hundreds of transactions, failover capacity |

#### Balance Monitoring

```python
from web3 import Web3

w3 = Web3(Web3.HTTPProvider('https://bsc-dataseed.binance.org'))
address = '0xYourWalletAddress'

# Query native token balance
balance_wei = w3.eth.get_balance(address)
balance_bnb = Web3.from_wei(balance_wei, 'ether')

print(f"BNB Balance: {balance_bnb:.6f} BNB")

# Alert threshold
MINIMUM_BALANCE = 0.005  # BNB
if balance_bnb < MINIMUM_BALANCE:
    print(f"WARNING: BNB balance below threshold. Current: {balance_bnb}, Required: {MINIMUM_BALANCE}")
```

#### Position Management Planning

Structure trading strategies to minimize split/merge operations.

**Example Strategy:**

1. Execute single split to create large token inventory
2. Trade via gas-free CLOB orders
3. Merge/redeem only when exiting position or market resolves

```python
# Initial setup (one-time gas cost)
client.split(market_id=813, amount=1000_000000)  # Create 1000 YES + 1000 NO

# Trading loop (no gas costs)
for i in range(100):
    order = PlaceOrderDataInput(...)
    client.place_order(order)  # Gas-free

# Position exit (one-time gas cost)
client.merge(market_id=813, amount=500 * 10**18)  # Merge remaining 500 pairs
```

### Next Steps

* **Precision** - Token decimal systems
