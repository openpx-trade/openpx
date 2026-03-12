# Type Reference

All types auto-generated from Rust source via `schema/openpx.schema.json`.
Run `just docs` to regenerate.

## Market Data

### Market

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `close_time` | `string | null` | No |  |
| `description` | `string` | No |  |
| `id` | `string` | Yes |  |
| `liquidity` | `number` | Yes |  |
| `metadata` | `unknown` | No |  |
| `outcomes` | `string[]` | Yes |  |
| `prices` | `Record<string, number>` | Yes | Outcome prices normalized to decimal format (0.0 to 1.0). |
| `question` | `string` | Yes |  |
| `tick_size` | `number` | Yes |  |
| `volume` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Market {
    pub close_time: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub id: String,
    pub liquidity: f64,
    pub metadata: Option<serde_json::Value>,
    pub outcomes: Vec<String>,
    pub prices: HashMap<String, f64>,
    pub question: String,
    pub tick_size: f64,
    pub volume: f64,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### MarketStatus

Enum with variants: `active`, `closed`, `resolved`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum MarketStatus {
    Active,
    Closed,
    Resolved,
}
```

**Python**
```python
class MarketStatus(str, Enum):
    ACTIVE = "active"
    CLOSED = "closed"
    RESOLVED = "resolved"
```

**TypeScript**
```typescript
type MarketStatus = "active" | "closed" | "resolved";
```

</details>

---

### OutcomeToken

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `outcome` | `string` | Yes |  |
| `token_id` | `string` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct OutcomeToken {
    pub outcome: String,
    pub token_id: String,
}
```

**Python**
```python
class OutcomeToken(BaseModel):
    outcome: str
    token_id: str
```

**TypeScript**
```typescript
interface OutcomeToken {
  outcome: string;
  token_id: string;
}
```

</details>

---

### UnifiedMarket

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `best_ask` | `number | null` | No | Best ask price (normalized 0-1) |
| `best_bid` | `number | null` | No | Best bid price (normalized 0-1) |
| `close_time` | `string | null` | No | Market close time |
| `condition_id` | `string | null` | No | Condition ID for CTF (nullable) |
| `description` | `string` | Yes | Full description/rules |
| `event_id` | `string | null` | No | Canonical OpenPX event ID used for cross-exchange event grouping. SDK users should prefer this over exchange-specific `group_id`. |
| `exchange` | `string` | Yes | Exchange identifier (kalshi, polymarket, etc.) |
| `group_id` | `string | null` | No | Source-native event/group ID from the exchange. Keep this raw so advanced users can reason about exchange internals. |
| `icon_url` | `string | null` | No | Market icon URL |
| `id` | `string` | Yes | Native exchange market ID |
| `image_url` | `string | null` | No | Market image URL |
| `last_trade_price` | `number | null` | No | Last trade price (normalized 0-1) |
| `liquidity` | `number | null` | No | Current liquidity (nullable) |
| `market_type` | `string` | Yes | Market type (binary, categorical, etc.) |
| `min_order_size` | `number | null` | No | Minimum order size (contracts). Exchange-specific: Polymarket varies per market (e.g. 5, 15); Kalshi defaults to 1. |
| `open_interest` | `number | null` | No | Current open interest (contracts/pairs) |
| `open_time` | `string | null` | No | Market open time |
| `openpx_id` | `string` | Yes | Primary key: {exchange}:{native_id} |
| `outcome_prices` | `Record<string, number>` | No | Outcome prices from the REST API (e.g., {"Yes": 0.65, "No": 0.35}) |
| `outcome_tokens` | `OutcomeToken[]` | No | Outcome-to-token mapping for orderbook subscriptions |
| `outcomes` | `string[]` | No | Outcome labels (e.g., ["Yes", "No"] for binary markets) |
| `price_change_1d` | `number | null` | No | 24-hour YES price change (decimal, e.g. 0.05 = +5%) |
| `price_change_1h` | `number | null` | No | 1-hour YES price change (decimal, e.g. -0.02 = -2%) |
| `price_change_1mo` | `number | null` | No | 30-day YES price change (decimal) |
| `price_change_1wk` | `number | null` | No | 7-day YES price change (decimal) |
| `question` | `string | null` | No | Market question (nullable, may differ from title) |
| `slug` | `string | null` | No | URL-friendly identifier (nullable) |
| `spread` | `number | null` | No | Bid-ask spread (decimal) |
| `status` | `unknown` | Yes | Normalized status: Active, Closed, Resolved |
| `tick_size` | `number | null` | No | Tick size (minimum price increment). Normalized to decimal (e.g. 0.01 = 1 cent). |
| `title` | `string` | Yes | Market title |
| `token_id_no` | `string | null` | No | No outcome token ID (nullable) |
| `token_id_yes` | `string | null` | No | Yes outcome token ID (nullable) |
| `volume` | `number` | Yes | Total volume (integer, coerced from f64/string) |
| `volume_1mo` | `number | null` | No | 30-day rolling trading volume (USDC) |
| `volume_1wk` | `number | null` | No | 7-day rolling trading volume (USDC) |
| `volume_24h` | `number | null` | No | 24-hour trading volume (USDC) |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct UnifiedMarket {
    pub best_ask: Option<f64>,
    pub best_bid: Option<f64>,
    pub close_time: Option<DateTime<Utc>>,
    pub condition_id: Option<String>,
    pub description: String,
    pub event_id: Option<String>,
    pub exchange: String,
    pub group_id: Option<String>,
    pub icon_url: Option<String>,
    pub id: String,
    pub image_url: Option<String>,
    pub last_trade_price: Option<f64>,
    pub liquidity: Option<i64>,
    pub market_type: String,
    pub min_order_size: Option<f64>,
    pub open_interest: Option<f64>,
    pub open_time: Option<DateTime<Utc>>,
    pub openpx_id: String,
    pub outcome_prices: Option<HashMap<String, f64>>,
    pub outcome_tokens: Option<Vec<OutcomeToken>>,
    pub outcomes: Option<Vec<String>>,
    pub price_change_1d: Option<f64>,
    pub price_change_1h: Option<f64>,
    pub price_change_1mo: Option<f64>,
    pub price_change_1wk: Option<f64>,
    pub question: Option<String>,
    pub slug: Option<String>,
    pub spread: Option<f64>,
    pub status: serde_json::Value,
    pub tick_size: Option<f64>,
    pub title: String,
    pub token_id_no: Option<String>,
    pub token_id_yes: Option<String>,
    pub volume: i64,
    pub volume_1mo: Option<i64>,
    pub volume_1wk: Option<i64>,
    pub volume_24h: Option<i64>,
}
```

**Python**
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

**TypeScript**
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

</details>

---

## Orders & Trading

### Fill

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `string` | Yes |  |
| `fee` | `number` | Yes |  |
| `fill_id` | `string` | Yes |  |
| `is_taker` | `boolean` | Yes |  |
| `market_id` | `string` | Yes |  |
| `order_id` | `string` | Yes |  |
| `outcome` | `string` | Yes |  |
| `price` | `number` | Yes |  |
| `side` | `OrderSide` | Yes |  |
| `size` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Fill {
    pub created_at: DateTime<Utc>,
    pub fee: f64,
    pub fill_id: String,
    pub is_taker: bool,
    pub market_id: String,
    pub order_id: String,
    pub outcome: String,
    pub price: f64,
    pub side: OrderSide,
    pub size: f64,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### LiquidityRole

Enum with variants: `maker`, `taker`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum LiquidityRole {
    Maker,
    Taker,
}
```

**Python**
```python
class LiquidityRole(str, Enum):
    MAKER = "maker"
    TAKER = "taker"
```

**TypeScript**
```typescript
type LiquidityRole = "maker" | "taker";
```

</details>

---

### Order

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `created_at` | `string` | Yes |  |
| `filled` | `number` | Yes |  |
| `id` | `string` | Yes |  |
| `market_id` | `string` | Yes |  |
| `outcome` | `string` | Yes |  |
| `price` | `number` | Yes |  |
| `side` | `OrderSide` | Yes |  |
| `size` | `number` | Yes |  |
| `status` | `OrderStatus` | Yes |  |
| `updated_at` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Order {
    pub created_at: DateTime<Utc>,
    pub filled: f64,
    pub id: String,
    pub market_id: String,
    pub outcome: String,
    pub price: f64,
    pub side: OrderSide,
    pub size: f64,
    pub status: OrderStatus,
    pub updated_at: Option<DateTime<Utc>>,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### OrderSide

Enum with variants: `buy`, `sell`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum OrderSide {
    Buy,
    Sell,
}
```

**Python**
```python
class OrderSide(str, Enum):
    BUY = "buy"
    SELL = "sell"
```

**TypeScript**
```typescript
type OrderSide = "buy" | "sell";
```

</details>

---

### OrderStatus

Enum with variants: `pending`, `open`, `filled`, `partially_filled`, `cancelled`, `rejected`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

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

</details>

---

### OrderType

Enum with variants: `gtc`, `ioc`, `fok`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum OrderType {
    Gtc,
    Ioc,
    Fok,
}
```

**Python**
```python
class OrderType(str, Enum):
    GTC = "gtc"
    IOC = "ioc"
    FOK = "fok"
```

**TypeScript**
```typescript
type OrderType = "gtc" | "ioc" | "fok";
```

</details>

---

## Account & Positions

### DeltaInfo

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `delta` | `number` | Yes |  |
| `max_outcome` | `string | null` | No |  |
| `max_position` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct DeltaInfo {
    pub delta: f64,
    pub max_outcome: Option<String>,
    pub max_position: f64,
}
```

**Python**
```python
class DeltaInfo(BaseModel):
    delta: float
    max_outcome: Optional[str]
    max_position: float
```

**TypeScript**
```typescript
interface DeltaInfo {
  delta: number;
  max_outcome?: string | null;
  max_position: number;
}
```

</details>

---

### Nav

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cash` | `number` | Yes |  |
| `nav` | `number` | Yes |  |
| `positions` | `PositionBreakdown[]` | Yes |  |
| `positions_value` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Nav {
    pub cash: f64,
    pub nav: f64,
    pub positions: Vec<PositionBreakdown>,
    pub positions_value: f64,
}
```

**Python**
```python
class Nav(BaseModel):
    cash: float
    nav: float
    positions: list[PositionBreakdown]
    positions_value: float
```

**TypeScript**
```typescript
interface Nav {
  cash: number;
  nav: number;
  positions: PositionBreakdown[];
  positions_value: number;
}
```

</details>

---

### Position

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `average_price` | `number` | Yes |  |
| `current_price` | `number` | Yes |  |
| `market_id` | `string` | Yes |  |
| `outcome` | `string` | Yes |  |
| `size` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Position {
    pub average_price: f64,
    pub current_price: f64,
    pub market_id: String,
    pub outcome: String,
    pub size: f64,
}
```

**Python**
```python
class Position(BaseModel):
    average_price: float
    current_price: float
    market_id: str
    outcome: str
    size: float
```

**TypeScript**
```typescript
interface Position {
  average_price: number;
  current_price: number;
  market_id: string;
  outcome: string;
  size: number;
}
```

</details>

---

### PositionBreakdown

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `current_price` | `number` | Yes |  |
| `outcome` | `string` | Yes |  |
| `size` | `number` | Yes |  |
| `value` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct PositionBreakdown {
    pub current_price: f64,
    pub outcome: String,
    pub size: f64,
    pub value: f64,
}
```

**Python**
```python
class PositionBreakdown(BaseModel):
    current_price: float
    outcome: str
    size: float
    value: float
```

**TypeScript**
```typescript
interface PositionBreakdown {
  current_price: number;
  outcome: string;
  size: number;
  value: number;
}
```

</details>

---

## Orderbook

### Orderbook

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asks` | `PriceLevel[]` | Yes |  |
| `asset_id` | `string` | Yes |  |
| `bids` | `PriceLevel[]` | Yes |  |
| `last_update_id` | `number | null` | No |  |
| `market_id` | `string` | Yes |  |
| `timestamp` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Orderbook {
    pub asks: Vec<PriceLevel>,
    pub asset_id: String,
    pub bids: Vec<PriceLevel>,
    pub last_update_id: Option<u64>,
    pub market_id: String,
    pub timestamp: Option<DateTime<Utc>>,
}
```

**Python**
```python
class Orderbook(BaseModel):
    asks: list[PriceLevel]
    asset_id: str
    bids: list[PriceLevel]
    last_update_id: Optional[int]
    market_id: str
    timestamp: Optional[datetime]
```

**TypeScript**
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

</details>

---

### OrderbookSnapshot

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asks` | `PriceLevel[]` | Yes |  |
| `bids` | `PriceLevel[]` | Yes |  |
| `hash` | `string | null` | No |  |
| `recorded_at` | `string | null` | No |  |
| `timestamp` | `string` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct OrderbookSnapshot {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
    pub hash: Option<String>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub timestamp: DateTime<Utc>,
}
```

**Python**
```python
class OrderbookSnapshot(BaseModel):
    asks: list[PriceLevel]
    bids: list[PriceLevel]
    hash: Optional[str]
    recorded_at: Optional[datetime]
    timestamp: datetime
```

**TypeScript**
```typescript
interface OrderbookSnapshot {
  asks: PriceLevel[];
  bids: PriceLevel[];
  hash?: string | null;
  recorded_at?: string | null;
  timestamp: string;
}
```

</details>

---

### PriceLevel

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `price` | `number` | Yes |  |
| `size` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}
```

**Python**
```python
class PriceLevel(BaseModel):
    price: float
    size: float
```

**TypeScript**
```typescript
interface PriceLevel {
  price: number;
  size: number;
}
```

</details>

---

### PriceLevelChange

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `price` | `number` | Yes |  |
| `side` | `PriceLevelSide` | Yes |  |
| `size` | `number` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct PriceLevelChange {
    pub price: f64,
    pub side: PriceLevelSide,
    pub size: f64,
}
```

**Python**
```python
class PriceLevelChange(BaseModel):
    price: float
    side: PriceLevelSide
    size: float
```

**TypeScript**
```typescript
interface PriceLevelChange {
  price: number;
  side: PriceLevelSide;
  size: number;
}
```

</details>

---

### PriceLevelSide

Enum with variants: `bid`, `ask`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum PriceLevelSide {
    Bid,
    Ask,
}
```

**Python**
```python
class PriceLevelSide(str, Enum):
    BID = "bid"
    ASK = "ask"
```

**TypeScript**
```typescript
type PriceLevelSide = "bid" | "ask";
```

</details>

---

## Trades & History

### Candlestick

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `close` | `number` | Yes |  |
| `high` | `number` | Yes |  |
| `low` | `number` | Yes |  |
| `open` | `number` | Yes |  |
| `open_interest` | `number | null` | No | Open interest at this candle's close. Only available from exchanges that report it (e.g., Kalshi). |
| `timestamp` | `string` | Yes | Period start timestamp (UTC). lightweight-charts expects start-of-period. |
| `volume` | `number` | Yes | Trade volume in contracts. 0.0 if exchange doesn't provide volume. |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct Candlestick {
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub open_interest: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub volume: f64,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### MarketTrade

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aggressor_side` | `string | null` | No |  |
| `id` | `string | null` | No |  |
| `no_price` | `number | null` | No |  |
| `outcome` | `string | null` | No |  |
| `price` | `number` | Yes |  |
| `side` | `string | null` | No |  |
| `size` | `number` | Yes |  |
| `source_channel` | `string` | Yes |  |
| `taker_address` | `string | null` | No |  |
| `timestamp` | `string` | Yes |  |
| `tx_hash` | `string | null` | No |  |
| `yes_price` | `number | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct MarketTrade {
    pub aggressor_side: Option<String>,
    pub id: Option<String>,
    pub no_price: Option<f64>,
    pub outcome: Option<String>,
    pub price: f64,
    pub side: Option<String>,
    pub size: f64,
    pub source_channel: String,
    pub taker_address: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: Option<String>,
    pub yes_price: Option<f64>,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### PriceHistoryInterval

Enum with variants: `1m`, `1h`, `6h`, `1d`, `1w`, `max`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum PriceHistoryInterval {
    1m,
    1h,
    6h,
    1d,
    1w,
    Max,
}
```

**Python**
```python
class PriceHistoryInterval(str, Enum):
    1M = "1m"
    1H = "1h"
    6H = "6h"
    1D = "1d"
    1W = "1w"
    MAX = "max"
```

**TypeScript**
```typescript
type PriceHistoryInterval = "1m" | "1h" | "6h" | "1d" | "1w" | "max";
```

</details>

---

## WebSocket & Streaming

### ActivityEvent

Enum with variants: `Trade`, `Fill`

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub enum ActivityEvent {
    Trade,
    Fill,
}
```

**Python**
```python
class ActivityEvent(str, Enum):
    TRADE = "Trade"
    FILL = "Fill"
```

**TypeScript**
```typescript
type ActivityEvent = "Trade" | "Fill";
```

</details>

---

### ActivityFill

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asset_id` | `string` | Yes |  |
| `fill_id` | `string | null` | No |  |
| `liquidity_role` | `LiquidityRole | null` | No |  |
| `market_id` | `string` | Yes |  |
| `order_id` | `string | null` | No |  |
| `outcome` | `string | null` | No |  |
| `price` | `number` | Yes |  |
| `side` | `string | null` | No |  |
| `size` | `number` | Yes |  |
| `source_channel` | `string` | Yes |  |
| `timestamp` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct ActivityFill {
    pub asset_id: String,
    pub fill_id: Option<String>,
    pub liquidity_role: Option<LiquidityRole>,
    pub market_id: String,
    pub order_id: Option<String>,
    pub outcome: Option<String>,
    pub price: f64,
    pub side: Option<String>,
    pub size: f64,
    pub source_channel: String,
    pub timestamp: Option<DateTime<Utc>>,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### ActivityTrade

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aggressor_side` | `string | null` | No |  |
| `asset_id` | `string` | Yes |  |
| `market_id` | `string` | Yes |  |
| `outcome` | `string | null` | No |  |
| `price` | `number` | Yes |  |
| `side` | `string | null` | No |  |
| `size` | `number` | Yes |  |
| `source_channel` | `string` | Yes |  |
| `timestamp` | `string | null` | No |  |
| `trade_id` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct ActivityTrade {
    pub aggressor_side: Option<String>,
    pub asset_id: String,
    pub market_id: String,
    pub outcome: Option<String>,
    pub price: f64,
    pub side: Option<String>,
    pub size: f64,
    pub source_channel: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub trade_id: Option<String>,
}
```

**Python**
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

**TypeScript**
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

</details>

---

## Configuration & Requests

### ExchangeInfo

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `has_approvals` | `boolean` | Yes |  |
| `has_cancel_order` | `boolean` | Yes |  |
| `has_create_order` | `boolean` | Yes |  |
| `has_fetch_balance` | `boolean` | Yes |  |
| `has_fetch_events` | `boolean` | Yes |  |
| `has_fetch_fills` | `boolean` | Yes |  |
| `has_fetch_markets` | `boolean` | Yes |  |
| `has_fetch_orderbook` | `boolean` | Yes |  |
| `has_fetch_orderbook_history` | `boolean` | Yes |  |
| `has_fetch_positions` | `boolean` | Yes |  |
| `has_fetch_price_history` | `boolean` | Yes |  |
| `has_fetch_trades` | `boolean` | Yes |  |
| `has_fetch_user_activity` | `boolean` | Yes |  |
| `has_refresh_balance` | `boolean` | Yes |  |
| `has_websocket` | `boolean` | Yes |  |
| `id` | `string` | Yes |  |
| `name` | `string` | Yes |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct ExchangeInfo {
    pub has_approvals: bool,
    pub has_cancel_order: bool,
    pub has_create_order: bool,
    pub has_fetch_balance: bool,
    pub has_fetch_events: bool,
    pub has_fetch_fills: bool,
    pub has_fetch_markets: bool,
    pub has_fetch_orderbook: bool,
    pub has_fetch_orderbook_history: bool,
    pub has_fetch_positions: bool,
    pub has_fetch_price_history: bool,
    pub has_fetch_trades: bool,
    pub has_fetch_user_activity: bool,
    pub has_refresh_balance: bool,
    pub has_websocket: bool,
    pub id: String,
    pub name: String,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### FetchMarketsParams

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cursor` | `string | null` | No | Exchange-specific cursor (offset, page number, or cursor string) |
| `limit` | `number | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct FetchMarketsParams {
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}
```

**Python**
```python
class FetchMarketsParams(BaseModel):
    cursor: Optional[str]
    limit: Optional[int]
```

**TypeScript**
```typescript
interface FetchMarketsParams {
  cursor?: string | null;
  limit?: number | null;
}
```

</details>

---

### FetchOrdersParams

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `market_id` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

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

</details>

---

### FetchUserActivityParams

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | `string` | Yes |  |
| `limit` | `number | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct FetchUserActivityParams {
    pub address: String,
    pub limit: Option<i64>,
}
```

**Python**
```python
class FetchUserActivityParams(BaseModel):
    address: str
    limit: Optional[int]
```

**TypeScript**
```typescript
interface FetchUserActivityParams {
  address: string;
  limit?: number | null;
}
```

</details>

---

### OrderbookHistoryRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cursor` | `string | null` | No |  |
| `end_ts` | `number | null` | No |  |
| `limit` | `number | null` | No |  |
| `market_id` | `string` | Yes |  |
| `start_ts` | `number | null` | No |  |
| `token_id` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct OrderbookHistoryRequest {
    pub cursor: Option<String>,
    pub end_ts: Option<i64>,
    pub limit: Option<i64>,
    pub market_id: String,
    pub start_ts: Option<i64>,
    pub token_id: Option<String>,
}
```

**Python**
```python
class OrderbookHistoryRequest(BaseModel):
    cursor: Optional[str]
    end_ts: Optional[int]
    limit: Optional[int]
    market_id: str
    start_ts: Optional[int]
    token_id: Optional[str]
```

**TypeScript**
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

</details>

---

### OrderbookRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `market_id` | `string` | Yes |  |
| `outcome` | `string | null` | No |  |
| `token_id` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct OrderbookRequest {
    pub market_id: String,
    pub outcome: Option<String>,
    pub token_id: Option<String>,
}
```

**Python**
```python
class OrderbookRequest(BaseModel):
    market_id: str
    outcome: Optional[str]
    token_id: Optional[str]
```

**TypeScript**
```typescript
interface OrderbookRequest {
  market_id: string;
  outcome?: string | null;
  token_id?: string | null;
}
```

</details>

---

### PriceHistoryRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `condition_id` | `string | null` | No | Condition ID for OI enrichment (Polymarket). |
| `end_ts` | `number | null` | No | Unix seconds |
| `interval` | `PriceHistoryInterval` | Yes |  |
| `market_id` | `string` | Yes |  |
| `outcome` | `string | null` | No |  |
| `start_ts` | `number | null` | No | Unix seconds |
| `token_id` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct PriceHistoryRequest {
    pub condition_id: Option<String>,
    pub end_ts: Option<i64>,
    pub interval: PriceHistoryInterval,
    pub market_id: String,
    pub outcome: Option<String>,
    pub start_ts: Option<i64>,
    pub token_id: Option<String>,
}
```

**Python**
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

**TypeScript**
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

</details>

---

### TradesRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cursor` | `string | null` | No | Opaque pagination cursor from a previous response. |
| `end_ts` | `number | null` | No | Unix seconds (inclusive) |
| `limit` | `number | null` | No | Max number of trades to return (exchange-specific caps may apply). |
| `market_id` | `string` | Yes | Exchange-native market identifier (as used by `UnifiedMarket.id` / `openpx_id`). |
| `market_ref` | `string | null` | No | Optional alternate market identifier for trade endpoints (e.g., Polymarket conditionId). When provided, exchanges should prefer this over `market_id`. |
| `outcome` | `string | null` | No |  |
| `start_ts` | `number | null` | No | Unix seconds (inclusive) |
| `token_id` | `string | null` | No |  |

<details>
<summary>Rust / Python / TypeScript definitions</summary>

**Rust**
```rust
pub struct TradesRequest {
    pub cursor: Option<String>,
    pub end_ts: Option<i64>,
    pub limit: Option<i64>,
    pub market_id: String,
    pub market_ref: Option<String>,
    pub outcome: Option<String>,
    pub start_ts: Option<i64>,
    pub token_id: Option<String>,
}
```

**Python**
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

**TypeScript**
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

</details>

---
