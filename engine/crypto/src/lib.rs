use std::sync::Arc;

use futures::StreamExt;
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, warn};

use px_core::error::WebSocketError;
use px_core::models::{CryptoPrice, CryptoPriceSource};
use px_core::websocket::{
    AtomicWebSocketState, CryptoPriceStream, WebSocketState, WS_CRYPTO_PING_INTERVAL,
    WS_MAX_RECONNECT_ATTEMPTS, WS_RECONNECT_BASE_DELAY, WS_RECONNECT_MAX_DELAY,
};

const CRYPTO_WS_URL: &str = "wss://ws-live-data.polymarket.com";
const BROADCAST_CAPACITY: usize = 16_384;

/// A stored subscription for replay on reconnect.
#[derive(Debug, Clone)]
struct Subscription {
    source: CryptoPriceSource,
    symbols: Vec<String>,
}

/// Outer envelope from the WebSocket.
#[derive(serde::Deserialize)]
struct Envelope {
    topic: String,
    #[serde(default)]
    #[allow(dead_code)]
    r#type: String,
    #[allow(dead_code)]
    #[serde(default)]
    timestamp: Option<u64>,
    payload: serde_json::Value,
}

/// Payload shape: `{ symbol, timestamp, value }`.
#[derive(serde::Deserialize)]
struct PricePayload {
    symbol: String,
    timestamp: u64,
    value: f64,
}

fn topic_for_source(source: CryptoPriceSource) -> &'static str {
    match source {
        CryptoPriceSource::Binance => "crypto_prices",
        CryptoPriceSource::Chainlink => "crypto_prices_chainlink",
    }
}

fn source_from_topic(topic: &str) -> Option<CryptoPriceSource> {
    match topic {
        "crypto_prices" => Some(CryptoPriceSource::Binance),
        "crypto_prices_chainlink" => Some(CryptoPriceSource::Chainlink),
        _ => None,
    }
}

fn build_subscribe_msg(source: CryptoPriceSource, symbols: &[String]) -> String {
    let topic = topic_for_source(source);
    if symbols.is_empty() {
        return serde_json::json!({
            "action": "subscribe",
            "subscriptions": [{
                "topic": topic,
                "type": "*",
                "filters": "",
            }]
        })
        .to_string();
    }
    let subs: Vec<serde_json::Value> = symbols
        .iter()
        .map(|sym| {
            let filter = serde_json::json!({ "symbol": sym }).to_string();
            serde_json::json!({
                "topic": topic,
                "type": "*",
                "filters": filter,
            })
        })
        .collect();
    serde_json::json!({
        "action": "subscribe",
        "subscriptions": subs,
    })
    .to_string()
}

fn build_unsubscribe_msg(source: CryptoPriceSource, symbols: &[String]) -> String {
    let topic = topic_for_source(source);
    if symbols.is_empty() {
        return serde_json::json!({
            "action": "unsubscribe",
            "subscriptions": [{
                "topic": topic,
                "type": "*",
                "filters": "",
            }]
        })
        .to_string();
    }
    let subs: Vec<serde_json::Value> = symbols
        .iter()
        .map(|sym| {
            let filter = serde_json::json!({ "symbol": sym }).to_string();
            serde_json::json!({
                "topic": topic,
                "type": "*",
                "filters": filter,
            })
        })
        .collect();
    serde_json::json!({
        "action": "unsubscribe",
        "subscriptions": subs,
    })
    .to_string()
}

/// Streams real-time crypto prices from a WebSocket feed.
///
/// Supports Binance and Chainlink price sources. Requires explicit subscribe/unsubscribe
/// messages and client-initiated PING every 5 seconds.
pub struct CryptoPriceWebSocket {
    state: Arc<AtomicWebSocketState>,
    sender: broadcast::Sender<Result<CryptoPrice, WebSocketError>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    subscriptions: Arc<Mutex<Vec<Subscription>>>,
}

impl CryptoPriceWebSocket {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            sender,
            write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn state(&self) -> WebSocketState {
        self.state.load()
    }

