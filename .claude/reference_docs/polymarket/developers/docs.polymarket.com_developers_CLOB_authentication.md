---
url: "https://docs.polymarket.com/developers/CLOB/authentication"
title: "Authentication - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/authentication#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Central Limit Order Book

Authentication

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Authentication Levels](https://docs.polymarket.com/developers/CLOB/authentication#authentication-levels)
- [L1 Authentication](https://docs.polymarket.com/developers/CLOB/authentication#l1-authentication)
- [What is L1?](https://docs.polymarket.com/developers/CLOB/authentication#what-is-l1)
- [What This Enables](https://docs.polymarket.com/developers/CLOB/authentication#what-this-enables)
- [CLOB Client](https://docs.polymarket.com/developers/CLOB/authentication#clob-client)
- [REST API](https://docs.polymarket.com/developers/CLOB/authentication#rest-api)
- [L2 Authentication](https://docs.polymarket.com/developers/CLOB/authentication#l2-authentication)
- [What is L2?](https://docs.polymarket.com/developers/CLOB/authentication#what-is-l2)
- [What This Enables](https://docs.polymarket.com/developers/CLOB/authentication#what-this-enables-2)
- [CLOB Client](https://docs.polymarket.com/developers/CLOB/authentication#clob-client-2)
- [REST API](https://docs.polymarket.com/developers/CLOB/authentication#rest-api-2)
- [Signature Types and Funder](https://docs.polymarket.com/developers/CLOB/authentication#signature-types-and-funder)
- [Troubleshooting](https://docs.polymarket.com/developers/CLOB/authentication#troubleshooting)
- [See Client Methods](https://docs.polymarket.com/developers/CLOB/authentication#see-client-methods)

The CLOB uses two levels of authentication: **L1 (Private Key)** and **L2 (API Key)**.
Either can be accomplished using the CLOB client or REST API. Authentication is not
required to access client public methods and public endpoints.

## [​](https://docs.polymarket.com/developers/CLOB/authentication\#authentication-levels)  Authentication Levels

[**L1 Authentication** \\
\\
Use the private key of the user’s account to sign messages](https://docs.polymarket.com/developers/CLOB/authentication#l1-authentication) [**L2 Authentication** \\
\\
Use API credentials (key, secret, passphrase) to authenticate requests to the CLOB](https://docs.polymarket.com/developers/CLOB/authentication#l2-authentication)

* * *

## [​](https://docs.polymarket.com/developers/CLOB/authentication\#l1-authentication)  L1 Authentication

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#what-is-l1)  What is L1?

L1 authentication uses the wallet’s private key to sign an EIP-712 message used in the
request header. It proves ownership and control over the private key. The private key
stays in control of the user and all trading activity remains non-custodial.

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#what-this-enables)  What This Enables

Access to L1 methods that create or derive L2 authentication headers.

- Create user API credentials
- Derive existing user API credentials
- Sign/create user’s orders locally

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#clob-client)  CLOB Client

- TypeScript

- Python


Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { Wallet } from "ethers"; // v5.8.0

const HOST = "https://clob.polymarket.com";
const CHAIN_ID = 137; // Polygon mainnet
const signer = new Wallet(process.env.PRIVATE_KEY);

const client = new ClobClient(
  HOST,
  CHAIN_ID,
  signer // Signer enables L1 methods
);

// Gets API key, or else creates
const apiCreds = await client.createOrDeriveApiKey();

/*
apiCreds = {
  "apiKey": "550e8400-e29b-41d4-a716-446655440000",
  "secret": "base64EncodedSecretString",
  "passphrase": "randomPassphraseString"
}
*/
```

Copy

Ask AI

```
from py_clob_client.client import ClobClient
import os

host = "https://clob.polymarket.com"
chain_id = 137 # Polygon mainnet
private_key = os.getenv("PRIVATE_KEY")

client = ClobClient(
    host=host,
    chain_id=chaind_id,
    key=private_key  # Signer enables L1 methods
)

# Gets API key, or else creates
api_creds = await client.create_or_derive_api_key()

# api_creds = {
#     "apiKey": "550e8400-e29b-41d4-a716-446655440000",
#     "secret": "base64EncodedSecretString",
#     "passphrase": "randomPassphraseString"
# }
```

**Never commit private keys to version control.** Always use environment
variables or secure key management systems.

* * *

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#rest-api)  REST API

While we highly recommend using our provided clients to handle signing
and authentication, the following is for developers who choose NOT to
use our [Python](https://github.com/Polymarket/py-clob-client) or
[TypeScript](https://github.com/Polymarket/clob-client) clients.When making direct REST API calls with L1 authentication, include these headers:

| Header | Required? | Description |
| --- | --- | --- |
| `POLY_ADDRESS` | yes | Polygon signer address |
| `POLY_SIGNATURE` | yes | CLOB EIP 712 signature |
| `POLY_TIMESTAMP` | yes | Current UNIX timestamp |
| `POLY_NONCE` | yes | Nonce. Default 0 |

The `POLY_SIGNATURE` is generated by signing the following EIP-712 struct.

EIP-712 Signing Example

Typescript

Python

Copy

Ask AI

```
const domain = {
  name: "ClobAuthDomain",
  version: "1",
  chainId: chainId, // Polygon Chain ID 137
};

const types = {
  ClobAuth: [\
    { name: "address", type: "address" },\
    { name: "timestamp", type: "string" },\
    { name: "nonce", type: "uint256" },\
    { name: "message", type: "string" },\
  ],
};

const value = {
  address: signingAddress, // The Signing address
  timestamp: ts,            // The CLOB API server timestamp
  nonce: nonce,             // The nonce used
  message: "This message attests that I control the given wallet",
};

const sig = await signer._signTypedData(domain, types, value);
```

Reference implementations:

- [TypeScript](https://github.com/Polymarket/clob-client/blob/main/src/signing/eip712.ts)
- [Python](https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/signing/eip712.py)

* * *

**Create API Credentials**Create new API credentials for user.

Copy

Ask AI

```
POST {clob-endpoint}/auth/api-key
```

**Derive API Credentials**Derive API credentials for user.

Copy

Ask AI

```
GET {clob-endpoint}/auth/derive-api-key
```

**Response**

Copy

Ask AI

```
{
  "apiKey": "550e8400-e29b-41d4-a716-446655440000",
  "secret": "base64EncodedSecretString",
  "passphrase": "randomPassphraseString"
}
```

**You’ll need all three values for L2 authentication.**

* * *

## [​](https://docs.polymarket.com/developers/CLOB/authentication\#l2-authentication)  L2 Authentication

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#what-is-l2)  What is L2?

The next level of authentication is called L2, and it consists of the
user’s API credentials (apiKey, secret, passphrase) generated from L1
authentication. These are used solely to authenticate requests made to
the CLOB API. Requests are signed using HMAC-SHA256.

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#what-this-enables-2)  What This Enables

Access to L2 methods such as posting signed/created orders, viewing open
orders, cancelling open orders, getting trades

- Cancel or get user’s open orders
- Check user’s balances and allowances
- Post user’s signed orders

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#clob-client-2)  CLOB Client

- TypeScript

- Python


Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { Wallet } from "ethers"; // v5.8.0

const HOST = "https://clob.polymarket.com";
const CHAIN_ID = 137; // Polygon mainnet
const signer = new Wallet(process.env.PRIVATE_KEY);

const client = new ClobClient(
  HOST,
  CHAIN_ID,
  signer,
  apiCreds, // Generated from L1 auth, API credentials enable L2 methods
  1, // signatureType explained below
  FUNDER // funder explained below
);

// Now you can trade!*
const order = await client.createAndPostOrder(
  { tokenID: "123456", price: 0.65, size: 100, side: "BUY" },
  { tickSize: "0.01", negRisk: false }
);
```

Copy

Ask AI

```
from py_clob_client.client import ClobClient
import os

host = "https://clob.polymarket.com"
chain_id = 137 # Polygon mainnet
private_key = os.getenv("PRIVATE_KEY")

client = ClobClient(
    host="https://clob.polymarket.com",
    chain_id=137,
    key=os.getenv("PRIVATE_KEY"),
    creds=api_creds,  # Generated from L1 auth, API credentials enable L2 methods
    signature_type=1,  # signatureType explained below
    funder=os.getenv("FUNDER_ADDRESS") # funder explained below
)

# Now you can trade!*
order = await client.create_and_post_order(
    {"token_id": "123456", "price": 0.65, "size": 100, "side": "BUY"},
    {"tick_size": "0.01", "neg_risk": False}
)
```

Even with L2 authentication headers, methods that create user orders still require the user to sign the order payload.

* * *

### [​](https://docs.polymarket.com/developers/CLOB/authentication\#rest-api-2)  REST API

While we highly recommend using our provided clients to handle signing
and authentication, the following is for developers who choose NOT to
use our [Python](https://github.com/Polymarket/py-clob-client) or
[TypeScript](https://github.com/Polymarket/clob-client) clients.When making direct REST API calls with L2 authentication, include these headers:

| Header | Required? | Description |
| --- | --- | --- |
| `POLY_ADDRESS` | yes | Polygon signer address |
| `POLY_SIGNATURE` | yes | HMAC signature for request |
| `POLY_TIMESTAMP` | yes | Current UNIX timestamp |
| `POLY_API_KEY` | yes | User’s API `apiKey` value |
| `POLY_PASSPHRASE` | yes | User’s API `passphrase` value |

The `POLY_SIGNATURE` for L2 is an HMAC-SHA256 signature created using the user’s API credentials `secret` value.
Reference implementations can be found in the [Typescript](https://github.com/Polymarket/clob-client/blob/main/src/signing/hmac.ts)
and [Python](https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/signing/hmac.py) clients.

* * *

## [​](https://docs.polymarket.com/developers/CLOB/authentication\#signature-types-and-funder)  Signature Types and Funder

When initializing the L2 client, you must specify your wallet **signatureType** and the **funder** address which holds the funds:

| Signature Type | Value | Description |
| --- | --- | --- |
| EOA | 0 | Standard Ethereum wallet (MetaMask). Funder is the EOA address and will need POL to pay gas on transactions. |
| POLY\_PROXY | 1 | A custom proxy wallet only used with users who logged in via Magic Link email/Google. Using this requires the user to have exported their PK from Polymarket.com and imported into your app. |
| GNOSIS\_SAFE | 2 | Gnosis Safe multisig proxy wallet (most common). Use this for any new or returning user who does not fit the other 2 types. |

The wallet addresses displayed to the user on Polymarket.com is the proxy wallet and should be used as the funder.
These can be deterministically derived or you can deploy them on behalf of the user.
These proxy wallets are automatically deployed for the user on their first login to Polymarket.com.

* * *

## [​](https://docs.polymarket.com/developers/CLOB/authentication\#troubleshooting)  Troubleshooting

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

## [​](https://docs.polymarket.com/developers/CLOB/authentication\#see-client-methods)  See Client Methods

[**Public Methods** \\
\\
Access market data, orderbooks, and prices.](https://docs.polymarket.com/developers/CLOB/clients/methods-public) [**L1 Methods** \\
\\
Private key authentication to create or derive API keys (L2 headers).](https://docs.polymarket.com/developers/CLOB/clients/methods-l1) [**L2 Methods** \\
\\
Manage and close orders. Creating orders requires signer.](https://docs.polymarket.com/developers/CLOB/clients/methods-l2) [**Builder Program Methods** \\
\\
Builder-specific operations for those in the Builders Program.](https://docs.polymarket.com/developers/CLOB/clients/methods-builder)

[Quickstart](https://docs.polymarket.com/developers/CLOB/quickstart) [Geographic Restrictions](https://docs.polymarket.com/developers/CLOB/geoblock)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.