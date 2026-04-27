# Kalshi × Polymarket overlap analysis — 2026-04-27

Sources: Kalshi `openapi.yaml` (78 paths) + `asyncapi.yaml` (12 channels);
Polymarket CLOB `clob-openapi.yaml`, Gamma `gamma-openapi.yaml`, Data
`data-openapi.yaml`, `asyncapi.json` (market WS) + `asyncapi-user.json` (user WS).

## 1. Strong overlap candidates (recommended new trait methods)

Sorted by UX win.

| Proposed method | Rationale | Kalshi | Polymarket | Return shape sketch |
|---|---|---|---|---|
| `fetch_events(params)` | Both venues group markets into "events". Today users have to drop down per-venue. High UX win for browsers, dashboards, search. | `GET /events`, `GET /events/{event_ticker}` | `GET /events` (Gamma), `GET /events/{id}`, `GET /events/slug/{slug}` | `(Vec<Event>, Option<String>)` — Event { id, slug, title, category, market_ids, start_ts, end_ts, … } |
| `fetch_event(id_or_slug)` | Single-event detail. Pairs with `fetch_events`. | `GET /events/{event_ticker}` | `GET /events/{id}` + `GET /events/slug/{slug}` | `Event` |
| `fetch_orderbooks_batch(ids)` | Both venues expose a multi-market book endpoint — currently the unified API forces N round-trips. Big latency win for HFT/MM users (the target audience). | `GET /markets/orderbooks` (multi-ticker) | `GET /books` / `POST /books` | `Vec<Orderbook>` keyed by id |
| `fetch_series(params)` | Both have a "series" concept (a recurring event family). Useful for sports + recurring politics. | `GET /series/{series_ticker}/...`, `GET /series/fee_changes` | `GET /series` (Gamma), `GET /series/{id}`, `GET /series-summary/{id}` | `(Vec<Series>, Option<String>)` |
| `fetch_series_one(id)` | Detail counterpart. | implicit via `/series/{series_ticker}/events/...` | `GET /series/{id}` | `Series` |
| `fetch_midpoint(req)` | Both expose a midpoint price. Cheaper than fetching a full orderbook when all you need is mark price (common in PnL calc). | not direct — derive from `/markets/{ticker}/orderbook` | `GET /midpoint` | `f64` |
| `fetch_midpoints_batch(ids)` | Batch counterpart; Polymarket has it natively. Kalshi falls back to N orderbook fetches or batch orderbook + derive. | derive from `/markets/orderbooks` | `GET /midpoints` (or POST) | `HashMap<String, f64>` |
| `fetch_spread(req)` | Both expose top-of-book; Polymarket has dedicated endpoint. Kalshi: derive from orderbook. Useful for liquidity scoring. | derive from `/markets/{ticker}/orderbook` | `GET /spread` | `f64` |
| `fetch_last_trade_price(req)` | "What just printed?" — common UI need. Kalshi: head of `/markets/trades` filtered. Polymarket: dedicated. | derive from `GET /markets/trades` (limit=1) | `GET /last-trade-price` | `{ price: f64, side: OrderSide, ts: i64 }` |
| `fetch_open_interest(market_id)` | Both publish OI; today users either parse from `Market` (sometimes) or call raw. Pulling it out as its own method is cleaner and matches CCXT. | OI is on `/markets/{ticker}` payload | `GET /oi` (Data API) | `f64` |
| `fetch_user_trades(params)` | Distinct from `fetch_fills` (which is venue-side execution rows): this is the user-facing trade history with PnL fields. Both venues expose it cleanly. | `GET /portfolio/fills` + `GET /historical/fills` | `GET /trades` (Data API) | `(Vec<Fill>, Option<String>)` — but extend `Fill` with realized_pnl, fee, role |
| `fetch_market_tags(market_id)` | Tag/category metadata is exposed on both. Useful for filtering & search UIs. | `GET /search/tags_by_categories` (global), tags on event payload | `GET /markets/{id}/tags`, `GET /events/{id}/tags` | `Vec<Tag>` |
| `cancel_all_orders(market_id?)` | Both have explicit "cancel everything" / "cancel for market" verbs. Today users bulk-cancel by calling `cancel_order` per id — way slower and racy. | `DELETE /portfolio/orders/batched` | `DELETE /cancel-all`, `DELETE /cancel-market-orders` | `Vec<Order>` (cancelled set) |
| `create_orders_batch(orders)` | Both expose batch create. Round-trip win for MM. | `POST /portfolio/orders/batched` | `POST /orders` (max 15) | `Vec<Order>` |

## 2. Weak / partial overlap (defer or add as opt-in)

