use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, warn};

use px_core::error::WebSocketError;
use px_core::models::SportResult;
use px_core::websocket::{
    AtomicWebSocketState, SportsStream, WebSocketState, WS_MAX_RECONNECT_ATTEMPTS,
    WS_RECONNECT_BASE_DELAY, WS_RECONNECT_MAX_DELAY,
};

const SPORTS_WS_URL: &str = "wss://sports-api.polymarket.com/ws";
const BROADCAST_CAPACITY: usize = 16_384;

/// Streams real-time sports scores from the Polymarket Sports WebSocket.
///
/// No auth, no subscription messages — connect and receive all active events.
/// Uses TEXT-level `"ping"`/`"pong"` keepalive (not WebSocket protocol pings).
pub struct SportsWebSocket {
    state: Arc<AtomicWebSocketState>,
    sender: broadcast::Sender<Result<SportResult, WebSocketError>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl SportsWebSocket {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            sender,
            write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn state(&self) -> WebSocketState {
        self.state.load()
    }

    pub fn stream(&self) -> SportsStream {
        let rx = self.sender.subscribe();
        Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|result| async move { result.ok() }),
        )
    }

    pub async fn connect(&mut self) -> Result<(), WebSocketError> {
        self.state.store(WebSocketState::Connecting);

        let (ws_stream, _) = connect_async(SPORTS_WS_URL)
            .await
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;

        let (write, read) = ws_stream.split();
        let (tx, rx) = futures::channel::mpsc::unbounded::<Message>();

        {
            let mut write_tx = self.write_tx.lock().await;
            *write_tx = Some(tx);
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        {
            let mut stx = self.shutdown_tx.lock().await;
            *stx = Some(shutdown_tx);
        }

        let state = self.state.clone();
        let sender = self.sender.clone();
        let write_tx_clone = self.write_tx.clone();

        tokio::spawn(async move {
            let write_future = rx.map(Ok).forward(write);

            let read_future = async {
                let mut read = read;
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // TEXT-level ping/pong — short-circuit before JSON parse
                            if text == "ping" {
                                if let Some(ref tx) = *write_tx_clone.lock().await {
                                    let _ = tx.unbounded_send(Message::Text("pong".into()));
                                }
                                continue;
                            }

                            match serde_json::from_str::<SportResult>(&text) {
                                Ok(result) => {
                                    let _ = sender.send(Ok(result));
                                }
                                Err(e) => {
                                    debug!(raw = %text, error = %e, "skipping non-SportResult message");
                                }
                            }
                        }
                        Ok(Message::Ping(data)) => {
                            if let Some(ref tx) = *write_tx_clone.lock().await {
                                let _ = tx.unbounded_send(Message::Pong(data));
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }
            };

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = shutdown_rx => {},
            }

            if state.load() == WebSocketState::Closed {
                return;
            }
            state.store(WebSocketState::Disconnected);

            // Auto-reconnect with exponential backoff
            let mut attempt = 1u32;
            while attempt <= WS_MAX_RECONNECT_ATTEMPTS {
                state.store(WebSocketState::Reconnecting);

                let delay = calculate_reconnect_delay(attempt);
                warn!(
                    attempt,
                    delay_ms = delay.as_millis() as u64,
                    "reconnecting sports websocket"
                );
                tokio::time::sleep(delay).await;

                match connect_async(SPORTS_WS_URL).await {
                    Ok((new_ws, _)) => {
                        let (new_write, new_read) = new_ws.split();
                        let (new_tx, new_rx) = futures::channel::mpsc::unbounded::<Message>();

                        {
                            let mut wtx = write_tx_clone.lock().await;
                            *wtx = Some(new_tx);
                        }

                        state.store(WebSocketState::Connected);
                        attempt = 0;

                        let sender_clone = sender.clone();
                        let wtx_clone = write_tx_clone.clone();

                        let write_future = new_rx.map(Ok).forward(new_write);
                        let read_future = async {
                            let mut read = new_read;
                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if text == "ping" {
                                            if let Some(ref tx) = *wtx_clone.lock().await {
                                                let _ =
                                                    tx.unbounded_send(Message::Text("pong".into()));
                                            }
                                            continue;
                                        }
                                        match serde_json::from_str::<SportResult>(&text) {
                                            Ok(result) => {
                                                let _ = sender_clone.send(Ok(result));
                                            }
                                            Err(e) => {
                                                debug!(raw = %text, error = %e, "skipping non-SportResult message");
                                            }
                                        }
                                    }
                                    Ok(Message::Ping(data)) => {
                                        if let Some(ref tx) = *wtx_clone.lock().await {
                                            let _ = tx.unbounded_send(Message::Pong(data));
                                        }
                                    }
                                    Ok(Message::Close(_)) => break,
                                    Err(_) => break,
                                    _ => {}
                                }
                            }
                        };

                        tokio::select! {
                            _ = write_future => {},
                            _ = read_future => {},
                        }

                        if state.load() == WebSocketState::Closed {
                            return;
                        }

                        attempt += 1;
                    }
                    Err(_) => {
                        attempt += 1;
                    }
                }
            }

            state.store(WebSocketState::Disconnected);
        });

        self.state.store(WebSocketState::Connected);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        self.state.store(WebSocketState::Closed);
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
        Ok(())
    }
}

