---
url: "https://docs.polymarket.com/developers/CLOB/websocket/market-channel"
title: "Market Channel - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Websocket

Market Channel

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [book Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#book-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#structure)
- [price\_change Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#price_change-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#structure-2)
- [tick\_size\_change Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#tick_size_change-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#structure-3)
- [last\_trade\_price Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#last_trade_price-message)
- [best\_bid\_ask Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#best_bid_ask-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#structure-4)
- [Example](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#example)
- [new\_market Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#new_market-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#structure-5)
- [Example](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#example-2)
- [market\_resolved Message](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#market_resolved-message)
- [Structure](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#structure-6)
- [Example](https://docs.polymarket.com/developers/CLOB/websocket/market-channel#example-3)

Public channel for updates related to market updates (level 2 price data).**SUBSCRIBE**`<wss-channel> market`

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#book-message)  book Message

Emitted When:

- First subscribed to a market
- When there is a trade that affects the book

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#structure)  Structure

| Name | Type | Description |
| --- | --- | --- |
| event\_type | string | ”book” |
| asset\_id | string | asset ID (token ID) |
| market | string | condition ID of market |
| timestamp | string | unix timestamp the current book generation in milliseconds (1/1,000 second) |
| hash | string | hash summary of the orderbook content |
| buys | OrderSummary\[\] | list of type (size, price) aggregate book levels for buys |
| sells | OrderSummary\[\] | list of type (size, price) aggregate book levels for sells |

Where a `OrderSummary` object is of the form:

| Name | Type | Description |
| --- | --- | --- |
| price | string | price of the orderbook level |
| size | string | size available at that price level |

Response

Copy

Ask AI

```
{
  "event_type": "book",
  "asset_id": "65818619657568813474341868652308942079804919287380422192892211131408793125422",
  "market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
  "bids": [\
    { "price": ".48", "size": "30" },\
    { "price": ".49", "size": "20" },\
    { "price": ".50", "size": "15" }\
  ],
  "asks": [\
    { "price": ".52", "size": "25" },\
    { "price": ".53", "size": "60" },\
    { "price": ".54", "size": "10" }\
  ],
  "timestamp": "123456789000",
  "hash": "0x0...."
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#price_change-message)  price\_change Message

**⚠️ Breaking Change Notice:** The price\_change message schema will be updated on September 15, 2025 at 11 PM UTC. Please see the [migration guide](https://docs.polymarket.com/developers/CLOB/websocket/market-channel-migration-guide) for details.

Emitted When:

- A new order is placed
- An order is cancelled

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#structure-2)  Structure

| Name | Type | Description |
| --- | --- | --- |
| event\_type | string | ”price\_change” |
| market | string | condition ID of market |
| price\_changes | PriceChange\[\] | array of price change objects |
| timestamp | string | unix timestamp in milliseconds |

Where a `PriceChange` object is of the form:

| Name | Type | Description |
| --- | --- | --- |
| asset\_id | string | asset ID (token ID) |
| price | string | price level affected |
| size | string | new aggregate size for price level |
| side | string | ”BUY” or “SELL” |
| hash | string | hash of the order |
| best\_bid | string | current best bid price |
| best\_ask | string | current best ask price |

Response

Copy

Ask AI

```
{
    "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
    "price_changes": [\
        {\
            "asset_id": "71321045679252212594626385532706912750332728571942532289631379312455583992563",\
            "price": "0.5",\
            "size": "200",\
            "side": "BUY",\
            "hash": "56621a121a47ed9333273e21c83b660cff37ae50",\
            "best_bid": "0.5",\
            "best_ask": "1"\
        },\
        {\
            "asset_id": "52114319501245915516055106046884209969926127482827954674443846427813813222426",\
            "price": "0.5",\
            "size": "200",\
            "side": "SELL",\
            "hash": "1895759e4df7a796bf4f1c5a5950b748306923e2",\
            "best_bid": "0",\
            "best_ask": "0.5"\
        }\
    ],
    "timestamp": "1757908892351",
    "event_type": "price_change"
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#tick_size_change-message)  tick\_size\_change Message

Emitted When:

- The minimum tick size of the market changes. This happens when the book’s price reaches the limits: price > 0.96 or price < 0.04

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#structure-3)  Structure

| Name | Type | Description |
| --- | --- | --- |
| event\_type | string | ”price\_change” |
| asset\_id | string | asset ID (token ID) |
| market | string | condition ID of market |
| old\_tick\_size | string | previous minimum tick size |
| new\_tick\_size | string | current minimum tick size |
| side | string | buy/sell |
| timestamp | string | time of event |

Response

Copy

Ask AI

```
{
"event_type": "tick_size_change",
"asset_id": "65818619657568813474341868652308942079804919287380422192892211131408793125422",\
"market": "0xbd31dc8a20211944f6b70f31557f1001557b59905b7738480ca09bd4532f84af",
"old_tick_size": "0.01",
"new_tick_size": "0.001",
"timestamp": "100000000"
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#last_trade_price-message)  last\_trade\_price Message

Emitted When:

- When a maker and taker order is matched creating a trade event.

Response

Copy

Ask AI

```
{
"asset_id":"114122071509644379678018727908709560226618148003371446110114509806601493071694",
"event_type":"last_trade_price",
"fee_rate_bps":"0",
"market":"0x6a67b9d828d53862160e470329ffea5246f338ecfffdf2cab45211ec578b0347",
"price":"0.456",
"side":"BUY",
"size":"219.217767",
"timestamp":"1750428146322"
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#best_bid_ask-message)  best\_bid\_ask Message

Emitted When:

- The best bid and ask prices for a market change.

(This message is behind the `custom_feature_enabled` flag)

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#structure-4)  Structure

| Name | Type | Description |
| --- | --- | --- |
| event\_type | string | ”best\_bid\_ask” |
| market | string | condition ID of market |
| asset\_id | string | asset ID (token ID) |
| best\_bid | string | current best bid price |
| best\_ask | string | current best ask price |
| spread | string | spread between best bid and ask |
| timestamp | string | unix timestamp in milliseconds |

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#example)  Example

Response

Copy

Ask AI

```
{
  "event_type": "best_bid_ask",
  "market": "0x0005c0d312de0be897668695bae9f32b624b4a1ae8b140c49f08447fcc74f442",
  "asset_id": "85354956062430465315924116860125388538595433819574542752031640332592237464430",
  "best_bid": "0.73",
  "best_ask": "0.77",
  "spread": "0.04",
  "timestamp": "1766789469958"
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#new_market-message)  new\_market Message

Emitted When:

- A new market is created.

(This message is behind the `custom_feature_enabled` flag)

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#structure-5)  Structure

| Name | Type | Description |
| --- | --- | --- |
| id | string | market ID |
| question | string | market question |
| market | string | condition ID of market |
| slug | string | market slug |
| description | string | market description |
| assets\_ids | string\[\] | list of asset IDs |
| outcomes | string\[\] | list of outcomes |
| event\_message | object | event message object |
| timestamp | string | unix timestamp in milliseconds |
| event\_type | string | ”new\_market” |

Where a `EventMessage` object is of the form:

| Name | Type | Description |
| --- | --- | --- |
| id | string | event message ID |
| ticker | string | event message ticker |
| slug | string | event message slug |
| title | string | event message title |
| description | string | event message description |

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#example-2)  Example

Response

Copy

Ask AI

```
{
    "id": "1031769",
    "question": "Will NVIDIA (NVDA) close above $240 end of January?",
    "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
    "slug": "nvda-above-240-on-january-30-2026",
    "description": "This market will resolve to \"Yes\" if the official closing price for NVIDIA (NVDA) on the final trading day of January 2026 is higher than the listed price. Otherwise, this market will resolve to \"No\".\n\nIf the final trading day of the month is shortened (for example, due to a market-holiday schedule), the official closing price published for that shortened session will still be used for resolution.\n\nIf no official closing price is published for that session (for example, due to a trading halt into the close, system issue, or other disruption), the market will use the last valid on-exchange trade price of the regular session as the effective closing price.\n\nThe resolution source for this market is Yahoo Finance — specifically, the NVIDIA (NVDA) \"Close\" prices available at https://finance.yahoo.com/quote/NVDA/history, published under \"Historical Prices.\"\n\nIn the event of a stock split, reverse stock split, or similar corporate action affecting the listed company during the listed time frame, this market will resolve based on split-adjusted prices as displayed on Yahoo Finance.",
    "assets_ids": [\
        "76043073756653678226373981964075571318267289248134717369284518995922789326425",\
        "31690934263385727664202099278545688007799199447969475608906331829650099442770"\
    ],
    "outcomes": [\
        "Yes",\
        "No"\
    ],
    "event_message": {
        "id": "125819",
        "ticker": "nvda-above-in-january-2026",
        "slug": "nvda-above-in-january-2026",
        "title": "Will NVIDIA (NVDA) close above ___ end of January?",
        "description": "This market will resolve to \"Yes\" if the official closing price for NVIDIA (NVDA) on the final trading day of January 2026 is higher than the listed price. Otherwise, this market will resolve to \"No\".\n\nIf the final trading day of the month is shortened (for example, due to a market-holiday schedule), the official closing price published for that shortened session will still be used for resolution.\n\nIf no official closing price is published for that session (for example, due to a trading halt into the close, system issue, or other disruption), the market will use the last valid on-exchange trade price of the regular session as the effective closing price.\n\nThe resolution source for this market is Yahoo Finance — specifically, the NVIDIA (NVDA) \"Close\" prices available at https://finance.yahoo.com/quote/NVDA/history, published under \"Historical Prices.\"\n\nIn the event of a stock split, reverse stock split, or similar corporate action affecting the listed company during the listed time frame, this market will resolve based on split-adjusted prices as displayed on Yahoo Finance."
    },
    "timestamp": "1766790415550",
    "event_type": "new_market"
}
```

## [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#market_resolved-message)  market\_resolved Message

Emitted When:

- A market is resolved.

(This message is behind the `custom_feature_enabled` flag)

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#structure-6)  Structure

| Name | Type | Description |
| --- | --- | --- |
| id | string | market ID |
| question | string | market question |
| market | string | condition ID of market |
| slug | string | market slug |
| description | string | market description |
| assets\_ids | string\[\] | list of asset IDs |
| outcomes | string\[\] | list of outcomes |
| winning\_asset\_id | string | winning asset ID |
| winning\_outcome | string | winning outcome |
| event\_message | object | event message object |
| timestamp | string | unix timestamp in milliseconds |
| event\_type | string | ”market\_resolved” |

Where a `EventMessage` object is of the form:

| Name | Type | Description |
| --- | --- | --- |
| id | string | event message ID |
| ticker | string | event message ticker |
| slug | string | event message slug |
| title | string | event message title |
| description | string | event message description |

### [​](https://docs.polymarket.com/developers/CLOB/websocket/market-channel\#example-3)  Example

Response

Copy

Ask AI

```
{
    "id": "1031769",
    "question": "Will NVIDIA (NVDA) close above $240 end of January?",
    "market": "0x311d0c4b6671ab54af4970c06fcf58662516f5168997bdda209ec3db5aa6b0c1",
    "slug": "nvda-above-240-on-january-30-2026",
    "description": "This market will resolve to \"Yes\" if the official closing price for NVIDIA (NVDA) on the final trading day of January 2026 is higher than the listed price. Otherwise, this market will resolve to \"No\".\n\nIf the final trading day of the month is shortened (for example, due to a market-holiday schedule), the official closing price published for that shortened session will still be used for resolution.\n\nIf no official closing price is published for that session (for example, due to a trading halt into the close, system issue, or other disruption), the market will use the last valid on-exchange trade price of the regular session as the effective closing price.\n\nThe resolution source for this market is Yahoo Finance — specifically, the NVIDIA (NVDA) \"Close\" prices available at https://finance.yahoo.com/quote/NVDA/history, published under \"Historical Prices.\"\n\nIn the event of a stock split, reverse stock split, or similar corporate action affecting the listed company during the listed time frame, this market will resolve based on split-adjusted prices as displayed on Yahoo Finance.",
    "assets_ids": [\
        "76043073756653678226373981964075571318267289248134717369284518995922789326425",\
        "31690934263385727664202099278545688007799199447969475608906331829650099442770"\
    ],
    "winning_asset_id": "76043073756653678226373981964075571318267289248134717369284518995922789326425",
    "winning_outcome": "Yes",
    "event_message": {
        "id": "125819",
        "ticker": "nvda-above-in-january-2026",
        "slug": "nvda-above-in-january-2026",
        "title": "Will NVIDIA (NVDA) close above ___ end of January?",
        "description": "This market will resolve to \"Yes\" if the official closing price for NVIDIA (NVDA) on the final trading day of January 2026 is higher than the listed price. Otherwise, this market will resolve to \"No\".\n\nIf the final trading day of the month is shortened (for example, due to a market-holiday schedule), the official closing price published for that shortened session will still be used for resolution.\n\nIf no official closing price is published for that session (for example, due to a trading halt into the close, system issue, or other disruption), the market will use the last valid on-exchange trade price of the regular session as the effective closing price.\n\nThe resolution source for this market is Yahoo Finance — specifically, the NVIDIA (NVDA) \"Close\" prices available at https://finance.yahoo.com/quote/NVDA/history, published under \"Historical Prices.\"\n\nIn the event of a stock split, reverse stock split, or similar corporate action affecting the listed company during the listed time frame, this market will resolve based on split-adjusted prices as displayed on Yahoo Finance."
    },
    "timestamp": "1766790415550",
    "event_type": "new_market"
}
```

[User Channel](https://docs.polymarket.com/developers/CLOB/websocket/user-channel) [Overview](https://docs.polymarket.com/developers/sports-websocket/overview)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.