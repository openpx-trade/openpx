//! `px-sports` — ESPN sports research data for OpenPX.
//!
//! ESPN's public endpoints provide the catalog (sports, leagues, games,
//! teams, athletes, standings, news, schedule, play-by-play) plus live
//! game state via an HTTP-polling stream.
//!
//! # Quickstart
//!
//! ```no_run
//! use px_sports::{GameFilter, Sports};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let sports = Sports::new()?;
//! let games = sports.list_games(GameFilter {
//!     league: Some("nfl".into()),
//!     ..Default::default()
//! }).await?;
//! # Ok(()) }
//! ```

pub mod espn;
pub mod sports;
pub mod types;

pub use espn::{Espn, EspnConfig, GameStateStream};
pub use sports::Sports;
pub use types::{Game, GameFilter, GameId, GameState, GameStatus, League, Score, Sport};
