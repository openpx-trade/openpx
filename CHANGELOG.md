# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0](https://github.com/openpx-trade/openpx/compare/v0.2.6...v0.3.0) (2026-05-03)


### ⚠ BREAKING CHANGES

* **api,mappings:** tighten doc surface — one-sentence params + 3-type mapping vocabulary
* **api:** order endpoints take asset_id, not market_ticker
* **api:** create_order takes asset_id, not market_ticker
* **api:** typed create_orders_batch — drop NewOrder, Kalshi V2 batch
* **kalshi:** cancel_order + cancel_all_orders on V2 endpoints
* **api:** lean create_order — typed request, Kalshi V2 endpoint
* **api:** lean fetch_trades — 5-field request, 10-field response
* **api:** replace pricing endpoints with orderbook insights
* **api:** drop fetch_market_tags, unify orderbook on asset_id
* **api:** remove 5 unused endpoints + dead code
* **api:** unify fetch_markets, add fetch_market_lineage, rename market_id → market_ticker
* **market:** reshape unified Market for cross-exchange clarity

### Features

* **api:** expose authenticated trading surface across CLI + Python + TS SDKs ([e70ba56](https://github.com/openpx-trade/openpx/commit/e70ba5637f5cbfe1f576263d83ee0f94b1e6af20))
* bootstrap autonomous-maintenance system ([#12](https://github.com/openpx-trade/openpx/issues/12)) ([a55cbb9](https://github.com/openpx-trade/openpx/commit/a55cbb9e29b78db4ef5aea3a480cc44f3f720e91))
* **core:** add fetch_server_time to Exchange trait (closes [#17](https://github.com/openpx-trade/openpx/issues/17)) ([ff3db65](https://github.com/openpx-trade/openpx/commit/ff3db65bd1884a5097e55bc6f5a69923179fc317))
* **core:** scaffold 14 unified Exchange-trait methods ([b59fcf1](https://github.com/openpx-trade/openpx/commit/b59fcf1ac3cdfc3955c40c3ee2ccc7fbe177b926))
* **docs:** add llms.txt generation pipeline ([1a1dcab](https://github.com/openpx-trade/openpx/commit/1a1dcab4b592e8fd764a04e0378676eec5109eb7))
* **docs:** add llms.txt generation pipeline ([a5fabb4](https://github.com/openpx-trade/openpx/commit/a5fabb477423287f4a94cd77a194a8d3befee04b))
* **docs:** auto-generate API reference from Rust trait + JSON Schema ([a6eb0bf](https://github.com/openpx-trade/openpx/commit/a6eb0bfd60de7b94857eedae9be303daa615efc7))
* **docs:** auto-generate WebSocket reference from JSON Schema via AsyncAPI ([f7b2e82](https://github.com/openpx-trade/openpx/commit/f7b2e82cc2134455c7ea1bb5c8ca62992776ad01))
* **exchanges:** batch order ops on both venues — cancel_all + create_orders_batch (Batch 5) ([5b20f72](https://github.com/openpx-trade/openpx/commit/5b20f724a5b159362d8f262fc3554b8555dcdd6e))
* **exchanges:** batch order ops on both venues — cancel_all + create_orders_batch (Batch 5) ([1ee8dd5](https://github.com/openpx-trade/openpx/commit/1ee8dd508ba4682d96b571443148eddde5ba0849))
* **exchanges:** implement events / series / tags methods on both venues ([bed47f2](https://github.com/openpx-trade/openpx/commit/bed47f2060c087ba53f27b3d2ef99d73c7109937))
* **exchanges:** implement events / series / tags methods on both venues (Batch 2) ([28419b0](https://github.com/openpx-trade/openpx/commit/28419b09cf021f610d3066ffb606042cca3a40ea))
* **exchanges:** implement fetch_user_trades on both venues (Batch 4) ([8f52922](https://github.com/openpx-trade/openpx/commit/8f52922673dc8ddf87ca59554599668dc56b9570))
* **exchanges:** implement fetch_user_trades on both venues (Batch 4) ([ac69aab](https://github.com/openpx-trade/openpx/commit/ac69aab0e4600e4cbfc81ed6938cdf3ac3c26880))
* **exchanges:** implement pricing/books methods on both venues (Batch 3) ([0bf1bce](https://github.com/openpx-trade/openpx/commit/0bf1bce1fd31d904a6d821444624b000c2f17a03))
* **exchanges:** implement pricing/books methods on both venues (Batch 3) ([083515f](https://github.com/openpx-trade/openpx/commit/083515fd3a927b96a80f728ab9dbae739f65aa06))
* **polymarket:** cf_clearance cookie-replay example for CLOB key bootstrap ([e3e0d03](https://github.com/openpx-trade/openpx/commit/e3e0d03e44a42a24a2da74f072babe4a0b8573b5))
* **polymarket:** per-failure auth diagnostics + funder/eoa auto-correct ([08e9e74](https://github.com/openpx-trade/openpx/commit/08e9e74978b0eb8426aefaab26f563ba2c031b77))
* **sports:** scaffold SportsProvider trait + unified schema (PR 1/N) ([7d2996d](https://github.com/openpx-trade/openpx/commit/7d2996d2c73264cf39f17973f8beb9517f37b022))
* **sports:** scaffold SportsProvider trait + unified schema (PR 1/N) ([2f2e5ca](https://github.com/openpx-trade/openpx/commit/2f2e5ca2da7c4eed8211abcd7483663792c74463))
* unified schema-mapping system + upstream drift pipeline ([b694841](https://github.com/openpx-trade/openpx/commit/b69484100fb202c3576d7d7a4d399a8561d50e1d))


### Bug Fixes

* **agents:** bot watches its own PRs through CI green, not just open ([bad177c](https://github.com/openpx-trade/openpx/commit/bad177c39197d6dcdc44b3cdde77389a532a6018))
* **agents:** stop issue duplication, require PR provenance, self-assign bot issues ([fa62f4e](https://github.com/openpx-trade/openpx/commit/fa62f4e83bbcb293c87d18b1f72df82bbdb9997a))
* **ci+agent:** correct python smoke check + give agent-tick the preflight tooling ([d45161a](https://github.com/openpx-trade/openpx/commit/d45161a95b637b820b270ba6ae60b2a5bade1118))
* **ci:** bypass per-tool permission prompts for orchestrator ([#15](https://github.com/openpx-trade/openpx/issues/15)) ([222acb3](https://github.com/openpx-trade/openpx/commit/222acb3a48c97917677442aae08ec9f39dbc0d03))
* **ci:** create real venv for Python SDK Build; skipLibCheck on TS smoke ([ea40f86](https://github.com/openpx-trade/openpx/commit/ea40f8692d062d9498b95785f385fd615b425586))
* **ci:** drop ci-success aggregator (was forever-pending) ([0a5775b](https://github.com/openpx-trade/openpx/commit/0a5775bb5e0764e6b9c2d9768185c69fe7f66147))
* **ci:** give claude-code-action a prompt instead of a fake `agent` input ([#14](https://github.com/openpx-trade/openpx/issues/14)) ([e5f27b2](https://github.com/openpx-trade/openpx/commit/e5f27b282a91bd49d68538a6626edcc4ff385e2c))
* **ci:** pin extractions/setup-just to 1.50.0 ([f44b45f](https://github.com/openpx-trade/openpx/commit/f44b45fe6e433455bdebdb3623bd2633195e18fa))
* **ci:** switch from extractions/setup-just to taiki-e/install-action ([a3a3674](https://github.com/openpx-trade/openpx/commit/a3a36743d581764ba4aa8ecd08a4e18e2ceff084))
* **ci:** typescript devDep + venv-path bug in justfile python-build ([3583ad2](https://github.com/openpx-trade/openpx/commit/3583ad2a98f6c806532a2d13bdac8218b9b216b4))
* **docs:** drop leading slash from asyncapi path ([5b010d1](https://github.com/openpx-trade/openpx/commit/5b010d1ac1ff30c2e19678c42c7cba3dccbd02d4))
* **docs:** drop Mintlify github component, use FA brand icon ([b61c872](https://github.com/openpx-trade/openpx/commit/b61c8729551eb4e9eed8b2de46318e48395a5cf8))
* **docs:** move asyncapi spec to docs/ root + use JSON + leading-slash path ([840da48](https://github.com/openpx-trade/openpx/commit/840da480ae2effb3d900ec7722a20fb3aa7498f1))
* **docs:** nest asyncapi auto-population inside a group ([aa4816c](https://github.com/openpx-trade/openpx/commit/aa4816c246ee86cfa59c033fde31d1466e2ea8ed))
* **docs:** render WebSocket stream via Mintlify asyncapi: frontmatter ([115aec0](https://github.com/openpx-trade/openpx/commit/115aec056b21aa8dcffaa9ad7fa418e10e98aa1c))
* **docs:** split WebSocket reference into 5 per-channel pages + co-locate spec ([4263e1f](https://github.com/openpx-trade/openpx/commit/4263e1f4ab9db58e82755ad8735da980cac45952))
* **docs:** unblock Mintlify renderer — leading-slash openapi path + asyncapi examples ([98aafab](https://github.com/openpx-trade/openpx/commit/98aafab395ceae953b4459281ed45fab56fc3759))
* **docs:** use 4-backtick fence to trigger Mintlify AsyncAPI rendering ([518de87](https://github.com/openpx-trade/openpx/commit/518de871bcd89c83a6a454183312bb275750551c))
* **docs:** use Mintlify asyncapi auto-population — drop per-channel MDX ([7b7282d](https://github.com/openpx-trade/openpx/commit/7b7282d7b47925ccd4eeecd1d3c6c1a1dcdcbdae))
* **kalshi:** use into_values() to satisfy clippy iter_kv_map (Batch 3) ([b1e22e2](https://github.com/openpx-trade/openpx/commit/b1e22e2f8eebc89bff59614c53ca84fb1e184a4d))
* **policy:** branch-protection contexts use bare check_run.name (no prefix, no suffix) ([cc12659](https://github.com/openpx-trade/openpx/commit/cc12659b4977395c5845ac0e53adeb2d5a3ca3a2))
* **polymarket:** check_credentials uses canonical host + standard env names ([ac13026](https://github.com/openpx-trade/openpx/commit/ac13026901aecc29c8575acb2428745e8cc4cb30))
* **polymarket:** dedupe fetch_markets across active+closed buckets ([e888d47](https://github.com/openpx-trade/openpx/commit/e888d475180214cd7e58c99936c4e03f55028700))
* **sdk-sync:** emit all schema $defs in TypeScript models.d.ts ([3f3b3b5](https://github.com/openpx-trade/openpx/commit/3f3b3b53c4370575657b295930dd13747c53dc02))


### Reverts

* "feat(sports): scaffold SportsProvider trait + unified schema (PR 1/N)" ([28456c0](https://github.com/openpx-trade/openpx/commit/28456c08e1f30e78062d0e71a5839eb82c3600f4))


### Documentation

* **api,mappings:** tighten doc surface — one-sentence params + 3-type mapping vocabulary ([88c206b](https://github.com/openpx-trade/openpx/commit/88c206bf9a2eb2fdaaf6b5ca560350d456f106f2))


### Code Refactoring

* **api:** create_order takes asset_id, not market_ticker ([e130eee](https://github.com/openpx-trade/openpx/commit/e130eeeb4cf7fea51c270ec3074ac5f89c6fb3af))
* **api:** drop fetch_market_tags, unify orderbook on asset_id ([5f5b438](https://github.com/openpx-trade/openpx/commit/5f5b438d14a2f4b65ce3838c5d01580dafdfca3c))
* **api:** lean create_order — typed request, Kalshi V2 endpoint ([7d9f548](https://github.com/openpx-trade/openpx/commit/7d9f548fabda416ba5e0dbf32f1bfeb2044ed983))
* **api:** lean fetch_trades — 5-field request, 10-field response ([d5675e7](https://github.com/openpx-trade/openpx/commit/d5675e714e69e5bc3bf834389ff8508294882a65))
* **api:** order endpoints take asset_id, not market_ticker ([ca7e835](https://github.com/openpx-trade/openpx/commit/ca7e835525995710ebd12b3693f00ac43fab6398))
* **api:** remove 5 unused endpoints + dead code ([685d149](https://github.com/openpx-trade/openpx/commit/685d14963b918f2ac36752137f4b1384e832e341))
* **api:** replace pricing endpoints with orderbook insights ([fd54ad1](https://github.com/openpx-trade/openpx/commit/fd54ad1914d535bb756239cec3b7d824e6e96185))
* **api:** typed create_orders_batch — drop NewOrder, Kalshi V2 batch ([0487055](https://github.com/openpx-trade/openpx/commit/04870557c88ce6b2dd82f79bce66e16cd4616900))
* **api:** unify fetch_markets, add fetch_market_lineage, rename market_id → market_ticker ([72383cc](https://github.com/openpx-trade/openpx/commit/72383cc4091fec96435cfd4301e585b7f2d0b077))
* **kalshi:** cancel_order + cancel_all_orders on V2 endpoints ([5fae811](https://github.com/openpx-trade/openpx/commit/5fae81197a3abedc51ce206bfb8b180bf75e3dc0))
* **market:** reshape unified Market for cross-exchange clarity ([1e56280](https://github.com/openpx-trade/openpx/commit/1e5628086e1df0c6fbd6da97aec12fb92d926556))

## [0.2.6](https://github.com/openpx-trade/openpx/compare/v0.2.5...v0.2.6) (2026-04-24)


### ⚠ BREAKING CHANGES

* **polymarket:** Polymarket `Market.id` now returns the condition_id hex string (e.g. `"0x311d0c4b..."`) rather than the REST numeric id (e.g. `"1031769"`). Callers storing `Market.id` to pass back to Polymarket REST are unaffected — both `?id=` and `?condition_id=` accept this value. Callers needing the numeric id should read `Market.native_numeric_id` instead.

### Features

* **polymarket:** use condition_id as Market.id; add native_numeric_id ([704c560](https://github.com/openpx-trade/openpx/commit/704c5606564503ee6ab2f2f158c12c46d06aa21d))

## [0.2.5](https://github.com/openpx-trade/openpx/compare/v0.2.3...v0.2.5) (2026-04-23)


### ⚠ BREAKING CHANGES

* **ws:** Polymarket WsUpdate::Snapshot.market_id and Delta.market_id now carry the parent condition ID (from the asset_to_market map), not the CLOB token. Consumers that were keying by the outer market_id expecting a token should key by the new asset_id field. Kalshi and Opinion behavior is unchanged.
* **kalshi:** cursor format changed. Old 0.2.3 cursor strings are not recognized; callers must restart pagination after upgrading.

### Features

* **ws:** asset_id on Delta, Clear variant, reconnect parity, Opinion snapshot ([3d4eaf0](https://github.com/openpx-trade/openpx/commit/3d4eaf07dd455a580cc7c54063b1b497603842b2))


### Bug Fixes

* **kalshi:** rewrite fetch_markets using /markets + /historical/markets ([267a74c](https://github.com/openpx-trade/openpx/commit/267a74c27b2d56e4827e28ff742c2306a7ccdb70))

## [0.2.3](https://github.com/openpx-trade/openpx/compare/v0.2.2...v0.2.3) (2026-04-23)


### Bug Fixes

* **npm:** remove prepublishOnly hook that double-runs napi prepublish ([19a4dc7](https://github.com/openpx-trade/openpx/commit/19a4dc71fc7e97b4c825d756df6765c2926ee614))

## [0.2.2](https://github.com/openpx-trade/openpx/compare/v0.2.1...v0.2.2) (2026-04-22)


### Bug Fixes

* **release:** re-publish to clear npm name-reservation hold on 0.2.1 ([19b3dad](https://github.com/openpx-trade/openpx/commit/19b3dad33a8692b3c254ae7fd766e8326e363788))

## [0.2.1](https://github.com/openpx-trade/openpx/compare/v0.2.0...v0.2.1) (2026-04-22)


### Features

* **bench:** add openpx-bench-ws — WebSocket first-book latency comparison ([45b5ca2](https://github.com/openpx-trade/openpx/commit/45b5ca20611edf53bc254121af49677aaf5db4a3))


### Performance Improvements

* **core,exchanges:** unify HTTP + WS decode tunings across every exchange ([0ff2593](https://github.com/openpx-trade/openpx/commit/0ff25930fc835bb2c39dae1c56f7eb738fc12230))
* **core,exchanges:** wire simd-json into WS decode on all exchanges ([adac103](https://github.com/openpx-trade/openpx/commit/adac103ef463b14ebcdbc990d9acdbe0344b233d))

## [0.2.0] - 2026-04-22

### BREAKING

- **WebSocket surface rewritten.** The 0.1 per-token `orderbook_stream(token_id)` / `activity_stream(token_id)` methods are gone. Consumers now call `ws.updates()` for a single multiplexed `WsUpdate` stream (Snapshot/Delta/Trade/Fill, tagged by `kind`) and `ws.session_events()` for connection-level `SessionEvent`s (Connected/Reconnected/Lagged/BookInvalidated/Error). This is the entire reason for the minor bump.
- **`updates()` and `session_events()` are take-once.** Both return `Option<Stream>` at the Rust layer and raise on the second call from Python/TypeScript. The underlying channel is single-consumer by contract — cloning a receiver would silently split messages between holders (a debug sidecar would quietly eat half the stream). Run one consumer that re-dispatches if you need fan-out.
- **`WsUpdate::Raw` removed.** The open escape-hatch variant was never produced and would have forced every future consumer `match` to either ignore an unstable payload or break when we eventually normalized it. If an exchange grows a payload we want to surface raw, it will land as a separate `raw_events()` stream rather than another `WsUpdate` variant.
- **`UpdateStream::into_stream()` / `SessionStream::into_stream()` removed.** They leaked the `async_channel::Receiver` type through the public API, locking us into that crate version forever. Consumers should use `.next()` / `.try_next()` / `.len()` on the stream types directly.
- **WS surface timestamps unified to `u64` millis since epoch.** `ActivityTrade.timestamp` and `ActivityFill.timestamp` (previously `Option<DateTime<Utc>>`) are now `exchange_ts_ms: Option<u64>`, matching `WsUpdate::{Snapshot, Delta}::exchange_ts`. Every timestamp on the WS surface is the same type, no mixed `DateTime` / `u64` representations at the FFI boundary.
- **Python `WsUpdate` / `SessionEvent` are now real classes, not dicts.** `Snapshot`, `Delta`, `Trade`, `Fill`, `Connected`, `Reconnected`, `Lagged`, `BookInvalidated`, `SessionError` are importable from `openpx` and support structural `match` with `__match_args__` plus `isinstance` dispatch. Consumers no longer reach for `update["kind"]`. Nested payloads (`book`, `changes`, `trade`, `fill`) remain dict-shaped for now — a full `Orderbook` / `PriceLevelChange` pyclass surface is a separate cut.
- **`tokio::sync::broadcast` replaced with `async-channel` + explicit lag signaling.** Under slow consumers, 0.1 silently skipped ahead and left caller books quietly corrupt; 0.2 emits `SessionEvent::Lagged` + `SessionEvent::BookInvalidated { reason: Lag }` per affected market so callers can discard and wait for the next `Snapshot`.
- **`InvalidationReason::SequenceGap { expected, received }`** surfaced from the engine — per-market gap detection no longer has to live in caller code.
- **`WebSocketState`** now has a stable `Display` / `.as_str()` with explicit labels (`"Connected"` etc.). Bindings moved off `format!("{state:?}")` so future `Debug` formatting changes can't silently break `state == "Connected"` checks.

### Fixed

- `KalshiConfig::demo()` no longer drops `api_url` / `private_key_path` fields when `demo: true` was combined with other overrides — the demo branch is now a base selection, not a rebuild.
- `Polymarket` WebSocket constructor now plumbs `private_key` / `funder` / `signature_type` through the shared parser. Previously only REST went through the full parser while WS silently ignored those fields.
- `ExchangeInner::new` and `WebSocketInner::new` now share one config parser (`engine/sdk/src/config.rs`) so new fields plumb to both call sites at once.

## [0.1.7](https://github.com/openpx-trade/openpx/compare/v0.1.6...v0.1.7) (2026-04-18)


### Bug Fixes

* clippy 1.95 unnecessary_sort_by in remaining exchange impls ([ff4c1fe](https://github.com/openpx-trade/openpx/commit/ff4c1fe78d6be07a33e748b881c6c9e878f20f5f))
* make python codegen reproducible by disabling timestamp header ([3e02273](https://github.com/openpx-trade/openpx/commit/3e02273932c3ee7a6dae4a6fe6b8da8e5f6f1b6a))
* **python:** install rustls ring crypto provider for websocket TLS ([c5f889b](https://github.com/openpx-trade/openpx/commit/c5f889b3c27acd3347a0ee0d86efe1b2feb4660d))
* satisfy clippy 1.95 + regenerate python models ([8211787](https://github.com/openpx-trade/openpx/commit/821178728bf729c504b0da2cea5fde54793cc881))

## [0.1.6](https://github.com/openpx-trade/openpx/compare/v0.1.5...v0.1.6) (2026-03-26)


### Features

* add MarketStatusFilter::All to fetch markets of any status ([02c97b2](https://github.com/openpx-trade/openpx/commit/02c97b24a14cf2171fc0e388f4a25c717b00da55))
* add release-please for automated versioning and publishing ([c5bab7e](https://github.com/openpx-trade/openpx/commit/c5bab7e84fd8f3a322d7fbdebffad7413973d1cd))
* add series_id and event_id filters to fetch_markets ([aef4508](https://github.com/openpx-trade/openpx/commit/aef4508c18bbc355e8664451022b945900b9ebef))
* add WsMessage envelope with per-market sequence numbers and NDJSON recording ([7575df6](https://github.com/openpx-trade/openpx/commit/7575df6af56b7c3bc198a33dada2f53b4ac0cb97))
* exchange manifest rate limits, WebSocket improvements, Opinion/Kalshi enhancements ([c2784e5](https://github.com/openpx-trade/openpx/commit/c2784e539b0b6dde66b4795bbda2481cb1a516c8))


### Bug Fixes

* add CI gate job to resolve branch protection check name mismatch ([1371cf4](https://github.com/openpx-trade/openpx/commit/1371cf4bef1a5975022e5c10ba9e80824d9784c4))
* update docs deps to Astro 6 + Starlight 0.38 for Amplify build ([7c3458a](https://github.com/openpx-trade/openpx/commit/7c3458a42403885c25af0e3135cce06a40cea66a))
* update Kalshi WebSocket to match current API field names ([fb5e8ab](https://github.com/openpx-trade/openpx/commit/fb5e8ab681d61de79f94520cd0b16101eb8655f6))
* use ANTHROPIC_API_KEY for Claude Code actions ([fa1f149](https://github.com/openpx-trade/openpx/commit/fa1f149c25e50fc928be4976afa421b94889e8eb))
* use PAT for release-please to trigger PR checks ([8d255f3](https://github.com/openpx-trade/openpx/commit/8d255f3086cf55dffde46c8730ff28d9d7452f79))
* use simple release type and add conventional commit hook ([5616429](https://github.com/openpx-trade/openpx/commit/561642929067a999c3a5b00fdb597b1f735442f8))

## [0.1.5] - 2026-03-26

### Added
- Exchange manifest rate limits and pagination configuration
- WebSocket improvements and reconnection handling
- Opinion and Kalshi exchange enhancements

## [0.1.4] - 2026-03-11

### Added
- Exchange implementations: Kalshi, Polymarket, Opinion, Limitless, Predict.fun
- Unified `Exchange` trait with market data, trading, positions, and balance methods
- WebSocket support for Kalshi, Polymarket, and Limitless (orderbook + activity streams)
- `ExchangeManifest` system for auditable field mappings and pagination config
- `UnifiedMarket` model with cross-exchange normalization
- `timed!` macro and `TimingGuard` for microsecond-precision instrumentation
- `ConcurrentRateLimiter` with semaphore-based concurrency control
- `OpenPxError` hierarchy with `is_retryable()` and `retry_after()` support
- Price utilities: tick rounding, validation, spread calculation
- Cross-exchange event identity mapping via `canonical_event_id()`
- `Orderbook` with `SmallVec`-based delta updates
- `Nav` portfolio calculation and `DeltaInfo` position exposure
- RSA-PSS signing (Kalshi), ECDSA/HMAC (Polymarket), EIP-191 (Limitless, Predict.fun)
- Examples for all five exchanges
- CI pipeline with fmt, clippy, test, and MSRV checks

## [0.1.0] - 2026-02-01

### Added
- Initial release
- Core trait definitions and model types
- Kalshi exchange implementation (reference)
