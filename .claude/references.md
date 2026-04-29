# OpenPX Upstream References

Single source of truth for every upstream doc URL the OpenPX codebase tracks.
`CLAUDE.md` and `.claude/commands/unify.md` both read from this file — update here, not in either of those.

Tier conventions:
- **Tier 1** — machine-readable specs, changelog, discovery index. Always fetch first when a keyword could touch them.
- **Tier 2** — operational prose pages. Fetch when the keyword maps to that area.

---

## Kalshi

Single base: `https://docs.kalshi.com`. One REST surface, one WS surface.

### Tier 1 — machine-readable specs
- REST: https://docs.kalshi.com/openapi.yaml
- WebSocket: https://docs.kalshi.com/asyncapi.yaml

### Tier 1 — changelog and discovery
- Changelog: https://docs.kalshi.com/changelog
- Discovery index: https://docs.kalshi.com/llms.txt
- Full discovery: https://docs.kalshi.com/llms-full.txt

### Tier 2 — operational pages
- https://docs.kalshi.com/getting_started/rate_limits.md
- https://docs.kalshi.com/getting_started/market_lifecycle.md
- https://docs.kalshi.com/getting_started/orderbook_responses.md
- https://docs.kalshi.com/getting_started/fee_rounding.md
- https://docs.kalshi.com/getting_started/pagination.md
- https://docs.kalshi.com/getting_started/fixed_point_migration.md
- https://docs.kalshi.com/getting_started/historical_data.md
- https://docs.kalshi.com/getting_started/targets_and_milestones.md
- https://docs.kalshi.com/getting_started/rfqs.md
- https://docs.kalshi.com/getting_started/terms.md

---

## Polymarket

The API is split across 4 services with separate base URLs. Specs are NOT at the conventional `/openapi.yaml` root — they live under `/api-spec/`.

### Service map
- **Gamma API** (`https://gamma-api.polymarket.com`) — markets/events/tags, no auth
- **Data API** (`https://data-api.polymarket.com`) — positions/trades/leaderboards, no auth
- **CLOB API** (`https://clob.polymarket.com`) — orderbook + trading, L1/L2 auth
- **Bridge API** (`https://bridge.polymarket.com`) — deposits/withdrawals

### Tier 1 — machine-readable specs
- CLOB: https://docs.polymarket.com/api-spec/clob-openapi.yaml
- Data: https://docs.polymarket.com/api-spec/data-openapi.yaml
- Gamma: https://docs.polymarket.com/api-spec/gamma-openapi.yaml
- Bridge: https://docs.polymarket.com/api-spec/bridge-openapi.yaml
- Relayer: https://docs.polymarket.com/api-spec/relayer-openapi.yaml
- Market WS: https://docs.polymarket.com/asyncapi.json
- User WS: https://docs.polymarket.com/asyncapi-user.json
- Sports WS (only if scope expands): https://docs.polymarket.com/asyncapi-sports.json

### Tier 1 — changelog, discovery, on-chain
- Changelog: https://docs.polymarket.com/changelog.md
- Discovery index: https://docs.polymarket.com/llms.txt
- Full discovery: https://docs.polymarket.com/llms-full.txt
- On-chain contracts (silent breakage if redeployed): https://docs.polymarket.com/resources/contracts.md

### Tier 2 — operational pages
- https://docs.polymarket.com/api-reference/rate-limits.md
- https://docs.polymarket.com/api-reference/authentication.md
- https://docs.polymarket.com/concepts/markets-events.md
- https://docs.polymarket.com/concepts/order-lifecycle.md
- https://docs.polymarket.com/concepts/prices-orderbook.md
- https://docs.polymarket.com/concepts/positions-tokens.md
- https://docs.polymarket.com/concepts/resolution.md
- https://docs.polymarket.com/concepts/pusd.md
- https://docs.polymarket.com/trading/fees.md
- https://docs.polymarket.com/trading/matching-engine.md
- https://docs.polymarket.com/trading/gasless.md
- https://docs.polymarket.com/trading/orderbook.md
- https://docs.polymarket.com/trading/orders/create.md
- https://docs.polymarket.com/trading/orders/cancel.md
- https://docs.polymarket.com/trading/clients/l1.md
- https://docs.polymarket.com/trading/clients/l2.md
- https://docs.polymarket.com/resources/error-codes.md
- https://docs.polymarket.com/resources/blockchain-data.md
- https://docs.polymarket.com/market-data/websocket/overview.md
- https://docs.polymarket.com/market-data/websocket/market-channel.md
- https://docs.polymarket.com/market-data/websocket/user-channel.md
- https://docs.polymarket.com/market-data/websocket/rtds.md
- https://docs.polymarket.com/advanced/neg-risk.md
