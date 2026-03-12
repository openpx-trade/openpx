---
url: "https://docs.polymarket.com/developers/CLOB/introduction"
title: "CLOB Introduction - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/introduction#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Central Limit Order Book

CLOB Introduction

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [System](https://docs.polymarket.com/developers/CLOB/introduction#system)
- [API](https://docs.polymarket.com/developers/CLOB/introduction#api)
- [Security](https://docs.polymarket.com/developers/CLOB/introduction#security)
- [Fees](https://docs.polymarket.com/developers/CLOB/introduction#fees)
- [Schedule](https://docs.polymarket.com/developers/CLOB/introduction#schedule)
- [Overview](https://docs.polymarket.com/developers/CLOB/introduction#overview)
- [Additional Resources](https://docs.polymarket.com/developers/CLOB/introduction#additional-resources)

Welcome to the Polymarket Order Book API! This documentation provides overviews, explanations, examples, and annotations to simplify interaction with the order book. The following sections detail the Polymarket Order Book and the API usage.

## [​](https://docs.polymarket.com/developers/CLOB/introduction\#system)  System

Polymarket’s Order Book, or CLOB (Central Limit Order Book), is hybrid-decentralized. It includes an operator for off-chain matching/ordering, with settlement executed on-chain, non-custodially, via signed order messages.The exchange uses a custom Exchange contract facilitating atomic swaps between binary Outcome Tokens (CTF ERC1155 assets and ERC20 PToken assets) and collateral assets (ERC20), following signed limit orders. Designed for binary markets, the contract enables complementary tokens to match across a unified order book.Orders are EIP712-signed structured data. Matched orders have one maker and one or more takers, with price improvements benefiting the taker. The operator handles off-chain order management and submits matched trades to the blockchain for on-chain execution.

## [​](https://docs.polymarket.com/developers/CLOB/introduction\#api)  API

The Polymarket Order Book API enables market makers and traders to programmatically manage market orders. Orders of any amount can be created, listed, fetched, or read from the market order books. Data includes all available markets, market prices, and order history via REST and WebSocket endpoints.

## [​](https://docs.polymarket.com/developers/CLOB/introduction\#security)  Security

Polymarket’s Exchange contract has been audited by Chainsecurity ( [View Audit](https://github.com/Polymarket/ctf-exchange/blob/main/audit/ChainSecurity_Polymarket_Exchange_audit.pdf)).The operator’s privileges are limited to order matching, non-censorship, and ensuring correct ordering. Operators can’t set prices or execute unauthorized trades. Users can cancel orders on-chain independently if trust issues arise.

## [​](https://docs.polymarket.com/developers/CLOB/introduction\#fees)  Fees

### [​](https://docs.polymarket.com/developers/CLOB/introduction\#schedule)  Schedule

> Subject to change

| Volume Level | Maker Fee Base Rate (bps) | Taker Fee Base Rate (bps) |
| --- | --- | --- |
| >0 USDC | 0 | 0 |

### [​](https://docs.polymarket.com/developers/CLOB/introduction\#overview)  Overview

Fees apply symmetrically in output assets (proceeds). This symmetry ensures fairness and market integrity. Fees are calculated differently depending on whether you are buying or selling:

- **Selling outcome tokens (base) for collateral (quote):**

feeQuote=baseRate×min⁡(price,1−price)×sizefeeQuote = baseRate \\times \\min(price, 1 - price) \\times sizefeeQuote=baseRate×min(price,1−price)×size

- **Buying outcome tokens (base) with collateral (quote):**

feeBase=baseRate×min⁡(price,1−price)×sizepricefeeBase = baseRate \\times \\min(price, 1 - price) \\times \\frac{size}{price}feeBase=baseRate×min(price,1−price)×pricesize​

## [​](https://docs.polymarket.com/developers/CLOB/introduction\#additional-resources)  Additional Resources

- [Exchange contract source code](https://github.com/Polymarket/ctf-exchange/tree/main/src)
- [Exchange contract documentation](https://github.com/Polymarket/ctf-exchange/blob/main/docs/Overview.md)

[Examples](https://docs.polymarket.com/developers/builders/examples) [Status](https://docs.polymarket.com/developers/CLOB/status)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.