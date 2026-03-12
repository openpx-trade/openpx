# TypeScript API Reference

All TypeScript types auto-generated from `schema/openpx.schema.json`.

## Market Data

### Market

```typescript
interface Market {
  close_time?: string | null;
  description?: string;
  id: string;
  liquidity: number;
  metadata?: unknown;
  outcomes: string[];
  prices: Record<string, number>;
  question: string;
  tick_size: number;
  volume: number;
}
```

### MarketStatus

```typescript
type MarketStatus = "active" | "closed" | "resolved";
```

### OutcomeToken

```typescript
interface OutcomeToken {
  outcome: string;
  token_id: string;
}
```

### UnifiedMarket

```typescript
interface UnifiedMarket {
  best_ask?: number | null;
  best_bid?: number | null;
  close_time?: string | null;
  condition_id?: string | null;
  description: string;
  event_id?: string | null;
  exchange: string;
  group_id?: string | null;
  icon_url?: string | null;
  id: string;
  image_url?: string | null;
  last_trade_price?: number | null;
  liquidity?: number | null;
  market_type: string;
  min_order_size?: number | null;
  open_interest?: number | null;
  open_time?: string | null;
  openpx_id: string;
  outcome_prices?: Record<string, number>;
  outcome_tokens?: OutcomeToken[];
  outcomes?: string[];
  price_change_1d?: number | null;
  price_change_1h?: number | null;
  price_change_1mo?: number | null;
  price_change_1wk?: number | null;
  question?: string | null;
  slug?: string | null;
  spread?: number | null;
  status: unknown;
  tick_size?: number | null;
  title: string;
  token_id_no?: string | null;
  token_id_yes?: string | null;
  volume: number;
  volume_1mo?: number | null;
  volume_1wk?: number | null;
  volume_24h?: number | null;
}
```

## Orders & Trading

### Fill

```typescript
interface Fill {
  created_at: string;
  fee: number;
  fill_id: string;
  is_taker: boolean;
  market_id: string;
  order_id: string;
  outcome: string;
  price: number;
  side: OrderSide;
  size: number;
}
```

### LiquidityRole

```typescript
type LiquidityRole = "maker" | "taker";
```

### Order

```typescript
interface Order {
  created_at: string;
  filled: number;
  id: string;
  market_id: string;
  outcome: string;
  price: number;
  side: OrderSide;
  size: number;
  status: OrderStatus;
  updated_at?: string | null;
}
```

### OrderSide

```typescript
type OrderSide = "buy" | "sell";
```

### OrderStatus

```typescript
type OrderStatus = "pending" | "open" | "filled" | "partially_filled" | "cancelled" | "rejected";
```

### OrderType

```typescript
type OrderType = "gtc" | "ioc" | "fok";
```

## Account & Positions

### DeltaInfo

```typescript
interface DeltaInfo {
  delta: number;
  max_outcome?: string | null;
  max_position: number;
}
```

### Nav

```typescript
interface Nav {
  cash: number;
  nav: number;
  positions: PositionBreakdown[];
  positions_value: number;
}
```

### Position

```typescript
interface Position {
  average_price: number;
  current_price: number;
  market_id: string;
  outcome: string;
  size: number;
}
```

### PositionBreakdown

```typescript
interface PositionBreakdown {
  current_price: number;
  outcome: string;
  size: number;
  value: number;
}
```

## Orderbook

### Orderbook

```typescript
interface Orderbook {
  asks: PriceLevel[];
  asset_id: string;
  bids: PriceLevel[];
  last_update_id?: number | null;
  market_id: string;
  timestamp?: string | null;
}
```

### OrderbookSnapshot

```typescript
interface OrderbookSnapshot {
  asks: PriceLevel[];
  bids: PriceLevel[];
  hash?: string | null;
  recorded_at?: string | null;
  timestamp: string;
}
```

### PriceLevel

```typescript
interface PriceLevel {
  price: number;
  size: number;
}
```

### PriceLevelChange

```typescript
interface PriceLevelChange {
  price: number;
  side: PriceLevelSide;
  size: number;
}
```

### PriceLevelSide

```typescript
type PriceLevelSide = "bid" | "ask";
```

## Trades & History

### Candlestick

```typescript
interface Candlestick {
  close: number;
  high: number;
  low: number;
  open: number;
  open_interest?: number | null;
  timestamp: string;
  volume: number;
}
```

### MarketTrade

```typescript
interface MarketTrade {
  aggressor_side?: string | null;
  id?: string | null;
  no_price?: number | null;
  outcome?: string | null;
  price: number;
  side?: string | null;
  size: number;
  source_channel: string;
  taker_address?: string | null;
  timestamp: string;
  tx_hash?: string | null;
  yes_price?: number | null;
}
```

### PriceHistoryInterval

```typescript
type PriceHistoryInterval = "1m" | "1h" | "6h" | "1d" | "1w" | "max";
```

## WebSocket & Streaming

### ActivityEvent

```typescript
type ActivityEvent = "Trade" | "Fill";
```

### ActivityFill

```typescript
interface ActivityFill {
  asset_id: string;
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
}
```

### ActivityTrade

```typescript
interface ActivityTrade {
  aggressor_side?: string | null;
  asset_id: string;
  market_id: string;
  outcome?: string | null;
  price: number;
  side?: string | null;
  size: number;
  source_channel: string;
  timestamp?: string | null;
  trade_id?: string | null;
}
```

## Configuration & Requests

### ExchangeInfo

```typescript
interface ExchangeInfo {
  has_approvals: boolean;
  has_cancel_order: boolean;
  has_create_order: boolean;
  has_fetch_balance: boolean;
  has_fetch_events: boolean;
  has_fetch_fills: boolean;
  has_fetch_markets: boolean;
  has_fetch_orderbook: boolean;
  has_fetch_orderbook_history: boolean;
  has_fetch_positions: boolean;
  has_fetch_price_history: boolean;
  has_fetch_trades: boolean;
  has_fetch_user_activity: boolean;
  has_refresh_balance: boolean;
  has_websocket: boolean;
  id: string;
  name: string;
}
```

### FetchMarketsParams

```typescript
interface FetchMarketsParams {
  cursor?: string | null;
  limit?: number | null;
}
```

### FetchOrdersParams

```typescript
interface FetchOrdersParams {
  market_id?: string | null;
}
```

### FetchUserActivityParams

```typescript
interface FetchUserActivityParams {
  address: string;
  limit?: number | null;
}
```

### OrderbookHistoryRequest

```typescript
interface OrderbookHistoryRequest {
  cursor?: string | null;
  end_ts?: number | null;
  limit?: number | null;
  market_id: string;
  start_ts?: number | null;
  token_id?: string | null;
}
```

### OrderbookRequest

```typescript
interface OrderbookRequest {
  market_id: string;
  outcome?: string | null;
  token_id?: string | null;
}
```

### PriceHistoryRequest

```typescript
interface PriceHistoryRequest {
  condition_id?: string | null;
  end_ts?: number | null;
  interval: PriceHistoryInterval;
  market_id: string;
  outcome?: string | null;
  start_ts?: number | null;
  token_id?: string | null;
}
```

### TradesRequest

```typescript
interface TradesRequest {
  cursor?: string | null;
  end_ts?: number | null;
  limit?: number | null;
  market_id: string;
  market_ref?: string | null;
  outcome?: string | null;
  start_ts?: number | null;
  token_id?: string | null;
}
```
