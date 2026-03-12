# Architecture

### System Architecture

The Opinion CLOB SDK implements a hybrid architecture that integrates off-chain order matching with on-chain settlement. This Python client provides programmatic access to the Opinion prediction market infrastructure deployed on BNB Chain.

#### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Your Application                              │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                  opinion_clob_sdk.Client                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  API Layer (opinion_api)                                 │   │
│  │  - Market data queries                                   │   │
│  │  - Order submission                                      │   │
│  │  - Position tracking                                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Chain Layer (Web3)                                      │   │
│  │  - Smart contract interactions                           │   │
│  │  - Token operations (split/merge/redeem)                 │   │
│  │  - Transaction signing                                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Order Utils (EIP712)                                    │   │
│  │  - Order building                                        │   │
│  │  - Cryptographic signing                                 │   │
│  │  - Gnosis Safe integration                               │   │
│  └─────────────────────────────────────────────────────────┘   │
└───────────┬──────────────────────────────────┬─────────────────┘
            │                                  │
            ▼                                  ▼
┌─────────────────────────┐      ┌──────────────────────────────┐
│  Opinion CLOB API       │      │  BNB Chain (BSC)             │
│  - Order matching       │      │  - ConditionalTokens         │
│  - Market data          │      │  - USDT (Collateral)         │
│  - User positions       │      │  - Gnosis Safe               │
└─────────────────────────┘      └──────────────────────────────┘
```

### Core Components

#### 1. Client (`sdk.py`)

The `Client` class serves as the primary interface, orchestrating interactions across API and blockchain layers.

**Responsibilities:**

* API connection management and authentication
* Method exposure for market data, trading, and contract operations
* Market data and quote token caching with configurable TTL
* Coordination between HTTP requests and Web3 transactions

**Key Configuration:**

```python
client = Client(
    host='https://proxy.opinion.trade:8443',  # API endpoint
    apikey='your_api_key',                     # Authentication
    chain_id=56,                                # BNB Chain
    rpc_url='https://bsc-dataseed.binance.org',
    private_key='0x...',                        # For signing
    multi_sig_addr='0x...'                      # Assets wallet
)
```

#### 2. API Layer (`opinion_api`)

Auto-generated OpenAPI client managing HTTP communication with the Opinion CLOB backend.

**Capabilities:**

* RESTful API invocation with type-safe request/response models
* Request serialization and response deserialization
* HTTP status code handling and error propagation
* Bearer token authentication (API key-based)

**Endpoints Covered:**

* Market discovery and details
* Orderbook snapshots
* Price data and candles
* Order placement and cancellation
* Position and balance queries
* Trade history

#### 3. Chain Layer (`chain/contract_caller.py`)

Web3 integration layer enabling direct blockchain interactions via the `web3.py` library.

**Smart Contracts Integrated:**

| Contract              | Address                                      | Purpose                                                         |
| --------------------- | -------------------------------------------- | --------------------------------------------------------------- |
| **ConditionalTokens** | `0xAD1a38cEc043e70E83a3eC30443dB285ED10D774` | Core prediction market contract for splitting/merging positions |
| **MultiSend**         | `0x998739BFdAAdde7C933B942a68053933098f9EDa` | Gnosis Safe batch transaction helper                            |
| **USDT**              | Native BNB Chain USDT                        | Collateral token for all markets                                |

**Operations:**

* `split()` - Convert USDT → outcome tokens (YES/NO)
* `merge()` - Convert outcome tokens → USDT
* `redeem()` - Claim winnings from resolved markets
* `enable_trading()` - Approve token allowances

#### 4. Order Utils (`chain/py_order_utils/`)

Order construction and signing module implementing EIP712 typed structured data signatures.

**Components:**

* **OrderBuilder** - Constructs valid order objects with all required fields
* **Signer** - Signs orders using private key (EOA) or Gnosis Safe
* **Model Classes** - Type-safe order representations

**EIP712 Signing Process:**

```
Order Data → EIP712 Hash → ECDSA Signature → Signed Order → API
```

#### 5. Gnosis Safe Integration (`chain/safe/`)

Support for multi-signature wallets using Gnosis Safe v1.3.0 contracts.

**Key Concepts:**

* **Signer Address** - The private key that signs orders (can be a Safe owner)
* **Multi-Sig Address** - The Safe contract holding your funds
* **Signature Type 2** - Indicates Gnosis Safe signature scheme

### Data Flow Patterns

#### Pattern 1: Market Data Query (Gas-Free)

```
Application
    ↓ call get_markets()
Client
    ↓ HTTP GET /markets
Opinion API
    ↓ response (JSON)
Client (caches result, TTL-based)
    ↓ return typed object
Application
```

**Characteristics:**

* Pure API interaction, no blockchain transaction
* Zero gas cost
* Response latency: 50-150ms (cached: <1ms)
* Configurable TTL-based caching

#### Pattern 2: Order Placement (Gas-Free via CLOB)

```
Application
    ↓ place_order(order_data)
Client
    ↓ construct order
OrderBuilder
    ↓ generate EIP712 hash
Signer
    ↓ ECDSA signature (secp256k1)
Client
    ↓ HTTP POST /orders + signature
Opinion API
    ↓ verify signature (ecrecover)
    ↓ match against orderbook
    ↓ settle on-chain (backend batch)
Client
    ↓ return order confirmation (trans_no)
