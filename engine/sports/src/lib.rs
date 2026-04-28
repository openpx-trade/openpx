//! `px-sports`: unified sports research surface.
//!
//! This crate owns the `SportsProvider` trait, the unified manifest scaffolding,
//! and the per-provider manifests. Concrete `impl SportsProvider` blocks live
//! in the provider's own crate:
//! - **Polymarket**: `engine/exchanges/polymarket/src/sports.rs`
//!   (driven internally by `polymarket::SportsWebSocket`).
//! - **Kalshi**: `engine/exchanges/kalshi/src/sports.rs` (next PR).
//! - **ESPN**: `engine/sports/src/providers/espn/` (later PR — pure-data
//!   provider lives inside this crate since it has no other home).

pub mod manifest;
pub mod manifests;
pub mod provider;

pub use manifest::{FieldMapping, SportsManifest, Transform};
pub use manifests::POLYMARKET_SPORTS_MANIFEST;
pub use provider::{GameStateStream, SportsCapabilities, SportsProvider};
