---
url: "https://docs.polymarket.com/developers/CLOB/websocket/wss-overview"
title: "WSS Overview - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Websocket

WSS Overview

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Overview](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview#overview)
- [Subscription](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview#subscription)
- [Subscribe to more assets](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview#subscribe-to-more-assets)

## [​](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview\#overview)  Overview

The Polymarket CLOB API provides websocket (wss) channels through which clients can get pushed updates. These endpoints allow clients to maintain almost real-time views of their orders, their trades and markets in general. There are two available channels `user` and `market`.

## [​](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview\#subscription)  Subscription

To subscribe send a message including the following authentication and intent information upon opening the connection.

| Field | Type | Description |
| --- | --- | --- |
| auth | Auth | see next page for auth information |
| markets | string\[\] | array of markets (condition IDs) to receive events for (for `user` channel) |
| assets\_ids | string\[\] | array of asset ids (token IDs) to receive events for (for `market` channel) |
| type | string | id of channel to subscribe to (USER or MARKET) |
| custom\_feature\_enabled | bool | enabling / disabling custom features |

Where the `auth` field is of type `Auth` which has the form described in the WSS Authentication section below.

### [​](https://docs.polymarket.com/developers/CLOB/websocket/wss-overview\#subscribe-to-more-assets)  Subscribe to more assets

Once connected, the client can subscribe and unsubscribe to `asset_ids` by sending the following message:

| Field | Type | Description |
| --- | --- | --- |
| assets\_ids | string\[\] | array of asset ids (token IDs) to receive events for (for `market` channel) |
| markets | string\[\] | array of market ids (condition IDs) to receive events for (for `user` channel) |
| operation | string | ”subscribe” or “unsubscribe” |
| custom\_feature\_enabled | bool | enabling / disabling custom features |

[Get Trades](https://docs.polymarket.com/developers/CLOB/trades/trades) [WSS Quickstart](https://docs.polymarket.com/quickstart/websocket/WSS-Quickstart)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.