[Skip to main content](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Orderbook

Get multiple order books summaries by request

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get multiple order books summaries by request

cURL

Copy

Ask AI

```
curl --request POST \
  --url https://clob.polymarket.com/books \
  --header 'Content-Type: application/json' \
  --data '
[\
  {\
    "token_id": "1234567890"\
  },\
  {\
    "token_id": "0987654321"\
  }\
]
'
```

200

Example

Copy

Ask AI

```
[\
  {\
    "market": "0x1b6f76e5b8587ee896c35847e12d11e75290a8c3934c5952e8a9d6e4c6f03cfa",\
    "asset_id": "1234567890",\
    "timestamp": "2023-10-01T12:00:00Z",\
    "hash": "0xabc123def456...",\
    "bids": [\
      {\
        "price": "1800.50",\
        "size": "10.5"\
      }\
    ],\
    "asks": [\
      {\
        "price": "1800.50",\
        "size": "10.5"\
      }\
    ],\
    "min_order_size": "0.001",\
    "tick_size": "0.01",\
    "neg_risk": false\
  }\
]
```

POST

/

books

Try it

Get multiple order books summaries by request

cURL

Copy

Ask AI

```
curl --request POST \
  --url https://clob.polymarket.com/books \
  --header 'Content-Type: application/json' \
  --data '
[\
  {\
    "token_id": "1234567890"\
  },\
  {\
    "token_id": "0987654321"\
  }\
]
'
```

200

Example

Copy

Ask AI

```
[\
  {\
    "market": "0x1b6f76e5b8587ee896c35847e12d11e75290a8c3934c5952e8a9d6e4c6f03cfa",\
    "asset_id": "1234567890",\
    "timestamp": "2023-10-01T12:00:00Z",\
    "hash": "0xabc123def456...",\
    "bids": [\
      {\
        "price": "1800.50",\
        "size": "10.5"\
      }\
    ],\
    "asks": [\
      {\
        "price": "1800.50",\
        "size": "10.5"\
      }\
    ],\
    "min_order_size": "0.001",\
    "tick_size": "0.01",\
    "neg_risk": false\
  }\
]
```

#### Body

application/json

Maximum array length: `500`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#body-items-token-id)

token\_id

string

required

The unique identifier for the token

Example:

`"1234567890"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#body-items-side)

side

enum<string>

Optional side parameter for certain operations

Available options:

`BUY`,

`SELL`

Example:

`"BUY"`

#### Response

200

application/json

Successful response

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-market)

market

string

required

Market identifier

Example:

`"0x1b6f76e5b8587ee896c35847e12d11e75290a8c3934c5952e8a9d6e4c6f03cfa"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-asset-id)

asset\_id

string

required

Asset identifier

Example:

`"1234567890"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-timestamp)

timestamp

string<date-time>

required

Timestamp of the order book snapshot

Example:

`"2023-10-01T12:00:00Z"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-hash)

hash

string

required

Hash of the order book state

Example:

`"0xabc123def456..."`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-bids)

bids

object\[\]

required

Array of bid levels

Showchild attributes

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-asks)

asks

object\[\]

required

Array of ask levels

Showchild attributes

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-min-order-size)

min\_order\_size

string

required

Minimum order size for this market

Example:

`"0.001"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-tick-size)

tick\_size

string

required

Minimum price increment

Example:

`"0.01"`

[​](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request#response-items-neg-risk)

neg\_risk

boolean

required

Whether negative risk is enabled

Example:

`false`

[Get order book summary](https://docs.polymarket.com/api-reference/orderbook/get-order-book-summary) [Get market price](https://docs.polymarket.com/api-reference/pricing/get-market-price)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.