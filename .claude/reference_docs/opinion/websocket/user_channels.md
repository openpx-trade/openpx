# User Channels

## Order Update

### Subscribe

Message will be sent once your order in this market has an update (new/cancel/match/confirm).

{% hint style="warning" %}
Please note that the matched trade does not guarantee successful execution on-chain.

And the final on-chain amount/share may vary if fee applied. For the accurate on-chain amount/share, please subscribe Trade Executed channel.
{% endhint %}

<table><thead><tr><th width="150">Field</th><th width="200">Value</th><th>Description</th></tr></thead><tbody><tr><td>channel</td><td>trade.order.update</td><td>Channel of user order update </td></tr><tr><td>marketId</td><td>{MARKET_ID}</td><td>MarketId of subscribed binary market</td></tr><tr><td>rootMarketId</td><td>{ROOT_MARKET_ID}</td><td>MarketId of subscribed categorical market</td></tr></tbody></table>

{% code title="Example" %}

```
For Binary market:
{"action":"SUBSCRIBE","channel":"trade.order.update","marketId":1274}
For Categorical market:
{"action":"SUBSCRIBE","channel":"trade.order.update","rootMarketId":61}
```

{% endcode %}

### Structure

<table><thead><tr><th width="170">Name</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>orderUpdateType</td><td>string</td><td>orderNew | orderFill | orderCancel | orderConfirm</td></tr><tr><td>marketId</td><td>number</td><td>market id</td></tr><tr><td>rootMarketId</td><td>number</td><td>root market id if belongs to a categorical market</td></tr><tr><td>orderId</td><td>string</td><td>order id</td></tr><tr><td>side</td><td>number</td><td>1 - buy, 2 - sell</td></tr><tr><td>outcomeSide</td><td>number</td><td>1 - yes, 2 - no</td></tr><tr><td>price</td><td>string</td><td>price</td></tr><tr><td>shares</td><td>string</td><td>amount of conditional token (e.g. "Yes","No")</td></tr><tr><td>amount</td><td>string</td><td>amount of quote token</td></tr><tr><td>status</td><td>number</td><td>1 - pending, 2 - finished, 3 - canceled, 4 - expired, 5 - failed</td></tr><tr><td>tradingMethod</td><td>number</td><td>1 - market order, 2 - limit order</td></tr><tr><td>quoteToken</td><td>string</td><td>contract address of quote token</td></tr><tr><td>createdAt</td><td>number</td><td>create unix timestamp</td></tr><tr><td>expiresAt</td><td>number</td><td>expire unix timestamp</td></tr><tr><td>chainId</td><td>string</td><td>chain id</td></tr><tr><td>filledShares</td><td>string</td><td>filled in shares, update after order confirmed on chain</td></tr><tr><td>filledAmount</td><td>string</td><td>filled in amount, update after order confirmed on chain</td></tr></tbody></table>

{% code title="Sample message" expandable="true" %}

```json
{
  "orderUpdateType": "orderConfirm",
  "marketId": 2770,
  "rootMarketId": 122,
  "orderId": "a11ee07e-e22f-11f0-9714-0a58a9feac02",
  "side": 1,
  "outcomeSide": 1,
  "price": "0.150000000000000000",
  "shares": "66.66",
  "amount": "9.999000000000000000",
  "status": 1,
  "tradingMethod": 2,
  "quoteToken": "0x55d398326f99059fF775485246999027B3197955",
  "createdAt": 1766735464,
  "expiresAt": 0,
  "chainId": "56",
  "filledShares": "10.000000000000000000",
  "filledAmount": "1.500000000000000000",
  "msgType": "trade.order.update"
}
```

{% endcode %}

## Trade Executed

### Subscribe

Message will be sent once your trade (matched order) has been confirmed on-chain, or a split/merge has been executed on-chain. Same order can have multiple fills that ends up multiple trades.

<table><thead><tr><th width="150">Field</th><th width="200">Value</th><th>Description</th></tr></thead><tbody><tr><td>channel</td><td>trade.record.new</td><td>Channel of user trade notice</td></tr><tr><td>marketId</td><td>{MARKET_ID}</td><td>MarketId of subscribed binary market</td></tr><tr><td>rootMarketId</td><td>{ROOT_MARKET_ID}</td><td>MarketId of subscribed categorical market</td></tr></tbody></table>

{% code title="Example" %}

```
For Binary market:
{"action":"SUBSCRIBE","channel":"trade.record.new","marketId":1274}
For Categorical market:
{"action":"SUBSCRIBE","channel":"trade.record.new","rootMarketId":61}
```

{% endcode %}

### Structure

<table><thead><tr><th width="185">Name</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>orderId</td><td>string</td><td>order id, same order can have multiple fills that ends up multiple trades</td></tr><tr><td>txHash</td><td>string</td><td>transaction hash on-chain, each trade has a unique txHash</td></tr><tr><td>marketId</td><td>number</td><td>market id</td></tr><tr><td>rootMarketId</td><td>number</td><td>root market id if belongs to a categorical market</td></tr><tr><td>side</td><td>string</td><td>Buy | Sell | Split | Merge</td></tr><tr><td>outcomeSide</td><td>number</td><td>1 - yes, 2 - no</td></tr><tr><td>price</td><td>string</td><td>price</td></tr><tr><td>shares</td><td>string</td><td>amount of conditional token (e.g. "Yes","No")</td></tr><tr><td>amount</td><td>string</td><td>amount of quote token</td></tr><tr><td>profit</td><td>string</td><td>realized profit in usd value, applicable for sell/merge</td></tr><tr><td>status</td><td>number</td><td>2 - finished, 3 - canceled, 5 - failed, 6 - onchain failed</td></tr><tr><td>quoteToken</td><td>string</td><td>contract address of quote token</td></tr><tr><td>quoteTokenUsdPrice</td><td>string</td><td>USD price of quote token at the moment</td></tr><tr><td>usdAmount</td><td>string</td><td>order value in USD value</td></tr><tr><td>fee</td><td>string</td><td>fee applied to this trade</td></tr><tr><td>chainId</td><td>string</td><td>chain id</td></tr><tr><td>createdAt</td><td>number</td><td>create unix timestamp</td></tr><tr><td>tradeNo</td><td>string</td><td>trade id for reference</td></tr></tbody></table>

{% code title="Sample message" expandable="true" %}

```json
{
  "orderId": "3c7af25f-e21f-11f0-9714-0a58a9feac02",
  "tradeNo": "e1403840-e22f-11f0-83af-0a58a9feac02",
  "marketId": 2770,
  "rootMarketId": 122,
  "txHash": "0x272c8d9b8f90f50564173cf624c0ac5a371978b72bcd12604b26312a27e24195",
  "side": "Buy",
  "outcomeSide": 2,
  "price": "0.100000000000000000",
  "shares": "9.44444",
  "amount": "0.944444",
  "profit": "0.000000000000000000",
  "status": 2,
  "quoteToken": "0x55d398326f99059fF775485246999027B3197955",
  "quoteTokenUsdPrice": "1.000000000000000000",
  "usdAmount": "1000000.000000000000000000",
  "fee": "0.000000000000000000",
  "chainId": "56",
  "createdAt": 1766735571,
  "msgType": "trade.record.new"
}
```

{% endcode %}
