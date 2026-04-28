//! Polymarket sports JSON → unified `Game` / `GameState` mapping.
//!
//! Source surfaces:
//! - Gamma API `/sports` and `/events/results` (HTTP, no auth).
//! - Sports WebSocket (`wss://sports-api.polymarket.com/ws`) for live state.
//!
//! The drift gate at `maintenance/tests/manifest_coverage.rs` asserts that
//! every JSON key read by `engine/exchanges/polymarket/src/sports.rs` is
//! either declared here or listed in
//! `maintenance/manifest-allowlists/sports-polymarket.txt`.

use px_core::models::GameStatus;

use crate::manifest::{FieldMapping, SportsManifest, Transform};

const FIELD_MAPPINGS: &[FieldMapping] = &[
    // Game-level fields, sourced from gamma `/events/results` and from the
    // Sports WS payload (camelCase names match the upstream JSON).
    FieldMapping {
        unified_field: "id",
        source_paths: &["gameId", "id"],
        transform: Transform::Direct,
        nullable: false,
    },
    FieldMapping {
        unified_field: "league",
        source_paths: &["leagueAbbreviation", "league"],
        transform: Transform::Direct,
        nullable: false,
    },
    FieldMapping {
        unified_field: "home_team",
        source_paths: &["homeTeam"],
        transform: Transform::Direct,
        nullable: true,
    },
    FieldMapping {
        unified_field: "away_team",
        source_paths: &["awayTeam"],
        transform: Transform::Direct,
        nullable: true,
    },
    FieldMapping {
        unified_field: "start_time",
        source_paths: &["gameStartTime", "eventDate"],
        transform: Transform::Iso8601ToDateTime,
        nullable: true,
    },
    FieldMapping {
        unified_field: "raw_status",
        source_paths: &["status", "gameStatus"],
        transform: Transform::Direct,
        nullable: true,
    },
    // GameState-level fields.
    FieldMapping {
        unified_field: "score.raw",
        source_paths: &["score"],
        transform: Transform::Direct,
        nullable: true,
    },
    FieldMapping {
        unified_field: "period",
        source_paths: &["period"],
        transform: Transform::Direct,
        nullable: true,
    },
    FieldMapping {
        unified_field: "clock",
        source_paths: &["elapsed"],
        transform: Transform::Direct,
        nullable: true,
    },
    FieldMapping {
        unified_field: "live",
        source_paths: &["live"],
        transform: Transform::Direct,
        nullable: false,
    },
    FieldMapping {
        unified_field: "ended",
        source_paths: &["ended"],
        transform: Transform::Direct,
        nullable: false,
    },
    FieldMapping {
        unified_field: "updated_at",
        source_paths: &["finished_timestamp"],
        transform: Transform::Iso8601ToDateTime,
        nullable: true,
    },
];

/// Polymarket-status-string → unified `GameStatus`. Lowercase entries; the
/// lookup is case-insensitive. Anything not listed normalizes to
/// `GameStatus::Unknown` with the raw value preserved on `Game::raw_status`.
const STATUS_MAP: &[(&str, GameStatus)] = &[
    ("scheduled", GameStatus::Scheduled),
    ("upcoming", GameStatus::Scheduled),
    ("pre", GameStatus::Scheduled),
    ("in progress", GameStatus::Live),
    ("in_progress", GameStatus::Live),
    ("live", GameStatus::Live),
    ("halftime", GameStatus::Live),
    ("final", GameStatus::Final),
    ("ended", GameStatus::Final),
    ("postponed", GameStatus::Postponed),
    ("delayed", GameStatus::Postponed),
    ("cancelled", GameStatus::Cancelled),
    ("canceled", GameStatus::Cancelled),
];

pub static POLYMARKET_SPORTS_MANIFEST: SportsManifest = SportsManifest {
    id: "polymarket",
    name: "Polymarket",
    base_url: "https://gamma-api.polymarket.com",
    games_endpoint: "/events/results",
    field_mappings: FIELD_MAPPINGS,
    status_map: STATUS_MAP,
};
