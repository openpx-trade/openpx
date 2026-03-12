[Skip to main content](https://docs.polymarket.com/api-reference/core/get-user-activity#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get user activity

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get user activity

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/activity?limit=100&sortBy=TIMESTAMP&sortDirection=DESC'
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
    "timestamp": 123,\
    "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
    "type": "TRADE",\
    "size": 123,\
    "usdcSize": 123,\
    "transactionHash": "<string>",\
    "price": 123,\
    "asset": "<string>",\
    "side": "BUY",\
    "outcomeIndex": 123,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "name": "<string>",\
    "pseudonym": "<string>",\
    "bio": "<string>",\
    "profileImage": "<string>",\
    "profileImageOptimized": "<string>"\
  }\
]
```

GET

/

activity

Try it

Get user activity

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/activity?limit=100&sortBy=TIMESTAMP&sortDirection=DESC'
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
    "timestamp": 123,\
    "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
    "type": "TRADE",\
    "size": 123,\
    "usdcSize": 123,\
    "transactionHash": "<string>",\
    "price": 123,\
    "asset": "<string>",\
    "side": "BUY",\
    "outcomeIndex": 123,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "name": "<string>",\
    "pseudonym": "<string>",\
    "bio": "<string>",\
    "profileImage": "<string>",\
    "profileImageOptimized": "<string>"\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-limit)

limit

integer

default:100

Required range: `0 <= x <= 500`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-offset)

offset

integer

default:0

Required range: `0 <= x <= 10000`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-user)

user

string

required

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-market)

market

string\[\]

Comma-separated list of condition IDs. Mutually exclusive with eventId.

0x-prefixed 64-hex string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-event-id)

eventId

integer\[\]

Comma-separated list of event IDs. Mutually exclusive with market.

Required range: `x >= 1`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-type)

type

enum<string>\[\]

Available options:

`TRADE`,

`SPLIT`,

`MERGE`,

`REDEEM`,

`REWARD`,

`CONVERSION`,

`MAKER_REBATE`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-start)

start

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-end)

end

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-sort-by)

sortBy

enum<string>

default:TIMESTAMP

Available options:

`TIMESTAMP`,

`TOKENS`,

`CASH`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-sort-direction)

sortDirection

enum<string>

default:DESC

Available options:

`ASC`,

`DESC`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#parameter-side)

side

enum<string>

Available options:

`BUY`,

`SELL`

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-proxy-wallet)

proxyWallet

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-timestamp)

timestamp

integer<int64>

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-condition-id)

conditionId

string

0x-prefixed 64-hex string

Example:

`"0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917"`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-type)

type

enum<string>

Available options:

`TRADE`,

`SPLIT`,

`MERGE`,

`REDEEM`,

`REWARD`,

`CONVERSION`,

`MAKER_REBATE`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-size)

size

number

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-usdc-size)

usdcSize

number

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-transaction-hash)

transactionHash

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-price)

price

number

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-asset)

asset

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-side)

side

enum<string>

Available options:

`BUY`,

`SELL`

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-outcome-index)

outcomeIndex

integer

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-title)

title

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-slug)

slug

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-icon)

icon

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-event-slug)

eventSlug

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-outcome)

outcome

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-name)

name

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-pseudonym)

pseudonym

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-bio)

bio

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-profile-image)

profileImage

string

[​](https://docs.polymarket.com/api-reference/core/get-user-activity#response-items-profile-image-optimized)

profileImageOptimized

string

[Get trades for a user or markets](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets) [Get top holders for markets](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.