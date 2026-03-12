[Skip to main content](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices-by-request#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Pricing

Get multiple market prices by request

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get multiple market prices by request

cURL

Copy

Ask AI

```
curl --request POST \
  --url https://clob.polymarket.com/prices \
  --header 'Content-Type: application/json' \
  --data '
[\
  {\
    "token_id": "1234567890",\
    "side": "BUY"\
  },\
  {\
    "token_id": "0987654321",\
    "side": "SELL"\
  }\
]
'
```

200

Example

Copy

Ask AI

```
{
  "1234567890": {
    "BUY": "1800.50",
    "SELL": "1801.00"
  },
  "0987654321": {
    "BUY": "50.25",
    "SELL": "50.30"
  }
}
```

POST

/

prices

Try it

Get multiple market prices by request

cURL

Copy

Ask AI

```
curl --request POST \
  --url https://clob.polymarket.com/prices \
  --header 'Content-Type: application/json' \
  --data '
[\
  {\
    "token_id": "1234567890",\
    "side": "BUY"\
  },\
  {\
    "token_id": "0987654321",\
    "side": "SELL"\
  }\
]
'
```

200

Example

Copy

Ask AI

```
{
  "1234567890": {
    "BUY": "1800.50",
    "SELL": "1801.00"
  },
  "0987654321": {
    "BUY": "50.25",
    "SELL": "50.30"
  }
}
```

#### Body

application/json

Maximum array length: `500`

[​](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices-by-request#body-items-token-id)

token\_id

string

required

The unique identifier for the token

Example:

`"1234567890"`

[​](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices-by-request#body-items-side)

side

enum<string>

required

The side of the market (BUY or SELL)

Available options:

`BUY`,

`SELL`

Example:

`"BUY"`

#### Response

200

application/json

Successful response

Map of token\_id to side to price

[​](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices-by-request#response-additional-properties)

{key}

object

Showchild attributes

[Get multiple market prices](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices) [Get midpoint price](https://docs.polymarket.com/api-reference/pricing/get-midpoint-price)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.