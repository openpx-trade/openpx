[Skip to main content](https://docs.polymarket.com/api-reference/sports/get-valid-sports-market-types#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Sports

Get valid sports market types

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get valid sports market types

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/sports/market-types
```

200

Copy

Ask AI

```
{
  "marketTypes": [\
    "<string>"\
  ]
}
```

GET

/

sports

/

market-types

Try it

Get valid sports market types

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://gamma-api.polymarket.com/sports/market-types
```

200

Copy

Ask AI

```
{
  "marketTypes": [\
    "<string>"\
  ]
}
```

#### Response

200 - application/json

List of valid sports market types

[​](https://docs.polymarket.com/api-reference/sports/get-valid-sports-market-types#response-market-types)

marketTypes

string\[\]

List of all valid sports market types

[Get sports metadata information](https://docs.polymarket.com/api-reference/sports/get-sports-metadata-information) [List tags](https://docs.polymarket.com/api-reference/tags/list-tags)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.