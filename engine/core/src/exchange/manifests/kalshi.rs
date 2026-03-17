use crate::exchange::manifest::{
    ExchangeManifest, FieldMapping, PaginationConfig, PaginationStyle, RateLimitConfig, Transform,
};
use crate::models::MarketStatus;

pub const KALSHI_MANIFEST: ExchangeManifest = ExchangeManifest {
    // ====== CONNECTION AUDIT ======
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
        requests_per_second: 20,
        burst: 5,
    },

    // ====== DATA AUDIT ======
    field_mappings: &[
        FieldMapping {
            unified_field: "id",
            source_paths: &["ticker"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "title",
            source_paths: &["title"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "question",
            source_paths: &["title"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "description",
            source_paths: &["rules_primary"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "volume",
            source_paths: &["volume"],
            transform: Transform::ParseInt,
            nullable: false,
        },
        FieldMapping {
            unified_field: "liquidity",
            source_paths: &["liquidity"],
            transform: Transform::ParseInt,
            nullable: true,
        },
        FieldMapping {
            unified_field: "close_time",
            source_paths: &["close_time"],
            transform: Transform::Iso8601ToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "open_time",
            source_paths: &["open_time"],
            transform: Transform::Iso8601ToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "group_id",
            source_paths: &["event_ticker"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "slug",
            source_paths: &["subtitle"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "market_type",
            source_paths: &["market_type"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "condition_id",
            source_paths: &[],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_yes",
            source_paths: &[],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_no",
            source_paths: &[],
            transform: Transform::Direct,
            nullable: true,
        },
    ],
    status_map: &[
        ("active", MarketStatus::Active),
        ("closed", MarketStatus::Closed),
        ("determined", MarketStatus::Resolved),
        ("finalized", MarketStatus::Resolved),
        ("initialized", MarketStatus::Closed),
        ("inactive", MarketStatus::Closed),
        ("disputed", MarketStatus::Closed),
        ("amended", MarketStatus::Closed),
    ],
};
