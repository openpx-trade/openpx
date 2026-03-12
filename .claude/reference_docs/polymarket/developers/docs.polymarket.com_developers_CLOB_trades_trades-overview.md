---
url: "https://docs.polymarket.com/developers/CLOB/trades/trades-overview"
title: "Trades Overview - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/trades/trades-overview#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Trades

Trades Overview

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Overview](https://docs.polymarket.com/developers/CLOB/trades/trades-overview#overview)
- [Statuses](https://docs.polymarket.com/developers/CLOB/trades/trades-overview#statuses)

## [​](https://docs.polymarket.com/developers/CLOB/trades/trades-overview\#overview)  Overview

All historical trades can be fetched via the Polymarket CLOB REST API. A trade is initiated by a “taker” who creates a marketable limit order. This limit order can be matched against one or more resting limit orders on the associated book. A trade can be in various states as described below. Note: in some cases (due to gas limitations) the execution of a “trade” must be broken into multiple transactions which case separate trade entities will be returned. To associate trade entities, there is a bucket\_index field and a match\_time field. Trades that have been broken into multiple trade objects can be reconciled by combining trade objects with the same market\_order\_id, match\_time and incrementing bucket\_index’s into a top level “trade” client side.

## [​](https://docs.polymarket.com/developers/CLOB/trades/trades-overview\#statuses)  Statuses

| Status | Terminal? | Description |
| --- | --- | --- |
| MATCHED | no | trade has been matched and sent to the executor service by the operator, the executor service submits the trade as a transaction to the Exchange contract |
| MINED | no | trade is observed to be mined into the chain, no finality threshold established |
| CONFIRMED | yes | trade has achieved strong probabilistic finality and was successful |
| RETRYING | no | trade transaction has failed (revert or reorg) and is being retried/resubmitted by the operator |
| FAILED | yes | trade has failed and is not being retried |

[Onchain Order Info](https://docs.polymarket.com/developers/CLOB/orders/onchain-order-info) [Get Trades](https://docs.polymarket.com/developers/CLOB/trades/trades)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.