---
url: "https://docs.polymarket.com/developers/CLOB/orders/cancel-orders"
title: "Cancel Orders(s) - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Order Management

Cancel Orders(s)

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Cancel an single Order](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#cancel-an-single-order)
- [Request Payload Parameters](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#request-payload-parameters)
- [Response Format](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#response-format)
- [Cancel Multiple Orders](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#cancel-multiple-orders)
- [Request Payload Parameters](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#request-payload-parameters-2)
- [Response Format](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#response-format-2)
- [Cancel ALL Orders](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#cancel-all-orders)
- [Response Format](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#response-format-3)
- [Cancel orders from market](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#cancel-orders-from-market)
- [Request Payload Parameters](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#request-payload-parameters-3)
- [Response Format](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders#response-format-4)

# [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#cancel-an-single-order)  Cancel an single Order

This endpoint requires a L2 Header.

Cancel an order.**HTTP REQUEST**`DELETE /<clob-endpoint>/order`

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#request-payload-parameters)  Request Payload Parameters

| Name | Required | Type | Description |
| --- | --- | --- | --- |
| orderID | yes | string | ID of order to cancel |

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#response-format)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| canceled | string\[\] | list of canceled orders |
| not\_canceled |  | a order id -> reason map that explains why that order couldn’t be canceled |

Python

Typescript

Copy

Ask AI

```
resp = client.cancel(order_id="0x38a73eed1e6d177545e9ab027abddfb7e08dbe975fa777123b1752d203d6ac88")
print(resp)
```

# [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#cancel-multiple-orders)  Cancel Multiple Orders

This endpoint requires a L2 Header.

**HTTP REQUEST**`DELETE /<clob-endpoint>/orders`

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#request-payload-parameters-2)  Request Payload Parameters

| Name | Required | Type | Description |
| --- | --- | --- | --- |
| null | yes | string\[\] | IDs of the orders to cancel |

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#response-format-2)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| canceled | string\[\] | list of canceled orders |
| not\_canceled |  | a order id -> reason map that explains why that order couldn’t be canceled |

Python

Typescript

Copy

Ask AI

```
resp = client.cancel_orders(["0x38a73eed1e6d177545e9ab027abddfb7e08dbe975fa777123b1752d203d6ac88", "0xaaaa..."])
print(resp)
```

# [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#cancel-all-orders)  Cancel ALL Orders

This endpoint requires a L2 Header.

Cancel all open orders posted by a user.**HTTP REQUEST**`DELETE /<clob-endpoint>/cancel-all`

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#response-format-3)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| canceled | string\[\] | list of canceled orders |
| not\_canceled |  | a order id -> reason map that explains why that order couldn’t be canceled |

Python

Typescript

Copy

Ask AI

```
resp = client.cancel_all()
print(resp)
print("Done!")
```

# [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#cancel-orders-from-market)  Cancel orders from market

This endpoint requires a L2 Header.

Cancel orders from market.**HTTP REQUEST**`DELETE /<clob-endpoint>/cancel-market-orders`

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#request-payload-parameters-3)  Request Payload Parameters

| Name | Required | Type | Description |
| --- | --- | --- | --- |
| market | no | string | condition id of the market |
| asset\_id | no | string | id of the asset/token |

### [​](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders\#response-format-4)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| canceled | string\[\] | list of canceled orders |
| not\_canceled |  | a order id -> reason map that explains why that order couldn’t be canceled |

Python

Typescript

Copy

Ask AI

```
resp = client.cancel_market_orders(market="0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af", asset_id="52114319501245915516055106046884209969926127482827954674443846427813813222426")
print(resp)
```

[Check Order Reward Scoring](https://docs.polymarket.com/developers/CLOB/orders/check-scoring) [Onchain Order Info](https://docs.polymarket.com/developers/CLOB/orders/onchain-order-info)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.