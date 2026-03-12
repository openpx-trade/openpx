---
url: "https://docs.polymarket.com/developers/CLOB/clients/methods-l1"
title: "L1 Methods - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

⌘KAsk AI

Search...

Navigation

Client

L1 Methods

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Client Initialization](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#client-initialization)
- [API Key Management](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#api-key-management)
- [createApiKey()](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#createapikey)
- [deriveApiKey()](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#deriveapikey)
- [createOrDeriveApiKey()](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#createorderiveapikey)
- [Order Signing](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#order-signing)
- [createOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#createorder)
- [createMarketOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#createmarketorder)
- [Troubleshooting](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#troubleshooting)
- [See Also](https://docs.polymarket.com/developers/CLOB/clients/methods-l1#see-also)

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#client-initialization)  Client Initialization

L1 methods require the client to initialize with a signer.

- TypeScript

- Python


Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { Wallet } from "ethers";

const signer = new Wallet(process.env.PRIVATE_KEY);

const client = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer // Signer required for L1 methods
);

// Ready to create user API credentials
const apiKey = await client.createApiKey();
```

Copy

Ask AI

```
from py_clob_client.client import ClobClient
import os

private_key = os.getenv("PRIVATE_KEY")

client = ClobClient(
    host="https://clob.polymarket.com",
    chain_id=137,
    key=private_key  # Signer required for L1 methods
)

# Ready to create user API credentials
api_key = await client.create_api_key()
```

**Security:** Never commit private keys to version control. Always use environment variables or secure key management systems.

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#api-key-management)  API Key Management

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#createapikey)  createApiKey()

Creates a new API key (L2 credentials) for the wallet signer. This generates a new set of credentials that can be used for L2 authenticated requests.
Each wallet can only have one active API key at a time. Creating a new key invalidates the previous one.

Signature

Copy

Ask AI

```
async createApiKey(nonce?: number): Promise<ApiKeyCreds>
```

Params

Copy

Ask AI

```
`nonce` (optional): Custom nonce for deterministic key generation. If not provided, a default derivation is used.
```

Response

Copy

Ask AI

```
interface ApiKeyCreds {
  apiKey: string;
  secret: string;
  passphrase: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#deriveapikey)  deriveApiKey()

Derives an existing API key (L2 credentials) using a specific nonce. If you’ve already created API credentials with a particular nonce, this method will return the same credentials again.

Signature

Copy

Ask AI

```
async deriveApiKey(nonce?: number): Promise<ApiKeyCreds>
```

Params

Copy

Ask AI

```
`nonce` (optional): Custom nonce for deterministic key generation. If not provided, a default derivation is used.
```

Response

Copy

Ask AI

```
interface ApiKeyCreds {
  apiKey: string;
  secret: string;
  passphrase: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#createorderiveapikey)  createOrDeriveApiKey()

Convenience method that attempts to derive an API key with the default nonce, or creates a new one if it doesn’t exist. This is the recommended method for initial setup if you’re unsure if credentials already exist.

Signature

Copy

Ask AI

```
async createOrDeriveApiKey(nonce?: number): Promise<ApiKeyCreds>
```

Params

Copy

Ask AI

```
`nonce` (optional): Custom nonce for deterministic key generation. If not provided, a default derivation is used.
```

Response

Copy

Ask AI

```
interface ApiKeyCreds {
  apiKey: string;
  secret: string;
  passphrase: string;
}
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#order-signing)  Order Signing

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#createorder)  createOrder()

Create and sign a limit order locally without posting it to the CLOB.
Use this when you want to sign orders in advance or implement custom order submission logic.
Place order via L2 methods postOrder or postOrders.

Signature

Copy

Ask AI

```
async createOrder(
  userOrder: UserOrder,
  options?: Partial<CreateOrderOptions>
): Promise<SignedOrder>
```

Params

Copy

Ask AI

```
interface UserOrder {
  tokenID: string;
  price: number;
  size: number;
  side: Side;
  feeRateBps?: number;
  nonce?: number;
  expiration?: number;
  taker?: string;
}

interface CreateOrderOptions {
  tickSize: TickSize;
  negRisk?: boolean;
}
```

Response

Copy

Ask AI

```
interface SignedOrder {
  salt: string;
  maker: string;
  signer: string;
  taker: string;
  tokenId: string;
  makerAmount: string;
  takerAmount: string;
  side: number;  // 0 = BUY, 1 = SELL
  expiration: string;
  nonce: string;
  feeRateBps: string;
  signatureType: number;
  signature: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#createmarketorder)  createMarketOrder()

Create and sign a market order locally without posting it to the CLOB.
Use this when you want to sign orders in advance or implement custom order submission logic.
Place orders via L2 methods postOrder or postOrders.

Signature

Copy

Ask AI

```
async createMarketOrder(
  userMarketOrder: UserMarketOrder,
  options?: Partial<CreateOrderOptions>
): Promise<SignedOrder>
```

Params

Copy

Ask AI

```
interface UserMarketOrder {
  tokenID: string;
  amount: number;  // BUY: dollar amount, SELL: number of shares
  side: Side;
  price?: number;  // Optional price limit
  feeRateBps?: number;
  nonce?: number;
  taker?: string;
  orderType?: OrderType.FOK | OrderType.FAK;
}
```

Response

Copy

Ask AI

```
interface SignedOrder {
  salt: string;
  maker: string;
  signer: string;
  taker: string;
  tokenId: string;
  makerAmount: string;
  takerAmount: string;
  side: number;  // 0 = BUY, 1 = SELL
  expiration: string;
  nonce: string;
  feeRateBps: string;
  signatureType: number;
  signature: string;
}
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#troubleshooting)  Troubleshooting

Error: INVALID\_SIGNATURE

Your wallet’s private key is incorrect or improperly formatted.**Solution:**

- Verify your private key is a valid hex string (starts with “0x”)
- Ensure you’re using the correct key for the intended address
- Check that the key has proper permissions

Error: NONCE\_ALREADY\_USED

The nonce you provided has already been used to create an API key.**Solution:**

- Use `deriveApiKey()` with the same nonce to retrieve existing credentials
- Or use a different nonce with `createApiKey()`

Error: Invalid Funder Address

Your funder address is incorrect or doesn’t match your wallet.**Solution:** Check your Polymarket profile address at [polymarket.com/settings](https://polymarket.com/settings).If it does not exist or user has never logged into Polymarket.com, deploy it first before creating L2 authentication.

Lost API credentials but have nonce

Copy

Ask AI

```
// Use deriveApiKey with the original nonce
const recovered = await client.deriveApiKey(originalNonce);
```

Lost both credentials and nonce

Unfortunately, there’s no way to recover lost API credentials without the nonce. You’ll need to create new credentials:

Copy

Ask AI

```
// Create fresh credentials with a new nonce
const newCreds = await client.createApiKey();
// Save the nonce this time!
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l1\#see-also)  See Also

[**Understand CLOB Authentication** \\
\\
Deep dive into L1 and L2 authentication](https://docs.polymarket.com/developers/CLOB/authentication) [**CLOB Quickstart Guide** \\
\\
Initialize the CLOB quickly and place your first order.](https://docs.polymarket.com/developers/CLOB/quickstart) [**Public Methods** \\
\\
Access market data, orderbooks, and prices.](https://docs.polymarket.com/developers/CLOB/clients/methods-l2) [**L2 Methods** \\
\\
Manage and close orders. Creating orders requires signer.](https://docs.polymarket.com/developers/CLOB/clients/methods-l2)

[Public Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-public) [L2 Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-l2)

⌘I