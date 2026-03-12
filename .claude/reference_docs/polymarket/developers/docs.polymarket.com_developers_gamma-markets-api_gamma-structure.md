---
url: "https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure"
title: "Gamma Structure - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Gamma Structure

Gamma Structure

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Detail](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure#detail)
- [Example](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure#example)

Gamma provides some organizational models. These include events, and markets. The most fundamental element is always markets and the other models simply provide additional organization.

# [​](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure\#detail)  Detail

1. **Market**   1. Contains data related to a market that is traded on. Maps onto a pair of clob token ids, a market address, a question id and a condition id
2. **Event**   1. Contains a set of markets
2. Variants:
      1. Event with 1 market (i.e., resulting in an SMP)
      2. Event with 2 or more markets (i.e., resulting in an GMP)

# [​](https://docs.polymarket.com/developers/gamma-markets-api/gamma-structure\#example)  Example

- **\[Event\]**Where will Barron Trump attend College?

  - **\[Market\]** Will Barron attend Georgetown?
  - **\[Market\]** Will Barron attend NYU?
  - **\[Market\]** Will Barron attend UPenn?
  - **\[Market\]** Will Barron attend Harvard?
  - **\[Market\]** Will Barron attend another college?

[Overview](https://docs.polymarket.com/developers/gamma-markets-api/overview) [Fetching Markets](https://docs.polymarket.com/developers/gamma-markets-api/fetch-markets-guide)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.