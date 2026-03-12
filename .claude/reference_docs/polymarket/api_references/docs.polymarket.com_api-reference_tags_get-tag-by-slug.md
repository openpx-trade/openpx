[Skip to main content](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Tags

Get tag by slug

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get tag by slug

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/tags/slug/{slug}
```

200

Copy

Ask AI

```
{
  "id": "<string>",
  "label": "<string>",
  "slug": "<string>",
  "forceShow": true,
  "publishedAt": "<string>",
  "createdBy": 123,
  "updatedBy": 123,
  "createdAt": "2023-11-07T05:31:56Z",
  "updatedAt": "2023-11-07T05:31:56Z",
  "forceHide": true,
  "isCarousel": true
}
```

GET

/

tags

/

slug

/

{slug}

Try it

Get tag by slug

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/tags/slug/{slug}
```

200

Copy

Ask AI

```
{
  "id": "<string>",
  "label": "<string>",
  "slug": "<string>",
  "forceShow": true,
  "publishedAt": "<string>",
  "createdBy": 123,
  "updatedBy": 123,
  "createdAt": "2023-11-07T05:31:56Z",
  "updatedAt": "2023-11-07T05:31:56Z",
  "forceHide": true,
  "isCarousel": true
}
```

#### Path Parameters

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#parameter-slug)

slug

string

required

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#parameter-include-template)

include\_template

boolean

#### Response

200

application/json

Tag

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-id)

id

string

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-label-one-of-0)

label

string \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-slug-one-of-0)

slug

string \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-force-show-one-of-0)

forceShow

boolean \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-published-at-one-of-0)

publishedAt

string \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-created-by-one-of-0)

createdBy

integer \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-updated-by-one-of-0)

updatedBy

integer \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-created-at-one-of-0)

createdAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-updated-at-one-of-0)

updatedAt

string<date-time> \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-force-hide-one-of-0)

forceHide

boolean \| null

[​](https://docs.polymarket.com/api-reference/tags/get-tag-by-slug#response-is-carousel-one-of-0)

isCarousel

boolean \| null

[Get tag by id](https://docs.polymarket.com/api-reference/tags/get-tag-by-id) [Get related tags (relationships) by tag id](https://docs.polymarket.com/api-reference/tags/get-related-tags-relationships-by-tag-id)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.