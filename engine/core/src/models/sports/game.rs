use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Opaque, provider-scoped game identifier. Only meaningful in the context of
/// the `Game::provider` that produced it.
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

/// A scheduled, live, or completed sporting event. The unified shape across
/// every `SportsProvider`. Provider-specific fields stay in the provider's
/// own DTO and are only surfaced via the provider's inherent methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Game {
    pub id: GameId,
    /// Canonical league id (`League::id`). Always set.
    pub league: String,
    /// Home team display name. May be `None` on providers that don't model
    /// teams as first-class (Kalshi).
    pub home_team: Option<String>,
    /// Away team display name. Same caveat as `home_team`.
    pub away_team: Option<String>,
    /// Scheduled start time. May be `None` for providers that publish
    /// games only after kickoff.
    pub start_time: Option<DateTime<Utc>>,
    pub status: GameStatus,
    /// Verbatim status string from the provider before normalization. Useful
    /// when `status` is `GameStatus::Unknown` (caller can branch on `raw_status`).
    pub raw_status: Option<String>,
    /// Provider id this game was fetched from (e.g., "espn", "kalshi", "polymarket").
    pub provider: String,
}

/// Normalized lifecycle status. Unit enum so manifest `status_map`s can be
/// declared as `&'static [(&'static str, GameStatus)]`. Providers ship a wide
/// vocabulary of status strings; anything we can't map normalizes to `Unknown`
/// and the original value is preserved in `Game::raw_status`.
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

/// Filter for `SportsProvider::list_games`. All fields optional — providers
/// apply only those they support and ignore the rest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GameFilter {
    /// Restrict to one league id (e.g., "nfl").
    pub league: Option<String>,
    /// Restrict to games on a specific calendar day (UTC).
    pub date: Option<DateTime<Utc>>,
    /// Restrict to a normalized status.
    pub status: Option<GameStatus>,
    /// Restrict to games involving a team (matches home or away).
    pub team: Option<String>,
    /// Page size hint; providers may clamp.
    pub limit: Option<usize>,
    /// Opaque pagination cursor from a previous response.
    pub cursor: Option<String>,
}
