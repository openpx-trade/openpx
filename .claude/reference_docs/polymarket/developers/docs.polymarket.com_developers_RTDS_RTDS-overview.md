---
url: "https://docs.polymarket.com/developers/RTDS/RTDS-overview"
title: "Real Time Data Socket - Polymarket Documentation"
---

[Skip to main content](https://docs.polymarket.com/developers/RTDS/RTDS-overview#content-area)

[Polymarket Documentation home page![light logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-black.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=aff81820f1f3d577fecb3956a8a3bee1)![dark logo](https://mintcdn.com/polymarket-292d1b1b/HmeJ4Y1FlVRRp8nd/images/logo-white.svg?fit=max&auto=format&n=HmeJ4Y1FlVRRp8nd&q=85&s=3bc6857b5dbe8b74b9a7d40975c19b2b)](https://docs.polymarket.com/)

Search...

Ctrl KAsk AI

Search...

Navigation

Real Time Data Stream

Real Time Data Socket

[User Guide](https://docs.polymarket.com/polymarket-learn/get-started/what-is-polymarket) [For Developers](https://docs.polymarket.com/quickstart/overview) [Changelog](https://docs.polymarket.com/changelog/changelog)

On this page

- [Overview](https://docs.polymarket.com/developers/RTDS/RTDS-overview#overview)
- [Connection Details](https://docs.polymarket.com/developers/RTDS/RTDS-overview#connection-details)
- [Authentication](https://docs.polymarket.com/developers/RTDS/RTDS-overview#authentication)
- [Connection Management](https://docs.polymarket.com/developers/RTDS/RTDS-overview#connection-management)
- [Available Subscription Types](https://docs.polymarket.com/developers/RTDS/RTDS-overview#available-subscription-types)
- [Message Structure](https://docs.polymarket.com/developers/RTDS/RTDS-overview#message-structure)
- [Subscription Management](https://docs.polymarket.com/developers/RTDS/RTDS-overview#subscription-management)
- [Subscribe to Topics](https://docs.polymarket.com/developers/RTDS/RTDS-overview#subscribe-to-topics)
- [Unsubscribe from Topics](https://docs.polymarket.com/developers/RTDS/RTDS-overview#unsubscribe-from-topics)
- [Error Handling](https://docs.polymarket.com/developers/RTDS/RTDS-overview#error-handling)

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#overview)  Overview

The Polymarket Real-Time Data Socket (RTDS) is a WebSocket-based streaming service that provides real-time updates for **comments** and **crypto prices**. [**TypeScript client** \\
\\
Official RTDS TypeScript client (`real-time-data-client`).](https://github.com/Polymarket/real-time-data-client)

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#connection-details)  Connection Details

- **WebSocket URL**: `wss://ws-live-data.polymarket.com`
- **Protocol**: WebSocket
- **Data Format**: JSON

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#authentication)  Authentication

Some user-specific streams may require `gamma_auth`:

- `address`: User wallet address

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#connection-management)  Connection Management

The WebSocket connection supports:

- **Dynamic Subscriptions**: Without disconnecting from the socket users can add, remove and modify topics and filters they are subscribed to.
- **Ping/Pong**: You should send PING messages (every 5 seconds ideally) to maintain connection

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#available-subscription-types)  Available Subscription Types

Only the subscription types documented below are supported.

The RTDS currently supports the following subscription types:

1. **[Crypto Prices](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices)** \- Real-time cryptocurrency price updates
2. **[Comments](https://docs.polymarket.com/developers/RTDS/RTDS-comments)** \- Comment-related events including reactions

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#message-structure)  Message Structure

All messages received from the WebSocket follow this structure:

Copy

Ask AI

```
{
  "topic": "string",
  "type": "string",
  "timestamp": "number",
  "payload": "object"
}
```

- `topic`: The subscription topic (e.g., “crypto\_prices”, “comments”)
- `type`: The message type/event (e.g., “update”, “reaction\_created”)
- `timestamp`: Unix timestamp in milliseconds
- `payload`: Event-specific data object

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#subscription-management)  Subscription Management

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#subscribe-to-topics)  Subscribe to Topics

To subscribe to data streams, send a JSON message with this structure:

Copy

Ask AI

```
{
  "action": "subscribe",
  "subscriptions": [\
    {\
      "topic": "topic_name",\
      "type": "message_type",\
      "filters": "optional_filter_string",\
      "gamma_auth": {\
        "address": "wallet_address"\
      }\
    }\
  ]
}
```

### [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#unsubscribe-from-topics)  Unsubscribe from Topics

To unsubscribe from data streams, send a similar message with `"action": "unsubscribe"`.

## [​](https://docs.polymarket.com/developers/RTDS/RTDS-overview\#error-handling)  Error Handling

- Connection errors will trigger automatic reconnection attempts
- Invalid subscription messages may result in connection closure
- Authentication failures will prevent successful subscription to protected topics

[Quickstart](https://docs.polymarket.com/developers/sports-websocket/quickstart) [RTDS Crypto Prices](https://docs.polymarket.com/developers/RTDS/RTDS-crypto-prices)

Ctrl+I

Assistant

Responses are generated using AI and may contain mistakes.