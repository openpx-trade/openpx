//! Comprehensive timing instrumentation for HFT-level latency tracking.
//!
//! This module provides utilities for recording detailed timing metrics at every
//! step of request processing. All timings are recorded as Prometheus histograms
//! in microseconds for maximum precision.
//!
//! HTTP timing across exchange crates follows a common pattern (send → body → parse).
//! Each exchange owns its timing to preserve exchange-specific metric labels and
//! auth header patterns. The `timed!` macro provides the building block.

use metrics::histogram;
use std::time::Instant;

/// Zero-cost timing macro. Expands inline at compile time.
#[macro_export]
macro_rules! timed {
    ($metric:expr, $($k:expr => $v:expr),+; $block:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $block;
        let __us = __start.elapsed().as_micros() as f64;
        metrics::histogram!($metric, $($k => $v),+).record(__us);
        __result
    }};
    ($metric:expr; $block:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $block;
        let __us = __start.elapsed().as_micros() as f64;
        metrics::histogram!($metric).record(__us);
        __result
    }};
}

/// A guard that automatically records timing when dropped.
/// Useful for measuring the total duration of a scope.
pub struct TimingGuard {
    name: &'static str,
    start: Instant,
    label_key: Option<&'static str>,
    label_value: Option<String>,
}

impl TimingGuard {
    /// Create a new timing guard with no labels.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            label_key: None,
            label_value: None,
        }
    }

    /// Create a new timing guard with a single label.
    pub fn with_label(name: &'static str, key: &'static str, value: impl Into<String>) -> Self {
        Self {
            name,
            start: Instant::now(),
            label_key: Some(key),
            label_value: Some(value.into()),
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let elapsed_us = self.start.elapsed().as_micros() as f64;
        if let (Some(key), Some(value)) = (self.label_key, self.label_value.take()) {
            histogram!(self.name, key => value).record(elapsed_us);
        } else {
            histogram!(self.name).record(elapsed_us);
        }
    }
}

// ============================================================================
// Metric name constants for consistency across crates
// ============================================================================

/// Exchange implementation metrics
pub mod exchange {
    /// HTTP request time to exchange API
    pub const HTTP_REQUEST: &str = "openpx.exchange.http_request_us";
    /// Response parsing time
    pub const PARSE_RESPONSE: &str = "openpx.exchange.parse_response_us";
    /// Signature generation time (for authenticated requests)
    pub const SIGN_REQUEST: &str = "openpx.exchange.sign_request_us";
    /// WebSocket message send time
    pub const WS_SEND: &str = "openpx.exchange.ws_send_us";
    /// WebSocket message receive time
    pub const WS_RECEIVE: &str = "openpx.exchange.ws_receive_us";
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timing_guard() {
        // Just ensure it compiles and doesn't panic
        let _guard = TimingGuard::new("test.metric");
        sleep(Duration::from_micros(100));
    }

    #[test]
    fn test_timing_guard_with_label() {
        let _guard = TimingGuard::with_label("test.metric", "exchange", "polymarket");
        sleep(Duration::from_micros(100));
    }
}
