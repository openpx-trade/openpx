---
url: "https://docs.polymarket.com/developers/CLOB/clients/methods-overview"
title: "Methods Overview - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/clients/methods-overview#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Client

Methods Overview

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Client Initialization by Use Case](https://docs.polymarket.com/developers/CLOB/clients/methods-overview#client-initialization-by-use-case)
- [Resources](https://docs.polymarket.com/developers/CLOB/clients/methods-overview#resources)

[**Public Methods** \\
\\
Access market data, orderbooks, and prices.](https://docs.polymarket.com/developers/CLOB/clients/methods-public) [**L1 Methods** \\
\\
Private key authentication to create or derive API keys (L2 headers).](https://docs.polymarket.com/developers/CLOB/clients/methods-l1) [**L2 Methods** \\
\\
Manage and close orders. Creating orders requires signer.](https://docs.polymarket.com/developers/CLOB/clients/methods-l2) [**Builder Program Methods** \\
\\
Builder-specific operations for those in the Builders Program.](https://docs.polymarket.com/developers/CLOB/clients/methods-builder)

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-overview\#client-initialization-by-use-case)  Client Initialization by Use Case

- Get Market Data

- Generate User API Credentials

- Create and Post Order

- Get Builders Orders


TypeScript

Python

Copy

Ask AI

```
// No signer or credentials needed
const client = new ClobClient(
  "https://clob.polymarket.com",
  137
);

// All public methods available
const markets = await client.getMarkets();
const book = await client.getOrderBook(tokenId);
const price = await client.getPrice(tokenId, "BUY");
```

TypeScript

Python

Copy

Ask AI

```
// Create client with signer
const client = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer
);

// All public and L1 methods available
const newCreds = createApiKey();
const derivedCreds = deriveApiKey();
const creds = createOrDeriveApiKey();
```

TypeScript

Python

Copy

Ask AI

```
// Create client with signer and creds
const client = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer,
  creds,
  2, // Indicates Gnosis Safe proxy
  funder // Safe wallet address holding funds
);

// All public, L1, and L2 methods available
const order = await client.createOrder({ /* ... */ });
const result = await client.postOrder(order);
const trades = await client.getTrades();
```

TypeScript

Python

Copy

Ask AI

```
// Create client with builder's authentication headers
import { BuilderConfig, BuilderApiKeyCreds } from "@polymarket/builder-signing-sdk";

const builderCreds: BuilderApiKeyCreds = {
  key: process.env.POLY_BUILDER_API_KEY!,
  secret: process.env.POLY_BUILDER_SECRET!,
  passphrase: process.env.POLY_BUILDER_PASSPHRASE!
};

const builderConfig: BuilderConfig = {
  localBuilderCreds: builderCreds
};

const client = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer,
  creds, // User's API credentials
  2,
  funder,
  undefined,
  false,
  builderConfig // Builder's API credentials
);

// You can call all methods including builder methods
const builderTrades = await client.getBuilderTrades();
```

Learn more about the Builders Program and Relay Client here

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-overview\#resources)  Resources

[**TypeScript Client** \\
\\
Open source TypeScript client on GitHub](https://github.com/Polymarket/clob-client) [**Python Client** \\
\\
Open source Python client for GitHub](https://github.com/Polymarket/py-clob-client) [**TypeScript Examples** \\
\\
TypeScript client method examples](https://github.com/Polymarket/clob-client/tree/main/examples) [**Python Examples** \\
\\
Python client method examples](https://github.com/Polymarket/py-clob-client/tree/main/examples) [**CLOB Rest API Reference** \\
\\
Complete REST endpoint documentation](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary) [**Web Socket API** \\
\\
Real-time market data streaming](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview)

[Geographic Restrictions](https://docs.polymarket.com/developers/CLOB/geoblock) [Public Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-public)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.