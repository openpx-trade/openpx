---
url: "https://docs.polymarket.com/quickstart/fetching-data"
title: "Fetching Market Data - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/quickstart/fetching-data#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

⌘KAsk AI

- [Main Site](https://polymarket.com/)
- [Main Site](https://polymarket.com/)

Search...

Navigation

Developer Quickstart

Fetching Market Data

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

- [Polymarket](https://polymarket.com/)
- [Discord Community](https://discord.gg/polymarket)
- [Twitter](https://x.com/polymarket)

##### Developer Quickstart

- [Developer Quickstart](https://docs.polymarket.com/quickstart/overview)
- [Fetching Market Data](https://docs.polymarket.com/quickstart/fetching-data)
- [Placing Your First Order](https://docs.polymarket.com/quickstart/first-order)
- [Glossary](https://docs.polymarket.com/quickstart/reference/glossary)
- [API Rate Limits](https://docs.polymarket.com/quickstart/introduction/rate-limits)
- [Endpoints](https://docs.polymarket.com/quickstart/reference/endpoints)

##### Market Makers

- [Market Maker Introduction](https://docs.polymarket.com/developers/market-makers/introduction)
- [Setup](https://docs.polymarket.com/developers/market-makers/setup)
- [Trading](https://docs.polymarket.com/developers/market-makers/trading)
- [Liquidity Rewards](https://docs.polymarket.com/developers/market-makers/liquidity-rewards)
- [Maker Rebates Program](https://docs.polymarket.com/developers/market-makers/maker-rebates-program)
- [Data Feeds](https://docs.polymarket.com/developers/market-makers/data-feeds)
- [Inventory Management](https://docs.polymarket.com/developers/market-makers/inventory)

##### Polymarket Builders Program

- [Builder Program Introduction](https://docs.polymarket.com/developers/builders/builder-intro)
- [Builder Tiers](https://docs.polymarket.com/developers/builders/builder-tiers)
- [Builder Profile & Keys](https://docs.polymarket.com/developers/builders/builder-profile)
- [Order Attribution](https://docs.polymarket.com/developers/builders/order-attribution)
- [Relayer Client](https://docs.polymarket.com/developers/builders/relayer-client)
- [Examples](https://docs.polymarket.com/developers/builders/examples)

##### Central Limit Order Book

- [CLOB Introduction](https://docs.polymarket.com/developers/CLOB/introduction)
- [Status](https://docs.polymarket.com/developers/CLOB/status)
- [Quickstart](https://docs.polymarket.com/developers/CLOB/quickstart)
- [Authentication](https://docs.polymarket.com/developers/CLOB/authentication)
- [Geographic Restrictions](https://docs.polymarket.com/developers/CLOB/geoblock)
- Client

- REST API

- Historical Timeseries Data

- Order Management

- Trades


##### Websocket

- [WSS Overview](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview)
- [WSS Quickstart](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart)
- [WSS Authentication](https://docs.polymarket.com/developers/CLOB/websocket/wss-auth)
- [User Channel](https://docs.polymarket.com/developers/CLOB/websocket/user-channel)
- [Market Channel](https://docs.polymarket.com/developers/CLOB/websocket/market-channel)
- Sports Websocket


##### Real Time Data Stream

- [RTDS Overview](https://docs.polymarket.com/developers/RTDS/RTDS-overview)
- [RTDS Crypto Prices](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices)
- [RTDS Comments](https://docs.polymarket.com/developers/RTDS/RTDS-comments)

##### Gamma Structure

- [Overview](https://docs.polymarket.com/developers/gamma-markets-api/overview)
- [Gamma Structure](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure)
- [Fetching Markets](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide)

##### Gamma Endpoints

- Gamma Status

- Sports

- Tags

- Events

- Markets

- Series

- Comments

- Profiles

- Search


##### Data-API

- Data API Status

- Misc

- Core

- Builders


##### Bridge & Swap

- [Overview](https://docs.polymarket.com/developers/misc-endpoints/bridge-overview)
- Bridge


##### Subgraph

- [Overview](https://docs.polymarket.com/developers/subgraph/overview)

##### Resolution

- [Resolution](https://docs.polymarket.com/developers/resolution/UMA)

##### Conditional Token Frameworks

- [Overview](https://docs.polymarket.com/developers/CTF/overview)
- [Splitting USDC](https://docs.polymarket.com/developers/CTF/split)
- [Merging Tokens](https://docs.polymarket.com/developers/CTF/merge)
- [Reedeeming Tokens](https://docs.polymarket.com/developers/CTF/redeem)
- [Deployment and Additional Information](https://docs.polymarket.com/developers/CTF/deployment-resources)

##### Proxy Wallets

- [Proxy wallet](https://docs.polymarket.com/developers/proxy-wallet)

##### Negative Risk

- [Overview](https://docs.polymarket.com/developers/neg-risk/overview)

On this page

- [Understanding the Data Model](https://docs.polymarket.com/quickstart/fetching-data#understanding-the-data-model)
- [Fetch Active Events](https://docs.polymarket.com/quickstart/fetching-data#fetch-active-events)
- [Market Discovery Best Practices](https://docs.polymarket.com/quickstart/fetching-data#market-discovery-best-practices)
- [For Sports Events](https://docs.polymarket.com/quickstart/fetching-data#for-sports-events)
- [For Non-Sports Topics](https://docs.polymarket.com/quickstart/fetching-data#for-non-sports-topics)
- [Get Market Details](https://docs.polymarket.com/quickstart/fetching-data#get-market-details)
- [Get Current Price](https://docs.polymarket.com/quickstart/fetching-data#get-current-price)
- [Get Orderbook Depth](https://docs.polymarket.com/quickstart/fetching-data#get-orderbook-depth)
- [More Data APIs](https://docs.polymarket.com/quickstart/fetching-data#more-data-apis)

Developer Quickstart

# Fetching Market Data

Fetch Polymarket data in minutes with no authentication required

Get market data with zero setup. No API key, no authentication, no wallet required.

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#understanding-the-data-model)  Understanding the Data Model

Before fetching data, understand how Polymarket structures its markets:

1

[Navigate to header](https://docs.polymarket.com/quickstart/fetching-data#)

Event

The top-level object representing a question like “Will X happen?”

2

[Navigate to header](https://docs.polymarket.com/quickstart/fetching-data#)

Market

Each event contains one or more markets. Each market is a specific tradable binary outcome.

3

[Navigate to header](https://docs.polymarket.com/quickstart/fetching-data#)

Outcomes & Prices

Markets have `outcomes` and `outcomePrices` arrays that map 1:1. These prices represent implied probabilities.

Copy

Ask AI

```
{
  "outcomes": "[\"Yes\", \"No\"]",
  "outcomePrices": "[\"0.20\", \"0.80\"]"
}
// Index 0: "Yes" → 0.20 (20% probability)
// Index 1: "No" → 0.80 (80% probability)
```

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#fetch-active-events)  Fetch Active Events

List all currently active events on Polymarket:

Copy

Ask AI

```
curl "https://gamma-api.polymarket.com/events?active=true&closed=false&limit=5"
```

Example Response

Copy

Ask AI

```
[\
  {\
    "id": "123456",\
    "slug": "will-bitcoin-reach-100k-by-2025",\
    "title": "Will Bitcoin reach $100k by 2025?",\
    "active": true,\
    "closed": false,\
    "tags": [\
      { "id": "21", "label": "Crypto", "slug": "crypto" }\
    ],\
    "markets": [\
      {\
        "id": "789",\
        "question": "Will Bitcoin reach $100k by 2025?",\
        "clobTokenIds": ["TOKEN_YES_ID", "TOKEN_NO_ID"],\
        "outcomes": "[\"Yes\", \"No\"]",\
        "outcomePrices": "[\"0.65\", \"0.35\"]"\
      }\
    ]\
  }\
]
```

Always use `active=true&closed=false` to filter for live, tradable events.

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#market-discovery-best-practices)  Market Discovery Best Practices

### [​](https://docs.polymarket.com/quickstart/fetching-data\#for-sports-events)  For Sports Events

Use the `/sports` endpoint to discover leagues, then query by `series_id`:

Copy

Ask AI

```
# Get all supported sports leagues
curl "https://gamma-api.polymarket.com/sports"

# Get events for a specific league (e.g., NBA series_id=10345)
curl "https://gamma-api.polymarket.com/events?series_id=10345&active=true&closed=false"

# Filter to just game bets (not futures) using tag_id=100639
curl "https://gamma-api.polymarket.com/events?series_id=10345&tag_id=100639&active=true&closed=false&order=startTime&ascending=true"
```

`/sports` only returns automated leagues. For others (UFC, Boxing, F1, Golf, Chess), use tag IDs via `/events?tag_id=<tag_id>`.

### [​](https://docs.polymarket.com/quickstart/fetching-data\#for-non-sports-topics)  For Non-Sports Topics

Use `/tags` to discover all available categories, then filter events:

Copy

Ask AI

```
# Get all available tags
curl "https://gamma-api.polymarket.com/tags?limit=100"

# Query events by topic
curl "https://gamma-api.polymarket.com/events?tag_id=2&active=true&closed=false"
```

Each event response includes a `tags` array, useful for discovering categories from live data and building your own tag mapping.

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#get-market-details)  Get Market Details

Once you have an event, get details for a specific market using its ID or slug:

Copy

Ask AI

```
curl "https://gamma-api.polymarket.com/markets?slug=will-bitcoin-reach-100k-by-2025"
```

The response includes `clobTokenIds`, you’ll need these to fetch prices and place orders.

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#get-current-price)  Get Current Price

Query the CLOB for the current price of any token:

Copy

Ask AI

```
curl "https://clob.polymarket.com/price?token_id=YOUR_TOKEN_ID&side=buy"
```

Example Response

Copy

Ask AI

```
{
  "price": "0.65"
}
```

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#get-orderbook-depth)  Get Orderbook Depth

See all bids and asks for a market:

Copy

Ask AI

```
curl "https://clob.polymarket.com/book?token_id=YOUR_TOKEN_ID"
```

Example Response

Copy

Ask AI

```
{
  "market": "0x...",
  "asset_id": "YOUR_TOKEN_ID",
  "bids": [\
    { "price": "0.64", "size": "500" },\
    { "price": "0.63", "size": "1200" }\
  ],
  "asks": [\
    { "price": "0.66", "size": "300" },\
    { "price": "0.67", "size": "800" }\
  ]
}
```

* * *

## [​](https://docs.polymarket.com/quickstart/fetching-data\#more-data-apis)  More Data APIs

[**Gamma API** \\
\\
Deep dive into market discovery](https://docs.polymarket.com/developers/gamma-markets-api/overview) [**Data API** \\
\\
Positions, activity, and holders data](https://docs.polymarket.com/developers/misc-endpoints/data-api-get-positions) [**WebSocket** \\
\\
Real-time orderbook updates](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview) [**RTDS** \\
\\
Real-time data streaming service](https://docs.polymarket.com/developers/RTDS/RTDS-overview)

[Developer Quickstart](https://docs.polymarket.com/quickstart/overview) [Placing Your First Order](https://docs.polymarket.com/quickstart/first-order)

⌘I

[github](https://github.com/polymarket)

[Powered by](https://www.mintlify.com/?utm_campaign=poweredBy&utm_medium=referral&utm_source=polymarket-292d1b1b)