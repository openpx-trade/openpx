# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
