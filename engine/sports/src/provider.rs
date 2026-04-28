use futures::Stream;
use serde::Serialize;
use std::pin::Pin;

use px_core::error::{OpenPxError, WebSocketError};
use px_core::models::{Game, GameFilter, GameId, GameState, Sport};

use crate::manifest::SportsManifest;

/// Stream of live `GameState` updates emitted by a `SportsProvider`. Mirrors
/// the `SportsStream` shape on the WebSocket side but carries the unified
/// `GameState` instead of a provider-specific DTO.
pub type GameStateStream = Pin<Box<dyn Stream<Item = Result<GameState, WebSocketError>> + Send>>;

/// Read-only sports data provider. Implemented by ESPN (pure data) and by the
/// existing exchanges (Kalshi, Polymarket) that also expose sports primitives.
///
/// The trait covers only the parity-core: concepts present across all current
/// providers. Provider-specific endpoints (ESPN's news/standings/play-by-play,
/// Kalshi's milestone game stats, Polymarket's team metadata) live as inherent
/// methods on the concrete provider type.
#[allow(async_fn_in_trait)]
pub trait SportsProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    /// Capability advertisement. Same pattern as `Exchange::describe()` —
    /// callers branch on flags rather than catching `NotSupported` errors at
    /// runtime.
    fn capabilities(&self) -> SportsCapabilities;

    /// Returns the provider's manifest containing the JSON-to-unified-schema
    /// field map. Mirrors `Exchange::manifest()`.
    fn manifest(&self) -> &'static SportsManifest;

    /// List the sports this provider covers.
    async fn list_sports(&self) -> Result<Vec<Sport>, OpenPxError>;

    /// List games matching the filter. Returns `(games, next_cursor)`.
    /// Pagination is opt-in: providers without a cursor return `None`.
    async fn list_games(
        &self,
        filter: GameFilter,
    ) -> Result<(Vec<Game>, Option<String>), OpenPxError>;

    /// Fetch a single game by id. The `id` must be a `GameId` previously
    /// returned by *this* provider.
    async fn get_game(&self, id: &GameId) -> Result<Game, OpenPxError>;

    /// Subscribe to live game-state updates. Implementations may use a
    /// long-lived WebSocket (Polymarket) or HTTP polling (ESPN, Kalshi); the
    /// stream contract is the same either way.
    fn subscribe_game_state(&self) -> GameStateStream;
}

/// Static capability flags for a `SportsProvider`. Includes both parity-core
/// methods (advertised because some providers can't honor them — e.g., Kalshi
/// has no first-class team primitive) and the inherent ESPN-only extensions
/// so callers can probe `capabilities().has_news` without downcasting.
#[derive(Debug, Clone, Copy, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SportsCapabilities {
    pub id: &'static str,
    pub name: &'static str,

    // Parity-core
    pub has_list_sports: bool,
    pub has_list_games: bool,
    pub has_get_game: bool,
    pub has_live_state_stream: bool,

    // Partial parity (some providers expose these as inherent methods)
    pub has_teams: bool,
    pub has_schedule: bool,
    pub has_standings: bool,

    // Provider-specific extensions
    pub has_athletes: bool,
    pub has_news: bool,
    pub has_rankings: bool,
    pub has_play_by_play: bool,
}

impl SportsCapabilities {
    pub const fn none(id: &'static str, name: &'static str) -> Self {
        Self {
            id,
            name,
            has_list_sports: false,
            has_list_games: false,
            has_get_game: false,
            has_live_state_stream: false,
            has_teams: false,
            has_schedule: false,
            has_standings: false,
            has_athletes: false,
            has_news: false,
            has_rankings: false,
            has_play_by_play: false,
        }
    }
}
