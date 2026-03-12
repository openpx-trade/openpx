use crate::exchange::manifest::{
    ExchangeManifest, FieldMapping, PaginationConfig, PaginationStyle, RateLimitConfig, Transform,
};

pub const LIMITLESS_MANIFEST: ExchangeManifest = ExchangeManifest {
    // ====== CONNECTION AUDIT ======
    id: "limitless",
    name: "Limitless",
    base_url: "https://api.limitless.exchange",
    markets_endpoint: "/markets/active",
    // Note: /markets/active endpoint does not support pagination parameters
    pagination: PaginationConfig {
        style: PaginationStyle::None,
        max_page_size: 0, // Not applicable - returns all in single call
        limit_param: "",
        cursor_param: "",
    },
    rate_limit: RateLimitConfig {
        requests_per_second: 5,
        burst: 2,
    },

    // ====== DATA AUDIT ======
    field_mappings: &[
        FieldMapping {
            unified_field: "id",
            source_paths: &["slug"],
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
            source_paths: &["question", "title"],
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
            source_paths: &[],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "open_time",
            source_paths: &["createdAt"],
            transform: Transform::Iso8601ToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "group_id",
            source_paths: &[],
            transform: Transform::Direct,
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
            source_paths: &["marketVariant"],
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
            source_paths: &["tokens.yes"],
            transform: Transform::NestedPath,
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_no",
            source_paths: &["tokens.no"],
            transform: Transform::NestedPath,
            nullable: true,
        },
    ],
    // Limitless /markets/active only returns active markets
    status_map: &[],
};
