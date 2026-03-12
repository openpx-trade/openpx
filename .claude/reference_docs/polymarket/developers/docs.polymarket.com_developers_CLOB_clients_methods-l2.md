---
url: "https://docs.polymarket.com/developers/CLOB/clients/methods-l2"
title: "L2 Methods - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Client

L2 Methods

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Client Initialization](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#client-initialization)
- [Order Creation and Management](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#order-creation-and-management)
- [createAndPostOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#createandpostorder)
- [createAndPostMarketOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#createandpostmarketorder)
- [postOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#postorder)
- [postOrders()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#postorders)
- [cancelOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#cancelorder)
- [cancelOrders()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#cancelorders)
- [cancelAll()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#cancelall)
- [cancelMarketOrders()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#cancelmarketorders)
- [Order and Trade Queries](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#order-and-trade-queries)
- [getOrder()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#getorder)
- [getOpenOrders()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#getopenorders)
- [getTrades()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#gettrades)
- [getTradesPaginated()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#gettradespaginated)
- [Balance and Allowances](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#balance-and-allowances)
- [getBalanceAllowance()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#getbalanceallowance)
- [updateBalanceAllowance()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#updatebalanceallowance)
- [API Key Management (L2)](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#api-key-management-l2)
- [getApiKeys()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#getapikeys)
- [deleteApiKey()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#deleteapikey)
- [Notifications](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#notifications)
- [getNotifications()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#getnotifications)
- [dropNotifications()](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#dropnotifications)
- [See Also](https://docs.polymarket.com/developers/CLOB/clients/methods-l2#see-also)

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#client-initialization)  Client Initialization

L2 methods require the client to initialize with the signer, signatureType, user API credentials, and funder.

- TypeScript

- Python


Copy

Ask AI

```
import { ClobClient } from "@polymarket/clob-client";
import { Wallet } from "ethers";

const signer = new Wallet(process.env.PRIVATE_KEY)

const apiCreds = {
  apiKey: process.env.API_KEY,
  secret: process.env.SECRET,
  passphrase: process.env.PASSPHRASE,
};

const client = new ClobClient(
  "https://clob.polymarket.com",
  137,
  signer,
  apiCreds,
  2, // Deployed Safe proxy wallet
  process.env.FUNDER_ADDRESS // Address of deployed Safe proxy wallet
);

// Ready to send authenticated requests to the CLOB API!
const order = await client.postOrder(signedOrder);
```

Copy

Ask AI

```
from py_clob_client.client import ClobClient
from py_clob_client.clob_types import ApiCreds
import os

api_creds = ApiCreds(
    api_key=os.getenv("API_KEY"),
    api_secret=os.getenv("SECRET"),
    api_passphrase=os.getenv("PASSPHRASE")
)

client = ClobClient(
    host="https://clob.polymarket.com",
    chain_id=137,
    key=os.getenv("PRIVATE_KEY"),
    creds=api_creds,
    signature_type=2, # Deployed Safe proxy wallet
    funder=os.getenv("FUNDER_ADDRESS") # Address of deployed Safe proxy wallet
)

# Ready to send authenticated requests to the CLOB API!
order = await client.post_order(signed_order)
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#order-creation-and-management)  Order Creation and Management

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#createandpostorder)  createAndPostOrder()

A convenience method that creates, prompts signature, and posts an order in a single call.
Use when you want to buy/sell at a specific price and can wait.

Signature

Copy

Ask AI

```
async createAndPostOrder(
  userOrder: UserOrder,
  options?: Partial<CreateOrderOptions>,
  orderType?: OrderType.GTC | OrderType.GTD, // Defaults to GTC
): Promise<OrderResponse>
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

type CreateOrderOptions = {
  tickSize: TickSize;
  negRisk?: boolean;
}

type TickSize = "0.1" | "0.01" | "0.001" | "0.0001";
```

Response

Copy

Ask AI

```
interface OrderResponse {
  success: boolean;
  errorMsg: string;
  orderID: string;
  transactionsHashes: string[];
  status: string;
  takingAmount: string;
  makingAmount: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#createandpostmarketorder)  createAndPostMarketOrder()

A convenience method that creates, prompts signature, and posts an order in a single call.
Use when you want to buy/sell right now at whatever the market price is.

Signature

Copy

Ask AI

```
async createAndPostMarketOrder(
  userMarketOrder: UserMarketOrder,
  options?: Partial<CreateOrderOptions>,
  orderType?: OrderType.FOK | OrderType.FAK, // Defaults to FOK
): Promise<OrderResponse>
```

Params

Copy

Ask AI

```
interface UserMarketOrder {
  tokenID: string;
  amount: number;
  side: Side;
  price?: number;
  feeRateBps?: number;
  nonce?: number;
  taker?: string;
  orderType?: OrderType.FOK | OrderType.FAK;
}

type CreateOrderOptions = {
  tickSize: TickSize;
  negRisk?: boolean;
}

type TickSize = "0.1" | "0.01" | "0.001" | "0.0001";
```

Response

Copy

Ask AI

```
interface OrderResponse {
  success: boolean;
  errorMsg: string;
  orderID: string;
  transactionsHashes: string[];
  status: string;
  takingAmount: string;
  makingAmount: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#postorder)  postOrder()

Posts a pre-signed and created order to the CLOB.

Signature

Copy

Ask AI

```
async postOrder(
  order: SignedOrder,
  orderType?: OrderType, // Defaults to GTC
  postOnly?: boolean, // Defaults to false
): Promise<OrderResponse>
```

Params

Copy

Ask AI

```
order: SignedOrder  // Pre-signed order from createOrder() or createMarketOrder()
orderType?: OrderType  // Optional, defaults to GTC
postOnly?: boolean  // Optional, defaults to false
```

Response

Copy

Ask AI

```
interface OrderResponse {
  success: boolean;
  errorMsg: string;
  orderID: string;
  transactionsHashes: string[];
  status: string;
  takingAmount: string;
  makingAmount: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#postorders)  postOrders()

Posts up to 15 pre-signed and created orders in a single batch.

Copy

Ask AI

```
async postOrders(
  args: PostOrdersArgs[],
): Promise<OrderResponse[]>
```

Params

Copy

Ask AI

```
interface PostOrdersArgs {
  order: SignedOrder;
  orderType: OrderType;
  postOnly?: boolean; // Defaults to false
}
```

Response

Copy

Ask AI

```
OrderResponse[]  // Array of OrderResponse objects

interface OrderResponse {
  success: boolean;
  errorMsg: string;
  orderID: string;
  transactionsHashes: string[];
  status: string;
  takingAmount: string;
  makingAmount: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#cancelorder)  cancelOrder()

Cancels a single open order.

Signature

Copy

Ask AI

```
async cancelOrder(orderID: string): Promise<CancelOrdersResponse>
```

Response

Copy

Ask AI

```
interface CancelOrdersResponse {
  canceled: string[];
  not_canceled: Record<string, any>;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#cancelorders)  cancelOrders()

Cancels multiple orders in a single batch.

Signature

Copy

Ask AI

```
async cancelOrders(orderIDs: string[]): Promise<CancelOrdersResponse>
```

Params

Copy

Ask AI

```
orderIDs: string[];
```

Response

Copy

Ask AI

```
interface CancelOrdersResponse {
  canceled: string[];
  not_canceled: Record<string, any>;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#cancelall)  cancelAll()

Cancels all open orders.

Signature

Copy

Ask AI

```
async cancelAll(): Promise<CancelResponse>
```

Response

Copy

Ask AI

```
interface CancelOrdersResponse {
  canceled: string[];
  not_canceled: Record<string, any>;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#cancelmarketorders)  cancelMarketOrders()

Cancels all open orders for a specific market.

Signature

Copy

Ask AI

```
async cancelMarketOrders(
  payload: OrderMarketCancelParams
): Promise<CancelOrdersResponse>
```

Parameters

Copy

Ask AI

```
interface OrderMarketCancelParams {
  market?: string;
  asset_id?: string;
}
```

Response

Copy

Ask AI

```
interface CancelOrdersResponse {
  canceled: string[];
  not_canceled: Record<string, any>;
}
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#order-and-trade-queries)  Order and Trade Queries

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#getorder)  getOrder()

Get details for a specific order.

Signature

Copy

Ask AI

```
async getOrder(orderID: string): Promise<OpenOrder>
```

Response

Copy

Ask AI

```
interface OpenOrder {
  id: string;
  status: string;
  owner: string;
  maker_address: string;
  market: string;
  asset_id: string;
  side: string;
  original_size: string;
  size_matched: string;
  price: string;
  associate_trades: string[];
  outcome: string;
  created_at: number;
  expiration: string;
  order_type: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#getopenorders)  getOpenOrders()

Get all your open orders.

Signature

Copy

Ask AI

```
async getOpenOrders(
  params?: OpenOrderParams,
  only_first_page?: boolean,
): Promise<OpenOrdersResponse>
```

Params

Copy

Ask AI

```
interface OpenOrderParams {
  id?: string; // Order ID
  market?: string; // Market condition ID
  asset_id?: string; // Token ID
}

only_first_page?: boolean  // Defaults to false
```

Response

Copy

Ask AI

```
type OpenOrdersResponse = OpenOrder[];

interface OpenOrder {
  id: string;
  status: string;
  owner: string;
  maker_address: string;
  market: string;
  asset_id: string;
  side: string;
  original_size: string;
  size_matched: string;
  price: string;
  associate_trades: string[];
  outcome: string;
  created_at: number;
  expiration: string;
  order_type: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#gettrades)  getTrades()

Get your trade history (filled orders).

Signature

Copy

Ask AI

```
async getTrades(
  params?: TradeParams,
  only_first_page?: boolean,
): Promise<Trade[]>
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

only_first_page?: boolean  // Defaults to false
```

Response

Copy

Ask AI

```
interface Trade {
  id: string;
  taker_order_id: string;
  market: string;
  asset_id: string;
  side: Side;
  size: string;
  fee_rate_bps: string;
  price: string;
  status: string;
  match_time: string;
  last_update: string;
  outcome: string;
  bucket_index: number;
  owner: string;
  maker_address: string;
  maker_orders: MakerOrder[];
  transaction_hash: string;
  trader_side: "TAKER" | "MAKER";
}

interface MakerOrder {
  order_id: string;
  owner: string;
  maker_address: string;
  matched_amount: string;
  price: string;
  fee_rate_bps: string;
  asset_id: string;
  outcome: string;
  side: Side;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#gettradespaginated)  getTradesPaginated()

Get trade history with pagination for large result sets.

Signature

Copy

Ask AI

```
async getTradesPaginated(
  params?: TradeParams,
): Promise<TradesPaginatedResponse>
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
interface TradesPaginatedResponse {
  trades: Trade[];
  limit: number;
  count: number;
}
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#balance-and-allowances)  Balance and Allowances

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#getbalanceallowance)  getBalanceAllowance()

Get your balance and allowance for specific tokens.

Signature

Copy

Ask AI

```
async getBalanceAllowance(
  params?: BalanceAllowanceParams
): Promise<BalanceAllowanceResponse>
```

Params

Copy

Ask AI

```
interface BalanceAllowanceParams {
  asset_type: AssetType;
  token_id?: string;
}

enum AssetType {
  COLLATERAL = "COLLATERAL",
  CONDITIONAL = "CONDITIONAL",
}
```

Response

Copy

Ask AI

```
interface BalanceAllowanceResponse {
  balance: string;
  allowance: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#updatebalanceallowance)  updateBalanceAllowance()

Updates the cached balance and allowance for specific tokens.

Signature

Copy

Ask AI

```
async updateBalanceAllowance(
  params?: BalanceAllowanceParams
): Promise<void>
```

Params

Copy

Ask AI

```
interface BalanceAllowanceParams {
  asset_type: AssetType;
  token_id?: string;
}

enum AssetType {
  COLLATERAL = "COLLATERAL",
  CONDITIONAL = "CONDITIONAL",
}
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#api-key-management-l2)  API Key Management (L2)

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#getapikeys)  getApiKeys()

Get all API keys associated with your account.

Signature

Copy

Ask AI

```
async getApiKeys(): Promise<ApiKeysResponse>
```

Response

Copy

Ask AI

```
interface ApiKeysResponse {
  apiKeys: ApiKeyCreds[];
}

interface ApiKeyCreds {
  key: string;
  secret: string;
  passphrase: string;
}
```

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#deleteapikey)  deleteApiKey()

Deletes (revokes) the currently authenticated API key.**TypeScript Signature:**

Copy

Ask AI

```
async deleteApiKey(): Promise<any>
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#notifications)  Notifications

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#getnotifications)  getNotifications()

Retrieves all event notifications for the L2 authenticated user.
Records are removed automatically after 48 hours or if manually removed via dropNotifications().

Signature

Copy

Ask AI

```
public async getNotifications(): Promise<Notification[]>
```

Response

Copy

Ask AI

```
interface Notification {
    id: number;           // Unique notification ID
    owner: string;        // User's L2 credential apiKey or empty string for global notifications
    payload: any;         // Type-specific payload data
    timestamp?: number;   // Unix timestamp
    type: number;         // Notification type (see type mapping below)
}
```

**Notification Type Mapping**

| Name | Value | Description |
| --- | --- | --- |
| Order Cancellation | 1 | User’s order was canceled |
| Order Fill | 2 | User’s order was filled (maker or taker) |
| Market Resolved | 4 | Market was resolved |

* * *

### [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#dropnotifications)  dropNotifications()

Mark notifications as read/dismissed.

Signature

Copy

Ask AI

```
public async dropNotifications(params?: DropNotificationParams): Promise<void>
```

Params

Copy

Ask AI

```
interface DropNotificationParams {
    ids: string[];  // Array of notification IDs to mark as read
}
```

* * *

## [​](https://docs.polymarket.com/developers/CLOB/clients/methods-l2\#see-also)  See Also

[**Understand CLOB Authentication** \\
\\
Deep dive into L1 and L2 authentication](https://docs.polymarket.com/developers/CLOB/authentication) [**Public Methods** \\
\\
Access market data, orderbooks, and prices.](https://docs.polymarket.com/developers/CLOB/clients/methods-l2) [**L1 Methods** \\
\\
Private key authentication to create or derive API keys (L2 headers)](https://docs.polymarket.com/developers/CLOB/clients/methods-l2) [**Web Socket API** \\
\\
Real-time market data streaming](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview)

[L1 Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-l1) [Builder Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-builder)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.