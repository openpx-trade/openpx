
OpenPX is an open-source, CCXT-style unified SDK for prediction markets.
Users bring their own exchange credentials and trade directly through a single interface.

## Supported Exchanges

| Exchange | Market Data | Trading | WebSocket |
|----------|------------|---------|-----------|
| Kalshi | Yes | Yes | Yes |
| Polymarket | Yes | Yes | Yes |
| Opinion | Yes | Yes | Yes |

## Architecture

```
Rust types (#[derive(JsonSchema)])
        |
        v
px-schema binary → schema/openpx.schema.json
        |                    |
        v                    v
datamodel-codegen      json-schema-to-typescript
        |                    |
        v                    v
Python SDK             TypeScript SDK
(Pydantic v2)          (TS interfaces)
        |                    |
        v                    v
PyO3 native ext        NAPI-RS native addon
```

All contributions go to **Rust only** — language SDKs are automatically
regenerated from Rust types via `just sync-all`.---
title: Installation
title: Quick Start
title: API Methods

## Exchange Info

### id / name

Get the exchange identifier and human-readable name.

No parameters.

**Returns:** `string`



**Rust**

```rust
let id: &str = exchange.id();     // "kalshi"
let name: &str = exchange.name(); // "Kalshi"
```


**Python**

```python
exchange.id    # "kalshi"
exchange.name  # "Kalshi"
```


**TypeScript**

```typescript
exchange.id;   // "kalshi"
exchange.name; // "Kalshi"
```


**CLI**

```bash
# The CLI implicitly uses the exchange ID you pass:
openpx kalshi fetch-markets --limit 1 | jq '.markets[0].exchange'
```



### describe

Returns capability flags for this exchange — which methods are supported.

No parameters.

