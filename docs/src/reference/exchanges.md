# Exchanges

## Supported Exchanges

### Kalshi

- **ID:** `kalshi`
- **Website:** [kalshi.com](https://kalshi.com)
- **API Docs:** [docs.kalshi.com](https://docs.kalshi.com)
- **Auth:** RSA key pair (`api_key_id` + `private_key_pem`)
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades, WebSocket

### Polymarket

- **ID:** `polymarket`
- **Website:** [polymarket.com](https://polymarket.com)
- **API Docs:** [docs.polymarket.com](https://docs.polymarket.com/developers/)
- **Auth:** Private key + optional CLOB API credentials
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades, WebSocket

### Opinion

- **ID:** `opinion`
- **Website:** [opinion.trade](https://opinion.trade)
- **API Docs:** [docs.opinion.trade](https://docs.opinion.trade/developer-guide/opinion-open-api)
- **Auth:** API key + private key + multi-sig address
- **Features:** Markets, Orders, Positions, Balance, Orderbook, WebSocket

### Limitless

- **ID:** `limitless`
- **Website:** [limitless.exchange](https://limitless.exchange)
- **API Docs:** [api.limitless.exchange](https://api.limitless.exchange/api-v1)
- **Auth:** Private key
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Trades, WebSocket

### Predict.fun

- **ID:** `predictfun`
- **Website:** [predict.fun](https://predict.fun)
- **API Docs:** [dev.predict.fun](https://dev.predict.fun/)
- **Auth:** API key + private key
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades

## Configuration

All exchanges accept a JSON config object. Pass exchange-specific fields:

```json
{
  "kalshi": {
    "api_key_id": "...",
    "private_key_pem": "...",
    "demo": false
  },
  "polymarket": {
    "private_key": "0x...",
    "funder": "0x...",
    "api_key": "...",
    "api_secret": "...",
    "api_passphrase": "..."
  },
  "opinion": {
    "api_key": "...",
    "private_key": "0x...",
    "multi_sig_addr": "0x..."
  },
  "limitless": {
    "private_key": "0x..."
  },
  "predictfun": {
    "api_key": "...",
    "private_key": "0x...",
    "testnet": false
  }
}
```
