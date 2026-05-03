# @openpx/sdk

[![npm](https://img.shields.io/npm/v/@openpx/sdk.svg)](https://www.npmjs.com/package/@openpx/sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Node.js SDK for OpenPX — a unified interface for prediction markets.

## Installation

```bash
npm install @openpx/sdk
```

## Quick Start

```javascript
const { Exchange } = require("@openpx/sdk");

const exchange = new Exchange("kalshi");
const markets = await exchange.fetchMarkets();

for (const market of markets.slice(0, 5)) {
  console.log(`${market.question}: ${JSON.stringify(market.prices)}`);
}
```

## Supported Exchanges

- **Kalshi** — US-regulated event contracts
- **Polymarket** — Crypto-native prediction markets

## Authentication

### Kalshi

```javascript
const exchange = new Exchange("kalshi", {
  api_key_id: process.env.KALSHI_API_KEY_ID,
  private_key_path: "./kalshi-private-key.pem", // or private_key_pem: "<inline PEM>"
});
```

Public market-data calls work without credentials.

### Polymarket

Pick the credential path that matches your wallet:

| Wallet model                           | Required keys                                               |
| -------------------------------------- | ----------------------------------------------------------- |
| MetaMask EOA + Polymarket Safe         | `private_key` (EOA) + `funder` (Safe)                       |
| Plain EOA (no Safe)                    | `private_key` only                                          |
| Pre-derived API keys (most reliable)   | `api_key` + `api_secret` + `api_passphrase` (+ `private_key` for order signing) |

```javascript
const exchange = new Exchange("polymarket", {
  private_key: process.env.POLYMARKET_PRIVATE_KEY,
  funder:      process.env.POLYMARKET_FUNDER, // omit for plain EOA
});
```

`signature_type` is auto-detected from `funder` and almost never needs to
be set explicitly. If you set it to `"eoa"` while a funder is configured,
the SDK overrides it to `"gnosis_safe"` with a warning — that combo is
rejected by Polymarket and is a common MetaMask misconfiguration.

If `derive-api-key` returns a Cloudflare WAF block (datacenter IPs are
often blocked), generate keys once via the Polymarket web app and pass
them as `api_key` / `api_secret` / `api_passphrase` to skip the derive
flow entirely.

Full credential matrix: [docs.openpx.io/setup/polymarket-credentials](https://docs.openpx.io/setup/polymarket-credentials).
