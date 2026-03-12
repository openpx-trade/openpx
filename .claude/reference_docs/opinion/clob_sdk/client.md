# Client

### Overview

The `Client` class provides the primary programmatic interface to the Opinion prediction market infrastructure. Configuration accuracy during initialization determines operational capability and security posture.

### Basic Initialization

#### Minimal Configuration

```python
from opinion_clob_sdk import Client

client = Client(
    host='https://proxy.opinion.trade:8443',
    apikey='your_api_key_here',
    chain_id=56,
    rpc_url='https://bsc-dataseed.binance.org',
    private_key='0x...',
    multi_sig_addr='0x...'
)
```

### Required Parameters

#### `host`

**Type:** `str`

**Description:** CLOB API endpoint URL for HTTP communication.

**Production Value:**

```python
host='https://proxy.opinion.trade:8443'
```

**Endpoint Functions:**

* Market data query routing
* Order submission and cancellation processing
* Position and balance data retrieval

#### `apikey`

**Type:** `str`

**Description:** Bearer token for API authentication and authorization.

**Acquisition:** Contact Opinion Labs at <support@opinion.trade> for API access provisioning.

**Security Best Practices:**

```python
import os
from dotenv import load_dotenv

load_dotenv()
apikey = os.getenv('API_KEY')  # Never hardcode!
```

#### `chain_id`

**Type:** `int`

**Description:** The blockchain network identifier.

**Supported Values:**

```python
from opinion_clob_sdk import CHAIN_ID_BNB_MAINNET

chain_id = CHAIN_ID_BNB_MAINNET  # 56
```

**Current Support:**

* **56** - BNB Chain (BSC) Mainnet

#### `rpc_url`

**Type:** `str`

**Description:** JSON-RPC endpoint URL for BNB Chain node communication.

**Available Providers:**

```python
# Public RPC endpoints (development and testing)
rpc_url = 'https://bsc-dataseed.binance.org'
rpc_url = 'https://bsc.nodereal.io'
```

#### `private_key`

**Type:** `str`

**Description:** secp256k1 private key for ECDSA signature generation (order signing and transaction authorization).

**Format Specification:**

```python
# 64-character hexadecimal string with 0x prefix (32 bytes)
private_key = '0x_______'
```

#### `multi_sig_addr`

**Type:** `str` (ChecksumAddress)

**Description:** Ethereum address (checksummed format) holding USDT collateral and outcome tokens.

**Gnosis Safe (Multi-Signature)**

```python
client = Client(
    private_key='0x...',           # Safe owner's key
    multi_sig_addr='0x8F58a1ab...', # Safe contract address
    ...
)
```

### Optional Parameters

#### `conditional_tokens_addr`

**Type:** `Optional[ChecksumAddress]`

**Default:** `0xAD1a38cEc043e70E83a3eC30443dB285ED10D774` (BNB Chain)

**Description:** The ConditionalTokens smart contract address for split/merge/redeem operations.

**When to Override:**

* Testing on custom networks
* Interacting with forked contracts
* Development environments

```python
# Usually you can omit this (uses default)
client = Client(...)

# Override only if needed
client = Client(
    conditional_tokens_addr='0xCustomAddress...',
    ...
)
```

#### `multisend_addr`

**Type:** `Optional[ChecksumAddress]`

**Default:** `0x998739BFdAAdde7C933B942a68053933098f9EDa` (BNB Chain)

**Description:** The Gnosis Safe MultiSend contract for batch transactions.

**When to Override:**

* Testing batch operations
* Custom Safe deployments

```python
# Usually you can omit this (uses default)
client = Client(...)
```

#### `market_cache_ttl_seconds`

**Type:** `int`

**Default:** `60` (1 minute)

**Description:** Time-to-live for cached market data.

**Tuning Guidelines:**

```python
# High-frequency trading (minimize stale data)
client = Client(market_cache_ttl_seconds=10, ...)

# Analytics/dashboards (reduce API load)
client = Client(market_cache_ttl_seconds=300, ...)

# One-time scripts (cache entire run)
client = Client(market_cache_ttl_seconds=3600, ...)

# Disable caching (always fresh data)
client = Client(market_cache_ttl_seconds=0, ...)
```

