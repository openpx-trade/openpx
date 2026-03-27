# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.7](https://github.com/openpx-trade/openpx/compare/v0.1.6...v0.1.7) (2026-03-27)


### Bug Fixes

* add npm platform package.json files for napi-rs publish ([7f37b74](https://github.com/openpx-trade/openpx/commit/7f37b74e4842ed54d12b9d2d58d311c4fbbeda45))
* reorder napi prepublish before artifacts in publish workflow ([8550148](https://github.com/openpx-trade/openpx/commit/8550148cb72d2fbaff2ce08520c1cdb5ccb0a88f))

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
