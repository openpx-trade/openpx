[Skip to main content](https://docs.polymarket.com/api-reference/pricing/get-market-price#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Pricing

Get market price

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get market price

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/price
```

200

Example

Copy

Ask AI

```
{
  "price": "1800.50"
}
```

GET

/

price

Try it

Get market price

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/price
```

200

Example

Copy

Ask AI

```
{
  "price": "1800.50"
}
```

#### Query Parameters

[​](https://docs.polymarket.com/api-reference/pricing/get-market-price#parameter-token-id)

token\_id

string

required

The unique identifier for the token

[​](https://docs.polymarket.com/api-reference/pricing/get-market-price#parameter-side)

side

enum<string>

required

The side of the market (BUY or SELL)

Available options:

`BUY`,

`SELL`

#### Response

200

application/json

Successful response

[​](https://docs.polymarket.com/api-reference/pricing/get-market-price#response-price)

price

string

required

The market price (as string to maintain precision)

Example:

`"1800.50"`

[Get multiple order books summaries by request](https://docs.polymarket.com/api-reference/orderbook/get-multiple-order-books-summaries-by-request) [Get multiple market prices](https://docs.polymarket.com/api-reference/pricing/get-multiple-market-prices)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.