**Trade-offs:**

| TTL  | API Calls | Data Freshness  | Use Case                |
| ---- | --------- | --------------- | ----------------------- |
| 0    | Maximum   | Real-time       | HFT, critical decisions |
| 60   | Moderate  | 1-min delay OK  | General trading         |
| 300+ | Minimal   | 5-min+ delay OK | Analytics, monitoring   |

#### `quote_token_cache_ttl_seconds`

**Type:** `int`

**Default:** `3600` (1 hour)

**Description:** Time-to-live for cached quote token (currency) information.

**Rationale:** Quote tokens (USDT, etc.) rarely change, so aggressive caching is safe.

```python
# Default is usually fine
client = Client(quote_token_cache_ttl_seconds=3600, ...)
```

### Validation and Error Handling

#### Validate Configuration

```python
from opinion_clob_sdk import Client, SUPPORTED_CHAIN_IDS
from opinion_clob_sdk.chain.exception import InvalidParamError

try:
    client = Client(
        host='https://proxy.opinion.trade:8443',
        apikey='test_key',
        chain_id=999,  # Invalid!
        rpc_url='https://bsc-dataseed.binance.org',
        private_key='0x...',
        multi_sig_addr='0x...'
    )
except InvalidParamError as e:
    print(f"Configuration error: {e}")
    print(f"Supported chain IDs: {SUPPORTED_CHAIN_IDS}")
```

#### Test Connection

```python
client = Client(...)

# Test API connectivity
try:
    currencies = client.get_quote_tokens()
    print(f"✓ API connected: {len(currencies)} currencies available")
except Exception as e:
    print(f"✗ API connection failed: {e}")

# Test blockchain connectivity
try:
    from web3 import Web3
    w3 = Web3(Web3.HTTPProvider(client.rpc_url))
    block = w3.eth.block_number
    print(f"✓ RPC connected: current block {block}")
except Exception as e:
    print(f"✗ RPC connection failed: {e}")

# Test wallet
from web3 import Web3
account = Web3().eth.account.from_key(client.private_key)
print(f"✓ Wallet: {account.address}")
print(f"✓ Multi-sig: {client.multi_sig_addr}")
```

### Common Initialization Errors

#### Error: Invalid Chain ID

```python
# Error message
InvalidParamError: chain_id must be one of [56]

# Solution
client = Client(chain_id=56, ...)  # Use BNB Chain
```

#### Error: Invalid Private Key Format

```python
# Common mistakes
private_key = 'ac0974bec...'  # Missing 0x prefix
private_key = '0xGGGG...'     # Invalid hex

# Correct format
private_key = '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80'
```

#### Error: RPC Connection Failed

```python
# Symptoms
requests.exceptions.ConnectionError: Max retries exceeded

# Solutions
1. Check internet connection
2. Try alternative RPC URL
3. Use paid RPC provider
4. Check firewall settings
```

#### Error: API Authentication Failed

```python
# Error response
{"errno": 40100, "errmsg": "Invalid API key"}

# Solutions
1. Verify API key is correct
2. Check for extra whitespace
3. Ensure key is active
4. Contact support@opinion.trade
```

### Performance Optimization

#### Connection Pooling

```python
# The SDK reuses HTTP connections automatically
# For long-running applications, create one client instance

# ✅ Good
client = Client(...)
for i in range(1000):
    markets = client.get_markets()

# ❌ Bad (creates 1000 connections)
for i in range(1000):
    client = Client(...)
    markets = client.get_markets()
```

#### Caching Strategy

```python
# Aggressive caching for read-heavy workloads
client = Client(
    market_cache_ttl_seconds=300,      # 5 minutes
    quote_token_cache_ttl_seconds=3600 # 1 hour
)

# Minimal caching for trading bots
client = Client(
    market_cache_ttl_seconds=10,       # 10 seconds
    quote_token_cache_ttl_seconds=60   # 1 minute
)
```

### Next Steps

* **Order** - Understanding market and limit orders
* **Gas Operations** - When you need BNB for gas
* **Quick Start Guide** - Your first API calls
