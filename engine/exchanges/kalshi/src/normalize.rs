//! Canonical Kalshi trade normalization functions.
//!
//! Kalshi APIs return prices as either decimals (0.01–0.99) or cent-like
//! integers (1–99). These helpers normalize both forms to probability space
//! [0, 1] and canonicalize outcome strings.

/// Normalize a raw Kalshi price to [0, 1] probability.
///
/// Returns `None` for non-finite, zero, or out-of-range values.
pub fn normalize_kalshi_trade_price(raw: f64) -> Option<f64> {
    if !raw.is_finite() {
        return None;
    }
    let mut p = raw;
    if p >= 1.0 {
        p /= 100.0;
    }
    // Avoid floating-point representation artifacts in API payloads.
    p = (p * 1_000_000.0).round() / 1_000_000.0;
    if p > 0.0 && p < 1.0 {
        Some(p)
    } else {
        None
    }
}

/// Canonicalize a Kalshi outcome string to `"Yes"` or `"No"`.
///
/// Returns `None` for unrecognized outcomes.
pub fn normalize_kalshi_outcome(outcome: Option<&str>) -> Option<String> {
    outcome.and_then(|o| match o.trim().to_ascii_lowercase().as_str() {
        "yes" => Some("Yes".to_string()),
        "no" => Some("No".to_string()),
        _ => None,
    })
}

/// Trait for types whose Kalshi price/outcome/size fields can be normalized
/// in-place. Implemented for `MarketTrade` and `TradeParquetRow`.
pub trait KalshiTradeLike {
    fn price(&self) -> f64;
    fn set_price(&mut self, p: f64);
    fn yes_price(&self) -> Option<f64>;
    fn set_yes_price(&mut self, p: Option<f64>);
    fn no_price(&self) -> Option<f64>;
    fn set_no_price(&mut self, p: Option<f64>);
    fn outcome(&self) -> Option<&str>;
    fn set_outcome(&mut self, o: Option<String>);
    fn size(&self) -> f64;
}

/// Normalize price, yes_price, no_price, outcome, and validate size on any
/// `KalshiTradeLike`. Returns `None` if the main price is invalid or size <= 0.
pub fn normalize_kalshi_trade<T: KalshiTradeLike>(mut trade: T) -> Option<T> {
    trade.set_price(normalize_kalshi_trade_price(trade.price())?);
    trade.set_yes_price(trade.yes_price().and_then(normalize_kalshi_trade_price));
    trade.set_no_price(trade.no_price().and_then(normalize_kalshi_trade_price));
    trade.set_outcome(normalize_kalshi_outcome(trade.outcome()));
    if trade.size() <= 0.0 {
        return None;
    }
    Some(trade)
}

impl KalshiTradeLike for px_core::MarketTrade {
    fn price(&self) -> f64 {
        self.price
    }
    fn set_price(&mut self, p: f64) {
        self.price = p;
    }
    fn yes_price(&self) -> Option<f64> {
        self.yes_price
    }
    fn set_yes_price(&mut self, p: Option<f64>) {
        self.yes_price = p;
    }
    fn no_price(&self) -> Option<f64> {
        self.no_price
    }
    fn set_no_price(&mut self, p: Option<f64>) {
        self.no_price = p;
    }
    fn outcome(&self) -> Option<&str> {
        self.outcome.as_deref()
    }
    fn set_outcome(&mut self, o: Option<String>) {
        self.outcome = o;
    }
    fn size(&self) -> f64 {
        self.size
    }
}
