---
url: "https://docs.polymarket.com/developers/CLOB/orders/onchain-order-info"
title: "Onchain Order Info - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/orders/onchain-order-info#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Order Management

Onchain Order Info

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [How do I interpret the OrderFilled onchain event?](https://docs.polymarket.com/developers/CLOB/orders/onchain-order-info#how-do-i-interpret-the-orderfilled-onchain-event)

## [​](https://docs.polymarket.com/developers/CLOB/orders/onchain-order-info\#how-do-i-interpret-the-orderfilled-onchain-event)  How do I interpret the OrderFilled onchain event?

Given an OrderFilled event:

- `orderHash`: a unique hash for the Order being filled
- `maker`: the user generating the order and the source of funds for the order
- `taker`: the user filling the order OR the Exchange contract if the order fills multiple limit orders
- `makerAssetId`: id of the asset that is given out. If 0, indicates that the Order is a BUY, giving USDC in exchange for Outcome tokens. Else, indicates that the Order is a SELL, giving Outcome tokens in exchange for USDC.
- `takerAssetId`: id of the asset that is received. If 0, indicates that the Order is a SELL, receiving USDC in exchange for Outcome tokens. Else, indicates that the Order is a BUY, receiving Outcome tokens in exchange for USDC.
- `makerAmountFilled`: the amount of the asset that is given out.
- `takerAmountFilled`: the amount of the asset that is received.
- `fee`: the fees paid by the order maker

[Cancel Orders(s)](https://docs.polymarket.com/developers/CLOB/orders/cancel-orders) [Trades Overview](https://docs.polymarket.com/developers/CLOB/trades/trades-overview)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.