Application
```

**Characteristics:**

* Gas abstraction: backend covers settlement costs
* Cryptographic proof of authorization via ECDSA signature
* Order matching latency: 200-500ms
* On-chain settlement batched asynchronously

#### Pattern 3: Smart Contract Operation (Gas Required)

```
Application
    ↓ split(market_id, amount)
Client
    ↓ construct transaction data
Web3 Provider
    ↓ gas estimation
    ↓ transaction signing (ECDSA)
    ↓ broadcast to BNB Chain RPC
BNB Chain
    ↓ transaction inclusion (block creation)
    ↓ ConditionalTokens.splitPosition() execution
    ↓ event emission (logs)
Client
    ↓ transaction receipt polling
    ↓ return transaction hash
Application
```

**Characteristics:**

* Gas payment required (BNB native token)
* Direct smart contract invocation
* Block confirmation time: \~3 seconds (BSC)
* Finality: \~10 blocks (\~15 seconds recommended)
* State changes immutable post-confirmation

### Authentication & Security

#### Blockchain Signing

Two-key system for enhanced security:

1. **Private Key (Signer)**
   * Signs orders and transactions
   * Can be a hot wallet for automated trading
   * Never leaves your application
2. **Multi-Sig Address (Funder)**
   * Holds your USDT and outcome tokens
   * Gnosis Safe v1.3.0 create via GnosisSafeProxyFactory(0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2) on BNB Chain
   * Requires approval for token operations

**Example Configuration:**

```python
# EOA (externally owned account) setup
private_key = "0x..."        # Signs orders
multi_sig_addr = "0x..."     # Same as signer (EOA holds funds)

# Gnosis Safe setup
private_key = "0x..."        # Safe owner key (signs orders)
multi_sig_addr = "0x..."     # Safe contract address (holds funds)
```

### Precision and Number Handling

#### Token Decimals

All tokens use **18 decimal places** (Wei standard):

```python
1 USDT = 1_000_000_000_000_000_000 wei
0.5 YES = 500_000_000_000_000_000 wei
```

#### Price Representation

Prices are quoted as **decimal strings** representing probability:

```python
"0.5"   = 50% probability (50¢ per share)
"0.75"  = 75% probability (75¢ per share)
```

**Valid Range:** `0.01` to `0.99` (1% to 99%)

#### Amount Specifications

Orders can specify amounts in two ways:

```python
# Quote token (USDT) amount
PlaceOrderDataInput(
    makerAmountInQuoteToken=10  # Spend 10 USDT
)

# Base token (outcome token) amount
PlaceOrderDataInput(
    makerAmountInBaseToken=5    # Buy/sell 5 YES tokens
)
```

### Caching Strategy

The SDK implements intelligent caching for frequently accessed data:

#### Market Cache

```python
client = Client(
    market_cache_ttl_seconds=60,  # Cache markets for 1 minute
    ...
)
```

**Rationale:** Market metadata rarely changes, reducing API load.

#### Quote Token Cache

```python
client = Client(
    quote_token_cache_ttl_seconds=3600,  # Cache for 1 hour
    ...
)
```

**Rationale:** Supported currencies (USDT) are static configuration.

#### Real-Time Data

**Never cached:**

* Orderbook snapshots
* Latest prices
* User positions
* Order status

### Error Handling Architecture

#### API Errors

Structured response format:

```json
{
  "errno": 400,
  "errmsg": "____"
}
```

**Common Error Codes:**

* `0` - Success
* `400` - Invalid request parameters
* `500` - Internal server error

#### Chain Errors

```python
from opinion_clob_sdk.chain.exception import (
    BalanceNotEnough,           # Insufficient tokens
    NoPositionsToRedeem,        # No winning positions
    InsufficientGasBalance      # Not enough BNB for gas
)
```

**Handling Strategy:**

```python
try:
    tx_hash = client.split(market_id=813, amount=1000000000000000000)
except BalanceNotEnough:
    print("Insufficient USDT balance")
except InsufficientGasBalance:
    print("Need more BNB for gas fees")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### Performance Benchmarks

#### API Response Times

| Operation     | Typical Latency | Factors                |
| ------------- | --------------- | ---------------------- |
| Get markets   | 50-150ms        | Cached: <10ms          |
| Get orderbook | 100-300ms       | Market depth           |
| Place order   | 200-500ms       | Signature verification |
| Get positions | 100-200ms       | Position count         |

#### Blockchain Transactions

| Operation | Block Confirmation | Finality          |
| --------- | ------------------ | ----------------- |
| Split     | 2 block (\~3s)     | 10 blocks (\~15s) |
| Merge     | 2 block (\~3s)     | 10 blocks (\~15s) |
| Redeem    | 2 block (\~3s)     | 10 blocks (\~15s) |
| Approve   | 2 block (\~3s)     | 10 blocks (\~15s) |

**Note:** BNB Chain (BSC) has \~1.5 second block time and recommends waiting 10 blocks for finality.

####

#### Python Compatibility

**Supported:** Python 3.8, 3.9, 3.10, 3.11, 3.12

**Recommended:** Python 3.10+ for best performance and type checking

### Extensibility

#### Custom RPC Providers

The SDK supports any Web3-compatible RPC provider:

```python
# Free public RPC
client = Client(rpc_url='https://bsc-dataseed.binance.org', ...)
```

### Next Steps

* [**Client**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts/client) - Detailed setup guide
* [**Order**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts/order) - Understanding market/limit orders
* [**Gas Operations**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts/gas-operations) - When you need BNB
* [**Precision**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts/precision) - Working with Wei units
