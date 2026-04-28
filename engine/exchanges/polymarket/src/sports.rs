//! Polymarket implementation of the unified `SportsProvider` trait.
//!
//! Three surfaces:
//! - **Catalog (HTTP, gamma API):** `list_sports` via `/sports`, `list_games` via
//!   `/events/results`, `get_game` via `/events/{id}`. No auth.
//! - **Live state (WebSocket):** `subscribe_game_state` wraps the
//!   [`SportsWebSocket`](crate::SportsWebSocket) driver and transforms each
//!   raw `SportResult` into a unified `GameState`.
//!
//! The JSON-to-unified-schema mapping lives in
//! `engine/sports/src/manifests/polymarket.rs` and is enforced by the
//! manifest-coverage drift gate at `maintenance/tests/manifest_coverage.rs`.

use std::pin::Pin;

use chrono::Utc;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use px_core::error::OpenPxError;
use px_core::models::{Game, GameFilter, GameId, GameState, GameStatus, Score, Sport, SportResult};
use px_sports::{
    manifests::POLYMARKET_SPORTS_MANIFEST, GameStateStream, SportsCapabilities, SportsManifest,
    SportsProvider,
};

use crate::exchange::Polymarket;
use crate::sports_ws::SportsWebSocket;

const STATE_CHANNEL_CAPACITY: usize = 1024;

impl SportsProvider for Polymarket {
    fn id(&self) -> &'static str {
        "polymarket"
    }

    fn name(&self) -> &'static str {
        "Polymarket"
    }

    fn capabilities(&self) -> SportsCapabilities {
        SportsCapabilities {
            id: "polymarket",
            name: "Polymarket",
            has_list_sports: true,
            has_list_games: true,
            has_get_game: true,
            has_live_state_stream: true,
            has_teams: true, // first-class /teams endpoint (not yet on this trait)
            has_schedule: true,
            has_standings: false,
            has_athletes: false,
            has_news: false,
            has_rankings: false,
            has_play_by_play: false,
        }
    }

    fn manifest(&self) -> &'static SportsManifest {
        &POLYMARKET_SPORTS_MANIFEST
    }

    async fn list_sports(&self) -> Result<Vec<Sport>, OpenPxError> {
        let raw: serde_json::Value = self
            .client
            .get_gamma("/sports")
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let arr = raw.as_array().cloned().unwrap_or_default();
        Ok(arr.into_iter().filter_map(parse_sport).collect())
    }

    async fn list_games(
        &self,
        filter: GameFilter,
    ) -> Result<(Vec<Game>, Option<String>), OpenPxError> {
        let limit = filter.limit.unwrap_or(50).clamp(1, 500);
        let mut endpoint = format!("/events/results?limit={limit}");
        if let Some(c) = filter.cursor.as_deref().filter(|c| !c.is_empty()) {
            endpoint.push_str(&format!("&after_cursor={c}"));
        }
        if let Some(league) = filter.league.as_deref() {
            endpoint.push_str(&format!("&league={league}"));
        }

        let raw: serde_json::Value = self
            .client
            .get_gamma(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let (events, next_cursor) = match &raw {
            serde_json::Value::Array(a) => (a.clone(), None),
            serde_json::Value::Object(o) => (
                o.get("events")
                    .or_else(|| o.get("data"))
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default(),
                o.get("next_cursor")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from),
            ),
            _ => (Vec::new(), None),
        };

        let games = events.into_iter().filter_map(parse_game).collect();
        Ok((games, next_cursor))
    }

    async fn get_game(&self, id: &GameId) -> Result<Game, OpenPxError> {
        let endpoint = format!("/events/{}", id.as_str());
        let raw: serde_json::Value = self
            .client
            .get_gamma(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;
        parse_game(raw).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "could not parse sports event: {id}"
            )))
        })
    }

    fn subscribe_game_state(&self) -> GameStateStream {
        let (tx, rx) = mpsc::channel::<Result<GameState, px_core::error::WebSocketError>>(
            STATE_CHANNEL_CAPACITY,
        );

        // Spawn a task that owns the SportsWebSocket for the lifetime of the
        // stream. When the receiver is dropped, send() returns Err and the
        // task exits, dropping the WS (which triggers its shutdown_rx).
        tokio::spawn(async move {
            let mut ws = SportsWebSocket::new();
            if let Err(e) = ws.connect().await {
                let _ = tx.send(Err(e)).await;
                return;
            }

            let mut inner = ws.stream();
            while let Some(item) = inner.next().await {
                let send = match item {
                    Ok(sr) => Ok(sport_result_to_game_state(sr)),
                    Err(e) => Err(e),
                };
                if tx.send(send).await.is_err() {
                    break;
                }
            }
        });

        Pin::from(Box::new(ReceiverStream::new(rx)))
    }
}

fn parse_sport(value: serde_json::Value) -> Option<Sport> {
    let obj = value.as_object()?;
    let id = obj
        .get("sport")
        .or_else(|| obj.get("id"))
        .or_else(|| obj.get("slug"))
        .and_then(|v| v.as_str())?
        .to_string();
    let name = obj
        .get("name")
        .or_else(|| obj.get("displayName"))
        .or_else(|| obj.get("sport"))
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();
    Some(Sport { id, name })
}

