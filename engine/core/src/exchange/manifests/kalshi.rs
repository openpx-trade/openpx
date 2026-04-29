use crate::exchange::manifest::{
    EndpointRateLimit, ExchangeManifest, PaginationConfig, PaginationStyle, RateLimitCategory,
    RateLimitConfig,
};

pub const KALSHI_MANIFEST: ExchangeManifest = ExchangeManifest {
    id: "kalshi",
    name: "Kalshi",
    base_url: "https://api.elections.kalshi.com/trade-api/v2",
    markets_endpoint: "/markets",
    pagination: PaginationConfig {
        style: PaginationStyle::Cursor,
        max_page_size: 1000,
        limit_param: "limit",
        cursor_param: "cursor",
    },
    rate_limit: RateLimitConfig {
        default_rps: 20,
        default_burst: 5,
        limits: &[
            EndpointRateLimit {
                category: RateLimitCategory::Write,
                requests_per_second: 10,
                burst: 3,
            },
            EndpointRateLimit {
                category: RateLimitCategory::Bulk,
                requests_per_second: 10,
                burst: 3,
            },
        ],
    },
};
