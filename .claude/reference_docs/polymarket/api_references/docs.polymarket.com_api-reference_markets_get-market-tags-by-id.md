[Skip to main content](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Markets

Get market tags by id

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get market tags by id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/markets/{id}/tags
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "label": "<string>",\
    "slug": "<string>",\
    "forceShow": true,\
    "publishedAt": "<string>",\
    "createdBy": 123,\
    "updatedBy": 123,\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z",\
    "forceHide": true,\
    "isCarousel": true\
  }\
]
```

GET

/

markets

/

{id}

/

tags

Try it

Get market tags by id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/markets/{id}/tags
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "label": "<string>",\
    "slug": "<string>",\
    "forceShow": true,\
    "publishedAt": "<string>",\
    "createdBy": 123,\
    "updatedBy": 123,\
    "createdAt": "2023-11-07T05:31:56Z",\
    "updatedAt": "2023-11-07T05:31:56Z",\
    "forceHide": true,\
    "isCarousel": true\
  }\
]
```

#### Path Parameters

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#parameter-id)

id

integer

required

#### Response

200

application/json

Tags attached to the market

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-id)

id

string

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-label-one-of-0)

label

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-slug-one-of-0)

slug

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-force-show-one-of-0)

forceShow

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-published-at-one-of-0)

publishedAt

string \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-created-by-one-of-0)

createdBy

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-updated-by-one-of-0)

updatedBy

integer \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-force-hide-one-of-0)

forceHide

boolean \| null

[​](https://docs.polymarket.com/api-reference/markets/get-market-tags-by-id#response-items-is-carousel-one-of-0)

isCarousel

boolean \| null

[Get market by id](https://docs.polymarket.com/api-reference/markets/get-market-by-id) [Get market by slug](https://docs.polymarket.com/api-reference/markets/get-market-by-slug)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.