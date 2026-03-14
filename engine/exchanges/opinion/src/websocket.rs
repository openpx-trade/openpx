use async_trait::async_trait;
use futures::StreamExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use px_core::{
    ActivityEvent, ActivityStream, ActivityTrade, AtomicWebSocketState, ChangeVec, FixedPrice,
    OrderBookWebSocket, OrderbookStream, OrderbookUpdate, PriceLevelChange, PriceLevelSide,
    WebSocketError, WebSocketState, WS_MAX_RECONNECT_ATTEMPTS, WS_RECONNECT_BASE_DELAY,
    WS_RECONNECT_MAX_DELAY,
};

use crate::config::OpinionConfig;

/// Opinion-specific heartbeat interval (30s per their WS protocol).
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

type OrderbookSender = broadcast::Sender<Result<OrderbookUpdate, WebSocketError>>;
type ActivitySender = broadcast::Sender<Result<ActivityEvent, WebSocketError>>;

pub struct OpinionWebSocket {
    config: OpinionConfig,
    state: Arc<AtomicWebSocketState>,
    subscriptions: Arc<RwLock<HashSet<String>>>,
    orderbook_senders: Arc<RwLock<HashMap<String, OrderbookSender>>>,
    activity_senders: Arc<RwLock<HashMap<String, ActivitySender>>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    auto_reconnect: bool,
    reconnect_attempts: Arc<Mutex<u32>>,
}

impl OpinionWebSocket {
    pub fn new(config: OpinionConfig) -> Result<Self, WebSocketError> {
        if config.api_key.is_none() {
            return Err(WebSocketError::Connection(
                "api_key required for Opinion WebSocket".into(),
            ));
        }
        Ok(Self {
            config,
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            orderbook_senders: Arc::new(RwLock::new(HashMap::new())),
            activity_senders: Arc::new(RwLock::new(HashMap::new())),
            write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect: true,
            reconnect_attempts: Arc::new(Mutex::new(0)),
        })
    }

    fn set_state(&self, new_state: WebSocketState) {
        self.state.store(new_state);
    }

    async fn reset_reconnect_attempts(&self) {
        let mut attempts = self.reconnect_attempts.lock().await;
        *attempts = 0;
    }

