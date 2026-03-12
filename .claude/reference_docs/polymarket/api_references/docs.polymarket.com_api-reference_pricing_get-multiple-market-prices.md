[Skip to main content](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Pricing

Get multiple market prices

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get multiple market prices

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/prices
```

200

400

500

Copy

Ask AI

```
{
  "1234567890": {
    "BUY": "1800.50",
    "SELL": "1801.00"
  },
  "0987654321": {
    "BUY": "50.25",
    "SELL": "50.30"
  }
}
```

GET

/

prices

Try it

Get multiple market prices

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/prices
```

200

400

500

Copy

Ask AI

```
{
  "1234567890": {
    "BUY": "1800.50",
    "SELL": "1801.00"
  },
  "0987654321": {
    "BUY": "50.25",
    "SELL": "50.30"
  }
}
```

#### Response

200

application/json

Successful response

Map of token\_id to side to price

[​](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices#response-additional-properties)

{key}

object

Showchild attributes

[Get market price](https://docs.polymarket.com/api-reference/pricing/get-market-price) [Get multiple market prices by request](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices-by-request)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.