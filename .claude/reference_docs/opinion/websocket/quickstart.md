# Quickstart

## Authentication

To use Opinion Websocket, establish a `wss` connection to `wss://ws.opinion.trade` with your `apikey` as query parameters:

```
wss://ws.opinion.trade?apikey={API_KEY}
```

## Maintain connection

To maintain connection, send a HEARTBEAT message (e.g. every 30 seconds) to keep it open.

```
{"action":"HEARTBEAT"}
```

## Subscribe

To subscribe a channel, send a `SUBSCRIBE` message, for example:

```
For Binary market:
{"action":"SUBSCRIBE","channel":"{CHANNEL}","marketId":{MARKET_ID}}
For Categorical market:
{"action":"SUBSCRIBE","channel":"{CHANNEL}","rootMarketId":{ROOT_MARKET_ID}}
```

The exact requied fields (e.g. `marketId` or `rootMarketId`) depend on the channel to be subscribed.

To unsubscribe a channel, send an UNSUBSCRIBE message with the same parameters:

```
For Binary market:
{"action":"UNSUBSCRIBE","channel":"{CHANNEL}","marketId":{MARKET_ID}}
For Categorical market:
{"action":"UNSUBSCRIBE","channel":"{CHANNEL}","rootMarketId":{ROOT_MARKET_ID}}
```