impl Default for SportsWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

fn calculate_reconnect_delay(attempt: u32) -> std::time::Duration {
    let delay = WS_RECONNECT_BASE_DELAY.as_millis() as f64 * 1.5_f64.powi(attempt as i32);
    let delay = delay.min(WS_RECONNECT_MAX_DELAY.as_millis() as f64) as u64;
    std::time::Duration::from_millis(delay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_nfl_in_progress() {
        let data = json!({
            "gameId": 401671793,
            "leagueAbbreviation": "nfl",
            "slug": "nfl-buf-kc-2025-01-26",
            "homeTeam": "Kansas City Chiefs",
            "awayTeam": "Buffalo Bills",
            "status": "In Progress",
            "score": "3-16",
            "period": "2nd Quarter",
            "elapsed": "12:34",
            "live": true,
            "ended": false,
            "turn": null,
            "finished_timestamp": null
        });

        let result: SportResult = serde_json::from_value(data).expect("should deserialize");
        assert_eq!(result.game_id, 401671793);
        assert_eq!(result.league_abbreviation, "nfl");
        assert_eq!(result.home_team, "Kansas City Chiefs");
        assert_eq!(result.away_team, "Buffalo Bills");
        assert_eq!(result.status, "In Progress");
        assert_eq!(result.score.as_deref(), Some("3-16"));
        assert_eq!(result.period.as_deref(), Some("2nd Quarter"));
        assert_eq!(result.elapsed.as_deref(), Some("12:34"));
        assert!(result.live);
        assert!(!result.ended);
        assert!(result.turn.is_none());
        assert!(result.finished_timestamp.is_none());
    }

    #[test]
    fn deserialize_esports_finished() {
        let data = json!({
            "gameId": 99001,
            "leagueAbbreviation": "lol-lck",
            "slug": "lol-lck-t1-geng-2025-02-01",
            "homeTeam": "T1",
            "awayTeam": "Gen.G",
            "status": "Final",
            "score": "000-000|2-0|Bo3",
            "period": null,
            "elapsed": null,
            "live": false,
            "ended": true,
            "turn": null,
            "finished_timestamp": "2025-02-01T14:30:00Z"
        });

        let result: SportResult = serde_json::from_value(data).expect("should deserialize");
        assert_eq!(result.game_id, 99001);
        assert_eq!(result.league_abbreviation, "lol-lck");
        assert!(result.ended);
        assert!(!result.live);
        assert!(result.finished_timestamp.is_some());
        assert_eq!(result.score.as_deref(), Some("000-000|2-0|Bo3"));
    }

    #[test]
    fn finished_timestamp_none_when_not_ended() {
        let data = json!({
            "gameId": 12345,
            "leagueAbbreviation": "nba",
            "slug": "nba-lal-bos-2025-01-30",
            "homeTeam": "Boston Celtics",
            "awayTeam": "Los Angeles Lakers",
            "status": "Halftime",
            "score": "55-48",
            "period": "Halftime",
            "elapsed": null,
            "live": true,
            "ended": false
        });

        let result: SportResult = serde_json::from_value(data).expect("should deserialize");
        assert!(result.finished_timestamp.is_none());
        assert!(!result.ended);
    }

    #[test]
    fn ping_is_not_valid_sport_result() {
        let result = serde_json::from_str::<SportResult>("ping");
        assert!(result.is_err());
    }

    #[test]
    fn optional_fields_absent() {
        let data = json!({
            "gameId": 77777,
            "leagueAbbreviation": "mlb",
            "slug": "mlb-nyy-bos-2025-04-01",
            "homeTeam": "Boston Red Sox",
            "awayTeam": "New York Yankees",
            "status": "Scheduled",
            "live": false,
            "ended": false
        });

        let result: SportResult = serde_json::from_value(data).expect("should deserialize");
        assert!(result.score.is_none());
        assert!(result.period.is_none());
        assert!(result.elapsed.is_none());
        assert!(result.turn.is_none());
        assert!(result.finished_timestamp.is_none());
    }
}
