use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{GameId, GameStatus};

/// Live or post-game state for a single `Game`. The unit emitted by
/// `SportsProvider::subscribe_game_state`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GameState {
    pub game_id: GameId,
    pub status: GameStatus,
    /// Verbatim status string from the provider; useful when `status` is `Unknown`.
    pub raw_status: Option<String>,
    /// Current score. `None` until the game starts.
    pub score: Option<Score>,
    /// Provider-formatted period label (e.g., "Q2", "Halftime", "9th Inning").
    /// Kept as a string because semantics differ per sport.
    pub period: Option<String>,
    /// Provider-formatted clock (e.g., "12:34"). Remaining-vs-elapsed semantics
    /// are sport-specific; surfaces verbatim.
    pub clock: Option<String>,
    pub live: bool,
    pub ended: bool,
    /// Timestamp this state was observed (provider-supplied if available, else
    /// receive-time on the client).
    pub updated_at: DateTime<Utc>,
}

/// Score in a head-to-head sport. For exotic formats (esports best-of-N,
/// golf cumulative, etc.) providers populate `raw` and leave home/away `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Score {
    pub home: Option<u32>,
    pub away: Option<u32>,
    /// Verbatim provider score string when home/away can't be normalized
    /// (e.g., "0-0|2-0|Bo3", "+5/-3").
    pub raw: Option<String>,
}
