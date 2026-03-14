use async_trait::async_trait;
use futures::StreamExt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::Duration;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
};

use px_core::{
    sort_asks, sort_bids, ActivityEvent, ActivityFill, ActivityStream, AtomicWebSocketState,
    OrderBookWebSocket, Orderbook, OrderbookStream, OrderbookUpdate, PriceLevel, WebSocketError,
    WebSocketState, WS_MAX_RECONNECT_ATTEMPTS, WS_RECONNECT_BASE_DELAY, WS_RECONNECT_MAX_DELAY,
};

use crate::PredictFunConfig;

/// Heartbeat interval: server sends heartbeat pushes every ~15s.
/// We echo back immediately. If we receive nothing for 45s, assume dead.
const HEARTBEAT_TIMEOUT_SECS: u64 = 45;
const HEARTBEAT_CHECK_INTERVAL_SECS: u64 = 5;

type OrderbookSender = broadcast::Sender<Result<OrderbookUpdate, WebSocketError>>;
type ActivitySender = broadcast::Sender<Result<ActivityEvent, WebSocketError>>;

pub struct PredictFunWebSocket {
    config: PredictFunConfig,
    state: Arc<AtomicWebSocketState>,
    subscriptions: Arc<RwLock<HashMap<String, Option<u64>>>>,
    pending: Arc<RwLock<HashMap<u64, String>>>,
    orderbook_senders: Arc<RwLock<HashMap<String, OrderbookSender>>>,
    activity_senders: Arc<RwLock<HashMap<String, ActivitySender>>>,
    orderbooks: Arc<RwLock<HashMap<String, Orderbook>>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    jwt_token: Option<String>,
    auto_reconnect: bool,
    reconnect_attempts: Arc<Mutex<u32>>,
    request_id: Arc<AtomicU64>,
    last_heartbeat: Arc<AtomicU64>,
}

impl PredictFunWebSocket {
    pub fn new(config: PredictFunConfig) -> Self {
        Self::with_config(config, None, true)
    }

    pub fn with_jwt(config: PredictFunConfig, jwt: String) -> Self {
        Self::with_config(config, Some(jwt), true)
    }

