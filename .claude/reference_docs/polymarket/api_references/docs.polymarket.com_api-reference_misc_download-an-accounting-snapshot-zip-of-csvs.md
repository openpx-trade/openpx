[Skip to main content](https://docs.polymarket.com/api-reference/misc/download-an-accounting-snapshot-zip-of-csvs#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Misc

Download an accounting snapshot (ZIP of CSVs)

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Download an accounting snapshot (ZIP of CSVs)

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://data-api.polymarket.com/v1/accounting/snapshot
```

200

400

500

Copy

Ask AI

```
"<string>"
```

GET

/

v1

/

accounting

/

snapshot

Try it

Download an accounting snapshot (ZIP of CSVs)

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://data-api.polymarket.com/v1/accounting/snapshot
```

200

400

500

Copy

Ask AI

```
"<string>"
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/misc/download-an-accounting-snapshot-zip-of-csvs#parameter-user)

user

string

required

User address (0x-prefixed)
User Profile Address (0x-prefixed, 40 hex chars)

Example:

`"0x56687bf447db6ffa42ffe2204a05edaa20f55839"`

#### Response

200

application/zip

ZIP file containing `positions.csv` and `equity.csv`.

The response is of type `file`.

[Data API Health check](https://docs.polymarket.com/api-reference/data-api-status/data-api-health-check) [Get total markets a user has traded](https://docs.polymarket.com/api-reference/misc/get-total-markets-a-user-has-traded)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.