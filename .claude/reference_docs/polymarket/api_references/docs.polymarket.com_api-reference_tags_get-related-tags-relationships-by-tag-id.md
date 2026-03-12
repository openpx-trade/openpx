[Skip to main content](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Tags

Get related tags (relationships) by tag id

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get related tags (relationships) by tag id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/tags/{id}/related-tags
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "tagID": 123,\
    "relatedTagID": 123,\
    "rank": 123\
  }\
]
```

GET

/

tags

/

{id}

/

related-tags

Try it

Get related tags (relationships) by tag id

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/tags/{id}/related-tags
```

200

Copy

Ask AI

```
[\
  {\
    "id": "<string>",\
    "tagID": 123,\
    "relatedTagID": 123,\
    "rank": 123\
  }\
]
```

#### Path Parameters

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#parameter-id)

id

integer

required

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#parameter-omit-empty)

omit\_empty

boolean

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#parameter-status)

status

enum<string>

Available options:

`active`,

`closed`,

`all`

#### Response

200 - application/json

Related tag relationships

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#response-items-id)

id

string

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#response-items-tag-id-one-of-0)

tagID

integer \| null

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#response-items-related-tag-id-one-of-0)

relatedTagID

integer \| null

[​](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id#response-items-rank-one-of-0)

rank

integer \| null

[Get tag by slug](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug) [Get related tags (relationships) by tag slug](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-slug)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.