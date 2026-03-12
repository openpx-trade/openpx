# Error Handling

## Error Hierarchy

```
OpenPxError
├── Network
│   ├── Http(String)
│   ├── Timeout(u64)
│   └── Connection(String)
├── Exchange
│   ├── MarketNotFound(String)
│   ├── InvalidOrder(String)
│   ├── OrderRejected(String)
│   ├── InsufficientFunds(String)
│   ├── Authentication(String)
│   ├── NotSupported(String)
│   └── Api(String)
├── WebSocket
│   ├── Connection(String)
│   ├── Closed
│   ├── Protocol(String)
│   └── Subscription(String)
├── Signing
│   ├── InvalidKey
│   ├── SigningFailed(String)
│   └── Unsupported(String)
├── RateLimitExceeded
├── Serialization(Error)
├── Config(String)
├── InvalidInput(String)
└── Other(String)
```

## Language Mapping

### Rust

```rust
use px_core::{OpenPxError, ExchangeError};

match result {
    Err(OpenPxError::Exchange(ExchangeError::Authentication(msg))) => {
        eprintln!("Auth failed: {msg}");
    }
    Err(OpenPxError::Network(e)) => {
        eprintln!("Network error: {e}");
    }
    Err(e) => eprintln!("Error: {e}"),
    Ok(v) => { /* success */ }
}
```

### Python

```python
from openpx import Exchange, OpenPxError, AuthenticationError, NetworkError

try:
    exchange.fetch_balance()
except AuthenticationError as e:
    print(f"Auth failed: {e}")
except NetworkError as e:
    print(f"Network error: {e}")
except OpenPxError as e:
    print(f"Error: {e}")
```

### TypeScript

```typescript
try {
  await exchange.fetchBalance();
} catch (e) {
  console.error(e.message);
}
```
