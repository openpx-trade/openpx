use crate::exchange::manifest::{
    ExchangeManifest, FieldMapping, PaginationConfig, PaginationStyle, RateLimitConfig, Transform,
};
use crate::models::MarketStatus;

pub const PREDICTFUN_MANIFEST: ExchangeManifest = ExchangeManifest {
    // ====== CONNECTION AUDIT ======
    id: "predictfun",
    name: "PredictFun",
    base_url: "https://api.predict.fun",
    markets_endpoint: "/v1/markets",
    pagination: PaginationConfig {
        style: PaginationStyle::Cursor,
        max_page_size: 100,
        limit_param: "first",
        cursor_param: "after",
    },
    rate_limit: RateLimitConfig {
        requests_per_second: 10,
        burst: 5,
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
            source_paths: &["title"],
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
            source_paths: &["categorySlug"],
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
            source_paths: &["conditionId"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_yes",
            source_paths: &["outcomes.0.id"],
            transform: Transform::JsonArrayIndex(0),
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_no",
            source_paths: &["outcomes.1.id"],
            transform: Transform::JsonArrayIndex(1),
            nullable: true,
        },
    ],
    status_map: &[
        ("OPEN", MarketStatus::Active),
        ("RESOLVED", MarketStatus::Resolved),
    ],
};
