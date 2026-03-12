[Skip to main content](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get trades for a user or markets

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get trades for a user or markets

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/trades?limit=100&takerOnly=true'
```

200

400

401

500

Copy

Ask AI

```
[\
  {\
    "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
    "side": "BUY",\
    "asset": "<string>",\
    "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
    "size": 123,\
    "price": 123,\
    "timestamp": 123,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "outcomeIndex": 123,\
    "name": "<string>",\
    "pseudonym": "<string>",\
    "bio": "<string>",\
    "profileImage": "<string>",\
    "profileImageOptimized": "<string>",\
    "transactionHash": "<string>"\
  }\
]
```

GET

/

trades

Try it

Get trades for a user or markets

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/trades?limit=100&takerOnly=true'
```

200

400

401

500

Copy

Ask AI

```
[\
  {\
    "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
    "side": "BUY",\
    "asset": "<string>",\
    "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
    "size": 123,\
    "price": 123,\
    "timestamp": 123,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "outcomeIndex": 123,\
    "name": "<string>",\
    "pseudonym": "<string>",\
    "bio": "<string>",\
    "profileImage": "<string>",\
    "profileImageOptimized": "<string>",\
    "transactionHash": "<string>"\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-limit)

limit

integer

default:100

Required range: `0 <= x <= 10000`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-offset)

offset

integer

default:0

Required range: `0 <= x <= 10000`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-taker-only)

takerOnly

boolean

default:true

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-filter-type)

filterType

enum<string>

Must be provided together with filterAmount.

Available options:

`CASH`,

`TOKENS`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-filter-amount)

filterAmount

number

Must be provided together with filterType.

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-market)

market

string\[\]

Comma-separated list of condition IDs. Mutually exclusive with eventId.

0x-prefixed 64-hex string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-event-id)

eventId

integer\[\]

Comma-separated list of event IDs. Mutually exclusive with market.

Required range: `x >= 1`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-user)

user

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#parameter-side)

side

enum<string>

Available options:

`BUY`,

`SELL`

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-proxy-wallet)

proxyWallet

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-side)

side

enum<string>

Available options:

`BUY`,

`SELL`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-asset)

asset

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-condition-id)

conditionId

string

0x-prefixed 64-hex string

Example:

`"0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917"`

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-size)

size

number

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-price)

price

number

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-timestamp)

timestamp

integer<int64>

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-title)

title

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-slug)

slug

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-icon)

icon

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-event-slug)

eventSlug

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-outcome)

outcome

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-outcome-index)

outcomeIndex

integer

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-name)

name

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-pseudonym)

pseudonym

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-bio)

bio

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-profile-image)

profileImage

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-profile-image-optimized)

profileImageOptimized

string

[​](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets#response-items-transaction-hash)

transactionHash

string

[Get current positions for a user](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user) [Get user activity](https://docs.polymarket.com/api-reference/core/get-user-activity)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.