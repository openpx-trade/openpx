[Skip to main content](https://docs.polymarket.com/api-reference/misc/get-live-volume-for-an-event#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Misc

Get live volume for an event

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get live volume for an event

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://data-api.polymarket.com/live-volume
```

200

400

500

Copy

Ask AI

```
[\
  {\
    "total": 123,\
    "markets": [\
      {\
        "market": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
        "value": 123\
      }\
    ]\
  }\
]
```

GET

/

live-volume

Try it

Get live volume for an event

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://data-api.polymarket.com/live-volume
```

200

400

500

Copy

Ask AI

```
[\
  {\
    "total": 123,\
    "markets": [\
      {\
        "market": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",\
        "value": 123\
      }\
    ]\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/misc/get-live-volume-for-an-event#parameter-id)

id

integer

required

Required range: `x >= 1`

#### Response

200

application/json

Success

[​](https://docs.polymarket.com/api-reference/misc/get-live-volume-for-an-event#response-items-total)

total

number

[​](https://docs.polymarket.com/api-reference/misc/get-live-volume-for-an-event#response-items-markets)

markets

object\[\]

Showchild attributes

[Get open interest](https://docs.polymarket.com/api-reference/misc/get-open-interest) [Get current positions for a user](https://docs.polymarket.com/api-reference/core/get-current-positions-for-a-user)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.