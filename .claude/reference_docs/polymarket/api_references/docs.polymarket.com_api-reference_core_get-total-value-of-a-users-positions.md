[Skip to main content](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get total value of a user's positions

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get total value of a user's positions

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://data-api.polymarket.com/value
```

200

400

500

Copy

Ask AI

```
[\
  {\
    "user": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
    "value": 123\
  }\
]
```

GET

/

value

Try it

Get total value of a user's positions

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://data-api.polymarket.com/value
```

200

400

500

Copy

Ask AI

```
[\
  {\
    "user": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
    "value": 123\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions#parameter-user)

user

string

required

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions#parameter-market)

market

string\[\]

0x-prefixed 64-hex string

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions#response-items-user)

user

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions#response-items-value)

value

number

[Get top holders for markets](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets) [Get closed positions for a user](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.