use crate::exchange::manifest::{
    EndpointRateLimit, ExchangeManifest, PaginationConfig, PaginationStyle, RateLimitCategory,
    RateLimitConfig,
};

pub const POLYMARKET_MANIFEST: ExchangeManifest = ExchangeManifest {
    id: "polymarket",
    name: "Polymarket",
    base_url: "https://gamma-api.polymarket.com",
    markets_endpoint: "/markets/keyset",
    pagination: PaginationConfig {
        style: PaginationStyle::Cursor,
        max_page_size: 500,
        limit_param: "limit",
        cursor_param: "after_cursor",
    },
    rate_limit: RateLimitConfig {
        default_rps: 150,
        default_burst: 10,
        limits: &[
            EndpointRateLimit {
                category: RateLimitCategory::Write,
                requests_per_second: 350,
                burst: 20,
            },
            EndpointRateLimit {
                category: RateLimitCategory::Bulk,
                requests_per_second: 20,
                burst: 5,
            },
        ],
    },
};
