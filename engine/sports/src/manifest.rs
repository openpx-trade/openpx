//! Per-provider field mapping manifest. Mirrors the exchange manifest pattern
//! (`engine/core/src/exchange/manifest.rs`): every JSON key a sports provider
//! reads must either be declared here or appear in the per-provider
//! `maintenance/manifest-allowlists/sports-<id>.txt` file. This is what
//! turns the unified schema into an auditable contract.

use px_core::models::GameStatus;

#[derive(Debug, Clone)]
pub struct SportsManifest {
    /// Provider id (e.g., "espn", "kalshi", "polymarket").
    pub id: &'static str,

    /// Human-readable name.
    pub name: &'static str,

    /// Base API URL.
    pub base_url: &'static str,

    /// Endpoint that returns a list of games (relative to `base_url`).
    pub games_endpoint: &'static str,

    /// Field mappings from raw provider JSON to unified `Game` / `GameState`.
    pub field_mappings: &'static [FieldMapping],

    /// Status value mappings (provider status string → `GameStatus`).
    /// Lowercase at definition time for case-insensitive lookup without alloc.
    pub status_map: &'static [(&'static str, GameStatus)],
}

impl SportsManifest {
    /// Look up the unified `GameStatus` for a provider-supplied status string.
    pub fn map_status(&self, raw: &str) -> Option<GameStatus> {
        self.status_map
            .iter()
            .find(|(s, _)| s.eq_ignore_ascii_case(raw))
            .map(|(_, status)| *status)
    }

    /// Get a field mapping by unified field name.
    pub fn get_field_mapping(&self, unified_field: &str) -> Option<&FieldMapping> {
        self.field_mappings
            .iter()
            .find(|m| m.unified_field == unified_field)
    }
}

/// Mapping from raw provider JSON field to unified field on `Game`/`GameState`.
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// Target field on the unified type (e.g., "home_team", "score.home").
    pub unified_field: &'static str,
    /// Source path(s) in raw JSON (fallback chain for providers that vary
    /// field names across endpoints).
    pub source_paths: &'static [&'static str],
    /// Transformation to apply when reading the source value.
    pub transform: Transform,
    /// Whether the field can be absent or null in the source payload.
    pub nullable: bool,
}

/// Transformation applied when mapping a source field. Intentionally smaller
/// than the exchange `Transform` enum — sports payloads don't have the
/// fixed-point quirks of trading endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    /// Use the value directly with no conversion.
    Direct,
    /// Unix timestamp (seconds) → `DateTime<Utc>`.
    UnixSecsToDateTime,
    /// Unix timestamp (milliseconds) → `DateTime<Utc>`.
    UnixMillisToDateTime,
    /// ISO8601 string → `DateTime<Utc>`.
    Iso8601ToDateTime,
    /// String/Float → integer.
    ParseInt,
    /// Nested path extraction (dot-notation handled by `source_paths`).
    NestedPath,
}
