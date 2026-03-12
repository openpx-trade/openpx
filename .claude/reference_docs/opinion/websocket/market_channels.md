# Market Channels

## Orderbook Change

### Subscribe

Message will be sent once the orderbook has any change (new/cancel/match order).

<table><thead><tr><th width="150">Field</th><th width="200">Value</th><th>Description</th></tr></thead><tbody><tr><td>channel</td><td>market.depth.diff</td><td>Channel of orderbook change</td></tr><tr><td>marketId</td><td>{MARKET_ID}</td><td>MarketId of subscribed market</td></tr></tbody></table>

{% hint style="info" %}
Orderbook Change applied to a single binary market only.

For categorical market, you should subscribe each `market_id` individually.
{% endhint %}

{% code title="Example" %}

```
{"action":"SUBSCRIBE","channel":"market.depth.diff","marketId":1274}
```

{% endcode %}

### Structure

<table><thead><tr><th width="180">Name</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>marketId</td><td>number</td><td>market id</td></tr><tr><td>rootMarketId</td><td>number</td><td>root market id if belongs to a categorical market</td></tr><tr><td>tokenId</td><td>string</td><td>token id of updated conditional token</td></tr><tr><td>outcomeSide</td><td>number</td><td>1 - yes, 2 - no</td></tr><tr><td>side</td><td>string</td><td>bids | asks</td></tr><tr><td>price</td><td>string</td><td>price</td></tr><tr><td>size</td><td>string</td><td>shares of conditional tokens</td></tr></tbody></table>

<pre class="language-json" data-title="Sample message" data-expandable="true"><code class="lang-json">{
<strong>    "marketId": 2764, 
</strong>    "tokenId": "19120407572139442221452465677574895365338028945317996490376653704877573103648", 
    "outcomeSide": 1, 
    "side": "bids", 
    "price": "0.2", 
    "size": "50", 
    "msgType": "market.depth.diff"
}
</code></pre>

## Market Price Change

Message will be sent once the latest match price has changed.

### Subscribe

<table><thead><tr><th width="150">Field</th><th width="200">Value</th><th>Description</th></tr></thead><tbody><tr><td>channel</td><td>market.last.price</td><td>Channel of market price change </td></tr><tr><td>marketId</td><td>{MARKET_ID}</td><td>MarketId of subscribed binary market</td></tr><tr><td>rootMarketId</td><td>{ROOT_MARKET_ID}</td><td>MarketId of subscribed categorical market</td></tr></tbody></table>

{% hint style="info" %}
If `rootMarketId` is defined, `marketId` will be omitted.

Subscribing root market will receive messages of all of its sub-markets.
{% endhint %}

<pre data-title="Example"><code><strong>For Binary market:
</strong>{"action":"SUBSCRIBE","channel":"market.last.price","marketId":1274}
For Categorical market:
{"action":"SUBSCRIBE","channel":"market.last.price","rootMarketId":61}
</code></pre>

### Structure

<table><thead><tr><th width="180">Name</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>marketId</td><td>number</td><td>market id</td></tr><tr><td>rootMarketId</td><td>number</td><td>root market id if belongs to a categorical market</td></tr><tr><td>tokenId</td><td>string</td><td>token id of updated conditional token</td></tr><tr><td>price</td><td>string</td><td>price</td></tr><tr><td>outcomeSide</td><td>number</td><td>1 - yes, 2 - no</td></tr></tbody></table>

{% code title="Sample message" expandable="true" %}

```json
{
    "tokenId": "19120407572139442221452465677574895365338028945317996490376653704877573103648", 
    "outcomeSide": 1, 
    "price": "0.85", 
    "marketId": 2764, 
    "msgType": "market.last.price"
}
```

{% endcode %}

## Market Last Trade

### Subscribe

Message will be sent once a trade matched in this market.

{% hint style="warning" %}
Please note that the matched trade does not guarantee successful execution on-chain.

And the final on-chain amount/share may vary if fee applied.
{% endhint %}

<table><thead><tr><th width="150">Field</th><th width="200">Value</th><th>Description</th></tr></thead><tbody><tr><td>channel</td><td>market.last.trade</td><td>Channel of market last trade </td></tr><tr><td>marketId</td><td>{MARKET_ID}</td><td>MarketId of subscribed binary market</td></tr><tr><td>rootMarketId</td><td>{ROOT_MARKET_ID}</td><td>MarketId of subscribed categorical market</td></tr></tbody></table>

{% code title="Example" %}

```
For Binary market:
{"action":"SUBSCRIBE","channel":"market.last.trade","marketId":1274}
For Categorical market:
{"action":"SUBSCRIBE","channel":"market.last.trade","rootMarketId":61}
```

{% endcode %}

### Structure

<table><thead><tr><th width="180">Name</th><th width="100">Type</th><th>Description</th></tr></thead><tbody><tr><td>marketId</td><td>number</td><td>market id</td></tr><tr><td>rootMarketId</td><td>number</td><td>root market id if belongs to a categorical market</td></tr><tr><td>tokenId</td><td>string</td><td>token id of updated conditional token</td></tr><tr><td>side</td><td>string</td><td>Buy | Sell | Split | Merge</td></tr><tr><td>outcomeSide</td><td>number</td><td>1 - yes, 2 - no</td></tr><tr><td>price</td><td>string</td><td>price</td></tr><tr><td>shares</td><td>string</td><td>amount of conditional token</td></tr><tr><td>amount</td><td>string</td><td>amount of quote token</td></tr></tbody></table>

{% code title="Sample message" expandable="true" %}

```json
{
    "tokenId": "19120407572139442221452465677574895365338028945317996490376653704877573103648", 
    "side": "Buy", 
    "outcomeSide": 1, 
    "price": "0.85", 
    "shares": "10", 
    "amount": "8.5", 
    "marketId": 2764, 
    "msgType": "market.last.trade"
}
```

{% endcode %}
