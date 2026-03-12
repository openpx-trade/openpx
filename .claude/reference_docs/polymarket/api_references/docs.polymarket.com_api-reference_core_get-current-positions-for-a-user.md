[Skip to main content](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get current positions for a user

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get current positions for a user

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/positions?sizeThreshold=1&limit=100&sortBy=TOKENS&sortDirection=DESC'
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
    "asset": "<string>",\
    "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
    "size": 123,\
    "avgPrice": 123,\
    "initialValue": 123,\
    "currentValue": 123,\
    "cashPnl": 123,\
    "percentPnl": 123,\
    "totalBought": 123,\
    "realizedPnl": 123,\
    "percentRealizedPnl": 123,\
    "curPrice": 123,\
    "redeemable": true,\
    "mergeable": true,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "outcomeIndex": 123,\
    "oppositeOutcome": "<string>",\
    "oppositeAsset": "<string>",\
    "endDate": "<string>",\
    "negativeRisk": true\
  }\
]
```

GET

/

positions

Try it

Get current positions for a user

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/positions?sizeThreshold=1&limit=100&sortBy=TOKENS&sortDirection=DESC'
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
    "asset": "<string>",\
    "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
    "size": 123,\
    "avgPrice": 123,\
    "initialValue": 123,\
    "currentValue": 123,\
    "cashPnl": 123,\
    "percentPnl": 123,\
    "totalBought": 123,\
    "realizedPnl": 123,\
    "percentRealizedPnl": 123,\
    "curPrice": 123,\
    "redeemable": true,\
    "mergeable": true,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "outcomeIndex": 123,\
    "oppositeOutcome": "<string>",\
    "oppositeAsset": "<string>",\
    "endDate": "<string>",\
    "negativeRisk": true\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-user)

user

string

required

User address (required)
User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-market)

market

string\[\]

Comma-separated list of condition IDs. Mutually exclusive with eventId.

0x-prefixed 64-hex string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-event-id)

eventId

integer\[\]

Comma-separated list of event IDs. Mutually exclusive with market.

Required range: `x >= 1`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-size-threshold)

sizeThreshold

number

default:1

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-redeemable)

redeemable

boolean

default:false

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-mergeable)

mergeable

boolean

default:false

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-limit)

limit

integer

default:100

Required range: `0 <= x <= 500`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-offset)

offset

integer

default:0

Required range: `0 <= x <= 10000`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-sort-by)

sortBy

enum<string>

default:TOKENS

Available options:

`CURRENT`,

`INITIAL`,

`TOKENS`,

`CASHPNL`,

`PERCENTPNL`,

`TITLE`,

`RESOLVING`,

`PRICE`,

`AVGPRICE`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-sort-direction)

sortDirection

enum<string>

default:DESC

Available options:

`ASC`,

`DESC`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#parameter-title)

title

string

Maximum string length: `100`

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-proxy-wallet)

proxyWallet

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-asset)

asset

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-condition-id)

conditionId

string

0x-prefixed 64-hex string

Example:

`"0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917"`

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-size)

size

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-avg-price)

avgPrice

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-initial-value)

initialValue

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-current-value)

currentValue

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-cash-pnl)

cashPnl

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-percent-pnl)

percentPnl

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-total-bought)

totalBought

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-realized-pnl)

realizedPnl

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-percent-realized-pnl)

percentRealizedPnl

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-cur-price)

curPrice

number

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-redeemable)

redeemable

boolean

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-mergeable)

mergeable

boolean

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-title)

title

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-slug)

slug

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-icon)

icon

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-event-slug)

eventSlug

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-outcome)

outcome

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-outcome-index)

outcomeIndex

integer

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-opposite-outcome)

oppositeOutcome

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-opposite-asset)

oppositeAsset

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-end-date)

endDate

string

[​](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user#response-items-negative-risk)

negativeRisk

boolean

[Get live volume for an event](https://docs.polymarket.com/api-reference/misc/get-live-volume-for-an-event) [Get trades for a user or markets](https://docs.polymarket.com/api-reference/core/get-trades-for-a-user-or-markets)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.