| Proposed method | Why weak | Notes |
|---|---|---|
| `fetch_tick_size(req)` | Polymarket has it as a first-class endpoint; Kalshi tick is implicit (cents) and exposed only via market metadata. | Surfacing it is a one-liner per venue but the value is low — most users hardcode. |
| `fetch_fee_rate(req)` | Polymarket: `GET /fee-rate`. Kalshi: `GET /series/fee_changes` is a *change log*, not a current rate. Shapes don't unify cleanly. | Better to surface fees on the `Market` model. |
| `search_markets(q)` | Polymarket: `GET /public-search`. Kalshi: `GET /search/tags_by_categories` is tag-driven, not text search. | Mismatch — defer. |
| `fetch_holders(market_id)` | Polymarket: `GET /holders` (top holders, on-chain). Kalshi: no public equivalent (off-chain CEX). | Polymarket-only; section 3. |
| `fetch_leaderboard()` | Polymarket: `GET /v1/leaderboard`. Kalshi: no equivalent. | Polymarket-only; section 3. |
| `fetch_announcements()` | Kalshi: `GET /exchange/announcements`. Polymarket: no equivalent. | Kalshi-only; section 3. |
| `fetch_status()` / `fetch_exchange_status()` | Kalshi: `GET /exchange/status`. Polymarket: no canonical health endpoint (Data API root `GET /` is undocumented health). | Weak — leave out. |
| `fetch_settlements()` | Kalshi: `GET /portfolio/settlements`. Polymarket: settlement is on-chain redemption (CTF redeem); shape is incompatible. | Skip until both sides have a unified `Settlement` model. |
| `amend_order` / `decrease_order` | Kalshi: `POST /portfolio/orders/{id}/amend` + `/decrease`. Polymarket: no amend (cancel+replace). | Kalshi-only; section 3. |

## 3. Single-venue methods worth adding as not-supported defaults

These are valuable enough to put on the trait with `Err(NotSupported)` defaults, so the unified API exposes them and users can detect via `describe()`.

| Method | Venue | Endpoint(s) | Why surface it |
|---|---|---|---|
| `fetch_server_time` *(already scaffolded)* | Polymarket, *not* Kalshi | Polymarket: `GET /time` (CLOB). Kalshi: no `/time` — `GET /exchange/user_data_timestamp` is approximate replication lag, not wall-clock. | Confirms current scaffolding is right: implement Polymarket; leave Kalshi at default Err. |
| `amend_order(order_id, price, size)` | Kalshi-only | `POST /portfolio/orders/{id}/amend` | HFT users care; Polymarket users get `NotSupported` and fall back to cancel+replace. |
| `fetch_order_queue_position(order_id)` | Kalshi-only | `GET /portfolio/orders/{id}/queue_position`, `GET /portfolio/orders/queue_positions` | MM-relevant; Polymarket's matching engine doesn't expose queue depth. |
| `fetch_holders(market_id)` | Polymarket-only | `GET /holders` (Data API) | On-chain transparency unique to Polymarket. |
| `fetch_leaderboard(params)` | Polymarket-only | `GET /v1/leaderboard` | Public ranking — useful for social/copy-trade UIs. |
| `fetch_announcements()` | Kalshi-only | `GET /exchange/announcements` | Operational notices. |
| `fetch_milestones(params)` / `fetch_live_data(milestone_id)` | Kalshi-only | `GET /milestones`, `GET /live_data/...` | Sports live-event hooks; no Polymarket parallel. |
| `fetch_rewards(params)` / `fetch_user_earnings(start, end)` | Polymarket-only | `GET /rewards/*` family | Liquidity reward program; Kalshi has incentive_programs (different shape). |
| `fetch_incentive_programs()` | Kalshi-only | `GET /incentive_programs` | Mirror of rewards; could co-evolve into a unified `fetch_incentives()` later — flag for re-review. |

## 4. WebSocket channel overlap

