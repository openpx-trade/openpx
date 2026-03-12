[Skip to main content](https://docs.polymarket.com/api-reference/pricing/get-midpoint-price#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Pricing

Get midpoint price

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get midpoint price

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/midpoint
```

200

400

404

500

Copy

Ask AI

```
{
  "mid": "1800.75"
}
```

GET

/

midpoint

Try it

Get midpoint price

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/midpoint
```

200

400

404

500

Copy

Ask AI

```
{
  "mid": "1800.75"
}
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/pricing/get-midpoint-price#parameter-token-id)

token\_id

string

required

The unique identifier for the token

#### Response

200

application/json

Successful response

[​](https://docs.polymarket.com/api-reference/pricing/get-midpoint-price#response-mid)

mid

string

required

The midpoint price (as string to maintain precision)

Example:

`"1800.75"`

[Get multiple market prices by request](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices-by-request) [Get price history for a traded token](https://docs.polymarket.com/api-reference/pricing/get-price-history-for-a-traded-token)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.