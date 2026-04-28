//! Unified sports research types. Shape is driven by ESPN — the only sports
//! data provider OpenPX exposes — but kept generic enough that callers don't
//! see ESPN-specific quirks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A top-level sport (e.g., football, basketball, hockey).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Sport {
    /// Canonical id used in ESPN URLs (e.g., "football", "basketball").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
}

/// A specific competition within a sport (e.g., NFL, NBA, EPL).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct League {
    /// Canonical id used in ESPN URLs (e.g., "nfl", "nba", "usa.1").
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Parent `Sport::id`.
    pub sport_id: String,
    /// Common abbreviation when present (e.g., "NFL").
    pub abbreviation: Option<String>,
}

/// Opaque game identifier scoped to ESPN's event ids.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GameId(pub String);

impl GameId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GameId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A scheduled, live, or completed sporting event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Game {
    pub id: GameId,
    /// Canonical league id (`League::id`).
    pub league: String,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
    /// Scheduled start time (UTC).
    pub start_time: Option<DateTime<Utc>>,
    pub status: GameStatus,
    /// Verbatim ESPN status before normalization. Useful when `status` is `Unknown`.
    pub raw_status: Option<String>,
    /// Optional venue name (stadium / arena).
    pub venue: Option<String>,
}

/// Normalized lifecycle status. Anything outside this list is `Unknown`,
/// with the original string preserved on `Game::raw_status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    Scheduled,
    Live,
    Final,
    Postponed,
    Cancelled,
    Unknown,
}

/// Filter for `Espn::list_games` / `Sports::list_games`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GameFilter {
    /// League id (e.g., "nfl"). Required for ESPN — scoreboard endpoints are league-scoped.
    pub league: Option<String>,
    /// Calendar day (UTC). When set, returns games scheduled on that day.
    pub date: Option<DateTime<Utc>>,
    pub status: Option<GameStatus>,
    /// Substring match against home/away team names.
    pub team: Option<String>,
}

/// Live or post-game state for a single game.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GameState {
    pub game_id: GameId,
    pub status: GameStatus,
    pub raw_status: Option<String>,
    pub score: Option<Score>,
    /// Provider-formatted period label (e.g., "Q2", "Halftime").
    pub period: Option<String>,
    /// Provider-formatted clock string (e.g., "12:34").
    pub clock: Option<String>,
    pub live: bool,
    pub ended: bool,
    pub updated_at: DateTime<Utc>,
}

/// Score in a head-to-head sport. Exotic formats (esports, golf cumulative)
/// fall through to `raw`-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Score {
    pub home: Option<u32>,
    pub away: Option<u32>,
    pub raw: Option<String>,
}
