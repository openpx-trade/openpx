use crate::exchange::manifest::{
    ExchangeManifest, FieldMapping, PaginationConfig, PaginationStyle, RateLimitConfig, Transform,
};
use crate::models::MarketStatus;

pub const OPINION_MANIFEST: ExchangeManifest = ExchangeManifest {
    // ====== CONNECTION AUDIT ======
    id: "opinion",
    name: "Opinion",
    base_url: "https://openapi.opinion.trade",
    markets_endpoint: "/openapi/market?marketType=2",
    pagination: PaginationConfig {
        style: PaginationStyle::PageNumber,
        max_page_size: 20,
        limit_param: "limit",
        cursor_param: "page",
    },
    rate_limit: RateLimitConfig {
        requests_per_second: 5,
        burst: 2,
    },

    // ====== DATA AUDIT ======
    field_mappings: &[
        FieldMapping {
            unified_field: "id",
            source_paths: &["marketId", "market_id", "topic_id", "id"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "title",
            source_paths: &["marketTitle", "market_title", "title"],
            transform: Transform::Direct,
            nullable: false,
        },
        FieldMapping {
            unified_field: "question",
            source_paths: &["marketTitle", "market_title", "title", "question"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "description",
            source_paths: &["rules"],
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
            source_paths: &["cutoffAt"],
            transform: Transform::UnixSecsToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "open_time",
            source_paths: &["createdAt"],
            transform: Transform::UnixSecsToDateTime,
            nullable: true,
        },
        FieldMapping {
            unified_field: "group_id",
            source_paths: &["collection.id"],
            transform: Transform::NestedPath,
            nullable: true,
        },
        FieldMapping {
            unified_field: "slug",
            source_paths: &[],
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
            source_paths: &["yesTokenId"],
            transform: Transform::Direct,
            nullable: true,
        },
        FieldMapping {
            unified_field: "token_id_no",
            source_paths: &["noTokenId"],
            transform: Transform::Direct,
            nullable: true,
        },
    ],
    status_map: &[
        ("activated", MarketStatus::Active),
        ("resolved", MarketStatus::Resolved),
    ],
};
