//! `Sports` — the top-level handle users interact with. Wraps the ESPN client
//! plus the venue-bridge primitive so the common research workflow ("look at
//! ESPN data, find the venues' markets that match") lives behind one struct.

use std::sync::Arc;

use px_core::error::OpenPxError;
use px_exchange_kalshi::{Kalshi, KalshiConfig};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};

use crate::bridge::{markets_for_game, MarketsByVenue};
use crate::espn::{Espn, EspnConfig, GameStateStream};
use crate::types::{Game, GameFilter, GameId, League, Sport};

/// Sports research handle. Construct with `Sports::new()` for the no-auth
/// default (fine for ESPN + the bridge against public market data), or
/// `Sports::with_venues(...)` to plug in pre-authenticated venue clients.
pub struct Sports {
    espn: Espn,
    kalshi: Arc<Kalshi>,
    polymarket: Arc<Polymarket>,
}

impl Sports {
    /// Default Sports handle. ESPN at `site.api.espn.com`; Kalshi and
    /// Polymarket spun up with no credentials (read-only public market data).
    pub fn new() -> Result<Self, OpenPxError> {
        let kalshi =
            Kalshi::new(KalshiConfig::default()).map_err(|e| OpenPxError::Config(e.to_string()))?;
        let polymarket = Polymarket::new(PolymarketConfig::default())
            .map_err(|e| OpenPxError::Config(e.to_string()))?;
        Ok(Self {
            espn: Espn::new(),
            kalshi: Arc::new(kalshi),
            polymarket: Arc::new(polymarket),
        })
    }

    /// Inject pre-built venue clients. Use this when the caller already has
    /// authenticated `Kalshi` / `Polymarket` instances and wants the bridge
    /// to share them.
    pub fn with_venues(espn: Espn, kalshi: Arc<Kalshi>, polymarket: Arc<Polymarket>) -> Self {
        Self {
            espn,
            kalshi,
            polymarket,
        }
    }

    /// Override the ESPN config (e.g., custom poll interval).
    pub fn with_espn_config(mut self, config: EspnConfig) -> Self {
        self.espn = Espn::with_config(config);
        self
    }

    /// Direct access to the ESPN client for the catalog methods (teams,
    /// athletes, standings, news, schedule, play_by_play).
    pub fn espn(&self) -> &Espn {
        &self.espn
    }

    // ---------- Convenience pass-throughs ----------

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

    // ---------- Bridge ----------

    /// Find the Kalshi and Polymarket events that match an ESPN game.
    /// Each returned `Event` carries `market_ids`; call
    /// `kalshi.fetch_market(id)` / `polymarket.fetch_market(id)` to drill
    /// into individual markets.
    pub async fn markets_for_game(&self, game: &Game) -> Result<MarketsByVenue, OpenPxError> {
        markets_for_game(game, self.kalshi.as_ref(), self.polymarket.as_ref()).await
    }
}