    pub fn with_config(
        config: PredictFunConfig,
        jwt_token: Option<String>,
        auto_reconnect: bool,
    ) -> Self {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            config,
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(HashMap::new())),
            orderbook_senders: Arc::new(RwLock::new(HashMap::new())),
            activity_senders: Arc::new(RwLock::new(HashMap::new())),
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            jwt_token,
            auto_reconnect,
            reconnect_attempts: Arc::new(Mutex::new(0)),
            request_id: Arc::new(AtomicU64::new(0)),
            last_heartbeat: Arc::new(AtomicU64::new(now_ms)),
        }
    }

    fn set_state(&self, new_state: WebSocketState) {
        self.state.store(new_state);
    }

    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn build_ws_url(&self) -> String {
        let mut url = self.config.ws_url.clone();
        if let Some(ref key) = self.config.api_key {
            url.push_str("?apiKey=");
            url.push_str(key);
        }
        url
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

    async fn send_heartbeat_response(&self, timestamp: u64) {
        let msg = serde_json::json!({
            "method": "heartbeat",
            "data": timestamp
        });
        if let Ok(json) = serde_json::to_string(&msg) {
            let _ = self.send_message(&json).await;
        }
    }

    async fn send_subscribe_msg(&self, market_id: &str) -> Result<u64, WebSocketError> {
        let id = self.next_request_id();
        let topic = format!("predictOrderbook/{market_id}");
        let payload = serde_json::json!({
            "method": "subscribe",
            "requestId": id,
            "params": [topic]
        });

        {
            let mut pending = self.pending.write().await;
            pending.insert(id, market_id.to_string());
        }

        let json =
            serde_json::to_string(&payload).map_err(|e| WebSocketError::Protocol(e.to_string()))?;
        self.send_message(&json).await?;
        Ok(id)
    }

    async fn send_unsubscribe_msg(&self, market_id: &str) -> Result<(), WebSocketError> {
        let id = self.next_request_id();
        let topic = format!("predictOrderbook/{market_id}");
        let payload = serde_json::json!({
            "method": "unsubscribe",
            "requestId": id,
            "params": [topic]
        });
        let json =
            serde_json::to_string(&payload).map_err(|e| WebSocketError::Protocol(e.to_string()))?;
        self.send_message(&json).await?;
        Ok(())
    }

    async fn subscribe_wallet_events(&self) -> Result<(), WebSocketError> {
        let jwt = match self.jwt_token {
            Some(ref t) => t.clone(),
            None => return Ok(()),
        };
        let id = self.next_request_id();
        let topic = format!("predictWalletEvents/{jwt}");
        let payload = serde_json::json!({
            "method": "subscribe",
            "requestId": id,
            "params": [topic]
        });
        let json =
            serde_json::to_string(&payload).map_err(|e| WebSocketError::Protocol(e.to_string()))?;
        self.send_message(&json).await?;
        Ok(())
    }

    async fn handle_message(&self, text: &str) {
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return,
        };

        let msg_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match msg_type {
            "M" => self.handle_push(&value).await,
            "R" => self.handle_response(&value).await,
            _ => {}
        }
    }

    async fn handle_push(&self, value: &serde_json::Value) {
        let topic = match value.get("topic").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return,
        };

        if topic == "heartbeat" {
            let timestamp = value
                .get("data")
                .and_then(|v| v.as_u64())
                .unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64
                });
            self.last_heartbeat
                .store(now_millis(), Ordering::Release);
            self.send_heartbeat_response(timestamp).await;
        } else if let Some(market_id) = topic.strip_prefix("predictOrderbook/") {
            self.handle_orderbook_push(market_id, value).await;
        } else if topic.starts_with("predictWalletEvents/") {
            self.handle_wallet_event(value).await;
        }
    }

    async fn handle_orderbook_push(&self, market_id: &str, value: &serde_json::Value) {
        let data = match value.get("data") {
            Some(d) => d,
            None => return,
        };

        // Parse bids/asks from the push data
        let mut bids: Vec<PriceLevel> = Vec::new();
        let mut asks: Vec<PriceLevel> = Vec::new();

        if let Some(buy_levels) = data.get("buys").and_then(|v| v.as_array()) {
            for level in buy_levels {
                if let (Some(price), Some(size)) = (
                    level.get("price").and_then(parse_f64),
                    level.get("size").and_then(parse_f64),
                ) {
                    if price > 0.0 && size > 0.0 {
                        bids.push(PriceLevel::new(price, size));
                    }
                }
            }
        }

        if let Some(sell_levels) = data.get("sells").and_then(|v| v.as_array()) {
            for level in sell_levels {
                if let (Some(price), Some(size)) = (
                    level.get("price").and_then(parse_f64),
                    level.get("size").and_then(parse_f64),
                ) {
                    if price > 0.0 && size > 0.0 {
                        asks.push(PriceLevel::new(price, size));
                    }
                }
            }
        }

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        let orderbook = Orderbook {
            market_id: market_id.to_string(),
            asset_id: market_id.to_string(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
        };

        {
            let mut obs = self.orderbooks.write().await;
            obs.insert(market_id.to_string(), orderbook.clone());
        }

        let senders = self.orderbook_senders.read().await;
        if let Some(sender) = senders.get(market_id) {
            let _ = sender.send(Ok(OrderbookUpdate::Snapshot(orderbook)));
        }
    }

    async fn handle_wallet_event(&self, value: &serde_json::Value) {
        let data = match value.get("data") {
            Some(d) => d,
            None => return,
        };

        let market_id = data
            .get("marketId")
            .or_else(|| data.get("market_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let price = data
            .get("price")
            .and_then(parse_f64)
            .unwrap_or(0.0);
        let size = data
            .get("size")
            .or_else(|| data.get("amount"))
            .and_then(parse_f64)
            .unwrap_or(0.0);

        if price <= 0.0 || size <= 0.0 {
            return;
        }

        let side = data.get("side").and_then(|v| v.as_str()).map(str::to_string);
        let order_id = data
            .get("orderId")
            .or_else(|| data.get("order_id"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let fill_id = data
            .get("fillId")
            .or_else(|| data.get("id"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let event = ActivityEvent::Fill(ActivityFill {
            market_id: market_id.clone(),
            asset_id: market_id.clone(),
            fill_id,
            order_id,
            price,
            size,
            side,
            outcome: None,
            timestamp: Some(chrono::Utc::now()),
            source_channel: Cow::Borrowed("predictfun_wallet_event"),
            liquidity_role: None,
        });

        // Broadcast to all activity senders (wallet events aren't market-specific subscriptions)
        let senders = self.activity_senders.read().await;
        if let Some(sender) = senders.get(&market_id) {
            let _ = sender.send(Ok(event));
        } else {
            // Broadcast to all senders if no market-specific one exists
            for sender in senders.values() {
                let _ = sender.send(Ok(event.clone()));
            }
        }
    }

    async fn handle_response(&self, value: &serde_json::Value) {
        let request_id = match value.get("requestId").and_then(|v| v.as_u64()) {
            Some(id) => id,
            None => return,
        };
        let success = value.get("success").and_then(|v| v.as_bool()).unwrap_or(false);

        let market = {
            let mut pending = self.pending.write().await;
            pending.remove(&request_id)
        };

        if let Some(market) = market {
            if success {
                let mut subs = self.subscriptions.write().await;
                subs.insert(market, Some(request_id));
            } else {
                let error_msg = value
                    .get("error")
                    .map(|e| {
                        let code = e.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
                        let message = e
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown error");
                        format!("predictfun ws error {code}: {message}")
                    })
                    .unwrap_or_else(|| "predictfun ws subscription error".to_string());

                let err = WebSocketError::Subscription(error_msg);
                let senders = self.orderbook_senders.read().await;
                if let Some(sender) = senders.get(&market) {
                    let _ = sender.send(Err(err));
                }
            }
        }
    }

    async fn resubscribe_all(&self) -> Result<(), WebSocketError> {
        let markets: Vec<String> = {
            let mut subs = self.subscriptions.write().await;
            let markets = subs.keys().cloned().collect::<Vec<_>>();
            for value in subs.values_mut() {
                *value = None;
            }
            markets
        };

        for market in markets {
            let _ = self.send_subscribe_msg(&market).await?;
        }

        // Re-subscribe to wallet events if JWT is set
        self.subscribe_wallet_events().await?;

        Ok(())
    }

    fn calculate_reconnect_delay(attempt: u32) -> Duration {
        let delay = WS_RECONNECT_BASE_DELAY.as_millis() as f64 * 1.5_f64.powi(attempt as i32);
        let delay = delay.min(WS_RECONNECT_MAX_DELAY.as_millis() as f64) as u64;
        Duration::from_millis(delay)
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn parse_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

#[async_trait]
impl OrderBookWebSocket for PredictFunWebSocket {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Connecting);

        let url = self.build_ws_url();
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

        // Reset heartbeat timer
        self.last_heartbeat.store(now_millis(), Ordering::Release);

        let state = self.state.clone();
        let orderbook_senders = self.orderbook_senders.clone();
        let activity_senders = self.activity_senders.clone();
        let orderbooks = self.orderbooks.clone();
        let subscriptions = self.subscriptions.clone();
        let pending = self.pending.clone();
        let write_tx_clone = self.write_tx.clone();
        let reconnect_attempts_clone = self.reconnect_attempts.clone();
        let auto_reconnect = self.auto_reconnect;
        let last_heartbeat = self.last_heartbeat.clone();
        let request_id = self.request_id.clone();

        let ws_self = PredictFunWebSocket {
            config: self.config.clone(),
            state: state.clone(),
            subscriptions: subscriptions.clone(),
            pending: pending.clone(),
            orderbook_senders: orderbook_senders.clone(),
            activity_senders: activity_senders.clone(),
            orderbooks: orderbooks.clone(),
            write_tx: write_tx_clone.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            jwt_token: self.jwt_token.clone(),
            auto_reconnect,
            reconnect_attempts: reconnect_attempts_clone.clone(),
            request_id: request_id.clone(),
            last_heartbeat: last_heartbeat.clone(),
        };

        let ws_url = url.clone();

        tokio::spawn(async move {
            let write_future = rx.map(Ok).forward(write);
            let read_future = async {
                let mut read = read;
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            ws_self.handle_message(&text).await;
                        }
                        Ok(Message::Ping(data)) => {
                            if let Some(ref tx) = *ws_self.write_tx.lock().await {
                                let _ = tx.unbounded_send(Message::Pong(data));
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }
            };

            // Heartbeat timeout watchdog
            let hb_last = last_heartbeat.clone();
            let heartbeat_watchdog = async {
                let mut check_interval =
                    tokio::time::interval(Duration::from_secs(HEARTBEAT_CHECK_INTERVAL_SECS));
                loop {
                    check_interval.tick().await;
                    let last = hb_last.load(Ordering::Acquire);
                    let elapsed = now_millis().saturating_sub(last);
                    if elapsed > HEARTBEAT_TIMEOUT_SECS * 1000 {
                        tracing::warn!(
                            "predictfun ws heartbeat timeout ({elapsed}ms), triggering reconnect"
                        );
                        break;
                    }
                }
            };

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = heartbeat_watchdog => {},
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

                    let delay = PredictFunWebSocket::calculate_reconnect_delay(attempt);
                    tokio::time::sleep(delay).await;

                    match connect_async(&ws_url).await {
                        Ok((new_ws, _)) => {
                            let (new_write, new_read) = new_ws.split();
                            let (new_tx, new_rx) =
                                futures::channel::mpsc::unbounded::<Message>();

                            {
                                let mut wtx = write_tx_clone.lock().await;
                                *wtx = Some(new_tx);
                            }

                            state.store(WebSocketState::Connected);
                            ws_self
                                .last_heartbeat
                                .store(now_millis(), Ordering::Release);

                            {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a = 0;
                            }

                            let _ = ws_self.resubscribe_all().await;

                            let write_future = new_rx.map(Ok).forward(new_write);
                            let read_future = async {
                                let mut read = new_read;
                                while let Some(msg) = read.next().await {
                                    match msg {
                                        Ok(Message::Text(text)) => {
                                            ws_self.handle_message(&text).await;
                                        }
                                        Ok(Message::Ping(data)) => {
                                            if let Some(ref tx) =
                                                *ws_self.write_tx.lock().await
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

                            let hb_last = ws_self.last_heartbeat.clone();
                            let heartbeat_watchdog = async {
                                let mut check_interval = tokio::time::interval(
                                    Duration::from_secs(HEARTBEAT_CHECK_INTERVAL_SECS),
                                );
                                loop {
                                    check_interval.tick().await;
                                    let last = hb_last.load(Ordering::Acquire);
                                    let elapsed = now_millis().saturating_sub(last);
                                    if elapsed > HEARTBEAT_TIMEOUT_SECS * 1000 {
                                        break;
                                    }
                                }
                            };

                            tokio::select! {
                                _ = write_future => {},
                                _ = read_future => {},
                                _ = heartbeat_watchdog => {},
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
        {
            let mut attempts = self.reconnect_attempts.lock().await;
            *attempts = 0;
        }
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
            subs.entry(market.clone()).or_insert(None);
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
            let _ = self.send_subscribe_msg(&market).await?;
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
            let _ = self.send_unsubscribe_msg(&market).await;
        }

        {
            let mut senders = self.orderbook_senders.write().await;
            senders.remove(&market);
        }

        {
            let mut senders = self.activity_senders.write().await;
            senders.remove(&market);
        }

        {
            let mut obs = self.orderbooks.write().await;
            obs.remove(&market);
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
    use serde_json::json;

    #[test]
    fn parse_heartbeat_push() {
        let msg = json!({
            "type": "M",
            "topic": "heartbeat",
            "data": 1710000000000_u64
        });
        assert_eq!(msg["type"], "M");
        assert_eq!(msg["topic"], "heartbeat");
        assert_eq!(msg["data"], 1710000000000_u64);
    }

    #[test]
    fn parse_subscribe_response_success() {
        let msg = json!({
            "type": "R",
            "requestId": 1,
            "success": true,
            "data": {}
        });
        assert_eq!(msg["type"], "R");
        assert_eq!(msg["requestId"], 1);
        assert!(msg["success"].as_bool().unwrap());
    }

    #[test]
    fn parse_subscribe_response_error() {
        let msg = json!({
            "type": "R",
            "requestId": 2,
            "success": false,
            "error": {
                "code": 400,
                "message": "invalid topic"
            }
        });
        assert!(!msg["success"].as_bool().unwrap());
        let error = msg.get("error").unwrap();
        assert_eq!(error["code"], 400);
        assert_eq!(error["message"], "invalid topic");
    }

    #[test]
    fn parse_orderbook_push() {
        let msg = json!({
            "type": "M",
            "topic": "predictOrderbook/0x1234",
            "data": {
                "buys": [
                    { "price": 0.55, "size": 100.0 },
                    { "price": 0.50, "size": 200.0 }
                ],
                "sells": [
                    { "price": 0.60, "size": 150.0 },
                    { "price": 0.65, "size": 50.0 }
                ]
            }
        });
        let topic = msg["topic"].as_str().unwrap();
        assert!(topic.starts_with("predictOrderbook/"));
        let market_id = topic.strip_prefix("predictOrderbook/").unwrap();
        assert_eq!(market_id, "0x1234");

        let data = msg.get("data").unwrap();
        let buys = data["buys"].as_array().unwrap();
        assert_eq!(buys.len(), 2);
        assert_eq!(buys[0]["price"].as_f64().unwrap(), 0.55);
        assert_eq!(buys[0]["size"].as_f64().unwrap(), 100.0);
    }

    #[test]
    fn parse_wallet_event() {
        let msg = json!({
            "type": "M",
            "topic": "predictWalletEvents/jwt_token_here",
            "data": {
                "marketId": "0xabcd",
                "price": 0.70,
                "size": 50.0,
                "side": "buy",
                "orderId": "order_123",
                "fillId": "fill_456"
            }
        });
        let topic = msg["topic"].as_str().unwrap();
        assert!(topic.starts_with("predictWalletEvents/"));
        let data = msg.get("data").unwrap();
        assert_eq!(data["marketId"], "0xabcd");
        assert_eq!(data["price"].as_f64().unwrap(), 0.70);
    }

    #[test]
    fn serialize_subscribe_message() {
        let payload = json!({
            "method": "subscribe",
            "requestId": 1,
            "params": ["predictOrderbook/0x1234"]
        });
        let json_str = serde_json::to_string(&payload).unwrap();
        // Must not contain "data" field
        assert!(!json_str.contains("\"data\""));
        assert!(json_str.contains("\"method\":\"subscribe\""));
        assert!(json_str.contains("\"requestId\":1"));
        assert!(json_str.contains("predictOrderbook/0x1234"));
    }

    #[test]
    fn serialize_heartbeat_response() {
        let payload = json!({
            "method": "heartbeat",
            "data": 1710000000000_u64
        });
        let json_str = serde_json::to_string(&payload).unwrap();
        // Must not contain "requestId" field
        assert!(!json_str.contains("\"requestId\""));
        assert!(json_str.contains("\"method\":\"heartbeat\""));
        assert!(json_str.contains("1710000000000"));
    }

    #[test]
    fn reconnect_delay_exponential_backoff() {
        let d0 = PredictFunWebSocket::calculate_reconnect_delay(0);
        let d1 = PredictFunWebSocket::calculate_reconnect_delay(1);
        let d2 = PredictFunWebSocket::calculate_reconnect_delay(2);
        let d3 = PredictFunWebSocket::calculate_reconnect_delay(3);

        // Base delay is 3000ms, multiplied by 1.5^attempt
        assert_eq!(d0.as_millis(), 3000);
        assert_eq!(d1.as_millis(), 4500);
        assert_eq!(d2.as_millis(), 6750);
        assert_eq!(d3.as_millis(), 10125);

        // Should cap at max delay (60s)
        let d_large = PredictFunWebSocket::calculate_reconnect_delay(20);
        assert!(d_large.as_millis() <= 60000);
    }

    #[test]
    fn topic_string_extraction() {
        let orderbook_topic = "predictOrderbook/0xabc123";
        assert_eq!(
            orderbook_topic.strip_prefix("predictOrderbook/"),
            Some("0xabc123")
        );

        let wallet_topic = "predictWalletEvents/eyJhbGciOiJIUzI1NiJ9";
        assert!(wallet_topic.starts_with("predictWalletEvents/"));

        let heartbeat_topic = "heartbeat";
        assert_eq!(heartbeat_topic, "heartbeat");
        assert!(!heartbeat_topic.starts_with("predictOrderbook/"));
    }

    #[test]
    fn parse_f64_from_number() {
        let v = json!(0.55);
        assert_eq!(parse_f64(&v), Some(0.55));
    }

    #[test]
    fn parse_f64_from_string() {
        let v = json!("0.55");
        assert_eq!(parse_f64(&v), Some(0.55));
    }

    #[test]
    fn parse_f64_from_null() {
        let v = json!(null);
        assert_eq!(parse_f64(&v), None);
    }

    #[test]
    fn build_ws_url_without_api_key() {
        let config = PredictFunConfig::new();
        let ws = PredictFunWebSocket::new(config);
        assert_eq!(ws.build_ws_url(), "wss://ws.predict.fun/ws");
    }

    #[test]
    fn build_ws_url_with_api_key() {
        let config = PredictFunConfig::new().with_api_key("test_key_123");
        let ws = PredictFunWebSocket::new(config);
        assert_eq!(
            ws.build_ws_url(),
            "wss://ws.predict.fun/ws?apiKey=test_key_123"
        );
    }

    #[test]
    fn config_testnet_ws_url() {
        let config = PredictFunConfig::testnet();
        assert_eq!(config.ws_url, "wss://ws-testnet.predict.fun/ws");
    }
}
