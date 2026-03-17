use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Real-time sports score/state from the Polymarket Sports WebSocket.
/// Serializes as snake_case for end-users; aliases accept the upstream camelCase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SportResult {
    #[serde(alias = "gameId")]
    pub game_id: u64,
    #[serde(alias = "leagueAbbreviation")]
    pub league_abbreviation: String,
    pub slug: String,
    #[serde(alias = "homeTeam")]
    pub home_team: String,
    #[serde(alias = "awayTeam")]
    pub away_team: String,
    pub status: String,
    pub score: Option<String>,
    pub period: Option<String>,
    pub elapsed: Option<String>,
    pub live: bool,
    pub ended: bool,
    pub turn: Option<String>,
    pub finished_timestamp: Option<DateTime<Utc>>,
}
