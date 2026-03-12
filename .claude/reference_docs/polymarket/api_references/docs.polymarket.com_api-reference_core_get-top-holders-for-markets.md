[Skip to main content](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Core

Get top holders for markets

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get top holders for markets

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/holders?limit=20&minBalance=1'
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
    "token": "<string>",\
    "holders": [\
      {\
        "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
        "bio": "<string>",\
        "asset": "<string>",\
        "pseudonym": "<string>",\
        "amount": 123,\
        "displayUsernamePublic": true,\
        "outcomeIndex": 123,\
        "name": "<string>",\
        "profileImage": "<string>",\
        "profileImageOptimized": "<string>"\
      }\
    ]\
  }\
]
```

GET

/

holders

Try it

Get top holders for markets

cURL

Copy

Ask AI

```
curl --request GET \
  --url 'https://data-api.polymarket.com/holders?limit=20&minBalance=1'
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
    "token": "<string>",\
    "holders": [\
      {\
        "proxyWallet": "0x56687bf447db6ffa42ffe2204a05edaa20f55839",\
        "bio": "<string>",\
        "asset": "<string>",\
        "pseudonym": "<string>",\
        "amount": 123,\
        "displayUsernamePublic": true,\
        "outcomeIndex": 123,\
        "name": "<string>",\
        "profileImage": "<string>",\
        "profileImageOptimized": "<string>"\
      }\
    ]\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets#parameter-limit)

limit

integer

default:20

Maximum number of holders to return per token. Capped at 20.

Required range: `0 <= x <= 20`

[​](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets#parameter-market)

market

string\[\]

required

Comma-separated list of condition IDs.

0x-prefixed 64-hex string

[​](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets#parameter-min-balance)

minBalance

integer

default:1

Required range: `0 <= x <= 999999`

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets#response-items-token)

token

string

[​](https://docs.polymarket.com/api-reference/core/get-top-holders-for-markets#response-items-holders)

holders

object\[\]

Showchild attributes

[Get user activity](https://docs.polymarket.com/api-reference/core/get-user-activity) [Get total value of a user's positions](https://docs.polymarket.com/api-reference/core/get-total-value-of-a-users-positions)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.