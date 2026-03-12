---
url: "https://docs.polymarket.com/developers/CLOB/orders/orders"
title: "Orders Overview - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/orders/orders#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Order Management

Orders Overview

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Allowances](https://docs.polymarket.com/developers/CLOB/orders/orders#allowances)
- [Signature Types](https://docs.polymarket.com/developers/CLOB/orders/orders#signature-types)
- [Validity Checks](https://docs.polymarket.com/developers/CLOB/orders/orders#validity-checks)

All orders are expressed as limit orders (can be marketable). The underlying order primitive must be in the form expected and executable by the on-chain binary limit order protocol contract. Preparing such an order is quite involved (structuring, hashing, signing), thus Polymarket suggests using the open source typescript, python and golang libraries.

## [​](https://docs.polymarket.com/developers/CLOB/orders/orders\#allowances)  Allowances

To place an order, allowances must be set by the funder address for the specified `maker` asset for the Exchange contract. When buying, this means the funder must have set a USDC allowance greater than or equal to the spending amount. When selling, the funder must have set an allowance for the conditional token that is greater than or equal to the selling amount. This allows the Exchange contract to execute settlement according to the signed order instructions created by a user and matched by the operator.

## [​](https://docs.polymarket.com/developers/CLOB/orders/orders\#signature-types)  Signature Types

Polymarket’s CLOB supports 3 signature types. Orders must identify what signature type they use. The available typescript and python clients abstract the complexity of signing and preparing orders with the following signature types by allowing a funder address and signer type to be specified on initialization. The supported signature types are:

| Type | ID | Description |
| --- | --- | --- |
| EOA | 0 | EIP712 signature signed by an EOA |
| POLY\_PROXY | 1 | EIP712 signatures signed by a signer associated with funding Polymarket proxy wallet |
| POLY\_GNOSIS\_SAFE | 2 | EIP712 signatures signed by a signer associated with funding Polymarket gnosis safe wallet |

## [​](https://docs.polymarket.com/developers/CLOB/orders/orders\#validity-checks)  Validity Checks

Orders are continually monitored to make sure they remain valid. Specifically, this includes continually tracking underlying balances, allowances and on-chain order cancellations. Any maker that is caught intentionally abusing these checks (which are essentially real time) will be blacklisted.Additionally, there are rails on order placement in a market. Specifically, you can only place orders that sum to less than or equal to your available balance for each market. For example if you have 500 USDC in your funding wallet, you can place one order to buy 1000 YES in marketA @ $.50, then any additional buy orders to that market will be rejected since your entire balance is reserved for the first (and only) buy order. More explicitly the max size you can place for an order is:maxOrderSize=underlyingAssetBalance−∑(orderSize−orderFillAmount)\\text{maxOrderSize} = \\text{underlyingAssetBalance} - \\sum(\\text{orderSize} - \\text{orderFillAmount})maxOrderSize=underlyingAssetBalance−∑(orderSize−orderFillAmount)

[Historical Timeseries Data](https://docs.polymarket.com/developers/CLOB/timeseries) [Place Single Order](https://docs.polymarket.com/developers/CLOB/orders/create-order)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.