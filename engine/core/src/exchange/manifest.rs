/// Connection-level descriptor for an exchange.
///
/// Holds the data the rate limiter, fetcher, and HTTP layer query at runtime:
/// base URL, markets endpoint, pagination shape, and rate-limit budgets.
/// Field-level mapping into the unified `Market` is handled directly in each
/// exchange's `parse_market()` — the manifest is not the contract for that.
#[derive(Debug, Clone)]
pub struct ExchangeManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub markets_endpoint: &'static str,
    pub pagination: PaginationConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Copy)]
pub struct PaginationConfig {
    pub style: PaginationStyle,
    pub max_page_size: usize,
    pub limit_param: &'static str,
    pub cursor_param: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationStyle {
    Cursor,
    Offset,
    PageNumber,
    None,
}

/// Endpoint category for per-endpoint rate limiting.
/// Each exchange maps these to its actual documented API rate limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RateLimitCategory {
    /// Read operations: fetch markets, orderbook, positions, balance, trades, price history
    Read = 0,
    /// Write operations: create order, cancel order
    Write = 1,
    /// Bulk/data operations: market fetcher, paginated full-catalog fetches
    Bulk = 2,
}

impl RateLimitCategory {
    pub const COUNT: usize = 3;
    pub const ALL: [RateLimitCategory; 3] = [
        RateLimitCategory::Read,
        RateLimitCategory::Write,
        RateLimitCategory::Bulk,
    ];
}

#[derive(Debug, Clone, Copy)]
pub struct EndpointRateLimit {
    pub category: RateLimitCategory,
    pub requests_per_second: u32,
    pub burst: u32,
}

/// Per-category rate limiting configuration for an exchange.
/// Categories not listed in `limits` inherit from `default_rps`/`default_burst`.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub default_rps: u32,
    pub default_burst: u32,
    pub limits: &'static [EndpointRateLimit],
}

impl RateLimitConfig {
    pub const fn get(&self, category: RateLimitCategory) -> (u32, u32) {
        let mut i = 0;
        while i < self.limits.len() {
            if self.limits[i].category as u8 == category as u8 {
                return (self.limits[i].requests_per_second, self.limits[i].burst);
            }
            i += 1;
        }
        (self.default_rps, self.default_burst)
    }

    pub const fn rps(&self, category: RateLimitCategory) -> u32 {
        self.get(category).0
    }

    pub const fn requests_per_second(&self) -> u32 {
        self.default_rps
    }
}