| Proposed unified channel | Kalshi channel | Polymarket message | Notes |
|---|---|---|---|
| `orderbook` (snapshot + delta) | `orderbook_delta` (auth required) | `book` (snapshot) + `price_change` (delta) | Direct overlap. Strong. Kalshi sends snapshot then deltas in same channel; Polymarket splits the two event_types. Unify under one stream of `OrderbookEvent::Snapshot \| Delta`. |
| `trades` (public tape) | `trade` | `last_trade_price` | Direct overlap. |
| `ticker` (top-of-book / mark) | `ticker` (with `send_initial_snapshot`) | `best_bid_ask` (gated by `custom_feature_enabled`) | Both exist. Polymarket version is opt-in; surface that via subscribe params. |
| `user_orders` (private) | `user_orders` (auth) | user WS `receiveOrder` (PLACEMENT/UPDATE/CANCELLATION) | Direct overlap. |
| `user_fills` (private) | `fill` (auth) | user WS `receiveTrade` (MATCHED/MINED/CONFIRMED…) | Direct overlap. Polymarket emits multiple lifecycle states per trade — collapse or surface. |
| `market_lifecycle` | `market_lifecycle_v2` | `new_market` + `market_resolved` (gated by `custom_feature_enabled`) | Overlap exists; Polymarket's are gated and split. Unify behind one stream. |
| `tick_size_change` | (none — Kalshi tick is constant) | `tick_size_change` | Polymarket-only; surface as venue-extension. |
| `positions` | `market_positions` (auth) | (none — derive from order/trade events) | Kalshi-only. Could synthesize on Polymarket side from trade events, but defer. |

### WS subscribe semantics (for the Connection API surface)

Both venues support multi-market subscriptions in one frame:
- Kalshi: `market_ticker(s)` array on subscribe + `sendUpdateSubscription` / `sendUpdateSubscriptionDelete` to add/remove without reconnect.
- Polymarket: `assets_ids` array on initial subscribe + `subscriptionRequestUpdate` with `subscribe` / `unsubscribe` operation.

Unify: a `subscribe(channel, market_ids)` + `update_subscription(sid, add, remove)` API on the WS trait.

## 5. Already-covered (sanity check — no work needed)

`fetch_markets`, `fetch_market`, `create_order`, `cancel_order`, `fetch_order`,
`fetch_open_orders`, `fetch_positions`, `fetch_balance`, `refresh_balance`,
`fetch_balance_raw`, `fetch_orderbook` (single), `fetch_price_history`
(candles), `fetch_trades` (public tape), `fetch_orderbook_history`,
`fetch_user_activity`, `fetch_fills`, `fetch_server_time` (scaffolded — needs
Polymarket impl).

## 6. Open questions / things to verify before scaffolding

1. **Kalshi server time** — `GET /exchange/user_data_timestamp` is *approximate data validation timestamp*, not a clock. Confirm with Kalshi support or raw probe whether any header (e.g. `Date` on `/exchange/status`) is the canonical wall-clock for skew correction. Until then, leave Kalshi `fetch_server_time` returning `NotSupported` as currently scaffolded.
2. **`Series` model shape** — Kalshi `series_ticker` is a code (e.g. `KXPRES`), Polymarket `series` has its own id/slug + summary endpoints. Unifying means picking which fields are required vs optional. Needs a model spike before scaffolding.
3. **`Event` model shape** — overlap is high-level but field-level mapping (start/end timestamps, status enum, market_ids vs condition_ids) needs the same model spike.
4. **`fetch_orderbooks_batch` semantics on Kalshi** — verify `/markets/orderbooks` accepts an arbitrary tickers list (vs paginated) and whether response is keyed map or ordered array. Probe before scaffolding the request type.
5. **`fetch_user_trades` vs `fetch_fills`** — Kalshi `/portfolio/fills` returns fills (one row per match leg). Polymarket Data `/trades` returns trades (aggregated). The current `Fill` model may not fit Polymarket trades cleanly — propose a separate `UserTrade` model or extend `Fill` with optional `realized_pnl`, `role`, `tx_hash`.
6. **WS auth shape** — Kalshi private channels need API-key signed auth on the WS handshake; Polymarket user WS expects `auth { key, secret, passphrase }` in the subscribe frame. Unifying requires a `WebSocketAuth` enum and a per-venue connector. Already partially exists in `engine/core/src/websocket/` — verify before adding `subscribe_user_orders`.
7. **`cancel_market_orders` (Polymarket) vs Kalshi** — Kalshi's `DELETE /portfolio/orders/batched` cancels by id list, not by market. To unify `cancel_all_orders(market_id: Some(...))` on Kalshi, we'd have to first list open orders and then batch-cancel — confirm latency budget is acceptable, or document this as a 2-call fallback.
8. **OI on Kalshi** — verified to live on `/markets/{ticker}` payload, but field name needs to be confirmed against the manifest before scaffolding `fetch_open_interest` (would be a derived/cached path on Kalshi vs a dedicated endpoint on Polymarket).
9. **Ticker/best_bid_ask gating** — Polymarket's `best_bid_ask` requires `custom_feature_enabled: true` on subscribe. If we expose `subscribe_ticker` unified, default to enabling it on Polymarket; document that this may be subject to permissioning.