    fn ws_url(&self) -> Result<String, WebSocketError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| WebSocketError::Connection("api_key required".into()))?;
        Ok(format!("{}?apikey={}", self.config.ws_url, api_key))
    }

    async fn send_message(&self, msg: &str) -> Result<(), WebSocketError> {
        let tx = self.write_tx.lock().await;
        if let Some(ref sender) = *tx {
            sender
                .unbounded_send(Message::Text(msg.into()))
                .map_err(|e| WebSocketError::Connection(format!("send failed: {e}")))?;
        }
        Ok(())
    }

    async fn send_subscribe(&self, market_id: &str) -> Result<(), WebSocketError> {
        let numeric_id: i64 = market_id
            .parse()
            .map_err(|_| WebSocketError::Subscription(format!("invalid market_id: {market_id}")))?;

        let depth_msg = serde_json::json!({
            "action": "SUBSCRIBE",
            "channel": "market.depth.diff",
            "marketId": numeric_id
        });
        let trade_msg = serde_json::json!({
            "action": "SUBSCRIBE",
            "channel": "market.last.trade",
            "marketId": numeric_id
        });

        self.send_message(&depth_msg.to_string()).await?;
        self.send_message(&trade_msg.to_string()).await?;
        Ok(())
    }

    async fn send_unsubscribe(&self, market_id: &str) -> Result<(), WebSocketError> {
        let numeric_id: i64 = market_id
            .parse()
            .map_err(|_| WebSocketError::Subscription(format!("invalid market_id: {market_id}")))?;

        let depth_msg = serde_json::json!({
            "action": "UNSUBSCRIBE",
            "channel": "market.depth.diff",
            "marketId": numeric_id
        });
        let trade_msg = serde_json::json!({
            "action": "UNSUBSCRIBE",
            "channel": "market.last.trade",
            "marketId": numeric_id
        });

        self.send_message(&depth_msg.to_string()).await?;
        self.send_message(&trade_msg.to_string()).await?;
        Ok(())
    }

    async fn handle_message(&self, text: &str) {
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return,
        };

        let msg_type = value.get("msgType").and_then(|v| v.as_str()).unwrap_or("");

        match msg_type {
            "market.depth.diff" => self.handle_depth_diff(&value).await,
            "market.last.trade" => self.handle_last_trade(&value).await,
            _ => {}
        }
    }

    async fn handle_depth_diff(&self, value: &serde_json::Value) {
        let market_id = match value
            .get("marketId")
            .and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(String::from)))
        {
            Some(id) => id,
            None => return,
        };

        let outcome_side = value.get("outcomeSide").and_then(|v| v.as_i64()).unwrap_or(0);
        let side_str = value
            .get("side")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let price = match value.get("price").and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }) {
            Some(p) => p,
            None => return,
        };

        let size = value
            .get("size")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        // Map outcome_side + side to PriceLevelSide and adjusted price:
        // outcomeSide 1 (yes): "bids" -> Bid, "asks" -> Ask (price as-is)
        // outcomeSide 2 (no): "bids" -> Ask at 1.0 - price, "asks" -> Bid at 1.0 - price
        let fp = FixedPrice::from_f64(price);
        let (plc_side, plc_fp) = match outcome_side {
            1 => {
                if side_str == "bids" {
                    (PriceLevelSide::Bid, fp)
                } else {
                    (PriceLevelSide::Ask, fp)
                }
            }
            2 => {
                if side_str == "bids" {
                    (PriceLevelSide::Ask, fp.complement())
                } else {
                    (PriceLevelSide::Bid, fp.complement())
                }
            }
            _ => return,
        };

        let change = PriceLevelChange {
            side: plc_side,
            price: plc_fp,
            size,
        };

        let mut changes = ChangeVec::new();
        changes.push(change);

        let timestamp = value
            .get("timestamp")
            .or_else(|| value.get("ts"))
            .and_then(|v| v.as_i64())
            .and_then(chrono::DateTime::from_timestamp_millis);

        let senders = self.orderbook_senders.read().await;
        if let Some(sender) = senders.get(&market_id) {
            let _ = sender.send(Ok(OrderbookUpdate::Delta { changes, timestamp }));
        }
    }

    async fn handle_last_trade(&self, value: &serde_json::Value) {
        let market_id = match value
            .get("marketId")
            .and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(String::from)))
        {
            Some(id) => id,
            None => return,
        };

        let price = match value.get("price").and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        }) {
            Some(p) => p,
            None => return,
        };

        let size = value
            .get("shares")
            .or_else(|| value.get("size"))
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let side = value
            .get("side")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let token_id = value
            .get("tokenId")
            .or_else(|| value.get("token_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let timestamp = value
            .get("timestamp")
            .or_else(|| value.get("ts"))
            .and_then(|v| v.as_i64())
            .and_then(chrono::DateTime::from_timestamp_millis);

        let event = ActivityEvent::Trade(ActivityTrade {
            market_id: market_id.clone(),
            asset_id: token_id,
            trade_id: None,
            price,
            size,
            side,
            aggressor_side: None,
            outcome: None,
            timestamp,
            source_channel: Cow::Borrowed("market.last.trade"),
        });

        let senders = self.activity_senders.read().await;
        if let Some(sender) = senders.get(&market_id) {
            let _ = sender.send(Ok(event));
        }
    }

    async fn resubscribe_all(&self) -> Result<(), WebSocketError> {
        let markets: Vec<String> = {
            let subs = self.subscriptions.read().await;
            subs.iter().cloned().collect()
        };

        for market in markets {
            self.send_subscribe(&market).await?;
        }

        Ok(())
    }

    fn calculate_reconnect_delay(attempt: u32) -> Duration {
        let delay = WS_RECONNECT_BASE_DELAY.as_millis() as f64 * 1.5_f64.powi(attempt as i32);
        let delay = delay.min(WS_RECONNECT_MAX_DELAY.as_millis() as f64) as u64;
        Duration::from_millis(delay)
    }
}

