# Configuration

This guide covers how to configure the Opinion CLOB SDK for different environments and use cases.

### Client Configuration

The `Client` class accepts multiple configuration parameters during initialization:

```python
from opinion_clob_sdk import Client

client = Client(
    host='https://proxy.opinion.trade:8443',
    apikey='your_api_key',
    chain_id=56,
    rpc_url='your_rpc_url',
    private_key='0x...',
    multi_sig_addr='0x...',
    conditional_tokens_addr='0xAD1a38cEc043e70E83a3eC30443dB285ED10D774',
    multisend_addr='0x998739BFdAAdde7C933B942a68053933098f9EDa',
    enable_trading_check_interval=3600,
    quote_tokens_cache_ttl=3600,
    market_cache_ttl=300
)
```

### Required Parameters

#### host

**Type**: `str` **Description**: Opinion API host URL **Default**: No default (required)

```python
# Production
host='https://proxy.opinion.trade:8443'
```

#### apikey

**Type**: `str` **Description**: API authentication key provided by Opinion Labs **Default**: No default (required)

**How to obtain**: fill out  this [short application form](https://docs.google.com/forms/d/1h7gp8UffZeXzYQ-lv4jcou9PoRNOqMAQhyW4IwZDnII)

```python
apikey='________'
```

> ⚠️ **Security**: Store API keys in environment variables, never in source code.

#### chain\_id

**Type**: `int` **Description**: Blockchain network chain ID **Supported values**:

* `56` - BNB Chain Mainnet (production)

```python
# Mainnet
chain_id=56
```

#### rpc\_url

**Type**: `str` **Description**: Blockchain RPC endpoint URL **Default**: No default (required)

**Common providers**:

* **BNB Chain Mainnet**: `https://bsc-dataseed.binance.org`
* **BNB Chain (Nodereal)**: [`https://bsc.nodereal.io`](https://bsc.nodereal.io)

```python
# Public RPC (rate limited)
rpc_url='https://bsc-dataseed.binance.org'

# Private RPC (recommended for production)
rpc_url='https://bsc.nodereal.io'
```

#### private\_key

**Type**: `str` (HexStr) **Description**: Private key for signing orders and transactions **Format**: 64-character hex string (with or without `0x` prefix)

```python
private_key='0x1234567890abcdef...'  # With 0x prefix
# or
private_key='1234567890abcdef...'    # Without 0x prefix
```

> ⚠️ **Critical Security**:
>
> * Never commit private keys to version control
> * Use environment variables or secure key management systems
> * Ensure the associated address has BNB for gas fees
> * This is the **signer** address, may differ from multi\_sig\_addr

#### multi\_sig\_addr

**Type**: `str` **Description**: Multi-signature wallet address (your assets/portfolio wallet) **Format**: Ethereum address (checksummed or lowercase)

```python
multi_sig_addr='0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
```

**Relationship to private\_key**:

* `private_key` → **Signer address** (signs orders/transactions)
* `multi_sig_addr` → **Assets address** (holds funds/positions)
* Can be the same address or different (e.g., hot wallet signs for cold wallet)

**Where to find**:

* Check your Opinion platform "My Profile" section
* Or use the wallet address where you hold USDT/positions

### Optional Parameters

#### conditional\_tokens\_addr

**Type**: `ChecksumAddress` (str) **Description**: ConditionalTokens contract address **Default**: `0xAD1a38cEc043e70E83a3eC30443dB285ED10D774` (BNB Chain mainnet)

```python
# Default for BNB Chain - no need to specify
client = Client(chain_id=56, ...)

# Custom deployment
conditional_tokens_addr='0xYourConditionalTokensContract...'
```

**When to set**: Only if using a custom deployment

#### multisend\_addr

**Type**: `ChecksumAddress` (str) **Description**: Gnosis Safe MultiSend contract address **Default**: `0x998739BFdAAdde7C933B942a68053933098f9EDa` (BNB Chain mainnet)

```python
# Default for BNB Chain - no need to specify
client = Client(chain_id=56, ...)

# Custom deployment
multisend_addr='0xYourMultiSendContract...'
```

**When to set**: Only if using a custom Gnosis Safe deployment

#### enable\_trading\_check\_interval

**Type**: `int` **Description**: Cache duration (in seconds) for trading approval checks **Default**: `3600` (1 hour) **Range**: `0` to `∞`

```python
# Default: check approval status every hour
enable_trading_check_interval=3600

# Check every time (no caching)
enable_trading_check_interval=0

# Check daily
enable_trading_check_interval=86400
```

**Impact**:

* Higher values → Fewer RPC calls → Faster performance
* `0` → Always check → Slower but always current
* Recommended: `3600` (approvals rarely change)

#### quote\_tokens\_cache\_ttl

**Type**: `int` **Description**: Cache duration (in seconds) for quote token data **Default**: `3600` (1 hour) **Range**: `0` to `∞`

```python
# Default: cache for 1 hour
quote_tokens_cache_ttl=3600

# No caching (always fresh)
quote_tokens_cache_ttl=0

# Cache for 6 hours
quote_tokens_cache_ttl=21600
```

**Impact**:

* Quote tokens rarely change
* Higher values improve performance
* Recommended: `3600` or higher

#### market\_cache\_ttl

**Type**: `int` **Description**: Cache duration (in seconds) for market data **Default**: `300` (5 minutes) **Range**: `0` to `∞`

```python
# Default: cache for 5 minutes
market_cache_ttl=300

# No caching (always fresh)
market_cache_ttl=0

# Cache for 1 hour
market_cache_ttl=3600
```

**Impact**:

* Markets change frequently (prices, status)
* Lower values → More current data
* Recommended: `300` for balance of performance and freshness

### Environment Variables

#### Using .env Files

Create a `.env` file in your project root:

```bash
# .env
API_KEY=opn_prod_abc123xyz789
RPC_URL=____
PRIVATE_KEY=0x1234567890abcdef...
MULTI_SIG_ADDRESS=0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
CHAIN_ID=56
```

Load in your Python code:

```python
import os
from dotenv import load_dotenv
from opinion_clob_sdk import Client

# Load .env file
load_dotenv()

# Use environment variables
client = Client(
    host='https://api.opinion.trade',
    apikey=os.getenv('API_KEY'),
    chain_id=int(os.getenv('CHAIN_ID', 56)),
    rpc_url=os.getenv('RPC_URL'),
    private_key=os.getenv('PRIVATE_KEY'),
    multi_sig_addr=os.getenv('MULTI_SIG_ADDRESS')
)
```

#### Using System Environment Variables

Set in shell:

```bash
# Linux/macOS
export API_KEY="opn_prod_abc123xyz789"
export RPC_URL=___
export PRIVATE_KEY="0x..."
export MULTI_SIG_ADDRESS="0x..."

# Windows (Command Prompt)
set API_KEY=opn_prod_abc123xyz789
set RPC_URL=___

# Windows (PowerShell)
$env:API_KEY="opn_prod_abc123xyz789"
$env:RPC_URL=___
```

Then access in Python:

```python
import os
client = Client(
    host='https://proxy.opinion.trade:8443',
    apikey=os.environ['API_KEY'],  # Raises error if not set
    # ... or ...
    apikey=os.getenv('API_KEY', 'default_value'),  # Returns default if not set
    # ...
)
```

### Configuration Patterns

#### Multi-Environment Setup

Manage different environments (dev, staging, prod):

```python
import os
from opinion_clob_sdk import Client

ENVIRONMENTS = {
    'production': {
        'host': 'https://proxy.opinion.trade:8443',
        'chain_id': 56,  # BNB Chain Mainnet
        'rpc_url': 'https://bsc-dataseed.binance.org'
    }
}

def create_client(env='production'):
    config = ENVIRONMENTS[env]

    return Client(
        host=config['host'],
        apikey=os.getenv(f'{env.upper()}_API_KEY'),
        chain_id=config['chain_id'],
        rpc_url=config['rpc_url'],
        private_key=os.getenv(f'{env.upper()}_PRIVATE_KEY'),
        multi_sig_addr=os.getenv(f'{env.upper()}_MULTI_SIG_ADDRESS')
    )

# Usage
dev_client = create_client('development')
prod_client = create_client('production')
```

#### Configuration Class

Organize configuration in a class:

```python
from dataclasses import dataclass
import os
from opinion_clob_sdk import Client

@dataclass
class OpinionConfig:
    api_key: str
    rpc_url: str
    private_key: str
    multi_sig_address: str
    chain_id: int = 56
    host: str = 'https://proxy.opinion.trade:8443'
    market_cache_ttl: int = 300

    @classmethod
    def from_env(cls):
        """Load configuration from environment variables"""
        return cls(
            api_key=os.environ['API_KEY'],
            rpc_url=os.environ['RPC_URL'],
            private_key=os.environ['PRIVATE_KEY'],
            multi_sig_address=os.environ['MULTI_SIG_ADDRESS'],
            chain_id=int(os.getenv('CHAIN_ID', 56))
        )

    def create_client(self):
        """Create Opinion Client from this configuration"""
        return Client(
            host=self.host,
            apikey=self.api_key,
            chain_id=self.chain_id,
            rpc_url=self.rpc_url,
            private_key=self.private_key,
            multi_sig_addr=self.multi_sig_address,
            market_cache_ttl=self.market_cache_ttl
        )

# Usage
config = OpinionConfig.from_env()
client = config.create_client()
```

#### Read-Only Client

For applications that only read data (no trading):

```python
# Minimal configuration for read-only access
client = Client(
    host='https://proxy.opinion.trade:8443',
    apikey=os.getenv('API_KEY'),
    chain_id=56,
    rpc_url='',           # Empty if not doing contract operations
    private_key='0x00',   # Dummy key if not placing orders
    multi_sig_addr='0x0000000000000000000000000000000000000000'
)

# Can use all GET methods
markets = client.get_markets()
market = client.get_market(123)
orderbook = client.get_orderbook('token_123')

# Cannot use trading or contract methods
# client.place_order(...)  # Would fail
# client.split(...)        # Would fail
```

### Performance Tuning

#### High-Frequency Trading

For trading bots with frequent API calls:

```python
client = Client(
    # ... required params ...
    market_cache_ttl=60,           # 1-minute cache for faster updates
    quote_tokens_cache_ttl=3600,   # 1-hour cache (rarely changes)
    enable_trading_check_interval=7200  # 2-hour cache (already approved)
)
```

#### Analytics/Research

For data analysis with less frequent updates:

```python
client = Client(
    # ... required params ...
    market_cache_ttl=1800,         # 30-minute cache
    quote_tokens_cache_ttl=86400,  # 24-hour cache
    enable_trading_check_interval=0  # Not trading
)
```

#### Real-Time Monitoring

For dashboards requiring fresh data:

```python
client = Client(
    # ... required params ...
    market_cache_ttl=0,            # No caching
    quote_tokens_cache_ttl=0,      # No caching
    enable_trading_check_interval=0
)
```

###

### Smart Contract Addresses

#### BNB Chain Mainnet (Chain ID: 56)

The following smart contract addresses are used by the Opinion CLOB SDK on BNB Chain mainnet:

| Contract              | Address                                      | Description                                            |
| --------------------- | -------------------------------------------- | ------------------------------------------------------ |
| **ConditionalTokens** | `0xAD1a38cEc043e70E83a3eC30443dB285ED10D774` | ERC1155 conditional tokens contract for outcome tokens |
| **MultiSend**         | `0x998739BFdAAdde7C933B942a68053933098f9EDa` | Gnosis Safe MultiSend contract for batch transactions  |

These addresses are automatically used by the SDK when you specify `chain_id=56`. You only need to provide custom addresses if you're using a custom deployment.

**Verification:**

* ConditionalTokens: [View on BscScan](https://bscscan.com/address/0xAD1a38cEc043e70E83a3eC30443dB285ED10D774)
* MultiSend: [View on BscScan](https://bscscan.com/address/0x998739BFdAAdde7C933B942a68053933098f9EDa)

### Next Steps

* [**API Reference**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/api-references): All Supported Methods
* [**Configuration Guide**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/getting-started/configuration): Configuration
* [**Core Concepts**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/core-concepts): Architecture
* [**Troubleshooting**](https://docs.opinion.trade/developer-guide/opinion-clob-sdk/support/troubleshooting): Common Issues
