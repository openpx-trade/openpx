---
url: "https://docs.polymarket.com/developers/CLOB/orders/get-active-order"
title: "Get Active Orders - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/orders/get-active-order#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Order Management

Get Active Orders

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Python

Typescript

Copy

Ask AI

```
from py_clob_client.clob_types import OpenOrderParams

resp = client.get_orders(
    OpenOrderParams(
        market="0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
    )
)
print(resp)
print("Done!")
```

This endpoint requires a L2 Header.

Get active order(s) for a specific market.**HTTP REQUEST**`GET /<clob-endpoint>/data/orders`

### [​](https://docs.polymarket.com/developers/CLOB/orders/get-active-order\#request-parameters)  Request Parameters

| Name | Required | Type | Description |
| --- | --- | --- | --- |
| id | no | string | id of order to get information about |
| market | no | string | condition id of market |
| asset\_id | no | string | id of the asset/token |

### [​](https://docs.polymarket.com/developers/CLOB/orders/get-active-order\#response-format)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| null | OpenOrder\[\] | list of open orders filtered by the query parameters |

[Get Order](https://docs.polymarket.com/developers/CLOB/orders/get-order) [Check Order Reward Scoring](https://docs.polymarket.com/developers/CLOB/orders/check-scoring)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.