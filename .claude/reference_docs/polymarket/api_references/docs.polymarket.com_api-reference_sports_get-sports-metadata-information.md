[Skip to main content](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Sports

Get sports metadata information

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get sports metadata information

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/sports
```

200

Copy

Ask AI

```
[\
  {\
    "sport": "<string>",\
    "image": "<string>",\
    "resolution": "<string>",\
    "ordering": "<string>",\
    "tags": "<string>",\
    "series": "<string>"\
  }\
]
```

GET

/

sports

Try it

Get sports metadata information

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/sports
```

200

Copy

Ask AI

```
[\
  {\
    "sport": "<string>",\
    "image": "<string>",\
    "resolution": "<string>",\
    "ordering": "<string>",\
    "tags": "<string>",\
    "series": "<string>"\
  }\
]
```

#### Response

200 - application/json

List of sports metadata objects containing sport configuration details, visual assets, and related identifiers

[​](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#response-items-sport)

sport

string

The sport identifier or abbreviation

[​](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#response-items-image)

image

string<uri>

URL to the sport's logo or image asset

[​](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#response-items-resolution)

resolution

string<uri>

URL to the official resolution source for the sport (e.g., league website)

[​](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#response-items-ordering)

ordering

string

Preferred ordering for sport display, typically "home" or "away"

[​](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#response-items-tags)

tags

string

Comma-separated list of tag IDs associated with the sport for categorization and filtering

[​](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information#response-items-series)

series

string

Series identifier linking the sport to a specific tournament or season series

[List teams](https://docs.polymarket.com/api-reference/sports/list-teams) [Get valid sports market types](https://docs.polymarket.com/api-reference/sports/get-valid-sports-market-types)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.