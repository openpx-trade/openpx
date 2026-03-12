[Skip to main content](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get closed positions for a user

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get closed positions for a user

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/closed-positions?limit=10&sortBy=REALIZEDPNL&sortDirection=DESC'
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
    "avgPrice": 123,\
    "totalBought": 123,\
    "realizedPnl": 123,\
    "curPrice": 123,\
    "timestamp": 123,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "outcomeIndex": 123,\
    "oppositeOutcome": "<string>",\
    "oppositeAsset": "<string>",\
    "endDate": "<string>"\
  }\
]
```

GET

/

closed-positions

Try it

Get closed positions for a user

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/closed-positions?limit=10&sortBy=REALIZEDPNL&sortDirection=DESC'
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
    "avgPrice": 123,\
    "totalBought": 123,\
    "realizedPnl": 123,\
    "curPrice": 123,\
    "timestamp": 123,\
    "title": "<string>",\
    "slug": "<string>",\
    "icon": "<string>",\
    "eventSlug": "<string>",\
    "outcome": "<string>",\
    "outcomeIndex": 123,\
    "oppositeOutcome": "<string>",\
    "oppositeAsset": "<string>",\
    "endDate": "<string>"\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-user)

user

string

required

The address of the user in question
User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-market)

market

string\[\]

The conditionId of the market in question. Supports multiple csv separated values. Cannot be used with the eventId param.

0x-prefixed 64-hex string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-title)

title

string

Filter by market title

Maximum string length: `100`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-event-id)

eventId

integer\[\]

The event id of the event in question. Supports multiple csv separated values. Returns positions for all markets for those event ids. Cannot be used with the market param.

Required range: `x >= 1`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-limit)

limit

integer

default:10

The max number of positions to return

Required range: `0 <= x <= 50`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-offset)

offset

integer

default:0

The starting index for pagination

Required range: `0 <= x <= 100000`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-sort-by)

sortBy

enum<string>

default:REALIZEDPNL

The sort criteria

Available options:

`REALIZEDPNL`,

`TITLE`,

`PRICE`,

`AVGPRICE`,

`TIMESTAMP`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#parameter-sort-direction)

sortDirection

enum<string>

default:DESC

The sort direction

Available options:

`ASC`,

`DESC`

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-proxy-wallet)

proxyWallet

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-asset)

asset

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-condition-id)

conditionId

string

0x-prefixed 64-hex string

Example:

`"0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917"`

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-avg-price)

avgPrice

number

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-total-bought)

totalBought

number

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-realized-pnl)

realizedPnl

number

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-cur-price)

curPrice

number

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-timestamp)

timestamp

integer<int64>

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-title)

title

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-slug)

slug

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-icon)

icon

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-event-slug)

eventSlug

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-outcome)

outcome

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-outcome-index)

outcomeIndex

integer

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-opposite-outcome)

oppositeOutcome

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-opposite-asset)

oppositeAsset

string

[​](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user#response-items-end-date)

endDate

string

[Get total value of a user's positions](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions) [Get trader leaderboard rankings](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.