    pub fn stream(&self) -> CryptoPriceStream {
        let rx = self.sender.subscribe();
        Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|result| async move { result.ok() }),
        )
    }

    /// Subscribe to crypto prices for the given source and symbols.
    /// Empty symbols subscribes to all available symbols.
    pub async fn subscribe(
        &self,
        source: CryptoPriceSource,
        symbols: &[String],
    ) -> Result<(), WebSocketError> {
        let msg = build_subscribe_msg(source, symbols);
        let write_tx = self.write_tx.lock().await;
        if let Some(ref tx) = *write_tx {
            tx.unbounded_send(Message::Text(msg))
                .map_err(|e| WebSocketError::Connection(e.to_string()))?;
        } else {
            return Err(WebSocketError::Connection("not connected".to_string()));
        }

        let mut subs = self.subscriptions.lock().await;
        subs.push(Subscription {
            source,
            symbols: symbols.to_vec(),
        });

        Ok(())
    }

    /// Unsubscribe from crypto prices for the given source and symbols.
    pub async fn unsubscribe(
        &self,
        source: CryptoPriceSource,
        symbols: &[String],
    ) -> Result<(), WebSocketError> {
        let msg = build_unsubscribe_msg(source, symbols);
        let write_tx = self.write_tx.lock().await;
        if let Some(ref tx) = *write_tx {
            tx.unbounded_send(Message::Text(msg))
                .map_err(|e| WebSocketError::Connection(e.to_string()))?;
        }

        let mut subs = self.subscriptions.lock().await;
        subs.retain(|s| !(s.source == source && s.symbols == symbols));

        Ok(())
    }

    pub async fn connect(&mut self) -> Result<(), WebSocketError> {
        self.state.store(WebSocketState::Connecting);

        let (ws_stream, _) = connect_async(CRYPTO_WS_URL)
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
        let subscriptions = self.subscriptions.clone();

        tokio::spawn(async move {
            let write_future = rx.map(Ok).forward(write);

            let read_future = {
                let sender = sender.clone();
                let write_tx_clone = write_tx_clone.clone();
                async move {
                    let mut read = read;
                    while let Some(msg) = read.next().await {
                        handle_message(msg, &sender, &write_tx_clone).await;
                    }
                }
            };

            let ping_future = {
                let write_tx_clone = write_tx_clone.clone();
                async move {
                    let mut interval = tokio::time::interval(WS_CRYPTO_PING_INTERVAL);
                    loop {
                        interval.tick().await;
                        let tx = write_tx_clone.lock().await;
                        if let Some(ref tx) = *tx {
                            if tx.unbounded_send(Message::Text("PING".into())).is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
            };

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = ping_future => {},
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
                    "reconnecting crypto websocket"
                );
                tokio::time::sleep(delay).await;

                match connect_async(CRYPTO_WS_URL).await {
                    Ok((new_ws, _)) => {
                        let (new_write, new_read) = new_ws.split();
                        let (new_tx, new_rx) = futures::channel::mpsc::unbounded::<Message>();

                        {
                            let mut wtx = write_tx_clone.lock().await;
                            *wtx = Some(new_tx);
                        }

                        state.store(WebSocketState::Connected);
                        attempt = 0;

                        // Replay stored subscriptions
                        {
                            let subs = subscriptions.lock().await;
                            let wtx = write_tx_clone.lock().await;
                            if let Some(ref tx) = *wtx {
                                for sub in subs.iter() {
                                    let msg = build_subscribe_msg(sub.source, &sub.symbols);
                                    let _ = tx.unbounded_send(Message::Text(msg));
                                }
                            }
                        }

                        let sender_clone = sender.clone();
                        let wtx_clone = write_tx_clone.clone();

                        let write_future = new_rx.map(Ok).forward(new_write);

                        let read_future = {
                            let sender = sender_clone;
                            let write_tx = wtx_clone.clone();
                            async move {
                                let mut read = new_read;
                                while let Some(msg) = read.next().await {
                                    handle_message(msg, &sender, &write_tx).await;
                                }
                            }
                        };

                        let ping_future = {
                            let write_tx = wtx_clone;
                            async move {
                                let mut interval = tokio::time::interval(WS_CRYPTO_PING_INTERVAL);
                                loop {
                                    interval.tick().await;
                                    let tx = write_tx.lock().await;
                                    if let Some(ref tx) = *tx {
                                        if tx.unbounded_send(Message::Text("PING".into())).is_err()
                                        {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }
                            }
                        };

                        tokio::select! {
                            _ = write_future => {},
                            _ = read_future => {},
                            _ = ping_future => {},
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

impl Default for CryptoPriceWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

async fn handle_message(
    msg: Result<Message, tokio_tungstenite::tungstenite::Error>,
    sender: &broadcast::Sender<Result<CryptoPrice, WebSocketError>>,
    write_tx: &Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
) {
    match msg {
        Ok(Message::Text(text)) => {
            // Ignore PONG responses
            if text == "PONG" {
                return;
            }

            let envelope: Envelope = match serde_json::from_str(&text) {
                Ok(e) => e,
                Err(e) => {
                    debug!(raw = %text, error = %e, "skipping non-envelope message");
                    return;
                }
            };

            let source = match source_from_topic(&envelope.topic) {
                Some(s) => s,
                None => {
                    debug!(topic = %envelope.topic, "skipping unknown topic");
                    return;
                }
            };

            let payload: PricePayload = match serde_json::from_value(envelope.payload) {
                Ok(p) => p,
                Err(e) => {
                    debug!(error = %e, "skipping malformed price payload");
                    return;
                }
            };

            let price = CryptoPrice {
                symbol: payload.symbol,
                timestamp: payload.timestamp,
                value: payload.value,
                source,
            };

            let _ = sender.send(Ok(price));
        }
        Ok(Message::Ping(data)) => {
            if let Some(ref tx) = *write_tx.lock().await {
                let _ = tx.unbounded_send(Message::Pong(data));
            }
        }
        Ok(Message::Close(_)) | Err(_) => {}
        _ => {}
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
    fn deserialize_binance_envelope() {
        let data = json!({
            "topic": "crypto_prices",
            "type": "update",
            "timestamp": 1700000000,
            "payload": {
                "symbol": "btcusdt",
                "timestamp": 1700000000u64,
                "value": 43250.5
            }
        });

        let envelope: Envelope = serde_json::from_value(data).expect("should deserialize");
        assert_eq!(envelope.topic, "crypto_prices");

        let source = source_from_topic(&envelope.topic).unwrap();
        assert_eq!(source, CryptoPriceSource::Binance);

        let payload: PricePayload =
            serde_json::from_value(envelope.payload).expect("should deserialize payload");
        assert_eq!(payload.symbol, "btcusdt");
        assert_eq!(payload.timestamp, 1700000000);
        assert!((payload.value - 43250.5).abs() < f64::EPSILON);
    }

    #[test]
    fn deserialize_chainlink_envelope() {
        let data = json!({
            "topic": "crypto_prices_chainlink",
            "type": "update",
            "timestamp": 1700000001,
            "payload": {
                "symbol": "eth/usd",
                "timestamp": 1700000001u64,
                "value": 2250.75
            }
        });

        let envelope: Envelope = serde_json::from_value(data).expect("should deserialize");
        assert_eq!(envelope.topic, "crypto_prices_chainlink");

        let source = source_from_topic(&envelope.topic).unwrap();
        assert_eq!(source, CryptoPriceSource::Chainlink);

        let payload: PricePayload =
            serde_json::from_value(envelope.payload).expect("should deserialize payload");
        assert_eq!(payload.symbol, "eth/usd");
        assert!((payload.value - 2250.75).abs() < f64::EPSILON);
    }

    #[test]
    fn serialize_binance_subscribe() {
        let msg = build_subscribe_msg(
            CryptoPriceSource::Binance,
            &["btcusdt".into(), "ethusdt".into()],
        );
        let parsed: serde_json::Value = serde_json::from_str(&msg).expect("valid JSON");
        assert_eq!(parsed["action"], "subscribe");
        // One subscription entry per symbol
        assert_eq!(parsed["subscriptions"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["subscriptions"][0]["topic"], "crypto_prices");
        assert_eq!(parsed["subscriptions"][0]["type"], "*");
        let f0: serde_json::Value =
            serde_json::from_str(parsed["subscriptions"][0]["filters"].as_str().unwrap())
                .expect("filters should be valid JSON");
        assert_eq!(f0["symbol"], "btcusdt");
        let f1: serde_json::Value =
            serde_json::from_str(parsed["subscriptions"][1]["filters"].as_str().unwrap())
                .expect("filters should be valid JSON");
        assert_eq!(f1["symbol"], "ethusdt");
    }

    #[test]
    fn serialize_chainlink_subscribe() {
        let msg = build_subscribe_msg(CryptoPriceSource::Chainlink, &["eth/usd".into()]);
        let parsed: serde_json::Value = serde_json::from_str(&msg).expect("valid JSON");
        assert_eq!(parsed["action"], "subscribe");
        assert_eq!(
            parsed["subscriptions"][0]["topic"],
            "crypto_prices_chainlink"
        );
        assert_eq!(parsed["subscriptions"][0]["type"], "*");
        let filters: serde_json::Value =
            serde_json::from_str(parsed["subscriptions"][0]["filters"].as_str().unwrap())
                .expect("filters should be valid JSON");
        assert_eq!(filters["symbol"], "eth/usd");
    }

    #[test]
    fn serialize_binance_subscribe_all() {
        let msg = build_subscribe_msg(CryptoPriceSource::Binance, &[]);
        let parsed: serde_json::Value = serde_json::from_str(&msg).expect("valid JSON");
        assert_eq!(parsed["subscriptions"][0]["type"], "*");
        assert_eq!(parsed["subscriptions"][0]["filters"], "");
    }

    #[test]
    fn serialize_unsubscribe() {
        let msg = build_unsubscribe_msg(CryptoPriceSource::Binance, &["btcusdt".into()]);
        let parsed: serde_json::Value = serde_json::from_str(&msg).expect("valid JSON");
        assert_eq!(parsed["action"], "unsubscribe");
        assert_eq!(parsed["subscriptions"][0]["topic"], "crypto_prices");
        let filters: serde_json::Value =
            serde_json::from_str(parsed["subscriptions"][0]["filters"].as_str().unwrap())
                .expect("filters should be valid JSON");
        assert_eq!(filters["symbol"], "btcusdt");
    }

    #[test]
    fn ping_is_not_valid_price() {
        let result = serde_json::from_str::<Envelope>("PING");
        assert!(result.is_err());
    }

    #[test]
    fn unknown_topic_returns_none() {
        assert!(source_from_topic("unknown_topic").is_none());
    }

    #[test]
    fn topic_round_trip() {
        assert_eq!(
            source_from_topic(topic_for_source(CryptoPriceSource::Binance)),
            Some(CryptoPriceSource::Binance)
        );
        assert_eq!(
            source_from_topic(topic_for_source(CryptoPriceSource::Chainlink)),
            Some(CryptoPriceSource::Chainlink)
        );
    }
}
