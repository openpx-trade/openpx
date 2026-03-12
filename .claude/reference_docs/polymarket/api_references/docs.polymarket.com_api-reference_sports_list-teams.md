[Skip to main content](https://docs.polymarket.com/api-reference/sports/list-teams#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Sports

List teams

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

List teams

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/teams
```

200

Copy

Ask AI

```
[\
  {\
    "id": 123,\
    "name": "<string>",\
    "league": "<string>",\
    "record": "<string>",\
    "logo": "<string>",\
    "abbreviation": "<string>",\
    "alias": "<string>",\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z"\
  }\
]
```

GET

/

teams

Try it

List teams

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/teams
```

200

Copy

Ask AI

```
[\
  {\
    "id": 123,\
    "name": "<string>",\
    "league": "<string>",\
    "record": "<string>",\
    "logo": "<string>",\
    "abbreviation": "<string>",\
    "alias": "<string>",\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z"\
  }\
]
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-limit)

limit

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-offset)

offset

integer

Required range: `x >= 0`

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-order)

order

string

Comma-separated list of fields to order by

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-ascending)

ascending

boolean

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-league)

league

string\[\]

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-name)

name

string\[\]

[​](https://docs.polymarket.com/api-reference/sports/list-teams#parameter-abbreviation)

abbreviation

string\[\]

#### Response

200 - application/json

List of teams

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-id)

id

integer

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-name-one-of-0)

name

string \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-league-one-of-0)

league

string \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-record-one-of-0)

record

string \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-logo-one-of-0)

logo

string \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-abbreviation-one-of-0)

abbreviation

string \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-alias-one-of-0)

alias

string \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/sports/list-teams#response-items-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[Gamma API Health check](https://docs.polymarket.com/api-reference/gamma-status/gamma-api-health-check) [Get sports metadata information](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.