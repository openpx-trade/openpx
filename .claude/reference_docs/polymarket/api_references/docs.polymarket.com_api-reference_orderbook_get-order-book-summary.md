[Skip to main content](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Orderbook

Get order book summary

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get order book summary

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/book
```

200

400

404

500

Copy

Ask AI

```
{
  "market": "0x1b6f76e5b8587ee896c35847e12d11e75290a8c3934c5952e8a9d6e4c6f03cfa",
  "asset_id": "1234567890",
  "timestamp": "2023-10-01T12:00:00Z",
  "hash": "0xabc123def456...",
  "bids": [\
    {\
      "price": "1800.50",\
      "size": "10.5"\
    }\
  ],
  "asks": [\
    {\
      "price": "1800.50",\
      "size": "10.5"\
    }\
  ],
  "min_order_size": "0.001",
  "tick_size": "0.01",
  "neg_risk": false
}
```

GET

/

book

Try it

Get order book summary

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/book
```

200

400

404

500

Copy

Ask AI

```
{
  "market": "0x1b6f76e5b8587ee896c35847e12d11e75290a8c3934c5952e8a9d6e4c6f03cfa",
  "asset_id": "1234567890",
  "timestamp": "2023-10-01T12:00:00Z",
  "hash": "0xabc123def456...",
  "bids": [\
    {\
      "price": "1800.50",\
      "size": "10.5"\
    }\
  ],
  "asks": [\
    {\
      "price": "1800.50",\
      "size": "10.5"\
    }\
  ],
  "min_order_size": "0.001",
  "tick_size": "0.01",
  "neg_risk": false
}
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#parameter-token-id)

token\_id

string

required

The unique identifier for the token

#### Response

200

application/json

Successful response

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-market)

market

string

required

Market identifier

Example:

`"0x1b6f76e5b8587ee896c35847e12d11e75290a8c3934c5952e8a9d6e4c6f03cfa"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-asset-id)

asset\_id

string

required

Asset identifier

Example:

`"1234567890"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-timestamp)

timestamp

string<date-time>

required

Timestamp of the order book snapshot

Example:

`"2023-10-01T12:00:00Z"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-hash)

hash

string

required

Hash of the order book state

Example:

`"0xabc123def456..."`

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-bids)

bids

object\[\]

required

Array of bid levels

Showchild attributes

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-asks)

asks

object\[\]

required

Array of ask levels

Showchild attributes

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-min-order-size)

min\_order\_size

string

required

Minimum order size for this market

Example:

`"0.001"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-tick-size)

tick\_size

string

required

Minimum price increment

Example:

`"0.01"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary#response-neg-risk)

neg\_risk

boolean

required

Whether negative risk is enabled

Example:

`false`

[Builder Methods](https://docs.polymarket.com/developers/CLOB/clients/methods-builder) [Get multiple order books summaries by request](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.