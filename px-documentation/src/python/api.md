# Python API Reference

All Python types auto-generated from `schema/openpx.schema.json`.

## Market Data

### Market

```python
class Market(BaseModel):
    close_time: Optional[datetime]
    description: Optional[str] = None
    id: str
    liquidity: float
    metadata: Optional[Any] = None
    outcomes: list[str]
    prices: dict[str, float]
    question: str
    tick_size: float
    volume: float
```

### MarketStatus

```python
class MarketStatus(str, Enum):
    ACTIVE = "active"
    CLOSED = "closed"
    RESOLVED = "resolved"
```

### OutcomeToken

```python
class OutcomeToken(BaseModel):
    outcome: str
    token_id: str
```

### UnifiedMarket

```python
class UnifiedMarket(BaseModel):
    best_ask: Optional[float]
    best_bid: Optional[float]
    close_time: Optional[datetime]
    condition_id: Optional[str]
    description: str
    event_id: Optional[str]
    exchange: str
    group_id: Optional[str]
    icon_url: Optional[str]
    id: str
    image_url: Optional[str]
    last_trade_price: Optional[float]
    liquidity: Optional[int]
    market_type: str
    min_order_size: Optional[float]
    open_interest: Optional[float]
    open_time: Optional[datetime]
    openpx_id: str
    outcome_prices: Optional[dict[str, float]] = None
    outcome_tokens: Optional[list[OutcomeToken]] = None
    outcomes: Optional[list[str]] = None
    price_change_1d: Optional[float]
    price_change_1h: Optional[float]
    price_change_1mo: Optional[float]
    price_change_1wk: Optional[float]
    question: Optional[str]
    slug: Optional[str]
    spread: Optional[float]
    status: Any
    tick_size: Optional[float]
    title: str
    token_id_no: Optional[str]
    token_id_yes: Optional[str]
    volume: int
    volume_1mo: Optional[int]
    volume_1wk: Optional[int]
    volume_24h: Optional[int]
```

## Orders & Trading

### Fill

```python
class Fill(BaseModel):
    created_at: datetime
    fee: float
    fill_id: str
    is_taker: bool
    market_id: str
    order_id: str
    outcome: str
    price: float
    side: OrderSide
    size: float
```

### LiquidityRole

```python
class LiquidityRole(str, Enum):
    MAKER = "maker"
    TAKER = "taker"
```

### Order

```python
class Order(BaseModel):
    created_at: datetime
    filled: float
    id: str
    market_id: str
    outcome: str
    price: float
    side: OrderSide
    size: float
    status: OrderStatus
    updated_at: Optional[datetime]
```

### OrderSide

```python
class OrderSide(str, Enum):
    BUY = "buy"
    SELL = "sell"
```

### OrderStatus

```python
class OrderStatus(str, Enum):
    PENDING = "pending"
    OPEN = "open"
    FILLED = "filled"
    PARTIALLY_FILLED = "partially_filled"
    CANCELLED = "cancelled"
    REJECTED = "rejected"
```

### OrderType

```python
class OrderType(str, Enum):
    GTC = "gtc"
    IOC = "ioc"
    FOK = "fok"
```

## Account & Positions

### DeltaInfo

```python
class DeltaInfo(BaseModel):
    delta: float
    max_outcome: Optional[str]
    max_position: float
```

### Nav

```python
class Nav(BaseModel):
    cash: float
    nav: float
    positions: list[PositionBreakdown]
    positions_value: float
```

### Position

```python
class Position(BaseModel):
    average_price: float
    current_price: float
    market_id: str
    outcome: str
    size: float
```

### PositionBreakdown

```python
class PositionBreakdown(BaseModel):
    current_price: float
    outcome: str
    size: float
    value: float
```

## Orderbook

### Orderbook

```python
class Orderbook(BaseModel):
    asks: list[PriceLevel]
    asset_id: str
    bids: list[PriceLevel]
    last_update_id: Optional[int]
    market_id: str
    timestamp: Optional[datetime]
```

### OrderbookSnapshot

