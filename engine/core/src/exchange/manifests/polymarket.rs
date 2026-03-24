use crate::exchange::manifest::{
    EndpointRateLimit, ExchangeManifest, FieldMapping, PaginationConfig, PaginationStyle,
    RateLimitCategory, RateLimitConfig, Transform,
};

pub const POLYMARKET_MANIFEST: ExchangeManifest = ExchangeManifest {
    // ====== CONNECTION AUDIT ======
    id: "polymarket",
    name: "Polymarket",
    base_url: "https://gamma-api.polymarket.com",
    markets_endpoint: "/markets",
    pagination: PaginationConfig {
        style: PaginationStyle::Offset,
        max_page_size: 500,
        limit_param: "limit",
        cursor_param: "offset",
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

    // ====== DATA AUDIT ======
    field_mappings: &[
        FieldMapping {
            unified_field: "id",
            source_paths: &["id"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "title",
            source_paths: &["question"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "question",
            source_paths: &["question"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "description",
            source_paths: &["description"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "volume",
            source_paths: &["volumeNum", "volume"],
            transform: Transform::ParseInt,
            nullable: false,
        },
        FieldMapping {
            unified_field: "liquidity",
            source_paths: &["liquidityNum", "liquidity"],
            transform: Transform::ParseInt,
            nullable: true,
        },
        FieldMapping {
            unified_field: "close_time",
            source_paths: &["endDate"],
            transform: Transform::Iso8601ToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "open_time",
            source_paths: &["startDate"],
            transform: Transform::Iso8601ToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "group_id",
            source_paths: &["events.0.id"],
            transform: Transform::NestedPath,
            nullable: true,
        },
        FieldMapping {
            unified_field: "slug",
            source_paths: &["slug"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "market_type",
            source_paths: &["marketType"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "condition_id",
            source_paths: &["conditionId"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_yes",
            source_paths: &["clobTokenIds.0"],
            transform: Transform::JsonArrayIndex(0),
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_no",
            source_paths: &["clobTokenIds.1"],
            transform: Transform::JsonArrayIndex(1),
            nullable: true,
        },
    ],
    // Polymarket uses boolean flags, handled specially in the adapter
    status_map: &[],
};
