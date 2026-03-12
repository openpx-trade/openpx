---
url: "https://docs.polymarket.com/developers/CLOB/quickstart"
title: "Quickstart - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/quickstart#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Central Limit Order Book

Quickstart

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Installation](https://docs.polymarket.com/developers/CLOB/quickstart#installation)
- [Quick Start](https://docs.polymarket.com/developers/CLOB/quickstart#quick-start)
- [1\. Setup Client](https://docs.polymarket.com/developers/CLOB/quickstart#1-setup-client)
- [2\. Place an Order](https://docs.polymarket.com/developers/CLOB/quickstart#2-place-an-order)
- [3\. Check Your Orders](https://docs.polymarket.com/developers/CLOB/quickstart#3-check-your-orders)
- [Complete Example](https://docs.polymarket.com/developers/CLOB/quickstart#complete-example)
- [Troubleshooting](https://docs.polymarket.com/developers/CLOB/quickstart#troubleshooting)
- [Next Steps](https://docs.polymarket.com/developers/CLOB/quickstart#next-steps)

## [​](https://docs.polymarket.com/developers/CLOB/quickstart\#installation)  Installation

TypeScript

Python

Copy

Ask AI

```
npm install @polymarket/clob-client ethers
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/quickstart\#quick-start)  Quick Start

### [​](https://docs.polymarket.com/developers/CLOB/quickstart\#1-setup-client)  1\. Setup Client

TypeScript

Python

Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { Wallet } from "ethers"; // v5.8.0

const HOST = "https://clob.polymarket.com";
const CHAIN_ID = 137; // Polygon mainnet
const signer = new Wallet(process.env.PRIVATE_KEY);

// Create or derive user API credentials
const tempClient = new ClobClient(HOST, CHAIN_ID, signer);
const apiCreds = await tempClient.createOrDeriveApiKey();

// See 'Signature Types' note below
const signatureType = 0;

// Initialize trading client
const client = new ClobClient(
  HOST,
  CHAIN_ID,
  signer,
  apiCreds,
  signatureType
);
```

This quick start sets your EOA as the trading account. You’ll need to fund this
wallet to trade and pay for gas on transactions. Gas-less transactions are only
available by deploying a proxy wallet and using Polymarket’s Polygon relayer
infrastructure.

Signature Types

| Wallet Type | ID | When to Use |
| --- | --- | --- |
| EOA | `0` | Standard Ethereum wallet (MetaMask) |
| Custom Proxy | `1` | Specific to Magic Link users from Polymarket only |
| Gnosis Safe | `2` | Injected providers (Metamask, Rabby, embedded wallets) |

* * *

### [​](https://docs.polymarket.com/developers/CLOB/quickstart\#2-place-an-order)  2\. Place an Order

TypeScript

Python

Copy

Ask AI

```
import { Side } from "@polymarket/clob-client";

// Place a limit order in one step
const response = await client.createAndPostOrder({
  tokenID: "YOUR_TOKEN_ID", // Get from Gamma API
  price: 0.65, // Price per share
  size: 10, // Number of shares
  side: Side.BUY, // or SELL
});

console.log(`Order placed! ID: ${response.orderID}`);
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/quickstart\#3-check-your-orders)  3\. Check Your Orders

TypeScript

Python

Copy

Ask AI

```
// View all open orders
const openOrders = await client.getOpenOrders();
console.log(`You have ${openOrders.length} open orders`);

// View your trade history
const trades = await client.getTrades();
console.log(`You've made ${trades.length} trades`);
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/quickstart\#complete-example)  Complete Example

TypeScript

Python

Copy

Ask AI

```
import { ClobClient, Side } from "@polymarket/clob-client";
import { Wallet } from "ethers";

async function trade() {
  const HOST = "https://clob.polymarket.com";
  const CHAIN_ID = 137; // Polygon mainnet
  const signer = new Wallet(process.env.PRIVATE_KEY);

  const tempClient = new ClobClient(HOST, CHAIN_ID, signer);
  const apiCreds = await tempClient.createOrDeriveApiKey();

  const signatureType = 0;

  const client = new ClobClient(
    HOST,
    CHAIN_ID,
    signer,
    apiCreds,
    signatureType
  );

  const response = await client.createAndPostOrder({
    tokenID: "YOUR_TOKEN_ID",
    price: 0.65,
    size: 10,
    side: Side.BUY,
  });

  console.log(`Order placed! ID: ${response.orderID}`);
}

trade();
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/quickstart\#troubleshooting)  Troubleshooting

Error: L2\_AUTH\_NOT\_AVAILABLE

You forgot to call `createOrDeriveApiKey()`. Make sure you initialize the client with API credentials:

Copy

Ask AI

```
const creds = await clobClient.createOrDeriveApiKey();
const client = new ClobClient(host, chainId, wallet, creds);
```

Order rejected: insufficient balance

Ensure you have:

- **USDC** in your funder address for BUY orders
- **Outcome tokens** in your funder address for SELL orders

Check your balance at [polymarket.com/portfolio](https://polymarket.com/portfolio).

Order rejected: insufficient allowance

You need to approve the Exchange contract to spend your tokens. This is typically done through the Polymarket UI on your first trade. Or use the CTF contract’s `setApprovalForAll()` method.

What's my funder address?

Your funder address is the Polymarket proxy wallet where you deposit funds. Find it:

1. Go to [polymarket.com/settings](https://polymarket.com/settings)
2. Look for “Wallet Address” or “Profile Address”
3. This is your `FUNDER_ADDRESS`

* * *

## [​](https://docs.polymarket.com/developers/CLOB/quickstart\#next-steps)  Next Steps

[**Full Example Implementations** \\
\\
Complete Next.js examples demonstrating integration of embedded wallets\\
(Privy, Magic, Turnkey, wagmi) and the CLOB and Builder Relay clients](https://docs.polymarket.com/developers/builders/examples)

[**Understand CLOB Authentication** \\
\\
Deep dive into L1 and L2 authentication](https://docs.polymarket.com/developers/CLOB/authentication) [**Browse Client Methods** \\
\\
Explore the complete client reference](https://docs.polymarket.com/developers/CLOB/clients/methods-overview) [**Find Markets to Trade** \\
\\
Use Gamma API to discover markets](https://docs.polymarket.com/developers/gamma-markets-api/get-markets) [**Monitor with WebSocket** \\
\\
Get real-time order updates](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview)

[Status](https://docs.polymarket.com/developers/CLOB/status) [Authentication](https://docs.polymarket.com/developers/CLOB/authentication)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.