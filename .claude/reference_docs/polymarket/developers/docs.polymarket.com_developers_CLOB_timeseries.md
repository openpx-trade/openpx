---
url: "https://docs.polymarket.com/developers/CLOB/timeseries"
title: "Historical Timeseries Data - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/timeseries#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Historical Timeseries Data

Historical Timeseries Data

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

Get price history for a traded token

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/prices-history
```

200

400

404

500

Copy

Ask AI

```
{
  "history": [\
    {\
      "t": 1697875200,\
      "p": 1800.75\
    }\
  ]
}
```

GET

/

prices-history

Try it

Get price history for a traded token

cURL

Copy

Ask AI

```
curl --request GET \
  --url https://clob.polymarket.com/prices-history
```

200

400

404

500

Copy

Ask AI

```
{
  "history": [\
    {\
      "t": 1697875200,\
      "p": 1800.75\
    }\
  ]
}
```

The CLOB provides detailed price history for each traded token.**HTTP REQUEST**`GET /<clob-endpoint>/prices-history`

We also have a Interactive Notebook to visualize the data from this endpoint available [here](https://colab.research.google.com/drive/1s4TCOR4K7fRP7EwAH1YmOactMakx24Cs?usp=sharing#scrollTo=mYCJBcfB9Zu4).

#### Query Parameters

[​](https://docs.polymarket.com/developers/CLOB/timeseries#parameter-market)

market

string

required

The CLOB token ID for which to fetch price history

[​](https://docs.polymarket.com/developers/CLOB/timeseries#parameter-start-ts)

startTs

number

The start time, a Unix timestamp in UTC

[​](https://docs.polymarket.com/developers/CLOB/timeseries#parameter-end-ts)

endTs

number

The end time, a Unix timestamp in UTC

[​](https://docs.polymarket.com/developers/CLOB/timeseries#parameter-interval)

interval

enum<string>

A string representing a duration ending at the current time. Mutually exclusive with startTs and endTs

Available options:

`1m`,

`1w`,

`1d`,

`6h`,

`1h`,

`max`

[​](https://docs.polymarket.com/developers/CLOB/timeseries#parameter-fidelity)

fidelity

number

The resolution of the data, in minutes

#### Response

200

application/json

A list of timestamp/price pairs

[​](https://docs.polymarket.com/developers/CLOB/timeseries#response-history)

history

object\[\]

required

Showchild attributes

[Get bid-ask spreads](https://docs.polymarket.com/api-reference/spreads/get-bid-ask-spreads) [Orders Overview](https://docs.polymarket.com/developers/CLOB/orders/orders)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.