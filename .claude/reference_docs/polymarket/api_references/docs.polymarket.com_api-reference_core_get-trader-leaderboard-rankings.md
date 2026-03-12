[Skip to main content](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get trader leaderboard rankings

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get trader leaderboard rankings

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/v1/leaderboard?category=OVERALL&timePeriod=DAY&orderBy=PNL&limit=25'
```

200

400

500

Copy

Ask AI

```
[\
  {\
    "rank": "<string>",\
    "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
    "userName": "<string>",\
    "vol": 123,\
    "pnl": 123,\
    "profileImage": "<string>",\
    "xUsername": "<string>",\
    "verifiedBadge": true\
  }\
]
```

GET

/

v1

/

leaderboard

Try it

Get trader leaderboard rankings

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/v1/leaderboard?category=OVERALL&timePeriod=DAY&orderBy=PNL&limit=25'
```

200

400

500

Copy

Ask AI

```
[\
  {\
    "rank": "<string>",\
    "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
    "userName": "<string>",\
    "vol": 123,\
    "pnl": 123,\
    "profileImage": "<string>",\
    "xUsername": "<string>",\
    "verifiedBadge": true\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-category)

category

enum<string>

default:OVERALL

Market category for the leaderboard

Available options:

`OVERALL`,

`POLITICS`,

`SPORTS`,

`CRYPTO`,

`CULTURE`,

`MENTIONS`,

`WEATHER`,

`ECONOMICS`,

`TECH`,

`FINANCE`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-time-period)

timePeriod

enum<string>

default:DAY

Time period for leaderboard results

Available options:

`DAY`,

`WEEK`,

`MONTH`,

`ALL`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-order-by)

orderBy

enum<string>

default:PNL

Leaderboard ordering criteria

Available options:

`PNL`,

`VOL`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-limit)

limit

integer

default:25

Max number of leaderboard traders to return

Required range: `1 <= x <= 50`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-offset)

offset

integer

default:0

Starting index for pagination

Required range: `0 <= x <= 1000`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-user)

user

string

Limit leaderboard to a single user by address
User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#parameter-user-name)

userName

string

Limit leaderboard to a single username

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-rank)

rank

string

The rank position of the trader

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-proxy-wallet)

proxyWallet

string

User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-user-name)

userName

string

The trader's username

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-vol)

vol

number

Trading volume for this trader

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-pnl)

pnl

number

Profit and loss for this trader

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-profile-image)

profileImage

string

URL to the trader's profile image

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-x-username)

xUsername

string

The trader's X (Twitter) username

[​](https://docs.polymarket.com/api-reference/core/get-trader-leaderboard-rankings#response-items-verified-badge)

verifiedBadge

boolean

Whether the trader has a verified badge

[Get closed positions for a user](https://docs.polymarket.com/api-reference/core/get-closed-positions-for-a-user) [Get aggregated builder leaderboard](https://docs.polymarket.com/api-reference/builders/get-aggregated-builder-leaderboard)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.