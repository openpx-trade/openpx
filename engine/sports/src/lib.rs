//! `px-sports` — sports research data for OpenPX.
//!
//! ESPN is the only sports data provider exposed here. The crate also ships
//! a venue-bridge primitive ([`Sports::markets_for_game`]) that finds the
//! Kalshi / Polymarket events resolving on a given ESPN game, so users can
//! go from "research a matchup" to "trade the market" without leaving the
//! SDK.
//!
//! # Quickstart
//!
//! ```no_run
//! use px_sports::{GameFilter, Sports};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let sports = Sports::new()?;
//!
//! // Research
//! let games = sports.list_games(GameFilter {
//!     league: Some("nfl".into()),
//!     ..Default::default()
//! }).await?;
//! let game = &games[0];
//!
//! // Bridge to venues
//! let venues = sports.markets_for_game(game).await?;
//! println!("{} kalshi events, {} polymarket events",
//!     venues.kalshi.len(), venues.polymarket.len());
//! # Ok(()) }
//! ```

pub mod bridge;
pub mod espn;
pub mod sports;
pub mod types;

pub use bridge::{markets_for_game, MarketsByVenue};
pub use espn::{Espn, EspnConfig, GameStateStream};
pub use sports::Sports;
pub use types::{Game, GameFilter, GameId, GameState, GameStatus, League, Score, Sport};
