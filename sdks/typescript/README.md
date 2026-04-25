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
