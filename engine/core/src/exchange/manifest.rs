use crate::models::MarketStatus;

/// Complete auditable manifest for an exchange.
/// When opened, shows SOURCE and TRANSFORMATION side-by-side.
#[derive(Debug, Clone)]
pub struct ExchangeManifest {
    // ========================================
    // SECTION 1: CONNECTION AUDIT (Where do we go?)
    // ========================================
    /// Exchange identifier (e.g., "kalshi", "polymarket")
    pub id: &'static str,

    /// Human-readable exchange name
    pub name: &'static str,

    /// Base API URL
    pub base_url: &'static str,

    /// Markets list endpoint (relative to base_url)
    pub markets_endpoint: &'static str,

    /// Pagination configuration
    pub pagination: PaginationConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    // ========================================
    // SECTION 2: DATA AUDIT (How do we map it?)
    // ========================================
    /// Field mappings from raw exchange JSON to Market
    pub field_mappings: &'static [FieldMapping],

    /// Status value mappings (exchange status -> MarketStatus)
    pub status_map: &'static [(&'static str, MarketStatus)],
}

impl ExchangeManifest {
    /// Look up the MarketStatus for a given exchange status string.
    /// Status map entries should be lowercase at definition time for O(n) without allocation.
    pub fn map_status(&self, exchange_status: &str) -> Option<MarketStatus> {
        self.status_map
            .iter()
            .find(|(s, _)| s.eq_ignore_ascii_case(exchange_status))
            .map(|(_, status)| *status)
    }

    /// Get a field mapping by unified field name.
    pub fn get_field_mapping(&self, unified_field: &str) -> Option<&FieldMapping> {
        self.field_mappings
            .iter()
            .find(|m| m.unified_field == unified_field)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PaginationConfig {
    /// Pagination style: Cursor, Offset, Page
    pub style: PaginationStyle,
    /// Maximum items per page
    pub max_page_size: usize,
    /// Query param name for limit
    pub limit_param: &'static str,
    /// Query param name for cursor/offset
    pub cursor_param: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaginationStyle {
    /// Cursor-based pagination (Kalshi)
    Cursor,
    /// Offset-based pagination (Polymarket)
    Offset,
    /// Page-number pagination (1-indexed)
    PageNumber,
    /// No pagination supported - endpoint returns all data in single call
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

/// Rate limit for a single endpoint category.
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
    /// Fallback rate for any category not explicitly listed.
    pub default_rps: u32,
    /// Default burst for any category not explicitly listed.
    pub default_burst: u32,
    /// Per-category overrides (searched linearly; at most 3 entries).
    pub limits: &'static [EndpointRateLimit],
}

impl RateLimitConfig {
    /// Look up (rps, burst) for a given category.
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

    /// Convenience: get requests_per_second for a category.
    pub const fn rps(&self, category: RateLimitCategory) -> u32 {
        self.get(category).0
    }

    /// Backwards-compat: overall requests_per_second (uses Read category).
    pub const fn requests_per_second(&self) -> u32 {
        self.default_rps
    }
}

/// Mapping from raw exchange JSON field to unified field.
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// Target field in Market
    pub unified_field: &'static str,
    /// Source path(s) in raw JSON (fallback chain)
    pub source_paths: &'static [&'static str],
    /// Transformation to apply
    pub transform: Transform,
    /// Whether field can be null
    pub nullable: bool,
}

/// Transformation to apply when mapping a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    /// No transformation - use value directly
    Direct,
    /// Divide by 100 (Kalshi prices: cents -> decimal)
    CentsToDollars,
    /// Unix timestamp (seconds) to DateTime
    UnixSecsToDateTime,
    /// Unix timestamp (milliseconds) to DateTime
    UnixMillisToDateTime,
    /// ISO8601 string to DateTime
    Iso8601ToDateTime,
    /// String/Float -> i64
    ParseInt,
    /// String -> f64
    ParseFloat,
    /// Extract element at index from JSON array
    JsonArrayIndex(usize),
    /// Nested path extraction (dot-notation handled by source_paths)
    NestedPath,
}
