//! `subscribe_game_state` — an HTTP-polling adapter that yields `GameState`
//! deltas as a `Stream`. ESPN has no public WebSocket; polling the scoreboard
//! at the configured cadence is the standard approach.

use std::pin::Pin;

use chrono::Utc;
use futures::Stream;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use px_core::error::OpenPxError;

use super::scoreboard::{map_status, parse_score};
use super::{split_league, Espn};
use crate::types::{GameId, GameState, GameStatus};

const CHANNEL_CAPACITY: usize = 1024;

/// Stream of `GameState` updates emitted by the ESPN poller.
pub type GameStateStream = Pin<Box<dyn Stream<Item = Result<GameState, OpenPxError>> + Send>>;

impl Espn {
    /// Subscribe to live `GameState` updates for a league. Yields one item
    /// per game per poll tick, only when state changed since the last tick
    /// (suppresses duplicates).
    pub fn subscribe_game_state(&self, league: &str) -> GameStateStream {
        let (tx, rx) = mpsc::channel::<Result<GameState, OpenPxError>>(CHANNEL_CAPACITY);
        let espn = self.clone();
        let league = league.to_string();
        let interval = espn.config.poll_interval;

        tokio::spawn(async move {
            let mut last_seen: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();

            loop {
                let (sport, lg) = split_league(&league);
                let path = format!("/{sport}/{lg}/scoreboard");
                let raw: Result<Value, _> = espn.get(&path).await;

                match raw {
                    Ok(v) => {
                        let events = v
                            .get("events")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        for e in events {
                            if let Some(state) = parse_state(&e) {
                                // Suppress duplicates: hash of (status,score,clock,period).
                                let key = state.game_id.as_str().to_string();
                                let signature = format!(
                                    "{}|{}|{}|{}",
                                    state.raw_status.as_deref().unwrap_or(""),
                                    state
                                        .score
                                        .as_ref()
                                        .and_then(|s| s.raw.as_deref())
                                        .unwrap_or(""),
                                    state.period.as_deref().unwrap_or(""),
                                    state.clock.as_deref().unwrap_or(""),
                                );
                                if last_seen.get(&key) == Some(&signature) {
                                    continue;
                                }
                                last_seen.insert(key, signature);
                                if tx.send(Ok(state)).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if tx.send(Err(e)).await.is_err() {
                            return;
                        }
                    }
                }

                tokio::time::sleep(interval).await;
            }
        });

        Box::pin(ReceiverStream::new(rx))
    }
}

fn parse_state(event: &Value) -> Option<GameState> {
    let id = event.get("id")?.as_str()?.to_string();
    let raw_status = event
        .get("status")
        .and_then(|s| s.get("type"))
        .and_then(|t| t.get("name"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let status = raw_status
        .as_deref()
        .map(map_status)
        .unwrap_or(GameStatus::Unknown);

    let comp = event
        .get("competitions")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());

    let period = comp
        .and_then(|c| c.get("status"))
        .and_then(|s| s.get("period"))
        .map(|v| v.to_string().trim_matches('"').to_string())
        .filter(|s| !s.is_empty() && s != "null");

    let clock = comp
        .and_then(|c| c.get("status"))
        .and_then(|s| s.get("displayClock"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let live = matches!(status, GameStatus::Live);
    let ended = matches!(status, GameStatus::Final);

    Some(GameState {
        game_id: GameId(id),
        status,
        raw_status,
        score: parse_score(event),
        period,
        clock,
        live,
        ended,
        updated_at: Utc::now(),
    })
}
