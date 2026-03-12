---
url: "https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices"
title: "RTDS Crypto Prices - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Real Time Data Stream

RTDS Crypto Prices

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Overview](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#overview)
- [Binance Source (crypto\_prices)](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#binance-source-crypto_prices)
- [Subscription Details](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#subscription-details)
- [Subscription Message](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#subscription-message)
- [With Symbol Filter](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#with-symbol-filter)
- [Chainlink Source (crypto\_prices\_chainlink)](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#chainlink-source-crypto_prices_chainlink)
- [Subscription Details](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#subscription-details-2)
- [Subscription Message](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#subscription-message-2)
- [With Symbol Filter](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#with-symbol-filter-2)
- [Message Format](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#message-format)
- [Binance Source Message Format](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#binance-source-message-format)
- [Chainlink Source Message Format](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#chainlink-source-message-format)
- [Payload Fields](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#payload-fields)
- [Example Messages](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#example-messages)
- [Binance Source Examples](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#binance-source-examples)
- [Solana Price Update (Binance)](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#solana-price-update-binance)
- [Bitcoin Price Update (Binance)](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#bitcoin-price-update-binance)
- [Chainlink Source Examples](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#chainlink-source-examples)
- [Ethereum Price Update (Chainlink)](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#ethereum-price-update-chainlink)
- [Bitcoin Price Update (Chainlink)](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#bitcoin-price-update-chainlink)
- [Supported Symbols](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#supported-symbols)
- [Binance Source Symbols](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#binance-source-symbols)
- [Chainlink Source Symbols](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#chainlink-source-symbols)
- [Notes](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#notes)
- [General](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices#general)

[**TypeScript client** \\
\\
Official RTDS TypeScript client (`real-time-data-client`).](https://github.com/Polymarket/real-time-data-client)

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#overview)  Overview

The crypto prices subscription provides real-time updates for cryptocurrency price data from two different sources:

- **Binance Source** (`crypto_prices`): Real-time price data from Binance exchange
- **Chainlink Source** (`crypto_prices_chainlink`): Price data from Chainlink oracle networks

Both streams deliver current market prices for various cryptocurrency trading pairs, but use different symbol formats and subscription structures.

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#binance-source-crypto_prices)  Binance Source (`crypto_prices`)

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#subscription-details)  Subscription Details

- **Topic**: `crypto_prices`
- **Type**: `update`
- **Authentication**: Not required
- **Filters**: Optional (specific symbols can be filtered)
- **Symbol Format**: Lowercase concatenated pairs (e.g., `solusdt`, `btcusdt`)

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#subscription-message)  Subscription Message

Copy

Ask AI

```
{
  "action": "subscribe",
  "subscriptions": [\
    {\
      "topic": "crypto_prices",\
      "type": "update"\
    }\
  ]
}
```

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#with-symbol-filter)  With Symbol Filter

To subscribe to specific cryptocurrency symbols, include a filters parameter:

Copy

Ask AI

```
{
  "action": "subscribe",
  "subscriptions": [\
    {\
      "topic": "crypto_prices",\
      "type": "update",\
      "filters": "solusdt,btcusdt,ethusdt"\
    }\
  ]
}
```

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#chainlink-source-crypto_prices_chainlink)  Chainlink Source (`crypto_prices_chainlink`)

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#subscription-details-2)  Subscription Details

- **Topic**: `crypto_prices_chainlink`
- **Type**: `*` (all types)
- **Authentication**: Not required
- **Filters**: Optional (JSON object with symbol specification)
- **Symbol Format**: Slash-separated pairs (e.g., `eth/usd`, `btc/usd`)

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#subscription-message-2)  Subscription Message

Copy

Ask AI

```
{
  "action": "subscribe",
  "subscriptions": [\
    {\
      "topic": "crypto_prices_chainlink",\
      "type": "*",\
      "filters": ""\
    }\
  ]
}
```

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#with-symbol-filter-2)  With Symbol Filter

To subscribe to specific cryptocurrency symbols, include a JSON filters parameter:

Copy

Ask AI

```
{
  "action": "subscribe",
  "subscriptions": [\
    {\
      "topic": "crypto_prices_chainlink",\
      "type": "*",\
      "filters": "{\"symbol\":\"eth/usd\"}"\
    }\
  ]
}
```

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#message-format)  Message Format

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#binance-source-message-format)  Binance Source Message Format

When subscribed to Binance crypto prices (`crypto_prices`), you’ll receive messages with the following structure:

Copy

Ask AI

```
{
  "topic": "crypto_prices",
  "type": "update",
  "timestamp": 1753314064237,
  "payload": {
    "symbol": "solusdt",
    "timestamp": 1753314064213,
    "value": 189.55
  }
}
```

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#chainlink-source-message-format)  Chainlink Source Message Format

When subscribed to Chainlink crypto prices (`crypto_prices_chainlink`), you’ll receive messages with the following structure:

Copy

Ask AI

```
{
  "topic": "crypto_prices_chainlink",
  "type": "update",
  "timestamp": 1753314064237,
  "payload": {
    "symbol": "eth/usd",
    "timestamp": 1753314064213,
    "value": 3456.78
  }
}
```

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#payload-fields)  Payload Fields

| Field | Type | Description |
| --- | --- | --- |
| `symbol` | string | Trading pair symbol<br>**Binance**: lowercase concatenated (e.g., “solusdt”, “btcusdt”)<br>**Chainlink**: slash-separated (e.g., “eth/usd”, “btc/usd”) |
| `timestamp` | number | Price timestamp in Unix milliseconds |
| `value` | number | Current price value in the quote currency |

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#example-messages)  Example Messages

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#binance-source-examples)  Binance Source Examples

#### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#solana-price-update-binance)  Solana Price Update (Binance)

Copy

Ask AI

```
{
  "topic": "crypto_prices",
  "type": "update",
  "timestamp": 1753314064237,
  "payload": {
    "symbol": "solusdt",
    "timestamp": 1753314064213,
    "value": 189.55
  }
}
```

#### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#bitcoin-price-update-binance)  Bitcoin Price Update (Binance)

Copy

Ask AI

```
{
  "topic": "crypto_prices",
  "type": "update",
  "timestamp": 1753314088421,
  "payload": {
    "symbol": "btcusdt",
    "timestamp": 1753314088395,
    "value": 67234.50
  }
}
```

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#chainlink-source-examples)  Chainlink Source Examples

#### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#ethereum-price-update-chainlink)  Ethereum Price Update (Chainlink)

Copy

Ask AI

```
{
  "topic": "crypto_prices_chainlink",
  "type": "update",
  "timestamp": 1753314064237,
  "payload": {
    "symbol": "eth/usd",
    "timestamp": 1753314064213,
    "value": 3456.78
  }
}
```

#### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#bitcoin-price-update-chainlink)  Bitcoin Price Update (Chainlink)

Copy

Ask AI

```
{
  "topic": "crypto_prices_chainlink",
  "type": "update",
  "timestamp": 1753314088421,
  "payload": {
    "symbol": "btc/usd",
    "timestamp": 1753314088395,
    "value": 67234.50
  }
}
```

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#supported-symbols)  Supported Symbols

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#binance-source-symbols)  Binance Source Symbols

The Binance source supports various cryptocurrency trading pairs using lowercase concatenated format:

- `btcusdt` \- Bitcoin to USDT
- `ethusdt` \- Ethereum to USDT
- `solusdt` \- Solana to USDT
- `xrpusdt` \- XRP to USDT

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#chainlink-source-symbols)  Chainlink Source Symbols

The Chainlink source supports cryptocurrency trading pairs using slash-separated format:

- `btc/usd` \- Bitcoin to USD
- `eth/usd` \- Ethereum to USD
- `sol/usd` \- Solana to USD
- `xrp/usd` \- XRP to USD

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#notes)  Notes

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices\#general)  General

- Price updates are sent as market prices change
- The timestamp in the payload represents when the price was recorded
- The outer timestamp represents when the message was sent via WebSocket
- No authentication is required for crypto price data

[RTDS Overview](https://docs.polymarket.com/developers/RTDS/RTDS-overview) [RTDS Comments](https://docs.polymarket.com/developers/RTDS/RTDS-comments)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.