**Returns:** [`ExchangeInfo`](/reference/models/#exchangeinfo)



**Rust**

```rust
let info = exchange.describe();
println!("Has WebSocket: {}", info.has_websocket);
println!("Has price history: {}", info.has_fetch_price_history);
```


**Python**

```python
info = exchange.describe()
print(f"Has WebSocket: {info['has_websocket']}")
print(f"Has price history: {info['has_fetch_price_history']}")
```


**TypeScript**

```typescript
const info = exchange.describe();
console.log(`Has WebSocket: ${info.has_websocket}`);
console.log(`Has price history: ${info.has_fetch_price_history}`);
```


**CLI**

```bash
# describe is available via the SDK; the CLI does not have a dedicated command for it.
```




## Trading

### create_order

Place a limit order on a market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to trade |
| `outcome` | `string` | **Yes** | Outcome to buy/sell (e.g. `"Yes"`, `"No"`) |
| `side` | `OrderSide` | **Yes** | `Buy` or `Sell` |
| `price` | `float` | **Yes** | Limit price (0.0 – 1.0) |
| `size` | `float` | **Yes** | Number of contracts |
| `params` | `map[string, string]` | No | Optional parameters. `order_type`: `gtc` (default), `ioc`, or `fok`. All three exchanges support all three order types |

**Returns:** [`Order`](/reference/models/#order)



**Rust**

```rust
use px_core::OrderSide;
use std::collections::HashMap;

let order = exchange.create_order(
    "KXBTC-25MAR14",
    "Yes",
    OrderSide::Buy,
    0.65,
    10.0,
    HashMap::new(),
).await?;

println!("Order {}: {:?}", order.id, order.status);
```


**Python**

```python
order = exchange.create_order(
    market_id="KXBTC-25MAR14",
    outcome="Yes",
    side="buy",
    price=0.65,
    size=10.0,
)
print(f"Order {order.id}: {order.status}")
```


**TypeScript**

```typescript
const order = await exchange.createOrder(
  "KXBTC-25MAR14", "Yes", "buy", 0.65, 10.0
);
console.log(`Order ${order.id}: ${order.status}`);
```


**CLI**

```bash
# create-order is available via the SDK; the CLI is read-only.
```



### cancel_order

Cancel an open order.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | `string` | **Yes** | Order to cancel |
| `market_id` | `string` | No | Market ID — required by some exchanges for faster lookup |

**Returns:** [`Order`](/reference/models/#order)



**Rust**

```rust
let cancelled = exchange.cancel_order("order-123", None).await?;
println!("Cancelled: {:?}", cancelled.status);
```


**Python**

```python
cancelled = exchange.cancel_order("order-123")
print(f"Cancelled: {cancelled.status}")
```


**TypeScript**

```typescript
const cancelled = await exchange.cancelOrder("order-123");
console.log(`Cancelled: ${cancelled.status}`);
```


**CLI**

```bash
# cancel-order is available via the SDK; the CLI is read-only.
```



### fetch_order

Fetch a single order by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | `string` | **Yes** | Order ID |
| `market_id` | `string` | No | Market ID — required by some exchanges for faster lookup |

**Returns:** [`Order`](/reference/models/#order)



**Rust**

```rust
let order = exchange.fetch_order("order-123", None).await?;
println!("Status: {:?}, Filled: {}", order.status, order.filled);
```


**Python**

```python
order = exchange.fetch_order("order-123")
print(f"Status: {order.status}, Filled: {order.filled}")
```


**TypeScript**

```typescript
const order = await exchange.fetchOrder("order-123");
console.log(`Status: ${order.status}, Filled: ${order.filled}`);
```


**CLI**

```bash
openpx kalshi fetch-order ORDER_ID
openpx polymarket fetch-order ORDER_ID --market-id "0x1234..."
```



### fetch_open_orders

Fetch all open orders, optionally filtered by market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | No | Filter to a specific market |

**Returns:** `list[Order]` — see [`Order`](/reference/models/#order)



**Rust**

```rust
use px_core::FetchOrdersParams;

// All open orders
let orders = exchange.fetch_open_orders(None).await?;

// For a specific market
let orders = exchange.fetch_open_orders(Some(FetchOrdersParams {
    market_id: Some("KXBTC-25MAR14".into()),
})).await?;

for o in &orders {
    println!("{}: {} @ {:.2}", o.id, o.side, o.price);
}
```


**Python**

```python
# All open orders
orders = exchange.fetch_open_orders()

# For a specific market
orders = exchange.fetch_open_orders(market_id="KXBTC-25MAR14")

for o in orders:
    print(f"{o.id}: {o.side} @ {o.price:.2f}")
```


**TypeScript**

```typescript
// All open orders
const orders = await exchange.fetchOpenOrders();

// For a specific market
const orders = await exchange.fetchOpenOrders("KXBTC-25MAR14");

for (const o of orders) {
  console.log(`${o.id}: ${o.side} @ ${o.price.toFixed(2)}`);
}
```


**CLI**

```bash
openpx kalshi fetch-open-orders
openpx polymarket fetch-open-orders --market-id "0x1234..."
```




## Orderbook

### fetch_orderbook

Fetch the L2 orderbook for a market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market ID |
| `outcome` | `string` | No | Filter by outcome (e.g. `"Yes"`) |
| `token_id` | `string` | No | Filter by CTF token ID (Polymarket) |

**Returns:** [`Orderbook`](/reference/models/#orderbook)



**Rust**

```rust
use px_core::OrderbookRequest;

let book = exchange.fetch_orderbook(OrderbookRequest {
    market_id: "KXBTC-25MAR14".into(),
    outcome: None,
    token_id: None,
}).await?;

println!("Best bid: {:?}", book.best_bid());
println!("Best ask: {:?}", book.best_ask());
println!("Spread: {:?}", book.spread());

for level in &book.bids[..5.min(book.bids.len())] {
    println!("  BID {:.2} x {}", level.price, level.size);
}
```


**Python**

```python
book = exchange.fetch_orderbook("KXBTC-25MAR14")

print(f"Best bid: {book['bids'][0]['price']}")
print(f"Best ask: {book['asks'][0]['price']}")

for level in book["bids"][:5]:
    print(f"  BID {level['price']:.2f} x {level['size']}")
```


**TypeScript**

```typescript
const book = await exchange.fetchOrderbook("KXBTC-25MAR14");

console.log(`Best bid: ${book.bids[0].price}`);
console.log(`Best ask: ${book.asks[0].price}`);

for (const level of book.bids.slice(0, 5)) {
  console.log(`  BID ${level.price.toFixed(2)} x ${level.size}`);
}
```


**CLI**

```bash
openpx kalshi fetch-orderbook KXBTC-25MAR14
openpx polymarket fetch-orderbook "0x1234..." --token-id "TOKEN_ID"

# Extract best bid/ask
openpx kalshi fetch-orderbook KXBTC-25MAR14 | jq '{
  best_bid: .bids[0].price,
  best_ask: .asks[0].price
}'
```



### fetch_orderbook_history

Fetch historical orderbook snapshots. Not all exchanges support this.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market ID |
| `token_id` | `string` | No | Token ID |
| `start_ts` | `int` | No | Start time (Unix seconds, inclusive) |
| `end_ts` | `int` | No | End time (Unix seconds, inclusive) |
| `limit` | `int` | No | Max snapshots per page |
| `cursor` | `string` | No | Pagination cursor from a previous response |

**Returns:** `(list[OrderbookSnapshot], cursor?)` — see [`OrderbookSnapshot`](/reference/models/#orderbooksnapshot).
The cursor is `null` when there are no more pages.



**Rust**

```rust
use px_core::OrderbookHistoryRequest;

let (snapshots, next_cursor) = exchange.fetch_orderbook_history(
    OrderbookHistoryRequest {
        market_id: "KXBTC-25MAR14".into(),
        limit: Some(10),
        ..Default::default()
    }
).await?;

for snap in &snapshots {
    println!("{}: {} bids, {} asks",
        snap.timestamp, snap.bids.len(), snap.asks.len());
}
```


**Python**

```python
result = exchange.fetch_orderbook_history("KXBTC-25MAR14", limit=10)

for snap in result["snapshots"]:
    print(f"{snap.timestamp}: {len(snap.bids)} bids, {len(snap.asks)} asks")

# Paginate
if result["cursor"]:
    next_page = exchange.fetch_orderbook_history("KXBTC-25MAR14", cursor=result["cursor"])
```


**TypeScript**

```typescript
const result = await exchange.fetchOrderbookHistory("KXBTC-25MAR14", undefined, undefined, undefined, 10);

for (const snap of result.snapshots) {
  console.log(`${snap.timestamp}: ${snap.bids.length} bids, ${snap.asks.length} asks`);
}

// Paginate
if (result.cursor) {
  const nextPage = await exchange.fetchOrderbookHistory("KXBTC-25MAR14", undefined, undefined, undefined, undefined, result.cursor);
}
```


**CLI**

```bash
openpx kalshi fetch-orderbook-history KXBTC-25MAR14
openpx kalshi fetch-orderbook-history KXBTC-25MAR14 --start-ts 1700000000 --limit 10
```




## WebSocket

Real-time streaming via WebSocket for orderbook updates, trades, and fills.

See the [WebSocket Streaming](/guides/websocket/) guide for full documentation.



**Rust**

```rust
use px_core::OrderBookWebSocket;
use futures::StreamExt;

let mut ws = exchange.websocket().unwrap();

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.orderbook_stream("KXBTC-25MAR14").await?;
while let Some(update) = stream.next().await {
    match update? {
        OrderbookUpdate::Snapshot(book) => {
            println!("Snapshot: {} bids, {} asks", book.bids.len(), book.asks.len());
        }
        OrderbookUpdate::Delta { changes, .. } => {
            for c in &changes {
                println!("  {:?} {:.2} x {}", c.side, c.price, c.size);
            }
        }
    }
}
```


**Python**

```python
ws = exchange.websocket()
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for update in ws.orderbook_stream("KXBTC-25MAR14"):
    if update["type"] == "Snapshot":
        book = update["Snapshot"]
        print(f"Snapshot: {len(book['bids'])} bids, {len(book['asks'])} asks")
    elif update["type"] == "Delta":
        for c in update["Delta"]["changes"]:
            print(f"  {c['side']} {c['price']:.2f} x {c['size']}")

ws.disconnect()
```


**TypeScript**

```typescript
const ws = exchange.websocket();
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onOrderbookUpdate("KXBTC-25MAR14", (err, update) => {
  if (err) { console.error(err); return; }
  if (update.type === "Snapshot") {
    console.log(`Snapshot: ${update.Snapshot.bids.length} bids`);
  } else {
    for (const c of update.Delta.changes) {
      console.log(`  ${c.side} ${c.price} x ${c.size}`);
    }
  }
});
```


**CLI**

```bash
# Stream live orderbook updates (JSON per line)
openpx kalshi ws-orderbook KXBTC-25MAR14
openpx polymarket ws-orderbook "0x1234..."

# Stream live trade and fill events
openpx kalshi ws-activity KXBTC-25MAR14
openpx polymarket ws-activity "0x1234..."

# Extract snapshots with jq
openpx kalshi ws-orderbook KXBTC-25MAR14 | jq 'select(.type == "Snapshot") | {
  best_bid: .Snapshot.bids[0].price,
  best_ask: .Snapshot.asks[0].price
}'
```




OpenPX provides real-time streaming via WebSocket for orderbook updates,
trades, and fill events across all supported exchanges.

## Exchange Support

| Exchange | Orderbook | Trades | Fills | Protocol |
|----------|-----------|--------|-------|----------|
| Kalshi | Yes | Yes | Yes | Native WS |
| Polymarket | Yes | Yes | Yes | Native WS (dual connection) |
| Opinion | Yes | Yes | Yes | Native WS |

## Connection Lifecycle



**Rust**

```rust
use px_core::OrderBookWebSocket;

// 1. Get WebSocket handle from exchange
let mut ws = exchange.websocket().unwrap();

// 2. Connect
ws.connect().await?;

// 3. Subscribe to markets
ws.subscribe("KXBTC-25MAR14").await?;
ws.subscribe("KXETH-25MAR14").await?;

// 4. Stream data (see sections below)
// ...

// 5. Unsubscribe when done
ws.unsubscribe("KXBTC-25MAR14").await?;

// 6. Disconnect
ws.disconnect().await?;
```


**Python**

```python
from openpx import Exchange

exchange = Exchange("kalshi", {
    "api_key_id": "your-key",
    "private_key_pem": "your-pem",
})
ws = exchange.websocket()

# 2. Connect
ws.connect()

# 3. Subscribe to markets
ws.subscribe("KXBTC-25MAR14")
ws.subscribe("KXETH-25MAR14")

# 4. Stream data (see sections below)
# ...

# 5. Unsubscribe when done
ws.unsubscribe("KXBTC-25MAR14")

# 6. Disconnect
ws.disconnect()
```


**TypeScript**

```typescript

const exchange = new Exchange("kalshi", {
  api_key_id: "your-key",
  private_key_pem: "your-pem",
});
const ws = exchange.websocket();

// 2. Connect
await ws.connect();

// 3. Subscribe to markets
await ws.subscribe("KXBTC-25MAR14");
await ws.subscribe("KXETH-25MAR14");

// 4. Stream data (see sections below)
// ...

// 5. Unsubscribe when done
await ws.unsubscribe("KXBTC-25MAR14");

// 6. Disconnect
await ws.disconnect();
```


**CLI**

```bash
# Stream orderbook updates (JSON per line, Ctrl+C to stop)
openpx kalshi ws-orderbook KXBTC-25MAR14

# Stream trade and fill events
openpx kalshi ws-activity KXBTC-25MAR14
```



## Method Reference

### connect

Open the WebSocket connection. Must be called before subscribing or streaming.

No parameters.

**Returns:** `void` — throws on connection failure.

### disconnect

Close the WebSocket connection and clean up resources.

No parameters.

**Returns:** `void`

### subscribe

Subscribe to a market to begin receiving updates. You can subscribe to
multiple markets on the same connection.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to subscribe to |

**Returns:** `void` — throws if the market ID is invalid or the connection is not open.

### unsubscribe

Stop receiving updates for a market.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to unsubscribe from |

**Returns:** `void`

### state

Check the connection state. Uses lock-free atomics for zero-cost reads.

No parameters.

**Returns:** `WebSocketState` — one of `Disconnected`, `Connecting`, `Connected`, `Reconnecting`, or `Closed`.



**Rust**

```rust
use px_core::WebSocketState;

match ws.state() {
    WebSocketState::Disconnected => println!("Not connected"),
    WebSocketState::Connecting => println!("Connecting..."),
    WebSocketState::Connected => println!("Ready"),
    WebSocketState::Reconnecting => println!("Reconnecting..."),
    WebSocketState::Closed => println!("Closed"),
}
```


**Python**

```python
state = ws.state  # "Connected", "Disconnected", etc.
print(f"WebSocket state: {state}")
```


**TypeScript**

```typescript
const state = ws.state; // "Connected", "Disconnected", etc.
console.log(`WebSocket state: ${state}`);
```



### orderbook_stream

Open a stream of real-time orderbook updates for a subscribed market. The
first message is always a full `Snapshot`, followed by incremental `Delta`
updates. You must call `subscribe` before opening a stream.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to stream (must be already subscribed) |

**Returns:** `Stream[OrderbookUpdate]` — yields `Snapshot` or `Delta` events.



**Rust**

```rust
use px_core::{OrderBookWebSocket, OrderbookUpdate};
use futures::StreamExt;

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.orderbook_stream("KXBTC-25MAR14").await?;

while let Some(update) = stream.next().await {
    match update? {
        OrderbookUpdate::Snapshot(book) => {
            println!("Full snapshot:");
            println!("  Best bid: {:?}", book.bids.first());
            println!("  Best ask: {:?}", book.asks.first());
            println!("  {} bids, {} asks", book.bids.len(), book.asks.len());
        }
        OrderbookUpdate::Delta { changes, timestamp } => {
            for change in &changes {
                let action = if change.size == 0.0 { "REMOVE" } else { "UPDATE" };
                println!("  {} {:?} {:.2} x {}",
                    action, change.side, change.price, change.size);
            }
        }
    }
}
```


**Python**

```python
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for update in ws.orderbook_stream("KXBTC-25MAR14"):
    if update["type"] == "Snapshot":
        book = update["Snapshot"]
        print(f"Full snapshot:")
        print(f"  Best bid: {book['bids'][0]}")
        print(f"  Best ask: {book['asks'][0]}")
        print(f"  {len(book['bids'])} bids, {len(book['asks'])} asks")
    elif update["type"] == "Delta":
        delta = update["Delta"]
        for change in delta["changes"]:
            action = "REMOVE" if change["size"] == 0 else "UPDATE"
            print(f"  {action} {change['side']} {change['price']:.2f} x {change['size']}")
```


**TypeScript**

```typescript
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onOrderbookUpdate("KXBTC-25MAR14", (err, update) => {
  if (err) { console.error(err); return; }
  if (update.type === "Snapshot") {
    const book = update.Snapshot;
    console.log(`Full snapshot:`);
    console.log(`  Best bid: ${JSON.stringify(book.bids[0])}`);
    console.log(`  Best ask: ${JSON.stringify(book.asks[0])}`);
    console.log(`  ${book.bids.length} bids, ${book.asks.length} asks`);
  } else if (update.type === "Delta") {
    for (const change of update.Delta.changes) {
      const action = change.size === 0 ? "REMOVE" : "UPDATE";
      console.log(`  ${action} ${change.side} ${change.price} x ${change.size}`);
    }
  }
});
```


**CLI**

```bash
# Stream live orderbook updates
openpx kalshi ws-orderbook KXBTC-25MAR14
openpx polymarket ws-orderbook "0x1234..."

# Extract best bid/ask from snapshots
openpx kalshi ws-orderbook KXBTC-25MAR14 | jq 'select(.type == "Snapshot") | {
  best_bid: .Snapshot.bids[0].price,
  best_ask: .Snapshot.asks[0].price
}'
```



#### Update Types

| Type | Description |
|------|-------------|
| **Snapshot** | Full orderbook state. Sent on first subscribe and after reconnection. Contains complete `bids` and `asks` arrays. |
| **Delta** | Incremental change. Each change has `side` (Bid/Ask), `price`, and `size`. A `size` of `0` means remove that price level. |

See the [Type Reference](/reference/models/#orderbook) for the full `Orderbook`,
`PriceLevel`, and `PriceLevelChange` type definitions.

### activity_stream

Open a stream of real-time trade and fill events for a subscribed market.
Trades are public market activity; fills are your personal order executions.
You must call `subscribe` before opening a stream.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market_id` | `string` | **Yes** | Market to stream (must be already subscribed) |

**Returns:** `Stream[ActivityEvent]` — yields `Trade` or `Fill` events.



**Rust**

```rust
use px_core::{OrderBookWebSocket, ActivityEvent};
use futures::StreamExt;

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.activity_stream("KXBTC-25MAR14").await?;

while let Some(event) = stream.next().await {
    match event? {
        ActivityEvent::Trade(trade) => {
            println!("TRADE: {} x {} @ {:.2} [{}]",
                trade.outcome.unwrap_or_default(),
                trade.size, trade.price,
                trade.source_channel);
        }
        ActivityEvent::Fill(fill) => {
            println!("FILL: {} x {} @ {:.2} ({})",
                fill.outcome.unwrap_or_default(),
                fill.size, fill.price,
                fill.liquidity_role
                    .map(|r| format!("{:?}", r))
                    .unwrap_or_default());
        }
    }
}
```


**Python**

```python
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for event in ws.activity_stream("KXBTC-25MAR14"):
    if "Trade" in event:
        t = event["Trade"]
        print(f"TRADE: {t.get('outcome', '')} x {t['size']} @ {t['price']:.2f}")
    elif "Fill" in event:
        f = event["Fill"]
        print(f"FILL: {f.get('outcome', '')} x {f['size']} @ {f['price']:.2f}")
```


**TypeScript**

```typescript
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onActivityUpdate("KXBTC-25MAR14", (err, event) => {
  if (err) { console.error(err); return; }
  if (event.Trade) {
    const t = event.Trade;
    console.log(`TRADE: ${t.outcome} x ${t.size} @ ${t.price}`);
  } else if (event.Fill) {
    const f = event.Fill;
    console.log(`FILL: ${f.outcome} x ${f.size} @ ${f.price}`);
  }
});
```


**CLI**

```bash
# Stream live trade and fill events
openpx kalshi ws-activity KXBTC-25MAR14
openpx polymarket ws-activity "0x1234..."
```



#### Event Types

See [`ActivityTrade`](/reference/models/#activitytrade) and
[`ActivityFill`](/reference/models/#activityfill) for field details.

| Event | Description | Exchanges |
|-------|-------------|-----------|
| **Trade** | Public market trade. Includes price, size, aggressor side, and outcome. | Kalshi, Polymarket, Opinion |
| **Fill** | Your order was filled. Includes fill ID, order ID, liquidity role (maker/taker), and fee info. | Kalshi, Polymarket, Opinion |

## Auto-Reconnect

WebSocket connections automatically reconnect on failure with exponential
backoff. No user intervention required.

| Setting | Value |
|---------|-------|
| Ping interval | 20 seconds |
| Initial reconnect delay | 3 seconds |
| Max reconnect delay | 60 seconds |
| Max reconnect attempts | 10 |

After reconnecting, subscriptions are automatically restored and a fresh
orderbook snapshot is sent.

## Error Handling



**Rust**

```rust
use px_core::WebSocketError;

match ws.connect().await {
    Ok(()) => println!("Connected"),
    Err(WebSocketError::Connection(msg)) => {
        // Retryable — auto-reconnect will handle this
        eprintln!("Connection failed: {msg}");
    }
    Err(WebSocketError::Subscription(msg)) => {
        // Bad market ID or unauthorized
        eprintln!("Subscription failed: {msg}");
    }
    Err(WebSocketError::Protocol(msg)) => {
        eprintln!("Protocol error: {msg}");
    }
    Err(WebSocketError::Closed) => {
        eprintln!("Connection was closed");
    }
}
```


**Python**

```python
from openpx import Exchange, OpenPxError

exchange = Exchange("kalshi", {"api_key_id": "...", "private_key_pem": "..."})
ws = exchange.websocket()

try:
    ws.connect()
except OpenPxError as e:
    print(f"WebSocket error: {e}")
```


**TypeScript**

```typescript
const ws = exchange.websocket();

try {
  await ws.connect();
} catch (e) {
  console.error(`WebSocket error: ${e.message}`);
}
```



## Performance Notes

The WebSocket implementation is optimized for low-latency trading:

- **Lock-free state reads** — `WebSocketState` uses atomic operations, no mutex contention
- **Stack-allocated deltas** — up to 4 price level changes per update use `SmallVec` (no heap allocation)
- **Broadcast channels** — 16K-slot capacity prevents slow consumers from blocking producers
- **Cached orderbooks** — full book state is maintained per-market so reconnects only need a snapshot diff


## Overview

The Sports WebSocket streams real-time scores, game state, and status for all active sports events on Polymarket. It requires **no authentication** and **no subscription messages** — connect and receive.

**Endpoint:** `wss://sports-api.polymarket.com/ws`

## Connection



**Rust**

```rust
use px_sports::SportsWebSocket;
use futures::StreamExt;

let mut ws = SportsWebSocket::new();
ws.connect().await?;

let mut stream = ws.stream();
while let Some(result) = stream.next().await {
    match result {
        Ok(sport) => println!("{} vs {} — {}", sport.home_team, sport.away_team, sport.status),
        Err(e) => eprintln!("error: {e}"),
    }
}
```


**Python**

```python
from openpx import SportsWebSocket

ws = SportsWebSocket()
ws.connect()

for score in ws.stream():
    print(f"{score['away_team']} @ {score['home_team']}: {score.get('score')}")

ws.disconnect()
```


**TypeScript**

```typescript

const ws = new SportsWebSocket();
await ws.connect();

await ws.onScoreUpdate((err, score) => {
  if (err) { console.error(err); return; }
  console.log(`${score.away_team} @ ${score.home_team}: ${score.score}`);
});
```


**CLI**

```bash
# Stream all sports
openpx sports

# Filter to NFL only
openpx sports --league nfl

# Only live games
openpx sports --live-only

# Combine filters
openpx sports --league nba --live-only
```



## Message Format

Each message is a `SportResult` with the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `game_id` | `u64` | Unique game identifier |
| `league_abbreviation` | `String` | League code (e.g. `nfl`, `nba`, `nhl`, `mlb`) |
| `slug` | `String` | URL-friendly game identifier |
| `home_team` | `String` | Home team name |
| `away_team` | `String` | Away team name |
| `status` | `String` | Game status (varies by sport) |
| `score` | `String?` | Current score (format varies by sport) |
| `period` | `String?` | Current period/quarter/half |
| `elapsed` | `String?` | Elapsed time in current period |
| `live` | `bool` | Whether the game is currently in progress |
| `ended` | `bool` | Whether the game has finished |
| `turn` | `String?` | Current turn (used in esports) |
| `finished_timestamp` | `DateTime?` | When the game ended (only present when `ended: true`) |

## Game Status Values by Sport

| Sport | Possible Status Values |
|-------|----------------------|
| NFL / CFB | `Scheduled`, `In Progress`, `Halftime`, `End of Period`, `Final`, `Final/OT`, `Delayed`, `Postponed` |
| NBA / CBB | `Scheduled`, `In Progress`, `Halftime`, `End of Period`, `Final`, `Final/OT` |
| NHL | `Scheduled`, `In Progress`, `End of Period`, `Final`, `Final/OT`, `Final/SO` |
| MLB | `Scheduled`, `In Progress`, `Delayed`, `Final`, `Postponed` |
| Soccer | `Scheduled`, `FirstHalf`, `Halftime`, `SecondHalf`, `ExtraTime`, `PenaltyShootout`, `Final` |
| Esports | `Scheduled`, `In Progress`, `Final` |
| Tennis | `Scheduled`, `In Progress`, `Final` |

## Score Formats

Score formats differ by sport:
- **NFL/NBA/NHL/CFB/CBB**: `"3-16"` (away-home)
- **MLB**: `"3-16"` (away-home)
- **Soccer**: `"1-0"` (home-away)
- **Esports**: `"000-000|2-0|Bo3"` (round scores, map score, series format)

## Filtering by League

Client-side filtering on the stream:

```rust
use futures::StreamExt;

let mut stream = ws.stream();
while let Some(Ok(result)) = stream.next().await {
    if result.league_abbreviation == "nfl" && result.live {
        println!("{}", serde_json::to_string(&result)?);
    }
}
```

## Auto-Reconnect

The `SportsWebSocket` automatically reconnects on disconnection using exponential backoff:

- **Base delay**: 3 seconds
- **Max delay**: 60 seconds
- **Max attempts**: 10
- **Backoff factor**: 1.5x per attempt

## Keepalive

The server sends `"ping"` as a TEXT message every ~5 seconds. The client must reply `"pong"` as TEXT within 10 seconds. This is handled automatically — no action needed.

## Error Handling

```rust
use px_core::error::WebSocketError;

let mut stream = ws.stream();
while let Some(result) = stream.next().await {
    match result {
        Ok(sport) => { /* handle */ },
        Err(WebSocketError::Connection(msg)) => eprintln!("connection lost: {msg}"),
        Err(WebSocketError::Closed) => break,
        Err(e) => eprintln!("error: {e}"),
    }
}
```


## Overview

The Crypto Price WebSocket streams real-time cryptocurrency prices from two sources: **Binance** and **Chainlink**. Unlike the Sports WebSocket, it requires **explicit subscribe/unsubscribe messages** and a **client-initiated PING every 5 seconds**.

**Endpoint:** `wss://ws-live-data.polymarket.com`

## Connection



**Rust**

```rust
use px_crypto::CryptoPriceWebSocket;
use px_core::models::CryptoPriceSource;
use futures::StreamExt;

let mut ws = CryptoPriceWebSocket::new();
ws.connect().await?;

// Subscribe to specific Binance symbols
ws.subscribe(CryptoPriceSource::Binance, &["btcusdt".into(), "ethusdt".into()]).await?;

let mut stream = ws.stream();
while let Some(result) = stream.next().await {
    match result {
        Ok(price) => println!("{} = {} ({:?})", price.symbol, price.value, price.source),
        Err(e) => eprintln!("error: {e}"),
    }
}
```


**Python**

```python
from openpx import CryptoPriceWebSocket

ws = CryptoPriceWebSocket()
ws.connect()
ws.subscribe("binance", ["btcusdt", "ethusdt"])

for price in ws.stream():
    print(f"{price['symbol']} = {price['value']} ({price['source']})")

ws.disconnect()
```


**TypeScript**

```typescript

const ws = new CryptoPriceWebSocket();
await ws.connect();
await ws.subscribe("binance", ["btcusdt", "ethusdt"]);

await ws.onPriceUpdate((err, price) => {
  if (err) { console.error(err); return; }
  console.log(`${price.symbol} = ${price.value} (${price.source})`);
});
```


**CLI**

```bash
# Stream all Binance crypto prices
openpx crypto

# Stream specific symbols
openpx crypto --symbols btcusdt,ethusdt

# Stream Chainlink prices
openpx crypto --source chainlink --symbols eth/usd,btc/usd
```



## Subscribe / Unsubscribe

Subscriptions require a source (Binance or Chainlink) and an optional list of symbols. Omitting symbols subscribes to all available prices for that source.

```rust
// Subscribe to all Binance prices
ws.subscribe(CryptoPriceSource::Binance, &[]).await?;

// Subscribe to specific Chainlink symbols
ws.subscribe(CryptoPriceSource::Chainlink, &["eth/usd".into()]).await?;

// Unsubscribe
ws.unsubscribe(CryptoPriceSource::Binance, &["btcusdt".into()]).await?;
```

### Binance Subscribe Format

```json
{
  "action": "subscribe",
  "subscriptions": [{
    "topic": "crypto_prices",
    "type": "update",
    "filters": "btcusdt,ethusdt"
  }]
}
```

### Chainlink Subscribe Format

```json
{
  "action": "subscribe",
  "subscriptions": [{
    "topic": "crypto_prices_chainlink",
    "type": "*",
    "filters": "{\"symbol\":\"eth/usd\"}"
  }]
}
```

## Message Format

Each message is a `CryptoPrice` with the following fields:

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Trading pair or price feed identifier |
| `timestamp` | `u64` | Unix timestamp of the price update |
| `value` | `f64` | Current price value |
| `source` | `CryptoPriceSource` | `binance` or `chainlink` |

### Raw Envelope

Messages arrive as JSON envelopes:

```json
{
  "topic": "crypto_prices",
  "type": "update",
  "timestamp": 1700000000,
  "payload": {
    "symbol": "btcusdt",
    "timestamp": 1700000000,
    "value": 43250.50
  }
}
```

## Supported Symbols

| Source | Symbol Format | Examples |
|--------|--------------|----------|
| Binance | Lowercase pair | `btcusdt`, `ethusdt`, `solusdt` |
| Chainlink | Slash-separated | `eth/usd`, `btc/usd`, `sol/usd` |

## Auto-Reconnect

The `CryptoPriceWebSocket` automatically reconnects on disconnection using exponential backoff. All stored subscriptions are replayed after reconnect.

- **Base delay**: 3 seconds
- **Max delay**: 60 seconds
- **Max attempts**: 10
- **Backoff factor**: 1.5x per attempt

## Keepalive

The client sends `"PING"` as a TEXT message every 5 seconds. The server responds with `"PONG"`. This is handled automatically — no action needed.

## Error Handling

```rust
use px_core::error::WebSocketError;

let mut stream = ws.stream();
while let Some(result) = stream.next().await {
    match result {
        Ok(price) => { /* handle */ },
        Err(WebSocketError::Connection(msg)) => eprintln!("connection lost: {msg}"),
        Err(WebSocketError::Closed) => break,
        Err(e) => eprintln!("error: {e}"),
    }
}
```

## Overview

The OpenPX CLI (`openpx`) lets you interact with any supported exchange from the terminal. All output is JSON, making it easy to pipe into `jq` or other tools.

## Installation

```bash
# Install globally
cargo install --path engine/cli

# Or build from workspace
cargo build --release -p px-cli
```

## Authentication

The CLI reads exchange credentials from environment variables. Create a `.env` file in your project root — the CLI loads it automatically via `dotenvy`.

### Kalshi

| Variable | Description |
|----------|-------------|
| `KALSHI_API_KEY_ID` | API key ID |
| `KALSHI_PRIVATE_KEY_PEM` | RSA private key (PEM string) |
| `KALSHI_PRIVATE_KEY_PATH` | Path to PEM file (alternative to inline PEM) |

### Polymarket

| Variable | Description |
|----------|-------------|
| `POLYMARKET_PRIVATE_KEY` | Ethereum private key (`0x...`) |
| `POLYMARKET_FUNDER` | Funder address (`0x...`) |
| `POLYMARKET_API_KEY` | CLOB API key |
| `POLYMARKET_API_SECRET` | CLOB API secret |
| `POLYMARKET_API_PASSPHRASE` | CLOB API passphrase |

### Opinion

| Variable | Description |
|----------|-------------|
| `OPINION_API_KEY` | API key |
| `OPINION_PRIVATE_KEY` | Private key (`0x...`) |
| `OPINION_MULTI_SIG_ADDR` | Multi-sig address (`0x...`) |

## Commands

All exchange-specific commands are namespaced under the exchange name:

```bash
openpx <exchange> <command> [options]
```

Where `<exchange>` is one of `kalshi`, `polymarket`, or `opinion`.

### Market Data

#### fetch-markets

Fetch a page of markets.

```bash
openpx kalshi fetch-markets
openpx kalshi fetch-markets --status active --limit 10
openpx polymarket fetch-markets --cursor "next_page_token"

# Filter by series
openpx kalshi fetch-markets --series-id KXBTC
openpx polymarket fetch-markets --series-id 10345

# All markets within a specific event
openpx kalshi fetch-markets --event-id KXBTC-25MAR14
openpx polymarket fetch-markets --event-id 903
openpx opinion fetch-markets --event-id btc-price-daily
```

| Flag | Description |
|------|-------------|
| `--status` | Filter by status: `active`, `closed`, `resolved`, `all` |
| `--cursor` | Pagination cursor from a previous response |
| `--limit` | Max markets to return |
| `--series-id` | Filter by series (Kalshi series ticker or Polymarket series ID) |
| `--event-id` | Fetch all markets within an event (Kalshi event ticker, Polymarket event ID or slug, or Opinion market slug) |

#### fetch-market

Fetch a single market by ID.

```bash
openpx kalshi fetch-market KXBTC-25MAR14
openpx polymarket fetch-market "0x1234..."
```

#### fetch-orderbook

Fetch the L2 orderbook for a market.

```bash
openpx kalshi fetch-orderbook KXBTC-25MAR14
openpx polymarket fetch-orderbook "0x1234..." --token-id "TOKEN_ID"
```

| Flag | Description |
|------|-------------|
| `--outcome` | Filter by outcome |
| `--token-id` | Token ID (Polymarket) |

#### fetch-price-history

Fetch OHLCV price history.

```bash
openpx kalshi fetch-price-history KXBTC-25MAR14 1h
openpx polymarket fetch-price-history "0x1234..." 1d --start-ts 1700000000
```

| Argument | Description |
|----------|-------------|
| `interval` | One of: `1m`, `1h`, `6h`, `1d`, `1w`, `max` |

| Flag | Description |
|------|-------------|
| `--outcome` | Filter by outcome |
| `--token-id` | Token ID |
| `--start-ts` | Start timestamp (unix seconds) |
| `--end-ts` | End timestamp (unix seconds) |

#### fetch-trades

Fetch recent trades.

```bash
openpx kalshi fetch-trades KXBTC-25MAR14
openpx polymarket fetch-trades "0x1234..." --limit 50
```

| Flag | Description |
|------|-------------|
| `--outcome` | Filter by outcome |
| `--token-id` | Token ID |
| `--limit` | Max trades to return |
| `--cursor` | Pagination cursor |

#### fetch-orderbook-history

Fetch historical orderbook snapshots.

```bash
openpx kalshi fetch-orderbook-history KXBTC-25MAR14
openpx kalshi fetch-orderbook-history KXBTC-25MAR14 --start-ts 1700000000 --limit 10
```

| Flag | Description |
|------|-------------|
| `--token-id` | Token ID |
| `--start-ts` | Start timestamp (unix seconds) |
| `--end-ts` | End timestamp (unix seconds) |
| `--limit` | Max snapshots to return |
| `--cursor` | Pagination cursor |

### Account Commands

These commands require authentication via environment variables.

#### fetch-balance

```bash
openpx kalshi fetch-balance
openpx polymarket fetch-balance
```

#### fetch-positions

```bash
openpx kalshi fetch-positions
openpx kalshi fetch-positions --market-id KXBTC-25MAR14
```

| Flag | Description |
|------|-------------|
| `--market-id` | Filter by market |

#### fetch-open-orders

```bash
openpx kalshi fetch-open-orders
openpx polymarket fetch-open-orders --market-id "0x1234..."
```

| Flag | Description |
|------|-------------|
| `--market-id` | Filter by market |

#### fetch-order

Fetch a single order by ID.

```bash
openpx kalshi fetch-order ORDER_ID
openpx polymarket fetch-order ORDER_ID --market-id "0x1234..."
```

| Flag | Description |
|------|-------------|
| `--market-id` | Market ID (required for some exchanges) |

#### fetch-fills

Fetch fill history.

```bash
openpx kalshi fetch-fills
openpx kalshi fetch-fills --market-id KXBTC-25MAR14 --limit 20
```

| Flag | Description |
|------|-------------|
| `--market-id` | Filter by market |
| `--limit` | Max fills to return |

### WebSocket Commands

#### ws-orderbook

Stream live orderbook updates.

```bash
openpx kalshi ws-orderbook KXBTC-25MAR14
openpx polymarket ws-orderbook "0x1234..."
```

#### ws-activity

Stream live trade and fill events.

```bash
openpx kalshi ws-activity KXBTC-25MAR14
openpx polymarket ws-activity "0x1234..."
```

### Sports Streaming

Stream real-time sports scores. No authentication required.

```bash
# Stream all sports
openpx sports

# Filter by league
openpx sports --league nfl

# Only live games
openpx sports --live-only

# Combine filters
openpx sports --league nba --live-only
```

| Flag | Description |
|------|-------------|
| `--league` | Filter by league abbreviation (e.g. `nfl`, `nba`, `nhl`, `mlb`) |
| `--live-only` | Only show games currently in progress |

### Crypto Price Streaming

Stream real-time crypto prices. No authentication required.

```bash
# Stream all Binance prices
openpx crypto

# Stream specific symbols
openpx crypto --symbols btcusdt,ethusdt

# Stream Chainlink prices
openpx crypto --source chainlink --symbols eth/usd,btc/usd
```

| Flag | Description |
|------|-------------|
| `--source` | Price source: `binance` (default) or `chainlink` |
| `--symbols` | Comma-separated symbols to subscribe to |

## Examples

### Pipe to jq

```bash
# Pretty-print first market title
openpx kalshi fetch-markets --limit 1 | jq '.markets[0].title'

# Get all active market IDs
openpx polymarket fetch-markets --status active | jq '.markets[].id'
```

### Monitor orderbook spread

```bash
openpx kalshi ws-orderbook KXBTC-25MAR14 | jq 'select(.type == "Snapshot") | {
  best_bid: .Snapshot.bids[0].price,
  best_ask: .Snapshot.asks[0].price
}'
```

### Watch live NBA scores

```bash
openpx sports --league nba --live-only | jq '{
  game: (.away_team + " @ " + .home_team),
  score: .score,
  period: .period
}'
```


OpenPX ships native SDKs for Rust, Python, and TypeScript. All three share
the same Rust engine — the Python and TypeScript SDKs are thin wrappers
compiled via PyO3 and NAPI-RS respectively.

## Installation



**Rust**

Add OpenPX crates to your `Cargo.toml`:

```toml
[dependencies]
px-core = "0.1"

# Individual exchanges
px-exchange-kalshi = "0.1"
px-exchange-polymarket = "0.1"
px-exchange-opinion = "0.1"

# Or use the unified SDK facade
px-sdk = "0.1"
```

### Crate Structure

| Crate | Description |
|-------|-------------|
| `px-core` | Core types, Exchange trait, error handling, timing |
| `px-sdk` | Unified facade — enum dispatch over all exchanges |
| `px-exchange-kalshi` | Kalshi exchange implementation |
| `px-exchange-polymarket` | Polymarket exchange implementation |
| `px-exchange-opinion` | Opinion exchange implementation |


**Python**

```bash
pip install openpx
```

Requires Python >= 3.9. The package includes a native Rust extension compiled
with PyO3 — no Rust toolchain needed on the user's machine.

### How It Works

```
User calls exchange.fetch_markets()
         |
exchange.py  (pure Python wrapper)
         |  calls _native.NativeExchange.fetch_markets()
         |
lib.rs  (PyO3 → Rust, returns Python dict via pythonize)
         |
exchange.py  receives list[dict]
         |  wraps: [Market(**d) for d in raw_dicts]
         |
User receives list[Market]  (Pydantic models with autocomplete)
```


**TypeScript**

```bash
npm install @openpx/sdk
```

Requires Node.js >= 18. The package includes a native Rust addon compiled
with NAPI-RS.



## Quick Start



**Rust**

```rust
use px_sdk::ExchangeInner;
use px_core::FetchMarketsParams;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exchange = ExchangeInner::new("polymarket", json!({}))?;

    let (markets, _cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await?;
    for m in &markets {
        println!("[{}] {} — {:?}", m.id, m.title, m.outcome_prices);
    }

    Ok(())
}
```


**Python**

```python
from openpx import Exchange

# Unauthenticated (market data only)
exchange = Exchange("polymarket")
markets = exchange.fetch_markets()
for m in markets:
    print(f"[{m.id}] {m.title}")

# Authenticated (trading)
exchange = Exchange("kalshi", {
    "api_key_id": "...",
    "private_key_pem": "...",
})
positions = exchange.fetch_positions()
balance = exchange.fetch_balance()
```


**TypeScript**

```typescript

// Unauthenticated (market data only)
const exchange = new Exchange("polymarket", {});
const markets = await exchange.fetchMarkets();
for (const m of markets) {
  console.log(`[${m.id}] ${m.title}`);
}

// Authenticated (trading)
const authed = new Exchange("kalshi", {
  api_key_id: "...",
  private_key_pem: "...",
});
const positions = await authed.fetchPositions();
const balance = await authed.fetchBalance();
```



## WebSocket Streaming

All three SDKs support real-time WebSocket streaming. See the full
[WebSocket guide](/guides/websocket/) for connection lifecycle details.



**Rust**

```rust
use px_core::OrderBookWebSocket;
use futures::StreamExt;

let mut ws = exchange.websocket().unwrap();

ws.connect().await?;
ws.subscribe("KXBTC-25MAR14").await?;

let mut stream = ws.orderbook_stream("KXBTC-25MAR14").await?;
while let Some(update) = stream.next().await {
    println!("{:?}", update?);
}

ws.disconnect().await?;
```


**Python**

```python
from openpx import Exchange

exchange = Exchange("kalshi", {
    "api_key_id": "...",
    "private_key_pem": "...",
})
ws = exchange.websocket()
ws.connect()
ws.subscribe("KXBTC-25MAR14")

for update in ws.orderbook_stream("KXBTC-25MAR14"):
    print(update)

ws.disconnect()
```


**TypeScript**

```typescript

const exchange = new Exchange("kalshi", {
  api_key_id: "...",
  private_key_pem: "...",
});
const ws = exchange.websocket();
await ws.connect();
await ws.subscribe("KXBTC-25MAR14");

await ws.onOrderbookUpdate("KXBTC-25MAR14", (err, update) => {
  if (err) { console.error(err); return; }
  console.log(update);
});
```



OpenPX also provides standalone WebSocket streams for
[sports scores](/guides/sports-websocket/) and
[crypto prices](/guides/crypto-websocket/) with Python and TypeScript bindings.

## Error Handling



**Rust**

```rust
use px_core::{OpenPxError, ExchangeError};

match exchange.fetch_balance().await {
    Ok(balance) => println!("Balance: {:?}", balance),
    Err(OpenPxError::Exchange(ExchangeError::Authentication(msg))) => {
        eprintln!("Auth failed: {msg}");
    }
    Err(e) => eprintln!("Error: {e}"),
}
```


**Python**

```python
from openpx import Exchange, OpenPxError, AuthenticationError

try:
    exchange = Exchange("kalshi", {"api_key_id": "bad"})
    exchange.fetch_balance()
except AuthenticationError as e:
    print(f"Auth failed: {e}")
except OpenPxError as e:
    print(f"Error: {e}")
```


**TypeScript**

```typescript

try {
  const exchange = new Exchange("kalshi", { api_key_id: "bad" });
  await exchange.fetchBalance();
} catch (e) {
  console.error(`Error: ${e.message}`);
}
```




All types auto-generated from Rust source via `schema/openpx.schema.json`.
Run `just docs` to regenerate.

**Exchange key:** K = Kalshi, P = Polymarket, O = Opinion

## Market Data

### Market

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `accepting_orders` | `boolean` | Yes | K P O | Whether the market is currently accepting new orders |
| `best_ask` | `number \| null` | No | K P | Lowest ask price on the Yes outcome orderbook, normalized 0-1 |
| `best_bid` | `number \| null` | No | K P | Highest bid price on the Yes outcome orderbook, normalized 0-1 |
| `can_close_early` | `boolean \| null` | No | K | Whether the exchange can close the market before the scheduled close time |
| `chain_id` | `string \| null` | No | O | Blockchain chain ID for on-chain settlement |
| `close_time` | `string \| null` | No | K P O | Scheduled market close time (ISO 8601). No new orders accepted after this time |
| `condition_id` | `string \| null` | No | P O | CTF (Conditional Token Framework) condition ID for on-chain settlement |
| `created_at` | `string \| null` | No | K P O | When the market was first created on the exchange (ISO 8601) |
| `denomination_token` | `string \| null` | No | P O | Address of the settlement token (e.g., USDC contract address) |
| `description` | `string` | Yes | K P O | Human-readable market description. Kalshi uses subtitle or rules; Polymarket and Opinion use description |
| `event_id` | `string \| null` | No | K P O | Canonical OpenPX event ID for cross-exchange grouping (derived from `group_id` via hash) |
| `exchange` | `string` | Yes | K P O | Exchange identifier: `"kalshi"`, `"polymarket"`, or `"opinion"` |
| `group_id` | `string \| null` | No | K P O | Native event/group ID from the exchange (e.g., Kalshi `event_ticker`, Polymarket event ID) |
| `icon_url` | `string \| null` | No | P | URL to the market's icon image |
| `id` | `string` | Yes | K P O | Native market ID on the exchange (e.g., Kalshi ticker, Polymarket market ID) |
| `image_url` | `string \| null` | No | P | URL to the market's banner image |
| `last_trade_price` | `number \| null` | No | K P | Most recent trade price for the Yes outcome, normalized 0-1 |
| `liquidity` | `number \| null` | No | P O | Total liquidity available in the market orderbook (USD) |
| `maker_fee_bps` | `number \| null` | No | P | Base maker fee rate in basis points (1 bps = 0.01%) |
| `market_type` | `MarketType` | Yes | K P O | `binary` (Yes/No), `categorical` (multiple outcomes), or `scalar` (numeric range) |
| `min_order_size` | `number \| null` | No | P | Minimum order size in contracts |
| `neg_risk` | `boolean \| null` | No | P | Whether this is a neg-risk market (Polymarket multi-outcome markets using the NegRisk adapter) |
| `neg_risk_market_id` | `string \| null` | No | P | Neg-risk market ID for multi-outcome Polymarket markets |
| `notional_value` | `number \| null` | No | K | Dollar value per contract at expiry |
| `open_interest` | `number \| null` | No | K O | Total number of outstanding contracts |
| `open_time` | `string \| null` | No | K P | When the market opened for trading (ISO 8601) |
| `openpx_id` | `string` | Yes | K P O | Primary key in `{exchange}:{native_id}` format (e.g., `kalshi:TICKER-123`) |
| `outcome_prices` | `Record<string, number>` | No | K P | Map of outcome label to current price, normalized 0-1 (e.g., `{"Yes": 0.65, "No": 0.35}`). Omitted when empty |
| `outcome_tokens` | `OutcomeToken[]` | Yes | P O | Maps each outcome to its on-chain token ID for orderbook subscriptions and trading. Empty array for Kalshi |
| `outcomes` | `string[]` | Yes | K P O | Ordered list of outcome labels (e.g., `["Yes", "No"]` for binary markets) |
| `previous_price` | `number \| null` | No | K | Previous settlement period's Yes price, normalized 0-1 |
| `price_change_1d` | `number \| null` | No | P | 24-hour price change as a decimal (e.g., 0.05 = +5%) |
| `price_change_1h` | `number \| null` | No | P | 1-hour price change as a decimal (e.g., 0.05 = +5%) |
| `price_change_1mo` | `number \| null` | No | P | 30-day price change as a decimal |
| `price_change_1wk` | `number \| null` | No | P | 7-day price change as a decimal |
| `price_level_structure` | `string \| null` | No | K | Sub-penny pricing tier structure identifier |
| `question` | `string \| null` | No | P O | Full market question text, may differ from `title` |
| `question_id` | `string \| null` | No | P O | Exchange-specific question identifier |
| `result` | `string \| null` | No | K | Resolution result after market settles (e.g., `"yes"`, `"no"`) |
| `rules` | `string \| null` | No | K O | Resolution criteria describing how the market outcome will be determined |
| `settlement_time` | `string \| null` | No | K O | When the market was or will be settled/resolved (ISO 8601) |
| `settlement_value` | `number \| null` | No | K | Final settlement price, normalized 0-1 |
| `slug` | `string \| null` | No | P | URL-friendly market identifier |
| `spread` | `number \| null` | No | K P | Bid-ask spread on the Yes outcome (`best_ask - best_bid`) |
| `status` | `MarketStatus` | Yes | K P O | Normalized lifecycle state: `active` (trading), `closed` (no trading), or `resolved` (settled) |
| `taker_fee_bps` | `number \| null` | No | P | Base taker fee rate in basis points (1 bps = 0.01%) |
| `tick_size` | `number \| null` | No | K P O | Minimum price increment as a decimal (e.g., 0.01 = 1 cent). Kalshi/Polymarket default 0.01, Opinion 0.001 |
| `title` | `string` | Yes | K P O | Market headline text |
| `token_id_no` | `string \| null` | No | P O | Token ID for the No outcome, used in CLOB order placement |
| `token_id_yes` | `string \| null` | No | P O | Token ID for the Yes outcome, used in CLOB order placement |
| `volume` | `number` | Yes | K P O | All-time total trading volume in USD |
| `volume_1mo` | `number \| null` | No | P | Rolling 30-day trading volume in USD |
| `volume_1wk` | `number \| null` | No | P O | Rolling 7-day trading volume in USD |
| `volume_24h` | `number \| null` | No | P O | Rolling 24-hour trading volume in USD |



**Rust**


```rust
pub struct Market {
    pub openpx_id: String,
    pub exchange: String,
    pub id: String,
    pub group_id: Option<String>,
    pub event_id: Option<String>,
    pub title: String,
    pub question: Option<String>,
    pub description: String,
    pub slug: Option<String>,
    pub rules: Option<String>,
    pub status: MarketStatus,
    pub market_type: MarketType,
    pub accepting_orders: bool,
    pub outcomes: Vec<String>,
    pub outcome_tokens: Vec<OutcomeToken>,
    pub outcome_prices: HashMap<String, f64>,
    pub token_id_yes: Option<String>,
    pub token_id_no: Option<String>,
    pub condition_id: Option<String>,
    pub question_id: Option<String>,
    pub volume: f64,
    pub volume_24h: Option<f64>,
    pub volume_1wk: Option<f64>,
    pub volume_1mo: Option<f64>,
    pub liquidity: Option<f64>,
    pub open_interest: Option<f64>,
    pub last_trade_price: Option<f64>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub spread: Option<f64>,
    pub price_change_1d: Option<f64>,
    pub price_change_1h: Option<f64>,
    pub price_change_1wk: Option<f64>,
    pub price_change_1mo: Option<f64>,
    pub tick_size: Option<f64>,
    pub min_order_size: Option<f64>,
    pub close_time: Option<DateTime<Utc>>,
    pub open_time: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub settlement_time: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub icon_url: Option<String>,
    pub neg_risk: Option<bool>,
    pub neg_risk_market_id: Option<String>,
    pub maker_fee_bps: Option<f64>,
    pub taker_fee_bps: Option<f64>,
    pub denomination_token: Option<String>,
    pub chain_id: Option<String>,
    pub notional_value: Option<f64>,
    pub price_level_structure: Option<String>,
    pub settlement_value: Option<f64>,
    pub previous_price: Option<f64>,
    pub can_close_early: Option<bool>,
    pub result: Option<String>,
}
```



**Python**


```python
class Market(BaseModel):
    openpx_id: str
    exchange: str
    id: str
    group_id: Optional[str]
    event_id: Optional[str]
    title: str
    question: Optional[str]
    description: str
    slug: Optional[str]
    rules: Optional[str]
    status: MarketStatus
    market_type: MarketType
    accepting_orders: bool
    outcomes: list[str]
    outcome_tokens: list[OutcomeToken]
    outcome_prices: Optional[dict[str, float]] = None
    token_id_yes: Optional[str]
    token_id_no: Optional[str]
    condition_id: Optional[str]
    question_id: Optional[str]
    volume: float
    volume_24h: Optional[float]
    volume_1wk: Optional[float]
    volume_1mo: Optional[float]
    liquidity: Optional[float]
    open_interest: Optional[float]
    last_trade_price: Optional[float]
    best_bid: Optional[float]
    best_ask: Optional[float]
    spread: Optional[float]
    price_change_1d: Optional[float]
    price_change_1h: Optional[float]
    price_change_1wk: Optional[float]
    price_change_1mo: Optional[float]
    tick_size: Optional[float]
    min_order_size: Optional[float]
    close_time: Optional[datetime]
    open_time: Optional[datetime]
    created_at: Optional[datetime]
    settlement_time: Optional[datetime]
    image_url: Optional[str]
    icon_url: Optional[str]
    neg_risk: Optional[bool]
    neg_risk_market_id: Optional[str]
    maker_fee_bps: Optional[float]
    taker_fee_bps: Optional[float]
    denomination_token: Optional[str]
    chain_id: Optional[str]
    notional_value: Optional[float]
    price_level_structure: Optional[str]
    settlement_value: Optional[float]
    previous_price: Optional[float]
    can_close_early: Optional[bool]
    result: Optional[str]
```



**TypeScript**


```typescript
interface Market {
  openpx_id: string;
  exchange: string;
  id: string;
  group_id?: string | null;
  event_id?: string | null;
  title: string;
  question?: string | null;
  description: string;
  slug?: string | null;
  rules?: string | null;
  status: MarketStatus;
  market_type: MarketType;
  accepting_orders: boolean;
  outcomes: string[];
  outcome_tokens: OutcomeToken[];
  outcome_prices?: Record<string, number>;
  token_id_yes?: string | null;
  token_id_no?: string | null;
  condition_id?: string | null;
  question_id?: string | null;
  volume: number;
  volume_24h?: number | null;
  volume_1wk?: number | null;
  volume_1mo?: number | null;
  liquidity?: number | null;
  open_interest?: number | null;
  last_trade_price?: number | null;
  best_bid?: number | null;
  best_ask?: number | null;
  spread?: number | null;
  price_change_1d?: number | null;
  price_change_1h?: number | null;
  price_change_1wk?: number | null;
  price_change_1mo?: number | null;
  tick_size?: number | null;
  min_order_size?: number | null;
  close_time?: string | null;
  open_time?: string | null;
  created_at?: string | null;
  settlement_time?: string | null;
  image_url?: string | null;
  icon_url?: string | null;
  neg_risk?: boolean | null;
  neg_risk_market_id?: string | null;
  maker_fee_bps?: number | null;
  taker_fee_bps?: number | null;
  denomination_token?: string | null;
  chain_id?: string | null;
  notional_value?: number | null;
  price_level_structure?: string | null;
  settlement_value?: number | null;
  previous_price?: number | null;
  can_close_early?: boolean | null;
  result?: string | null;
}
```





### MarketType

Enum with variants: `binary`, `categorical`, `scalar`



**Rust**


```rust
pub enum MarketType {
    Binary,
    Categorical,
    Scalar,
}
```



**Python**


```python
class MarketType(str, Enum):
    BINARY = "binary"
    CATEGORICAL = "categorical"
    SCALAR = "scalar"
```



**TypeScript**


```typescript
type MarketType = "binary" | "categorical" | "scalar";
```





## Orders & Trading

### Fill

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `created_at` | `string` | Yes | K P | Fill execution timestamp (ISO 8601) |
| `fee` | `number` | Yes | K | Fee charged for this fill in USD. Polymarket: always `0.0` via REST |
| `fill_id` | `string` | Yes | K P | Unique fill identifier |
| `is_taker` | `boolean` | Yes | K | Whether this fill was on the taker side. Polymarket: always `false` via REST |
| `market_id` | `string` | Yes | K P | Native market ID the fill occurred on |
| `order_id` | `string` | Yes | K | Parent order ID that generated this fill. Polymarket: empty string via REST |
| `outcome` | `string` | Yes | K P | Outcome that was traded (e.g., `"Yes"`, `"No"`) |
| `price` | `number` | Yes | K P | Execution price, normalized 0-1 |
| `side` | `OrderSide` | Yes | K P | Whether the order was a `buy` or `sell` |
| `size` | `number` | Yes | K P | Number of contracts filled |



**Rust**


```rust
pub struct Fill {
    pub fill_id: String,
    pub order_id: String,
    pub market_id: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub is_taker: bool,
    pub fee: f64,
    pub created_at: DateTime<Utc>,
}
```



**Python**


```python
class Fill(BaseModel):
    fill_id: str
    order_id: str
    market_id: str
    outcome: str
    side: OrderSide
    price: float
    size: float
    is_taker: bool
    fee: float
    created_at: datetime
```



**TypeScript**


```typescript
interface Fill {
  fill_id: string;
  order_id: string;
  market_id: string;
  outcome: string;
  side: OrderSide;
  price: number;
  size: number;
  is_taker: boolean;
  fee: number;
  created_at: string;
}
```





### Order

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `created_at` | `string` | Yes | K P O | When the order was placed (ISO 8601) |
| `filled` | `number` | Yes | K P O | Number of contracts filled so far |
| `id` | `string` | Yes | K P O | Exchange-assigned order identifier |
| `market_id` | `string` | Yes | K P O | Native market ID this order belongs to |
| `outcome` | `string` | Yes | K P O | Outcome being traded (e.g., `"Yes"`, `"No"`) |
| `price` | `number` | Yes | K P O | Limit price, normalized 0-1 |
| `side` | `OrderSide` | Yes | K P O | Whether the order is a `buy` or `sell` |
| `size` | `number` | Yes | K P O | Total order size in contracts |
| `status` | `OrderStatus` | Yes | K P O | Current lifecycle state of the order |
| `updated_at` | `string \| null` | No | K | When the order was last modified (ISO 8601) |



**Rust**


```rust
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub filled: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
```



**Python**


```python
class Order(BaseModel):
    id: str
    market_id: str
    outcome: str
    side: OrderSide
    price: float
    size: float
    filled: float
    status: OrderStatus
    created_at: datetime
    updated_at: Optional[datetime]
```



**TypeScript**


```typescript
interface Order {
  id: string;
  market_id: string;
  outcome: string;
  side: OrderSide;
  price: number;
  size: number;
  filled: number;
  status: OrderStatus;
  created_at: string;
  updated_at?: string | null;
}
```





### OrderStatus

Enum with variants: `pending`, `open`, `filled`, `partially_filled`, `cancelled`, `rejected`



**Rust**


```rust
pub enum OrderStatus {
    Pending,
    Open,
    Filled,
    PartiallyFilled,
    Cancelled,
    Rejected,
}
```



**Python**


```python
class OrderStatus(str, Enum):
    PENDING = "pending"
    OPEN = "open"
    FILLED = "filled"
    PARTIALLY_FILLED = "partially_filled"
    CANCELLED = "cancelled"
    REJECTED = "rejected"
```



**TypeScript**


```typescript
type OrderStatus = "pending" | "open" | "filled" | "partially_filled" | "cancelled" | "rejected";
```





## Account & Positions

### Position

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `average_price` | `number` | Yes | K P O | Volume-weighted average entry price, normalized 0-1 |
| `current_price` | `number` | Yes | K P O | Current market price for this outcome, normalized 0-1 |
| `market_id` | `string` | Yes | K P O | Native market ID this position belongs to |
| `outcome` | `string` | Yes | K P O | Outcome held (e.g., `"Yes"`, `"No"`) |
| `size` | `number` | Yes | K P O | Number of contracts held |



**Rust**


```rust
pub struct Position {
    pub market_id: String,
    pub outcome: String,
    pub size: f64,
    pub average_price: f64,
    pub current_price: f64,
}
```



**Python**


```python
class Position(BaseModel):
    market_id: str
    outcome: str
    size: float
    average_price: float
    current_price: float
```



**TypeScript**


```typescript
interface Position {
  market_id: string;
  outcome: string;
  size: number;
  average_price: number;
  current_price: number;
}
```





### OrderbookSnapshot

A point-in-time L2 orderbook snapshot, used for historical orderbook data.

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `asks` | `PriceLevel[]` | Yes | P | Ask-side price levels |
| `bids` | `PriceLevel[]` | Yes | P | Bid-side price levels |
| `hash` | `string \| null` | No | P | Content hash for deduplication |
| `recorded_at` | `string \| null` | No | P | When this snapshot was recorded by the collector (ISO 8601) |
| `timestamp` | `string` | Yes | P | Exchange-reported snapshot timestamp (ISO 8601) |



**Rust**


```rust
pub struct OrderbookSnapshot {
    pub timestamp: DateTime<Utc>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub hash: Option<String>,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}
```



**Python**


```python
class OrderbookSnapshot(BaseModel):
    timestamp: datetime
    recorded_at: Optional[datetime]
    hash: Optional[str]
    bids: list[PriceLevel]
    asks: list[PriceLevel]
```



**TypeScript**


```typescript
interface OrderbookSnapshot {
  timestamp: string;
  recorded_at?: string | null;
  hash?: string | null;
  bids: PriceLevel[];
  asks: PriceLevel[];
}
```





### PriceLevelChange

Incremental orderbook update with absolute replacement semantics: `size > 0` sets the level to this size, `size == 0` removes the level.

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `price` | `number` | Yes | K P | Price level being updated, normalized 0-1 |
| `side` | `PriceLevelSide` | Yes | K P | Whether this change applies to the `bid` or `ask` side |
| `size` | `number` | Yes | K P | New size at this price level (0 = remove level) |



**Rust**


```rust
pub struct PriceLevelChange {
    pub side: PriceLevelSide,
    pub price: FixedPrice, // serializes as f64
    pub size: f64,
}
```



**Python**


```python
class PriceLevelChange(BaseModel):
    side: PriceLevelSide
    price: float
    size: float
```



**TypeScript**


```typescript
interface PriceLevelChange {
  side: PriceLevelSide;
  price: number;
  size: number;
}
```





## Trades & History

### Candlestick

OHLCV candlestick data. All prices are normalized 0-1. Timestamp is the period **start** (not end).

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `close` | `number` | Yes | K P | Closing price for this period, normalized 0-1 |
| `high` | `number` | Yes | K P | Highest price during this period, normalized 0-1 |
| `low` | `number` | Yes | K P | Lowest price during this period, normalized 0-1 |
| `open` | `number` | Yes | K P | Opening price for this period, normalized 0-1 |
| `open_interest` | `number \| null` | No | K | Open interest at this candle's close. Only available from Kalshi |
| `timestamp` | `string` | Yes | K P | Period start timestamp (ISO 8601, UTC) |
| `volume` | `number` | Yes | K | Trade volume in contracts for this period. Polymarket: always `0.0` |



**Rust**


```rust
pub struct Candlestick {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub open_interest: Option<f64>,
}
```



**Python**


```python
class Candlestick(BaseModel):
    timestamp: datetime
    open: float
    high: float
    low: float
    close: float
    volume: float
    open_interest: Optional[float]
```



**TypeScript**


```typescript
interface Candlestick {
  timestamp: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  open_interest?: number | null;
}
```





### PriceHistoryInterval

Enum with variants: `1m`, `1h`, `6h`, `1d`, `1w`, `max`



**Rust**


```rust
pub enum PriceHistoryInterval {
    #[serde(rename = "1m")] OneMinute,
    #[serde(rename = "1h")] OneHour,
    #[serde(rename = "6h")] SixHours,
    #[serde(rename = "1d")] OneDay,
    #[serde(rename = "1w")] OneWeek,
    #[serde(rename = "max")] Max,
}
```



**Python**


```python
class PriceHistoryInterval(str, Enum):
    ONE_MINUTE = "1m"
    ONE_HOUR = "1h"
    SIX_HOURS = "6h"
    ONE_DAY = "1d"
    ONE_WEEK = "1w"
    MAX = "max"
```



**TypeScript**


```typescript
type PriceHistoryInterval = "1m" | "1h" | "6h" | "1d" | "1w" | "max";
```





### ActivityFill

A fill event received via WebSocket, representing a trade execution on one of your orders.

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `asset_id` | `string` | Yes | K P O | Token ID that was traded |
| `fee` | `number \| null` | No | O | Fee charged for this fill (from on-chain confirmation) |
| `fill_id` | `string \| null` | No | K P O | Unique fill identifier. Opinion: `tradeNo` from `trade.record.new` |
| `liquidity_role` | `LiquidityRole \| null` | No | K P O | Whether you were `maker` or `taker` on this fill |
| `market_id` | `string` | Yes | K P O | Native market ID |
| `order_id` | `string \| null` | No | K P O | Parent order ID |
| `outcome` | `string \| null` | No | K P O | Outcome that was filled (e.g., `"Yes"`, `"No"`) |
| `price` | `number` | Yes | K P O | Execution price, normalized 0-1 |
| `side` | `string \| null` | No | K P O | `"buy"` or `"sell"` |
| `size` | `number` | Yes | K P O | Number of contracts filled |
| `source_channel` | `string` | Yes | K P O | Data source identifier (e.g., `"kalshi_user_fill"`, `"trade.record.new"`) |
| `timestamp` | `string \| null` | No | K P O | Fill timestamp (ISO 8601) |
| `tx_hash` | `string \| null` | No | O | On-chain transaction hash (from on-chain confirmation) |



**Rust**


```rust
pub struct ActivityFill {
    pub market_id: String,
    pub asset_id: String,
    pub fill_id: Option<String>,
    pub order_id: Option<String>,
    pub price: f64,
    pub size: f64,
    pub side: Option<String>,
    pub outcome: Option<String>,
    pub tx_hash: Option<String>,
    pub fee: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub source_channel: Cow<'static, str>,
    pub liquidity_role: Option<LiquidityRole>,
}
```



**Python**


```python
class ActivityFill(BaseModel):
    asset_id: str
    fee: Optional[float]
    fill_id: Optional[str]
    liquidity_role: Optional[LiquidityRole]
    market_id: str
    order_id: Optional[str]
    outcome: Optional[str]
    price: float
    side: Optional[str]
    size: float
    source_channel: str
    timestamp: Optional[datetime]
    tx_hash: Optional[str]
```



**TypeScript**


```typescript
interface ActivityFill {
  asset_id: string;
  fee?: number | null;
  fill_id?: string | null;
  liquidity_role?: LiquidityRole | null;
  market_id: string;
  order_id?: string | null;
  outcome?: string | null;
  price: number;
  side?: string | null;
  size: number;
  source_channel: string;
  timestamp?: string | null;
  tx_hash?: string | null;
}
```





## Configuration & Requests

### ExchangeInfo

Describes the capabilities of an exchange instance. Boolean fields indicate which operations are available (some depend on authentication).

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `has_approvals` | `boolean` | Yes | P | Whether on-chain token approval management is available |
| `has_cancel_order` | `boolean` | Yes | K P O | Whether order cancellation is supported |
| `has_create_order` | `boolean` | Yes | K P O | Whether order placement is supported (requires auth) |
| `has_fetch_balance` | `boolean` | Yes | K P O | Whether account balance queries are supported |
| `has_fetch_fills` | `boolean` | Yes | K P | Whether fill/trade history queries are supported |
| `has_fetch_markets` | `boolean` | Yes | K P O | Whether market listing is supported |
| `has_fetch_orderbook` | `boolean` | Yes | K P O | Whether L2 orderbook snapshots are available |
| `has_fetch_orderbook_history` | `boolean` | Yes | — | Whether historical orderbook snapshots are available (not currently supported) |
| `has_fetch_positions` | `boolean` | Yes | K P O | Whether position queries are supported |
| `has_fetch_price_history` | `boolean` | Yes | K P | Whether OHLCV candlestick data is available |
| `has_fetch_trades` | `boolean` | Yes | K P | Whether public trade tape queries are supported |
| `has_fetch_user_activity` | `boolean` | Yes | P O | Whether user activity feed is available |
| `has_refresh_balance` | `boolean` | Yes | P | Whether cached balance refresh is available |
| `has_websocket` | `boolean` | Yes | K P O | Whether real-time WebSocket streaming is supported (may require auth) |
| `id` | `string` | Yes | K P O | Exchange identifier (e.g., `"kalshi"`, `"polymarket"`, `"opinion"`) |
| `name` | `string` | Yes | K P O | Human-readable exchange name (e.g., `"Kalshi"`, `"Polymarket"`, `"Opinion"`) |



**Rust**


```rust
pub struct ExchangeInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub has_fetch_markets: bool,
    pub has_create_order: bool,
    pub has_cancel_order: bool,
    pub has_fetch_positions: bool,
    pub has_fetch_balance: bool,
    pub has_fetch_orderbook: bool,
    pub has_fetch_price_history: bool,
    pub has_fetch_trades: bool,
    pub has_fetch_user_activity: bool,
    pub has_fetch_fills: bool,
    pub has_approvals: bool,
    pub has_refresh_balance: bool,
    pub has_websocket: bool,
    pub has_fetch_orderbook_history: bool,
}
```



**Python**


```python
class ExchangeInfo(BaseModel):
    id: str
    name: str
    has_fetch_markets: bool
    has_create_order: bool
    has_cancel_order: bool
    has_fetch_positions: bool
    has_fetch_balance: bool
    has_fetch_orderbook: bool
    has_fetch_price_history: bool
    has_fetch_trades: bool
    has_fetch_user_activity: bool
    has_fetch_fills: bool
    has_approvals: bool
    has_refresh_balance: bool
    has_websocket: bool
    has_fetch_orderbook_history: bool
```



**TypeScript**


```typescript
interface ExchangeInfo {
  id: string;
  name: string;
  has_fetch_markets: boolean;
  has_create_order: boolean;
  has_cancel_order: boolean;
  has_fetch_positions: boolean;
  has_fetch_balance: boolean;
  has_fetch_orderbook: boolean;
  has_fetch_price_history: boolean;
  has_fetch_trades: boolean;
  has_fetch_user_activity: boolean;
  has_fetch_fills: boolean;
  has_approvals: boolean;
  has_refresh_balance: boolean;
  has_websocket: boolean;
  has_fetch_orderbook_history: boolean;
}
```





### FetchOrdersParams

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `market_id` | `string \| null` | No | K P O | Filter orders to a specific market |



**Rust**


```rust
pub struct FetchOrdersParams {
    pub market_id: Option<String>,
}
```



**Python**


```python
class FetchOrdersParams(BaseModel):
    market_id: Optional[str]
```



**TypeScript**


```typescript
interface FetchOrdersParams {
  market_id?: string | null;
}
```





### OrderbookHistoryRequest

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `cursor` | `string \| null` | No | P | Opaque pagination cursor from a previous response |
| `end_ts` | `number \| null` | No | P | End time filter (Unix seconds) |
| `limit` | `number \| null` | No | P | Maximum number of snapshots per page |
| `market_id` | `string` | Yes | P | Native market ID to fetch history for |
| `start_ts` | `number \| null` | No | P | Start time filter (Unix seconds) |
| `token_id` | `string \| null` | No | P | Filter to a specific token ID |



**Rust**


```rust
pub struct OrderbookHistoryRequest {
    pub market_id: String,
    pub token_id: Option<String>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}
```



**Python**


```python
class OrderbookHistoryRequest(BaseModel):
    market_id: str
    token_id: Optional[str]
    start_ts: Optional[int]
    end_ts: Optional[int]
    limit: Optional[int]
    cursor: Optional[str]
```



**TypeScript**


```typescript
interface OrderbookHistoryRequest {
  market_id: string;
  token_id?: string | null;
  start_ts?: number | null;
  end_ts?: number | null;
  limit?: number | null;
  cursor?: string | null;
}
```





### PriceHistoryRequest

| Field | Type | Required | Exchanges | Description |
|-------|------|----------|-----------|-------------|
| `condition_id` | `string \| null` | No | P | Condition ID for open interest enrichment (Polymarket only) |
| `end_ts` | `number \| null` | No | K P | End time filter (Unix seconds, inclusive) |
| `interval` | `PriceHistoryInterval` | Yes | K P | Candle interval (e.g., `"1h"`, `"1d"`) |
| `market_id` | `string` | Yes | K P | Native market ID to fetch candles for |
| `outcome` | `string \| null` | No | K | Filter by outcome name (Kalshi only) |
| `start_ts` | `number \| null` | No | K P | Start time filter (Unix seconds, inclusive) |
| `token_id` | `string \| null` | No | P | Filter by token ID (Polymarket only) |



**Rust**


```rust
pub struct PriceHistoryRequest {
    pub market_id: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
    pub condition_id: Option<String>,
    pub interval: PriceHistoryInterval,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
}
```



**Python**


```python
class PriceHistoryRequest(BaseModel):
    market_id: str
    interval: PriceHistoryInterval
    token_id: Optional[str]
    start_ts: Optional[int]
    end_ts: Optional[int]
    outcome: Optional[str]
    condition_id: Optional[str]
```



**TypeScript**


```typescript
interface PriceHistoryRequest {
  market_id: string;
  interval: PriceHistoryInterval;
  token_id?: string | null;
  start_ts?: number | null;
  end_ts?: number | null;
  outcome?: string | null;
  condition_id?: string | null;
}
```





## Supported Exchanges

### Kalshi

- **ID:** `kalshi`
- **Website:** [kalshi.com](https://kalshi.com)
- **API Docs:** [docs.kalshi.com](https://docs.kalshi.com)
- **Auth:** RSA key pair (`api_key_id` + `private_key_pem`)
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades, WebSocket

### Polymarket

- **ID:** `polymarket`
- **Website:** [polymarket.com](https://polymarket.com)
- **API Docs:** [docs.polymarket.com](https://docs.polymarket.com/developers/)
- **Auth:** Private key + optional CLOB API credentials
- **Features:** Markets, Orders, Positions, Balance, Orderbook, Price History, Trades, WebSocket

### Opinion

- **ID:** `opinion`
- **Website:** [opinion.trade](https://opinion.trade)
- **API Docs:** [docs.opinion.trade](https://docs.opinion.trade/developer-guide/opinion-open-api)
- **Auth:** API key + private key + multi-sig address
- **Features:** Markets, Orders, Positions, Balance, Orderbook, WebSocket (requires auth)

## Configuration

All exchanges accept a JSON config object. Pass exchange-specific fields:

```json
{
  "kalshi": {
    "api_key_id": "...",
    "private_key_pem": "...",
    "demo": false
  },
  "polymarket": {
    "private_key": "0x...",
    "funder": "0x...",
    "api_key": "...",
    "api_secret": "...",
    "api_passphrase": "..."
  },
  "opinion": {
    "api_key": "...",
    "private_key": "0x...",
    "multi_sig_addr": "0x..."
  }
}
```


## Error Hierarchy

```
OpenPxError
├── Network
│   ├── Http(String)
│   ├── Timeout(u64)
│   └── Connection(String)
├── Exchange
│   ├── MarketNotFound(String)
│   ├── InvalidOrder(String)
│   ├── OrderRejected(String)
│   ├── InsufficientFunds(String)
│   ├── Authentication(String)
│   ├── NotSupported(String)
│   └── Api(String)
├── WebSocket
│   ├── Connection(String)
│   ├── Closed
│   ├── Protocol(String)
│   └── Subscription(String)
├── Signing
│   ├── InvalidKey
│   ├── SigningFailed(String)
│   └── Unsupported(String)
├── RateLimitExceeded
├── Serialization(Error)
├── Config(String)
├── InvalidInput(String)
└── Other(String)
```

## Language Mapping



**Rust**

```rust
use px_core::{OpenPxError, ExchangeError};

match result {
    Err(OpenPxError::Exchange(ExchangeError::Authentication(msg))) => {
        eprintln!("Auth failed: {msg}");
    }
    Err(OpenPxError::Network(e)) => {
        eprintln!("Network error: {e}");
    }
    Err(e) => eprintln!("Error: {e}"),
    Ok(v) => { /* success */ }
}
```


**Python**

```python
from openpx import Exchange, OpenPxError, AuthenticationError, NetworkError

try:
    exchange.fetch_balance()
except AuthenticationError as e:
    print(f"Auth failed: {e}")
except NetworkError as e:
    print(f"Network error: {e}")
except OpenPxError as e:
    print(f"Error: {e}")
```


**TypeScript**

```typescript
try {
  await exchange.fetchBalance();
} catch (e) {
  console.error(e.message);
}
```