```python
class OrderbookSnapshot(BaseModel):
    asks: list[PriceLevel]
    bids: list[PriceLevel]
    hash: Optional[str]
    recorded_at: Optional[datetime]
    timestamp: datetime
```

### PriceLevel

```python
class PriceLevel(BaseModel):
    price: float
    size: float
```

### PriceLevelChange

```python
class PriceLevelChange(BaseModel):
    price: float
    side: PriceLevelSide
    size: float
```

### PriceLevelSide

```python
class PriceLevelSide(str, Enum):
    BID = "bid"
    ASK = "ask"
```

## Trades & History

### Candlestick

```python
class Candlestick(BaseModel):
    close: float
    high: float
    low: float
    open: float
    open_interest: Optional[float]
    timestamp: datetime
    volume: float
```

### MarketTrade

```python
class MarketTrade(BaseModel):
    aggressor_side: Optional[str]
    id: Optional[str]
    no_price: Optional[float]
    outcome: Optional[str]
    price: float
    side: Optional[str]
    size: float
    source_channel: str
    taker_address: Optional[str]
    timestamp: datetime
    tx_hash: Optional[str]
    yes_price: Optional[float]
```

### PriceHistoryInterval

```python
class PriceHistoryInterval(str, Enum):
    1M = "1m"
    1H = "1h"
    6H = "6h"
    1D = "1d"
    1W = "1w"
    MAX = "max"
```

## WebSocket & Streaming

### ActivityEvent

```python
class ActivityEvent(str, Enum):
    TRADE = "Trade"
    FILL = "Fill"
```

### ActivityFill

```python
class ActivityFill(BaseModel):
    asset_id: str
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
```

### ActivityTrade

```python
class ActivityTrade(BaseModel):
    aggressor_side: Optional[str]
    asset_id: str
    market_id: str
    outcome: Optional[str]
    price: float
    side: Optional[str]
    size: float
    source_channel: str
    timestamp: Optional[datetime]
    trade_id: Optional[str]
```

## Configuration & Requests

### ExchangeInfo

```python
class ExchangeInfo(BaseModel):
    has_approvals: bool
    has_cancel_order: bool
    has_create_order: bool
    has_fetch_balance: bool
    has_fetch_events: bool
    has_fetch_fills: bool
    has_fetch_markets: bool
    has_fetch_orderbook: bool
    has_fetch_orderbook_history: bool
    has_fetch_positions: bool
    has_fetch_price_history: bool
    has_fetch_trades: bool
    has_fetch_user_activity: bool
    has_refresh_balance: bool
    has_websocket: bool
    id: str
    name: str
```

### FetchMarketsParams

```python
class FetchMarketsParams(BaseModel):
    cursor: Optional[str]
    limit: Optional[int]
```

### FetchOrdersParams

```python
class FetchOrdersParams(BaseModel):
    market_id: Optional[str]
```

### FetchUserActivityParams

```python
class FetchUserActivityParams(BaseModel):
    address: str
    limit: Optional[int]
```

### OrderbookHistoryRequest

```python
class OrderbookHistoryRequest(BaseModel):
    cursor: Optional[str]
    end_ts: Optional[int]
    limit: Optional[int]
    market_id: str
    start_ts: Optional[int]
    token_id: Optional[str]
```

### OrderbookRequest

```python
class OrderbookRequest(BaseModel):
    market_id: str
    outcome: Optional[str]
    token_id: Optional[str]
```

### PriceHistoryRequest

```python
class PriceHistoryRequest(BaseModel):
    condition_id: Optional[str]
    end_ts: Optional[int]
    interval: PriceHistoryInterval
    market_id: str
    outcome: Optional[str]
    start_ts: Optional[int]
    token_id: Optional[str]
```

### TradesRequest

```python
class TradesRequest(BaseModel):
    cursor: Optional[str]
    end_ts: Optional[int]
    limit: Optional[int]
    market_id: str
    market_ref: Optional[str]
    outcome: Optional[str]
    start_ts: Optional[int]
    token_id: Optional[str]
```
