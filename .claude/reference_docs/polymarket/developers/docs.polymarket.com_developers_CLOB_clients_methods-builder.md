---
url: "https://docs.polymarket.com/developers/CLOB/clients/methods-builder"
title: "Builder Methods - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/clients/methods-builder#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Client

Builder Methods

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Client Initialization](https://docs.polymarket.com/developers/CLOB/clients/methods-builder#client-initialization)
- [Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-builder#methods)
- [getBuilderTrades()](https://docs.polymarket.com/developers/CLOB/clients/methods-builder#getbuildertrades)
- [revokeBuilderApiKey()](https://docs.polymarket.com/developers/CLOB/clients/methods-builder#revokebuilderapikey)
- [See Also](https://docs.polymarket.com/developers/CLOB/clients/methods-builder#see-also)

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-builder\#client-initialization)  Client Initialization

Builder methods require the client to initialize with a separate authentication setup using
builder configs acquired from [Polymarket.com](https://polymarket.com/settings?tab=builder)
and the `@polymarket/builder-signing-sdk` package.

- Local Builder Credentials

- Remote Builder Signing


TypeScript

Python

Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { BuilderConfig, BuilderApiKeyCreds } from "@polymarket/builder-signing-sdk";

const builderConfig = new BuilderConfig({
  localBuilderCreds: new BuilderApiKeyCreds({
    key: process.env.BUILDER_API_KEY,
    secret: process.env.BUILDER_SECRET,
    passphrase: process.env.BUILDER_PASS_PHRASE,
  }),
});

const clobClient = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer,
  apiCreds, // The user's API credentials generated from L1 authentication
  signatureType,
  funderAddress,
  undefined,
  false,
  builderConfig
);
```

TypeScript

Python

Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { BuilderConfig } from "@polymarket/builder-signing-sdk";

const builderConfig = new BuilderConfig({
    remoteBuilderConfig: {url: "http://localhost:3000/sign"}
});

const clobClient = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer,
  apiCreds, // The user's API credentials generated from L1 authentication
  signatureType,
  funder,
  undefined,
  false,
  builderConfig
);
```

[More information on builder signing](https://docs.polymarket.com/developers/builders/order-attribution)

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-builder\#methods)  Methods

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-builder\#getbuildertrades)  getBuilderTrades()

Retrieves all trades attributed to your builder account.
This method allows builders to track which trades were routed through your platform.

Signature

Copy

Ask AI

```
async getBuilderTrades(
  params?: TradeParams,
): Promise<BuilderTradesPaginatedResponse>
```

Params

Copy

Ask AI

```
interface TradeParams {
  id?: string;
  maker_address?: string;
  market?: string;
  asset_id?: string;
  before?: string;
  after?: string;
}
```

Response

Copy

Ask AI

```
interface BuilderTradesPaginatedResponse {
  trades: BuilderTrade[];
  next_cursor: string;
  limit: number;
  count: number;
}

interface BuilderTrade {
  id: string;
  tradeType: string;
  takerOrderHash: string;
  builder: string;
  market: string;
  assetId: string;
  side: string;
  size: string;
  sizeUsdc: string;
  price: string;
  status: string;
  outcome: string;
  outcomeIndex: number;
  owner: string;
  maker: string;
  transactionHash: string;
  matchTime: string;
  bucketIndex: number;
  fee: string;
  feeUsdc: string;
  err_msg?: string | null;
  createdAt: string | null;
  updatedAt: string | null;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-builder\#revokebuilderapikey)  revokeBuilderApiKey()

Revokes the builder API key used to authenticate the current request.
After revocation, the key can no longer be used to make builder-authenticated requests.

Signature

Copy

Ask AI

```
async revokeBuilderApiKey(): Promise<any>
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-builder\#see-also)  See Also

[**Builders Program Introduction** \\
\\
Learn the benefits, how to implement, and more.](https://docs.polymarket.com/developers/builders/builder-intro) [**Implement Builders Signing** \\
\\
Attribute orders to you, and pre-requisite to using the Relayer Client.](https://docs.polymarket.com/developers/builders/order-attribution) [**Relayer Client** \\
\\
The relayer executes other gasless transactions for your users, on your app.](https://docs.polymarket.com/developers/builders/relayer-client) [**Full Example Implementations** \\
\\
Complete Next.js examples integrated with embedded wallets (Privy, Magic, Turnkey, wagmi)](https://docs.polymarket.com/developers/builders/examples)

[L2 Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-l2) [Get order book summary](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.