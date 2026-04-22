# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
