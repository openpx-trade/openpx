---
url: "https://docs.polymarket.com/developers/CLOB/websocket/user-channel"
title: "User Channel - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/websocket/user-channel#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Websocket

User Channel

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Trade Message](https://docs.polymarket.com/developers/CLOB/websocket/user-channel#trade-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/user-channel#structure)
- [Order Message](https://docs.polymarket.com/developers/CLOB/websocket/user-channel#order-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/user-channel#structure-2)

Authenticated channel for updates related to user activities (orders, trades), filtered for authenticated user by apikey.**SUBSCRIBE**`<wss-channel> user`

## [​](https://docs.polymarket.com/developers/CLOB/websocket/user-channel\#trade-message)  Trade Message

Emitted when:

- when a market order is matched (“MATCHED”)
- when a limit order for the user is included in a trade (“MATCHED”)
- subsequent status changes for trade (“MINED”, “CONFIRMED”, “RETRYING”, “FAILED”)

### [​](https://docs.polymarket.com/developers/CLOB/websocket/user-channel\#structure)  Structure

| Name | Type | Description |
| --- | --- | --- |
| asset\_id | string | asset id (token ID) of order (market order) |
| event\_type | string | ”trade” |
| id | string | trade id |
| last\_update | string | time of last update to trade |
| maker\_orders | MakerOrder\[\] | array of maker order details |
| market | string | market identifier (condition ID) |
| matchtime | string | time trade was matched |
| outcome | string | outcome |
| owner | string | api key of event owner |
| price | string | price |
| side | string | BUY/SELL |
| size | string | size |
| status | string | trade status |
| taker\_order\_id | string | id of taker order |
| timestamp | string | time of event |
| trade\_owner | string | api key of trade owner |
| type | string | ”TRADE” |

Where a `MakerOrder` object is of the form:

| Name | Type | Description |
| --- | --- | --- |
| asset\_id | string | asset of the maker order |
| matched\_amount | string | amount of maker order matched in trade |
| order\_id | string | maker order ID |
| outcome | string | outcome |
| owner | string | owner of maker order |
| price | string | price of maker order |

Response

Copy

Ask AI

```
{
  "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",
  "event_type": "trade",
  "id": "28c4d2eb-bbea-40e7-a9f0-b2fdb56b2c2e",
  "last_update": "1672290701",
  "maker_orders": [\
    {\
      "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",\
      "matched_amount": "10",\
      "order_id": "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b",\
      "outcome": "YES",\
      "owner": "9180014b-33c8-9240-a14b-bdca11c0a465",\
      "price": "0.57"\
    }\
  ],
  "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
  "matchtime": "1672290701",
  "outcome": "YES",
  "owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
  "price": "0.57",
  "side": "BUY",
  "size": "10",
  "status": "MATCHED",
  "taker_order_id": "0x06bc63e346ed4ceddce9efd6b3af37c8f8f440c92fe7da6b2d0f9e4ccbc50c42",
  "timestamp": "1672290701",
  "trade_owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
  "type": "TRADE"
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/user-channel\#order-message)  Order Message

Emitted when:

- When an order is placed (PLACEMENT)
- When an order is updated (some of it is matched) (UPDATE)
- When an order is canceled (CANCELLATION)

### [​](https://docs.polymarket.com/developers/CLOB/websocket/user-channel\#structure-2)  Structure

| Name | Type | Description |
| --- | --- | --- |
| asset\_id | string | asset ID (token ID) of order |
| associate\_trades | string\[\] | array of ids referencing trades that the order has been included in |
| event\_type | string | ”order” |
| id | string | order id |
| market | string | condition ID of market |
| order\_owner | string | owner of order |
| original\_size | string | original order size |
| outcome | string | outcome |
| owner | string | owner of orders |
| price | string | price of order |
| side | string | BUY/SELL |
| size\_matched | string | size of order that has been matched |
| timestamp | string | time of event |
| type | string | PLACEMENT/UPDATE/CANCELLATION |

Response

Copy

Ask AI

```
{
  "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",
  "associate_trades": null,
  "event_type": "order",
  "id": "0xff354cd7ca7539dfa9c28d90943ab5779a4eac34b9b37a757d7b32bdfb11790b",
  "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
  "order_owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
  "original_size": "10",
  "outcome": "YES",
  "owner": "9180014b-33c8-9240-a14b-bdca11c0a465",
  "price": "0.57",
  "side": "SELL",
  "size_matched": "0",
  "timestamp": "1672290687",
  "type": "PLACEMENT"
}
```

[WSS Authentication](https://docs.polymarket.com/developers/CLOB/websocket/wss-auth) [Market Channel](https://docs.polymarket.com/developers/CLOB/websocket/market-channel)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.