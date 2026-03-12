---
url: "https://docs.polymarket.com/developers/CLOB/orders/check-scoring"
title: "Check Order Reward Scoring - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/orders/check-scoring#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Order Management

Check Order Reward Scoring

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Python

Typescript

Copy

Ask AI

```
scoring = client.is_order_scoring(
    OrderScoringParams(
        orderId="0x..."
    )
)
print(scoring)

scoring = client.are_orders_scoring(
    OrdersScoringParams(
        orderIds=["0x..."]
    )
)
print(scoring)
```

This endpoint requires a L2 Header.

Returns a boolean value where it is indicated if an order is scoring or not.**HTTP REQUEST**`GET /<clob-endpoint>/order-scoring?order_id={...}`

### [​](https://docs.polymarket.com/developers/CLOB/orders/check-scoring\#request-parameters)  Request Parameters

| Name | Required | Type | Description |
| --- | --- | --- | --- |
| orderId | yes | string | id of order to get information about |

### [​](https://docs.polymarket.com/developers/CLOB/orders/check-scoring\#response-format)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| null | OrdersScoring | order scoring data |

An `OrdersScoring` object is of the form:

| Name | Type | Description |
| --- | --- | --- |
| scoring | boolean | indicates if the order is scoring or not |

# [​](https://docs.polymarket.com/developers/CLOB/orders/check-scoring\#check-if-some-orders-are-scoring)  Check if some orders are scoring

> This endpoint requires a L2 Header.

Returns to a dictionary with boolean value where it is indicated if an order is scoring or not.**HTTP REQUEST**`POST /<clob-endpoint>/orders-scoring`

### [​](https://docs.polymarket.com/developers/CLOB/orders/check-scoring\#request-parameters-2)  Request Parameters

| Name | Required | Type | Description |
| --- | --- | --- | --- |
| orderIds | yes | string\[\] | ids of the orders to get information about |

### [​](https://docs.polymarket.com/developers/CLOB/orders/check-scoring\#response-format-2)  Response Format

| Name | Type | Description |
| --- | --- | --- |
| null | OrdersScoring | orders scoring data |

An `OrdersScoring` object is a dictionary that indicates the order by if it score.

[Get Active Orders](https://docs.polymarket.com/developers/CLOB/orders/get-active-order) [Cancel Orders(s)](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.