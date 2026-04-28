//! `Sports` — the top-level handle users interact with. Wraps the ESPN client.

use px_core::error::OpenPxError;

use crate::espn::{Espn, EspnConfig, GameStateStream};
use crate::types::{Game, GameFilter, GameId, League, Sport};

/// Sports research handle. Wraps the ESPN client.
///
/// ESPN endpoints are public and require no authentication, so `Sports::new()`
/// is infallible in practice — the constructor returns a `Result` only to
/// match the rest of the OpenPX builder pattern.
pub struct Sports {
    espn: Espn,
}

impl Sports {
    pub fn new() -> Result<Self, OpenPxError> {
        Ok(Self { espn: Espn::new() })
    }

    /// Override the ESPN config (e.g., custom poll interval, base URL).
    pub fn with_espn_config(mut self, config: EspnConfig) -> Self {
        self.espn = Espn::with_config(config);
        self
    }

    /// Direct access to the ESPN client for the catalog methods (teams,
    /// athletes, standings, news, schedule, play_by_play).
    pub fn espn(&self) -> &Espn {
        &self.espn
    }

    pub async fn list_sports(&self) -> Result<Vec<Sport>, OpenPxError> {
        self.espn.list_sports().await
    }

    pub async fn list_leagues(&self, sport_id: Option<&str>) -> Result<Vec<League>, OpenPxError> {
        self.espn.list_leagues(sport_id).await
    }

    pub async fn list_games(&self, filter: GameFilter) -> Result<Vec<Game>, OpenPxError> {
        self.espn.list_games(filter).await
    }

    pub async fn get_game(&self, league: &str, id: &GameId) -> Result<Game, OpenPxError> {
        self.espn.get_game(league, id).await
    }

    pub fn subscribe_game_state(&self, league: &str) -> GameStateStream {
        self.espn.subscribe_game_state(league)
    }
}
