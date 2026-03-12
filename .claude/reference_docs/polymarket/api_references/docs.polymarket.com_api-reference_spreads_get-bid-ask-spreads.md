[Skip to main content](https://docs.polymarket.com/api-reference/spreads/get-bid-ask-spreads#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Spreads

Get bid-ask spreads

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get bid-ask spreads

cURL

Copy

Ask AI

```
curl --request POST \
  --url https://clob.polymarket.com/spreads \
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
{
  "1234567890": "0.50",
  "0987654321": "0.05"
}
```

POST

/

spreads

Try it

Get bid-ask spreads

cURL

Copy

Ask AI

```
curl --request POST \
  --url https://clob.polymarket.com/spreads \
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
{
  "1234567890": "0.50",
  "0987654321": "0.05"
}
```

#### Body

application/json

Maximum array length: `500`

[​](https://docs.polymarket.com/api-reference/spreads/get-bid-ask-spreads#body-items-token-id)

token\_id

string

required

The unique identifier for the token

Example:

`"1234567890"`

[​](https://docs.polymarket.com/api-reference/spreads/get-bid-ask-spreads#body-items-side)

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

Map of token\_id to spread value

[​](https://docs.polymarket.com/api-reference/spreads/get-bid-ask-spreads#response-additional-properties)

{key}

string

[Get price history for a traded token](https://docs.polymarket.com/api-reference/pricing/get-price-history-for-a-traded-token) [Historical Timeseries Data](https://docs.polymarket.com/developers/CLOB/timeseries)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.