fn parse_game(value: serde_json::Value) -> Option<Game> {
    let obj = value.as_object()?;

    let id = obj
        .get("gameId")
        .or_else(|| obj.get("id"))
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })?;

    let league = obj
        .get("leagueAbbreviation")
        .or_else(|| obj.get("league"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let home_team = obj
        .get("homeTeam")
        .and_then(|v| v.as_str())
        .map(String::from);
    let away_team = obj
        .get("awayTeam")
        .and_then(|v| v.as_str())
        .map(String::from);

    let start_time = obj
        .get("gameStartTime")
        .or_else(|| obj.get("eventDate"))
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let raw_status = obj
        .get("status")
        .or_else(|| obj.get("gameStatus"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let status = raw_status
        .as_deref()
        .and_then(|s| POLYMARKET_SPORTS_MANIFEST.map_status(s))
        .unwrap_or(GameStatus::Unknown);

    Some(Game {
        id: GameId(id),
        league,
        home_team,
        away_team,
        start_time,
        status,
        raw_status,
        provider: "polymarket".into(),
    })
}

fn sport_result_to_game_state(sr: SportResult) -> GameState {
    let status = POLYMARKET_SPORTS_MANIFEST
        .map_status(&sr.status)
        .unwrap_or(GameStatus::Unknown);

    GameState {
        game_id: GameId(sr.game_id.to_string()),
        status,
        raw_status: Some(sr.status),
        score: sr.score.as_deref().map(parse_score),
        period: sr.period,
        clock: sr.elapsed,
        live: sr.live,
        ended: sr.ended,
        updated_at: sr.finished_timestamp.unwrap_or_else(Utc::now),
    }
}

/// Best-effort parse of the Polymarket score string. Standard
/// head-to-head sports ship `"home-away"` (e.g. `"3-16"`); exotic
/// formats (esports `"0-0|2-0|Bo3"`, golf cumulative) fall through to
/// `raw`-only.
fn parse_score(raw: &str) -> Score {
    if let Some((h, a)) = raw.split_once('-') {
        if !h.contains('|') && !a.contains('|') {
            if let (Ok(home), Ok(away)) = (h.trim().parse::<u32>(), a.trim().parse::<u32>()) {
                return Score {
                    home: Some(home),
                    away: Some(away),
                    raw: Some(raw.to_string()),
                };
            }
        }
    }
    Score {
        home: None,
        away: None,
        raw: Some(raw.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_standard_score() {
        let s = parse_score("3-16");
        assert_eq!(s.home, Some(3));
        assert_eq!(s.away, Some(16));
        assert_eq!(s.raw.as_deref(), Some("3-16"));
    }

    #[test]
    fn parses_two_digit_score() {
        let s = parse_score("110-108");
        assert_eq!(s.home, Some(110));
        assert_eq!(s.away, Some(108));
    }

    #[test]
    fn esports_score_falls_through_to_raw() {
        let s = parse_score("000-000|2-0|Bo3");
        assert!(s.home.is_none());
        assert!(s.away.is_none());
        assert_eq!(s.raw.as_deref(), Some("000-000|2-0|Bo3"));
    }

    #[test]
    fn malformed_score_falls_through() {
        let s = parse_score("--");
        assert!(s.home.is_none());
        assert!(s.away.is_none());
    }

    #[test]
    fn maps_in_progress_to_live() {
        let sr = SportResult {
            game_id: 1,
            league_abbreviation: "nfl".into(),
            slug: "x".into(),
            home_team: "A".into(),
            away_team: "B".into(),
            status: "In Progress".into(),
            score: Some("3-0".into()),
            period: None,
            elapsed: None,
            live: true,
            ended: false,
            turn: None,
            finished_timestamp: None,
        };
        let gs = sport_result_to_game_state(sr);
        assert_eq!(gs.status, GameStatus::Live);
        assert_eq!(gs.raw_status.as_deref(), Some("In Progress"));
        assert!(gs.live);
        assert_eq!(gs.score.as_ref().unwrap().home, Some(3));
    }

    #[test]
    fn unknown_status_preserves_raw() {
        let sr = SportResult {
            game_id: 1,
            league_abbreviation: "nfl".into(),
            slug: "x".into(),
            home_team: "A".into(),
            away_team: "B".into(),
            status: "WeirdStatus".into(),
            score: None,
            period: None,
            elapsed: None,
            live: false,
            ended: false,
            turn: None,
            finished_timestamp: None,
        };
        let gs = sport_result_to_game_state(sr);
        assert_eq!(gs.status, GameStatus::Unknown);
        assert_eq!(gs.raw_status.as_deref(), Some("WeirdStatus"));
    }

    #[test]
    fn parses_game_from_event_json() {
        let v = json!({
            "gameId": "401671793",
            "leagueAbbreviation": "nfl",
            "homeTeam": "Kansas City Chiefs",
            "awayTeam": "Buffalo Bills",
            "gameStartTime": "2025-01-26T18:30:00Z",
            "status": "Scheduled",
        });
        let g = parse_game(v).expect("should parse");
        assert_eq!(g.id.as_str(), "401671793");
        assert_eq!(g.league, "nfl");
        assert_eq!(g.home_team.as_deref(), Some("Kansas City Chiefs"));
        assert_eq!(g.status, GameStatus::Scheduled);
        assert_eq!(g.provider, "polymarket");
        assert!(g.start_time.is_some());
    }
}
