//! Shared HTTP client tunings.
//!
//! Every exchange client (Polymarket, Kalshi, Opinion, and any future
//! addition) should build its `reqwest::Client` via `tuned_client_builder()`.
//! Centralising the tunings here means one edit rolls out to every exchange
//! — we don't want performance fixes to drift across three independent
//! builders.
//!
//! Flags applied:
//!
//! - `http2_adaptive_window(true)`
//! - `http2_initial_stream_window_size(512 KB)` — empirically optimal for
//!   the ~480 KB `/simplified-markets` response class.
//! - `tcp_nodelay(true)` — disable Nagle's. Right call for HTTP/2.
//! - `pool_max_idle_per_host(10)` — deep enough for burst patterns.
//! - `http2_keep_alive_interval(15s)` — keeps the HTTP/2 connection hot
//!   across bursty call patterns.
//! - `no_proxy()` — skip the slow OS proxy lookup.
//!
//! Exchange-specific settings (`.timeout(…)`, `.user_agent(…)`, `.connector(…)`)
//! layer on top of the returned builder.

use std::time::Duration;

pub const HTTP2_INITIAL_STREAM_WINDOW_BYTES: u32 = 512 * 1024;
pub const POOL_MAX_IDLE_PER_HOST: usize = 10;
pub const HTTP2_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

/// A `reqwest::ClientBuilder` preloaded with the openpx-wide HTTP tunings.
/// Layer exchange-specific settings on top before calling `.build()`.
#[cfg(feature = "http")]
pub fn tuned_client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .http2_adaptive_window(true)
        .http2_initial_stream_window_size(HTTP2_INITIAL_STREAM_WINDOW_BYTES)
        .tcp_nodelay(true)
        .pool_max_idle_per_host(POOL_MAX_IDLE_PER_HOST)
        .http2_keep_alive_interval(HTTP2_KEEP_ALIVE_INTERVAL)
        .no_proxy()
}