#[async_trait]
impl OrderBookWebSocket for OpinionWebSocket {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Connecting);

        let url = self.ws_url()?;
        let (ws_stream, _) = connect_async(&url)
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
        let orderbook_senders = self.orderbook_senders.clone();
        let activity_senders = self.activity_senders.clone();
        let subscriptions = self.subscriptions.clone();
        let write_tx_clone = self.write_tx.clone();
        let reconnect_attempts_clone = self.reconnect_attempts.clone();
        let auto_reconnect = self.auto_reconnect;
        let config = self.config.clone();

        // Build a lightweight handle for the spawned task to dispatch messages
        let ws_handle = OpinionWebSocket {
            config: config.clone(),
            state: state.clone(),
            subscriptions: subscriptions.clone(),
            orderbook_senders: orderbook_senders.clone(),
            activity_senders: activity_senders.clone(),
            write_tx: write_tx_clone.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect,
            reconnect_attempts: reconnect_attempts_clone.clone(),
        };

        tokio::spawn(async move {
            let write_future = rx.map(Ok).forward(write);
            let read_future = async {
                let mut read = read;
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            ws_handle.handle_message(&text).await;
                        }
                        Ok(Message::Ping(data)) => {
                            if let Some(ref tx) = *ws_handle.write_tx.lock().await {
                                let _ = tx.unbounded_send(Message::Pong(data));
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }
            };

            let heartbeat_future = async {
                let mut heartbeat = interval(HEARTBEAT_INTERVAL);
                loop {
                    heartbeat.tick().await;
                    if let Some(ref tx) = *ws_handle.write_tx.lock().await {
                        let msg = r#"{"action":"HEARTBEAT"}"#;
                        let _ = tx.unbounded_send(Message::Text(msg.into()));
                    }
                }
            };

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = heartbeat_future => {},
                _ = shutdown_rx => {},
            }

            if state.load() == WebSocketState::Closed {
                return;
            }
            state.store(WebSocketState::Disconnected);

            if auto_reconnect {
                let mut attempt = {
                    let mut a = reconnect_attempts_clone.lock().await;
                    *a += 1;
                    *a
                };

                while attempt <= WS_MAX_RECONNECT_ATTEMPTS {
                    state.store(WebSocketState::Reconnecting);

                    let delay = OpinionWebSocket::calculate_reconnect_delay(attempt);
                    tokio::time::sleep(delay).await;

                    let url = match ws_handle.ws_url() {
                        Ok(u) => u,
                        Err(_) => break,
                    };

                    match connect_async(&url).await {
                        Ok((new_ws, _)) => {
                            let (new_write, new_read) = new_ws.split();
                            let (new_tx, new_rx) = futures::channel::mpsc::unbounded::<Message>();

                            {
                                let mut wtx = write_tx_clone.lock().await;
                                *wtx = Some(new_tx);
                            }

                            state.store(WebSocketState::Connected);

                            {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a = 0;
                            }

                            let _ = ws_handle.resubscribe_all().await;

                            let write_future = new_rx.map(Ok).forward(new_write);
                            let read_future = async {
                                let mut read = new_read;
                                while let Some(msg) = read.next().await {
                                    match msg {
                                        Ok(Message::Text(text)) => {
                                            ws_handle.handle_message(&text).await;
                                        }
                                        Ok(Message::Ping(data)) => {
                                            if let Some(ref tx) =
                                                *ws_handle.write_tx.lock().await
                                            {
                                                let _ = tx.unbounded_send(Message::Pong(data));
                                            }
                                        }
                                        Ok(Message::Close(_)) => break,
                                        Err(_) => break,
                                        _ => {}
                                    }
                                }
                            };

                            let heartbeat_future = async {
                                let mut heartbeat = interval(HEARTBEAT_INTERVAL);
                                loop {
                                    heartbeat.tick().await;
                                    if let Some(ref tx) = *ws_handle.write_tx.lock().await {
                                        let msg = r#"{"action":"HEARTBEAT"}"#;
                                        let _ = tx.unbounded_send(Message::Text(msg.into()));
                                    }
                                }
                            };

                            tokio::select! {
                                _ = write_future => {},
                                _ = read_future => {},
                                _ = heartbeat_future => {},
                            }

                            if state.load() == WebSocketState::Closed {
                                return;
                            }

                            attempt = {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a += 1;
                                *a
                            };
                        }
                        Err(_) => {
                            attempt = {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a += 1;
                                *a
                            };
                        }
                    }
                }

                state.store(WebSocketState::Disconnected);
            }
        });

        self.set_state(WebSocketState::Connected);
        self.reset_reconnect_attempts().await;
        self.resubscribe_all().await?;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Closed);
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    async fn subscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        let market = market_id.to_string();

        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(market.clone());
        }

        {
            let mut senders = self.orderbook_senders.write().await;
            if !senders.contains_key(&market) {
                let (tx, _) = broadcast::channel(16_384);
                senders.insert(market.clone(), tx);
            }
        }

        {
            let mut senders = self.activity_senders.write().await;
            if !senders.contains_key(&market) {
                let (tx, _) = broadcast::channel(16_384);
                senders.insert(market.clone(), tx);
            }
        }

        if self.state.load() == WebSocketState::Connected {
            self.send_subscribe(&market).await?;
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        let market = market_id.to_string();

        {
            let mut subs = self.subscriptions.write().await;
            subs.remove(&market);
        }

        if self.state.load() == WebSocketState::Connected {
            let _ = self.send_unsubscribe(&market).await;
        }

        {
            let mut senders = self.orderbook_senders.write().await;
            senders.remove(&market);
        }

        {
            let mut senders = self.activity_senders.write().await;
            senders.remove(&market);
        }

        Ok(())
    }

    fn state(&self) -> WebSocketState {
        self.state.load()
    }

    async fn orderbook_stream(
        &mut self,
        market_id: &str,
    ) -> Result<OrderbookStream, WebSocketError> {
        {
            let mut senders = self.orderbook_senders.write().await;
            if !senders.contains_key(market_id) {
                let (tx, _) = broadcast::channel(16_384);
                senders.insert(market_id.to_string(), tx);
            }
        }

        let senders = self.orderbook_senders.read().await;
        let sender = senders.get(market_id).ok_or_else(|| {
            WebSocketError::Subscription(format!("no orderbook sender for market: {market_id}"))
        })?;
        let rx = sender.subscribe();

        Ok(Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|result| async move { result.ok() }),
        ))
    }

    async fn activity_stream(&mut self, market_id: &str) -> Result<ActivityStream, WebSocketError> {
        {
            let mut senders = self.activity_senders.write().await;
            if !senders.contains_key(market_id) {
                let (tx, _) = broadcast::channel(16_384);
                senders.insert(market_id.to_string(), tx);
            }
        }

        let senders = self.activity_senders.read().await;
        let sender = senders.get(market_id).ok_or_else(|| {
            WebSocketError::Subscription(format!("no activity sender for market: {market_id}"))
        })?;
        let rx = sender.subscribe();

        Ok(Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|result| async move { result.ok() }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use px_core::{FixedPrice, OrderbookUpdate, PriceLevelSide};

    fn make_ws() -> OpinionWebSocket {
        let config = OpinionConfig::new().with_api_key("test_key");
        OpinionWebSocket::new(config).unwrap()
    }

    #[test]
    fn new_requires_api_key() {
        let config = OpinionConfig::new();
        assert!(OpinionWebSocket::new(config).is_err());
    }

    #[test]
    fn ws_url_includes_api_key() {
        let ws = make_ws();
        let url = ws.ws_url().unwrap();
        assert!(url.contains("apikey=test_key"));
        assert!(url.starts_with("wss://ws.opinion.trade"));
    }

    #[tokio::test]
    async fn handle_depth_diff_yes_bid() {
        let ws = make_ws();
        let market_id = "123";

        // Set up sender
        {
            let mut senders = ws.orderbook_senders.write().await;
            let (tx, _) = broadcast::channel(16_384);
            senders.insert(market_id.to_string(), tx);
        }

        let mut rx = {
            let senders = ws.orderbook_senders.read().await;
            senders.get(market_id).unwrap().subscribe()
        };

        let msg = serde_json::json!({
            "msgType": "market.depth.diff",
            "marketId": 123,
            "outcomeSide": 1,
            "side": "bids",
            "price": "0.65",
            "size": "100"
        });

        ws.handle_depth_diff(&msg).await;

        let update = rx.try_recv().unwrap().unwrap();
        match update {
            OrderbookUpdate::Delta { changes, .. } => {
                assert_eq!(changes.len(), 1);
                assert_eq!(changes[0].side, PriceLevelSide::Bid);
                assert_eq!(changes[0].price, FixedPrice::from_f64(0.65));
                assert!((changes[0].size - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Delta"),
        }
    }

    #[tokio::test]
    async fn handle_depth_diff_no_bid_inverts() {
        let ws = make_ws();
        let market_id = "456";

        {
            let mut senders = ws.orderbook_senders.write().await;
            let (tx, _) = broadcast::channel(16_384);
            senders.insert(market_id.to_string(), tx);
        }

        let mut rx = {
            let senders = ws.orderbook_senders.read().await;
            senders.get(market_id).unwrap().subscribe()
        };

        // outcomeSide 2 (no), side "bids" -> Ask at 1.0 - price
        let msg = serde_json::json!({
            "msgType": "market.depth.diff",
            "marketId": 456,
            "outcomeSide": 2,
            "side": "bids",
            "price": 0.3,
            "size": 50
        });

        ws.handle_depth_diff(&msg).await;

        let update = rx.try_recv().unwrap().unwrap();
        match update {
            OrderbookUpdate::Delta { changes, .. } => {
                assert_eq!(changes[0].side, PriceLevelSide::Ask);
                assert_eq!(changes[0].price, FixedPrice::from_f64(0.7));
                assert!((changes[0].size - 50.0).abs() < f64::EPSILON);
            }
            _ => panic!("expected Delta"),
        }
    }

    #[tokio::test]
    async fn handle_last_trade_broadcasts() {
        let ws = make_ws();
        let market_id = "789";

        {
            let mut senders = ws.activity_senders.write().await;
            let (tx, _) = broadcast::channel(16_384);
            senders.insert(market_id.to_string(), tx);
        }

        let mut rx = {
            let senders = ws.activity_senders.read().await;
            senders.get(market_id).unwrap().subscribe()
        };

        let msg = serde_json::json!({
            "msgType": "market.last.trade",
            "marketId": 789,
            "price": "0.55",
            "shares": 25,
            "side": "buy",
            "tokenId": "token_yes_789"
        });

        ws.handle_last_trade(&msg).await;

        let event = rx.try_recv().unwrap().unwrap();
        match event {
            ActivityEvent::Trade(trade) => {
                assert_eq!(trade.market_id, "789");
                assert_eq!(trade.asset_id, "token_yes_789");
                assert!((trade.price - 0.55).abs() < f64::EPSILON);
                assert!((trade.size - 25.0).abs() < f64::EPSILON);
                assert_eq!(trade.side.as_deref(), Some("buy"));
                assert_eq!(trade.source_channel, "market.last.trade");
            }
            _ => panic!("expected Trade"),
        }
    }

    #[test]
    fn reconnect_delay_increases_exponentially() {
        let d0 = OpinionWebSocket::calculate_reconnect_delay(0);
        let d1 = OpinionWebSocket::calculate_reconnect_delay(1);
        let d2 = OpinionWebSocket::calculate_reconnect_delay(2);
        assert!(d1 > d0);
        assert!(d2 > d1);
        // Should cap at max
        let d_max = OpinionWebSocket::calculate_reconnect_delay(100);
        assert_eq!(d_max, WS_RECONNECT_MAX_DELAY);
    }
}
