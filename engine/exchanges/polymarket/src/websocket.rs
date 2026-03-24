use futures::StreamExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use chrono::{DateTime, Utc};
use px_core::websocket::ndjson::NdjsonWriter;
use px_core::{
    insert_ask, insert_bid, ActivityEvent, ActivityFill, ActivityStream, ActivityTrade,
    AtomicWebSocketState, ChangeVec, FixedPrice, LiquidityRole, OrderBookWebSocket, Orderbook,
    OrderbookStream, OrderbookUpdate, PriceLevel, PriceLevelChange, PriceLevelSide, WebSocketError,
    WebSocketState, WsMessage, WS_MAX_RECONNECT_ATTEMPTS, WS_PING_INTERVAL,
    WS_RECONNECT_BASE_DELAY, WS_RECONNECT_MAX_DELAY,
};
use smallvec::SmallVec;

const WS_MARKET_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";
const WS_USER_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/user";

#[derive(Debug, Clone)]
struct UserAuth {
    api_key: String,
    secret: String,
    passphrase: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SubscribeMessage {
    auth: HashMap<String, String>,
    markets: Vec<String>,
    assets_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_feature_enabled: Option<bool>,
    #[serde(rename = "type")]
    msg_type: String,
}

/// Dynamic subscribe/unsubscribe message for an already-connected WS.
/// Polymarket requires `operation: "subscribe"` (not `type: "market"`) after
/// the initial handshake.
#[derive(Debug, Clone, serde::Serialize)]
struct DynamicSubscribeMessage {
    assets_ids: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    markets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_feature_enabled: Option<bool>,
    operation: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct RawWsMessage {
    event_type: Option<String>,
    asset_id: Option<String>,
    market: Option<String>,
    bids: Option<Vec<WsPriceLevel>>,
    asks: Option<Vec<WsPriceLevel>>,
    price_changes: Option<Vec<WsPriceChange>>,
    price: Option<String>,
    size: Option<String>,
    side: Option<String>,
    timestamp: Option<serde_json::Value>,
    id: Option<String>,
    order_id: Option<String>,
    #[serde(alias = "takerOrderId")]
    taker_order_id: Option<String>,
    #[serde(alias = "tradeOwner")]
    trade_owner: Option<String>,
    #[serde(default, alias = "makerOrders")]
    maker_orders: Option<Vec<WsMakerOrder>>,
    /// Book-state hash from `book` events, used for integrity verification.
    hash: Option<String>,
    /// Fee rate in basis points from `last_trade_price` events.
    #[serde(alias = "fee_rate_bps")]
    fee_rate_bps: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct WsPriceLevel {
    price: String,
    size: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
struct WsPriceChange {
    asset_id: String,
    price: Option<String>,
    size: Option<String>,
    side: Option<String>,
    best_bid: Option<String>,
    best_ask: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct WsMakerOrder {
    #[serde(alias = "orderId")]
    order_id: Option<String>,
    owner: Option<String>,
}

type OrderbookSender = broadcast::Sender<Result<WsMessage<OrderbookUpdate>, WebSocketError>>;
type ActivitySender = broadcast::Sender<Result<WsMessage<ActivityEvent>, WebSocketError>>;
type OrderbookSenderMap = Arc<RwLock<HashMap<String, (OrderbookSender, Arc<AtomicU64>)>>>;
type ActivitySenderMap = Arc<RwLock<HashMap<String, (ActivitySender, Arc<AtomicU64>)>>>;
type BroadcastEntry = (String, Option<DateTime<Utc>>, ChangeVec);

pub struct PolymarketWebSocket {
    state: Arc<AtomicWebSocketState>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    orderbook_senders: OrderbookSenderMap,
    activity_senders: ActivitySenderMap,
    orderbooks: Arc<RwLock<HashMap<String, Orderbook>>>,
    asset_to_market: Arc<RwLock<HashMap<String, String>>>,
    market_to_assets: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    user_subscribed_markets: Arc<RwLock<HashSet<String>>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    user_write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    user_shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    auto_reconnect: bool,
    reconnect_attempts: Arc<Mutex<u32>>,
    user_auth: Option<UserAuth>,
    /// Tracks companion relationships: primary -> companion and vice versa.
    /// When a pair is subscribed, both directions are stored.
    companions: Arc<RwLock<HashMap<String, String>>>,
    /// Maps asset_id → outcome name ("Yes" / "No"). Populated via `register_outcomes`.
    outcome_map: Arc<RwLock<HashMap<String, String>>>,
    /// True once the initial `type: "market"` subscribe message has been sent.
    /// Subsequent subscriptions must use `operation: "subscribe"` instead.
    initial_subscribed: Arc<std::sync::atomic::AtomicBool>,
    ndjson_writer: Option<Arc<NdjsonWriter>>,
}

impl PolymarketWebSocket {
    pub fn new() -> Self {
        Self::with_config(true)
    }

    pub fn with_config(auto_reconnect: bool) -> Self {
        Self::with_auth_config(auto_reconnect, None)
    }

    pub fn with_auth(api_key: String, secret: String, passphrase: String) -> Self {
        Self::with_auth_config(
            true,
            Some(UserAuth {
                api_key,
                secret,
                passphrase,
            }),
        )
    }

    fn with_auth_config(auto_reconnect: bool, user_auth: Option<UserAuth>) -> Self {
        Self {
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            orderbook_senders: Arc::new(RwLock::new(HashMap::new())),
            activity_senders: Arc::new(RwLock::new(HashMap::new())),
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            asset_to_market: Arc::new(RwLock::new(HashMap::new())),
            market_to_assets: Arc::new(RwLock::new(HashMap::new())),
            user_subscribed_markets: Arc::new(RwLock::new(HashSet::new())),
            write_tx: Arc::new(Mutex::new(None)),
            user_write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            user_shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect,
            reconnect_attempts: Arc::new(Mutex::new(0)),
            user_auth,
            companions: Arc::new(RwLock::new(HashMap::new())),
            outcome_map: Arc::new(RwLock::new(HashMap::new())),
            initial_subscribed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            ndjson_writer: None,
        }
    }

    pub fn with_ndjson_writer(mut self, writer: impl std::io::Write + Send + 'static) -> Self {
        self.ndjson_writer = Some(Arc::new(NdjsonWriter::new(writer)));
        self
    }

    /// Register outcome names for asset IDs so activity events include "Yes"/"No".
    /// Call with the two token IDs from the market's `clob_token_ids` and `outcomes`.
    pub async fn register_outcomes(&self, yes_token_id: &str, no_token_id: &str) {
        let mut map = self.outcome_map.write().await;
        map.insert(yes_token_id.to_string(), "Yes".to_string());
        map.insert(no_token_id.to_string(), "No".to_string());
    }

    async fn reset_reconnect_attempts(&self) {
        let mut attempts = self.reconnect_attempts.lock().await;
        *attempts = 0;
    }

    #[allow(dead_code)]
    async fn increment_reconnect_attempts(&self) -> u32 {
        let mut attempts = self.reconnect_attempts.lock().await;
        *attempts += 1;
        *attempts
    }

    #[allow(dead_code)]
    pub async fn get_reconnect_attempts(&self) -> u32 {
        *self.reconnect_attempts.lock().await
    }

    fn set_state(&self, new_state: WebSocketState) {
        self.state.store(new_state);
    }

    /// Send the right subscribe message depending on whether this is the initial
    /// subscription (uses `type: "market"`) or a dynamic add (uses `operation: "subscribe"`).
    async fn send_market_subscribe(&self, asset_ids: Vec<String>) -> Result<(), WebSocketError> {
        if self.state.load() != WebSocketState::Connected {
            return Ok(());
        }
        let json = if self
            .initial_subscribed
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            // Already sent initial subscribe — use dynamic operation
            serde_json::to_string(&DynamicSubscribeMessage {
                assets_ids: asset_ids,
                markets: vec![],
                custom_feature_enabled: Some(true),
                operation: "subscribe".into(),
            })
        } else {
            // First subscribe on this connection — use type: "market"
            serde_json::to_string(&SubscribeMessage {
                auth: HashMap::new(),
                markets: vec![],
                assets_ids: asset_ids,
                custom_feature_enabled: Some(true),
                msg_type: "market".into(),
            })
        }
        .map_err(|e| WebSocketError::Protocol(e.to_string()))?;
        self.send_message(&json).await
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

    async fn send_user_message(&self, msg: &str) -> Result<(), WebSocketError> {
        let tx = self.user_write_tx.lock().await;
        if let Some(ref sender) = *tx {
            sender
                .unbounded_send(Message::Text(msg.into()))
                .map_err(|e| WebSocketError::Connection(format!("send failed: {e}")))?;
        }
        Ok(())
    }

    fn parse_timestamp(value: Option<&serde_json::Value>) -> Option<chrono::DateTime<chrono::Utc>> {
        let value = value?;
        if let Some(s) = value.as_str() {
            if let Ok(ts) = s.parse::<i64>() {
                return chrono::DateTime::from_timestamp_millis(ts)
                    .or_else(|| chrono::DateTime::from_timestamp(ts, 0));
            }
            return chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc));
        }

        if let Some(ts) = value.as_i64() {
            return chrono::DateTime::from_timestamp_millis(ts)
                .or_else(|| chrono::DateTime::from_timestamp(ts, 0));
        }

        None
    }

    async fn handle_message(&self, text: &str) {
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(_) => return,
        };

        if let Some(items) = value.as_array() {
            for item in items {
                self.handle_single_message(item).await;
            }
        } else {
            self.handle_single_message(&value).await;
        }
    }

    async fn handle_single_message(&self, value: &serde_json::Value) {
        let msg: RawWsMessage = match serde_json::from_value(value.clone()) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to parse RawWsMessage: {}", e);
                return;
            }
        };

        match msg.event_type.as_deref() {
            Some("book") => {
                tracing::debug!(
                    "polymarket_ws: book event asset_id={:?} bids={} asks={}",
                    msg.asset_id,
                    msg.bids.as_ref().map(|b| b.len()).unwrap_or(0),
                    msg.asks.as_ref().map(|a| a.len()).unwrap_or(0),
                );
                self.handle_book_message(&msg).await;
            }
            Some("price_change") => {
                tracing::debug!(
                    "Received price_change event with {} changes",
                    msg.price_changes.as_ref().map(|c| c.len()).unwrap_or(0)
                );
                self.handle_price_change(&msg).await;
            }
            Some("last_trade_price") => self.handle_last_trade_price(&msg).await,
            Some("trade") => self.handle_user_trade(&msg).await,
            Some(other) => {
                tracing::trace!("Ignoring event_type: {}", other);
            }
            _ => {}
        }
    }

    async fn handle_book_message(&self, msg: &RawWsMessage) {
        let asset_id = match &msg.asset_id {
            Some(id) => id.clone(),
            None => return,
        };

        let market_id = msg.market.clone().unwrap_or_default();

        let bids: Vec<PriceLevel> = msg
            .bids
            .as_ref()
            .map(|b| {
                b.iter()
                    .filter_map(|l| {
                        let price = l.price.parse::<f64>().ok()?;
                        let size = l.size.parse::<f64>().ok()?;
                        if price > 0.0 && size > 0.0 {
                            Some(PriceLevel::new(price, size))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let asks: Vec<PriceLevel> = msg
            .asks
            .as_ref()
            .map(|a| {
                a.iter()
                    .filter_map(|l| {
                        let price = l.price.parse::<f64>().ok()?;
                        let size = l.size.parse::<f64>().ok()?;
                        if price > 0.0 && size > 0.0 {
                            Some(PriceLevel::new(price, size))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let orderbook = Orderbook {
            market_id: market_id.clone(),
            asset_id: asset_id.clone(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
            hash: msg.hash.clone(),
        };

        {
            let mut obs = self.orderbooks.write().await;
            obs.insert(asset_id.clone(), orderbook.clone());
        }

        if !market_id.is_empty() {
            {
                let mut map = self.asset_to_market.write().await;
                map.insert(asset_id.clone(), market_id.clone());
            }
            {
                let mut map = self.market_to_assets.write().await;
                map.entry(market_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(asset_id.clone());
            }
            self.ensure_user_market_subscription(&market_id).await;
        }

        self.broadcast_orderbook(&asset_id, orderbook).await;
    }

    async fn handle_price_change(&self, msg: &RawWsMessage) {
        let raw_changes = match &msg.price_changes {
            Some(c) => c,
            None => return,
        };

        let mut obs = self.orderbooks.write().await;
        // Group changes by asset_id
        let mut asset_changes: SmallVec<[(String, ChangeVec); 2]> = SmallVec::new();

        for change in raw_changes {
            let asset_id = &change.asset_id;
            if let Some(ob) = obs.get_mut(asset_id) {
                if let (Some(price_str), Some(size_str), Some(side)) =
                    (&change.price, &change.size, &change.side)
                {
                    if let (Ok(price), Ok(size)) =
                        (price_str.parse::<f64>(), size_str.parse::<f64>())
                    {
                        let fp = FixedPrice::from_f64(price);
                        let is_bid = side.eq_ignore_ascii_case("BUY");
                        let levels = if is_bid { &mut ob.bids } else { &mut ob.asks };

                        // Apply to internal book
                        if let Some(existing) = levels.iter_mut().find(|l| l.price == fp) {
                            if size > 0.0 {
                                existing.size = size;
                            } else {
                                // Remove will happen via retain below
                            }
                        }
                        if size > 0.0 {
                            if !levels.iter().any(|l| l.price == fp) {
                                let level = PriceLevel::with_fixed(fp, size);
                                if is_bid {
                                    insert_bid(levels, level);
                                } else {
                                    insert_ask(levels, level);
                                }
                            }
                        } else {
                            levels.retain(|l| l.price != fp);
                        }

                        // Collect the change
                        let plc = PriceLevelChange {
                            side: if is_bid {
                                PriceLevelSide::Bid
                            } else {
                                PriceLevelSide::Ask
                            },
                            price: fp,
                            size,
                        };

                        if let Some(entry) = asset_changes.iter_mut().find(|(id, _)| id == asset_id)
                        {
                            entry.1.push(plc);
                        } else {
                            let mut cv = ChangeVec::new();
                            cv.push(plc);
                            asset_changes.push((asset_id.clone(), cv));
                        }
                    }
                }
            } else {
                // Expected for brief window before book snapshot arrives,
                // or for tokens we haven't subscribed to.
                tracing::trace!("price_change: no orderbook found for asset_id={}", asset_id);
            }
        }

        // Collect broadcast data BEFORE dropping the lock
        let mut to_broadcast: SmallVec<[BroadcastEntry; 2]> = SmallVec::new();
        for (asset_id, changes) in asset_changes {
            if let Some(ob) = obs.get_mut(&asset_id) {
                ob.timestamp = Some(Utc::now());
                to_broadcast.push((asset_id, ob.timestamp, changes));
            }
        }

        // CRITICAL: Drop orderbooks write lock BEFORE broadcasting
        drop(obs);

        let senders = self.orderbook_senders.read().await;
        for (asset_id, timestamp, changes) in to_broadcast {
            if let Some((sender, seq)) = senders.get(&asset_id) {
                let msg = WsMessage {
                    seq: seq.fetch_add(1, Ordering::Relaxed),
                    received_at: chrono::Utc::now(),
                    data: OrderbookUpdate::Delta { changes, timestamp },
                };
                if let Some(ref writer) = self.ndjson_writer {
                    writer.write_record(&msg);
                }
                let _ = sender.send(Ok(msg));
            }
        }
    }

    async fn handle_last_trade_price(&self, msg: &RawWsMessage) {
        let Some(asset_id) = msg.asset_id.clone() else {
            return;
        };
        let Some(price) = msg.price.as_deref().and_then(|s| s.parse::<f64>().ok()) else {
            return;
        };
        let size = msg
            .size
            .as_deref()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        if size <= 0.0 {
            return;
        }

        let timestamp = Self::parse_timestamp(msg.timestamp.as_ref());
        let fee_rate_bps = msg
            .fee_rate_bps
            .as_deref()
            .and_then(|s| s.parse::<u32>().ok());

        let outcome = {
            let map = self.outcome_map.read().await;
            map.get(&asset_id).cloned()
        };

        let event = ActivityEvent::Trade(ActivityTrade {
            market_id: msg.market.clone().unwrap_or_default(),
            asset_id: asset_id.clone(),
            trade_id: msg.id.clone(),
            price,
            size,
            side: msg.side.clone(),
            aggressor_side: msg.side.clone(),
            outcome,
            fee_rate_bps,
            timestamp,
            source_channel: Cow::Borrowed("polymarket_last_trade_price"),
        });
        self.broadcast_activity(&asset_id, event).await;
    }

    async fn handle_user_trade(&self, msg: &RawWsMessage) {
        let asset_id = msg.asset_id.clone();
        let market_id = msg.market.clone();
        let price = msg
            .price
            .as_deref()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let size = msg
            .size
            .as_deref()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        if price <= 0.0 || size <= 0.0 {
            return;
        }

        let user_key = self.user_auth.as_ref().map(|auth| auth.api_key.as_str());
        let maker_order_for_user = user_key.and_then(|key| {
            msg.maker_orders.as_ref().and_then(|orders| {
                orders
                    .iter()
                    .find(|o| o.owner.as_deref() == Some(key))
                    .and_then(|o| o.order_id.clone())
            })
        });

        let order_id = if maker_order_for_user.is_some() {
            maker_order_for_user
        } else if user_key.is_some() {
            msg.taker_order_id.clone().or_else(|| msg.order_id.clone())
        } else {
            msg.taker_order_id
                .clone()
                .or_else(|| {
                    msg.maker_orders
                        .as_ref()
                        .and_then(|orders| orders.iter().find_map(|o| o.order_id.clone()))
                })
                .or_else(|| msg.order_id.clone())
        };

        let liquidity_role = user_key.and_then(|key| {
            let is_maker = msg
                .maker_orders
                .as_ref()
                .is_some_and(|orders| orders.iter().any(|o| o.owner.as_deref() == Some(key)));
            if is_maker {
                Some(LiquidityRole::Maker)
            } else if msg.trade_owner.as_deref() == Some(key) {
                Some(LiquidityRole::Taker)
            } else {
                None
            }
        });

        let timestamp = Self::parse_timestamp(msg.timestamp.as_ref());
        let outcome = if let Some(ref aid) = asset_id {
            let map = self.outcome_map.read().await;
            map.get(aid).cloned()
        } else {
            None
        };

        let fill = ActivityEvent::Fill(ActivityFill {
            market_id: market_id.clone().unwrap_or_default(),
            asset_id: asset_id.clone().unwrap_or_default(),
            fill_id: msg.id.clone(),
            order_id,
            price,
            size,
            side: msg.side.clone(),
            outcome,
            tx_hash: None,
            fee: None,
            timestamp,
            source_channel: Cow::Borrowed("polymarket_user_trade"),
            liquidity_role,
        });

        if let Some(asset_id) = asset_id {
            self.broadcast_activity(&asset_id, fill).await;
            return;
        }

        if let Some(market_id) = market_id {
            let assets = {
                let map = self.market_to_assets.read().await;
                map.get(&market_id).cloned().unwrap_or_default()
            };
            for asset in assets {
                self.broadcast_activity(&asset, fill.clone()).await;
            }
        }
    }

    async fn broadcast_orderbook(&self, asset_id: &str, orderbook: Orderbook) {
        let senders = self.orderbook_senders.read().await;
        if let Some((sender, seq)) = senders.get(asset_id) {
            let receivers = sender.receiver_count();
            let msg = WsMessage {
                seq: seq.fetch_add(1, Ordering::Relaxed),
                received_at: chrono::Utc::now(),
                data: OrderbookUpdate::Snapshot(orderbook),
            };
            if let Some(ref writer) = self.ndjson_writer {
                writer.write_record(&msg);
            }
            match sender.send(Ok(msg)) {
                Ok(n) => {
                    tracing::debug!(
                        asset_id = %asset_id,
                        receivers = receivers,
                        sent_to = n,
                        "polymarket_ws: broadcast orderbook snapshot"
                    );
                }
                Err(_) => {
                    tracing::debug!(
                        asset_id = %asset_id,
                        receivers = receivers,
                        "polymarket_ws: broadcast snapshot — no receivers"
                    );
                }
            }
        } else {
            tracing::trace!(
                asset_id = %asset_id,
                "polymarket_ws: no sender registered for asset_id"
            );
        }
    }

    async fn broadcast_activity(&self, asset_id: &str, activity: ActivityEvent) {
        let senders = self.activity_senders.read().await;
        if let Some((sender, seq)) = senders.get(asset_id) {
            let msg = WsMessage {
                seq: seq.fetch_add(1, Ordering::Relaxed),
                received_at: chrono::Utc::now(),
                data: activity,
            };
            if let Some(ref writer) = self.ndjson_writer {
                writer.write_record(&msg);
            }
            let _ = sender.send(Ok(msg));
        }
    }

    async fn ensure_user_market_subscription(&self, market_id: &str) {
        if self.user_auth.is_none() {
            return;
        }

        {
            let subscribed = self.user_subscribed_markets.read().await;
            if subscribed.contains(market_id) {
                return;
            }
        }

        let Some(auth) = self.user_auth.clone() else {
            return;
        };

        let mut auth_payload = HashMap::new();
        auth_payload.insert("apiKey".to_string(), auth.api_key);
        auth_payload.insert("secret".to_string(), auth.secret);
        auth_payload.insert("passphrase".to_string(), auth.passphrase);

        let msg = SubscribeMessage {
            auth: auth_payload,
            markets: vec![market_id.to_string()],
            assets_ids: vec![],
            custom_feature_enabled: None,
            msg_type: "user".to_string(),
        };

        let Ok(json) = serde_json::to_string(&msg) else {
            return;
        };

        if self.send_user_message(&json).await.is_ok() {
            let mut subscribed = self.user_subscribed_markets.write().await;
            subscribed.insert(market_id.to_string());
        }
    }

    async fn resubscribe_all(&self) -> Result<(), WebSocketError> {
        // Fresh connection — reset so first subscribe uses `type: "market"`.
        self.initial_subscribed
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let subs = self.subscriptions.read().await;
        // Deduplicate: companion pairs share the same asset_ids vec,
        // so we only need to send one subscribe per unique set.
        let mut sent: HashSet<Vec<String>> = HashSet::new();
        for (_asset_id, asset_ids) in subs.iter() {
            let mut sorted = asset_ids.clone();
            sorted.sort();
            if !sent.insert(sorted) {
                continue;
            }
            self.send_market_subscribe(asset_ids.clone()).await?;
        }

        if self.user_auth.is_some() {
            let markets: Vec<String> = {
                let markets = self.user_subscribed_markets.read().await;
                markets.iter().cloned().collect()
            };
            for market in markets {
                self.ensure_user_market_subscription(&market).await;
            }
        }

        Ok(())
    }

    async fn connect_user_channel(&self) -> Result<(), WebSocketError> {
        if self.user_auth.is_none() {
            return Ok(());
        }

        let (user_ws, _) = connect_async(WS_USER_URL)
            .await
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;
        let (user_write, user_read) = user_ws.split();
        let (user_tx, user_rx) = futures::channel::mpsc::unbounded::<Message>();

        {
            let mut write_tx = self.user_write_tx.lock().await;
            *write_tx = Some(user_tx);
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        {
            let mut stx = self.user_shutdown_tx.lock().await;
            *stx = Some(shutdown_tx);
        }

        let ws_self = Self {
            state: self.state.clone(),
            subscriptions: self.subscriptions.clone(),
            orderbook_senders: self.orderbook_senders.clone(),
            activity_senders: self.activity_senders.clone(),
            orderbooks: self.orderbooks.clone(),
            asset_to_market: self.asset_to_market.clone(),
            market_to_assets: self.market_to_assets.clone(),
            user_subscribed_markets: self.user_subscribed_markets.clone(),
            write_tx: self.write_tx.clone(),
            user_write_tx: self.user_write_tx.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            user_shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect: self.auto_reconnect,
            reconnect_attempts: self.reconnect_attempts.clone(),
            user_auth: self.user_auth.clone(),
            companions: self.companions.clone(),
            outcome_map: self.outcome_map.clone(),
            initial_subscribed: self.initial_subscribed.clone(),
            ndjson_writer: self.ndjson_writer.clone(),
        };

        tokio::spawn(async move {
            let write_future = user_rx.map(Ok).forward(user_write);
            let read_future = async {
                let mut read = user_read;
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            ws_self.handle_message(&text).await;
                        }
                        Ok(Message::Ping(data)) => {
                            if let Some(ref tx) = *ws_self.user_write_tx.lock().await {
                                let _ = tx.unbounded_send(Message::Pong(data));
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }
            };

            let ping_future = async {
                let mut ping_interval = interval(WS_PING_INTERVAL);
                loop {
                    ping_interval.tick().await;
                    if let Some(ref tx) = *ws_self.user_write_tx.lock().await {
                        let _ = tx.unbounded_send(Message::Ping(vec![]));
                    }
                }
            };

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = ping_future => {},
                _ = shutdown_rx => {},
            }
        });

        Ok(())
    }

    fn calculate_reconnect_delay(attempt: u32) -> Duration {
        let delay = WS_RECONNECT_BASE_DELAY.as_millis() as f64 * 1.5_f64.powi(attempt as i32);
        let delay = delay.min(WS_RECONNECT_MAX_DELAY.as_millis() as f64) as u64;
        Duration::from_millis(delay)
    }
}

impl PolymarketWebSocket {
    /// Subscribe to both tokens of a binary market pair.
    /// The primary token gets full subscribe treatment; the companion token
    /// gets orderbook/activity senders and tracking so incoming price_change
    /// events for both sides are processed instead of dropped.
    pub async fn subscribe_pair(
        &mut self,
        primary_token_id: &str,
        companion_token_id: &str,
    ) -> Result<(), WebSocketError> {
        // Guard: if somehow the same token, fall back to single subscribe
        if primary_token_id == companion_token_id {
            return self.subscribe(primary_token_id).await;
        }
        let asset_ids = vec![primary_token_id.to_string(), companion_token_id.to_string()];

        // Register subscriptions for both tokens
        for token_id in &asset_ids {
            let mut subs = self.subscriptions.write().await;
            subs.insert(token_id.clone(), asset_ids.clone());
        }

        // Ensure senders exist for both tokens
        for token_id in &asset_ids {
            {
                let mut senders = self.orderbook_senders.write().await;
                if !senders.contains_key(token_id.as_str()) {
                    let (tx, _) = broadcast::channel(16_384);
                    senders.insert(token_id.clone(), (tx, Arc::new(AtomicU64::new(0))));
                }
            }
            {
                let mut senders = self.activity_senders.write().await;
                if !senders.contains_key(token_id.as_str()) {
                    let (tx, _) = broadcast::channel(16_384);
                    senders.insert(token_id.clone(), (tx, Arc::new(AtomicU64::new(0))));
                }
            }
        }

        // Track companion relationship (bidirectional)
        {
            let mut comps = self.companions.write().await;
            comps.insert(primary_token_id.to_string(), companion_token_id.to_string());
            comps.insert(companion_token_id.to_string(), primary_token_id.to_string());
        }

        // Send subscribe with both asset_ids
        self.send_market_subscribe(asset_ids).await?;

        Ok(())
    }

    /// Unsubscribe a token, also cleaning up its companion if no other
    /// subscribers remain for the companion.
    pub async fn unsubscribe_with_companion(&mut self, token_id: &str) {
        // Read-only lookup — don't nuke the mapping yet
        let companion = {
            let comps = self.companions.read().await;
            comps.get(token_id).cloned()
        };

        // Unsubscribe primary and remove its side of the mapping
        self.unsubscribe_single(token_id).await;
        {
            let mut comps = self.companions.write().await;
            comps.remove(token_id);
        }

        // Only tear down companion if nobody is still listening
        if let Some(comp) = companion {
            let has_receivers = {
                let senders = self.orderbook_senders.read().await;
                senders
                    .get(&comp)
                    .map(|(s, _)| s.receiver_count() > 0)
                    .unwrap_or(false)
            };
            if !has_receivers {
                self.unsubscribe_single(&comp).await;
                let mut comps = self.companions.write().await;
                comps.remove(&comp);
            }
        }
    }

    async fn unsubscribe_single(&mut self, token_id: &str) {
        {
            let mut subs = self.subscriptions.write().await;
            subs.remove(token_id);
        }
        {
            let mut senders = self.orderbook_senders.write().await;
            senders.remove(token_id);
        }
        {
            let mut senders = self.activity_senders.write().await;
            senders.remove(token_id);
        }
        // Keep self.orderbooks and asset/market mappings intact.
        // Polymarket WS has no unsubscribe protocol, so re-subscribing to the
        // same asset_id won't trigger a new BOOK snapshot. orderbook_stream()
        // uses the cached book to immediately seed new subscribers, preventing
        // an indefinite wait for a snapshot that may never arrive.
    }
}

impl Default for PolymarketWebSocket {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBookWebSocket for PolymarketWebSocket {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Connecting);

        let (ws_stream, _) = connect_async(WS_MARKET_URL)
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
        let subscriptions = self.subscriptions.clone();
        let orderbook_senders = self.orderbook_senders.clone();
        let activity_senders = self.activity_senders.clone();
        let orderbooks = self.orderbooks.clone();
        let asset_to_market = self.asset_to_market.clone();
        let market_to_assets = self.market_to_assets.clone();
        let user_subscribed_markets = self.user_subscribed_markets.clone();
        let write_tx_clone = self.write_tx.clone();
        let user_write_tx_clone = self.user_write_tx.clone();
        let user_auth = self.user_auth.clone();

        let ndjson_writer = self.ndjson_writer.clone();

        let ws_self = PolymarketWebSocket {
            state: state.clone(),
            subscriptions: subscriptions.clone(),
            orderbook_senders: orderbook_senders.clone(),
            activity_senders: activity_senders.clone(),
            orderbooks: orderbooks.clone(),
            asset_to_market: asset_to_market.clone(),
            market_to_assets: market_to_assets.clone(),
            user_subscribed_markets: user_subscribed_markets.clone(),
            write_tx: write_tx_clone.clone(),
            user_write_tx: user_write_tx_clone.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            user_shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect: self.auto_reconnect,
            reconnect_attempts: self.reconnect_attempts.clone(),
            user_auth,
            companions: self.companions.clone(),
            outcome_map: self.outcome_map.clone(),
            initial_subscribed: self.initial_subscribed.clone(),
            ndjson_writer: ndjson_writer.clone(),
        };

        let auto_reconnect = self.auto_reconnect;
        let reconnect_attempts_clone = self.reconnect_attempts.clone();

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

            let ping_future = async {
                let mut ping_interval = interval(WS_PING_INTERVAL);
                loop {
                    ping_interval.tick().await;
                    if let Some(ref tx) = *ws_self.write_tx.lock().await {
                        let _ = tx.unbounded_send(Message::Ping(vec![]));
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

            if auto_reconnect {
                let mut attempt = {
                    let mut a = reconnect_attempts_clone.lock().await;
                    *a += 1;
                    *a
                };

                tracing::warn!(
                    exchange = "polymarket",
                    attempt,
                    max = WS_MAX_RECONNECT_ATTEMPTS,
                    "websocket connection lost, starting reconnect"
                );

                while attempt <= WS_MAX_RECONNECT_ATTEMPTS {
                    state.store(WebSocketState::Reconnecting);

                    let delay = Self::calculate_reconnect_delay(attempt);
                    tracing::info!(
                        exchange = "polymarket",
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        "reconnect attempt starting"
                    );
                    tokio::time::sleep(delay).await;

                    match connect_async(WS_MARKET_URL).await {
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

                            // Broadcast reconnect event to all orderbook subscribers
                            {
                                let senders = orderbook_senders.read().await;
                                for (sender, seq) in senders.values() {
                                    let msg = WsMessage {
                                        seq: seq.fetch_add(1, Ordering::Relaxed),
                                        received_at: chrono::Utc::now(),
                                        data: OrderbookUpdate::Reconnected,
                                    };
                                    if let Some(ref writer) = ndjson_writer {
                                        writer.write_record(&msg);
                                    }
                                    let _ = sender.send(Ok(msg));
                                }
                            }

                            match ws_self.resubscribe_all().await {
                                Ok(()) => {
                                    let market_count = ws_self.subscriptions.read().await.len();
                                    tracing::info!(
                                        exchange = "polymarket",
                                        markets = market_count,
                                        "reconnected and resubscribed to all markets"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(exchange = "polymarket", error = %e, "resubscription failed after reconnect");
                                }
                            }
                            let _ = ws_self.connect_user_channel().await;

                            let write_future = new_rx.map(Ok).forward(new_write);
                            let read_future = async {
                                let mut read = new_read;
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

                            tokio::select! {
                                _ = write_future => {},
                                _ = read_future => {},
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
                        Err(e) => {
                            tracing::warn!(exchange = "polymarket", attempt, error = %e, "reconnect attempt failed");
                            attempt = {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a += 1;
                                *a
                            };
                        }
                    }
                }

                tracing::error!(
                    exchange = "polymarket",
                    max = WS_MAX_RECONNECT_ATTEMPTS,
                    "max reconnect attempts exhausted, giving up"
                );
                state.store(WebSocketState::Disconnected);
            }
        });

        self.set_state(WebSocketState::Connected);
        self.reset_reconnect_attempts().await;
        self.resubscribe_all().await?;
        let _ = self.connect_user_channel().await;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Closed);
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.user_shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }
        Ok(())
    }

    async fn subscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        let asset_ids = vec![market_id.to_string()];

        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(market_id.to_string(), asset_ids.clone());
        }

        {
            let mut senders = self.orderbook_senders.write().await;
            if !senders.contains_key(market_id) {
                let (tx, _) = broadcast::channel(16_384);
                senders.insert(market_id.to_string(), (tx, Arc::new(AtomicU64::new(0))));
            }
        }

        {
            let mut senders = self.activity_senders.write().await;
            if !senders.contains_key(market_id) {
                let (tx, _) = broadcast::channel(16_384);
                senders.insert(market_id.to_string(), (tx, Arc::new(AtomicU64::new(0))));
            }
        }

        self.send_market_subscribe(asset_ids).await?;

        if self.state.load() == WebSocketState::Connected {
            let maybe_market = {
                let map = self.asset_to_market.read().await;
                map.get(market_id).cloned()
            };
            if let Some(market_id) = maybe_market {
                self.ensure_user_market_subscription(&market_id).await;
            }
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        {
            let mut subs = self.subscriptions.write().await;
            subs.remove(market_id);
        }
        {
            let mut senders = self.orderbook_senders.write().await;
            senders.remove(market_id);
        }
        {
            let mut senders = self.activity_senders.write().await;
            senders.remove(market_id);
        }
        // Keep self.orderbooks and asset/market mappings intact (same
        // rationale as unsubscribe_single — Polymarket has no unsubscribe
        // protocol so cached books are needed to seed re-subscribers).
        Ok(())
    }

    fn state(&self) -> WebSocketState {
        self.state.load()
    }

    async fn orderbook_stream(
        &mut self,
        market_id: &str,
    ) -> Result<OrderbookStream, WebSocketError> {
        // Ensure sender exists — may be called before subscribe() to avoid the race where
        // an initial snapshot is broadcast with no receivers.
        let (sender, seq) = {
            let mut senders = self.orderbook_senders.write().await;
            if let Some((sender, seq)) = senders.get(market_id) {
                (sender.clone(), seq.clone())
            } else {
                let (tx, _) = broadcast::channel(16_384);
                let seq = Arc::new(AtomicU64::new(0));
                senders.insert(market_id.to_string(), (tx.clone(), seq.clone()));
                (tx, seq)
            }
        };

        let rx = sender.subscribe();

        // If we already have a cached book for this asset_id, immediately emit it.
        // This prevents "delta-only" streams where a late subscriber never sees a
        // full snapshot (common when subscribing via paired tokens).
        if let Some(cached) = self.orderbooks.read().await.get(market_id).cloned() {
            let msg = WsMessage {
                seq: seq.fetch_add(1, Ordering::Relaxed),
                received_at: chrono::Utc::now(),
                data: OrderbookUpdate::Snapshot(cached),
            };
            if let Some(ref writer) = self.ndjson_writer {
                writer.write_record(&msg);
            }
            let _ = sender.send(Ok(msg));
        }

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
                senders.insert(market_id.to_string(), (tx, Arc::new(AtomicU64::new(0))));
            }
        }
        let senders = self.activity_senders.read().await;
        let (sender, _seq) = senders.get(market_id).ok_or_else(|| {
            WebSocketError::Subscription(format!("no activity sender for market: {market_id}"))
        })?;
        let rx = sender.subscribe();
        Ok(Box::pin(
            tokio_stream::wrappers::BroadcastStream::new(rx)
                .filter_map(|result| async move { result.ok() }),
        ))
    }
}

pub async fn get_orderbook_snapshot(ws: &PolymarketWebSocket, asset_id: &str) -> Option<Orderbook> {
    let obs = ws.orderbooks.read().await;
    obs.get(asset_id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn user_trade_event_emits_fill_activity() {
        let mut ws = PolymarketWebSocket::new();
        ws.subscribe("asset-1")
            .await
            .expect("subscribe should succeed");
        let mut stream = ws
            .activity_stream("asset-1")
            .await
            .expect("activity stream should exist");

        let msg = r#"{
            "event_type":"trade",
            "asset_id":"asset-1",
            "market":"market-1",
            "id":"fill-1",
            "taker_order_id":"order-1",
            "price":"0.52",
            "size":"100",
            "side":"buy",
            "timestamp":"1736000000000"
        }"#;
        ws.handle_message(msg).await;

        let ws_msg = timeout(Duration::from_millis(300), stream.next())
            .await
            .expect("expected fill event")
            .expect("stream unexpectedly closed")
            .expect("stream event should be ok");

        match ws_msg.data {
            ActivityEvent::Fill(fill) => {
                assert_eq!(fill.market_id, "market-1");
                assert_eq!(fill.asset_id, "asset-1");
                assert_eq!(fill.fill_id.as_deref(), Some("fill-1"));
                assert_eq!(fill.order_id.as_deref(), Some("order-1"));
                assert_eq!(fill.source_channel, "polymarket_user_trade");
                assert!(fill.liquidity_role.is_none());
            }
            _ => panic!("expected fill event"),
        }
    }

    #[tokio::test]
    async fn user_trade_prefers_maker_order_for_user() {
        let mut ws = PolymarketWebSocket::with_auth(
            "user-key-1".to_string(),
            "secret".to_string(),
            "passphrase".to_string(),
        );
        ws.subscribe("asset-2")
            .await
            .expect("subscribe should succeed");
        let mut stream = ws
            .activity_stream("asset-2")
            .await
            .expect("activity stream should exist");

        let msg = r#"{
            "event_type":"trade",
            "asset_id":"asset-2",
            "market":"market-2",
            "id":"fill-2",
            "taker_order_id":"taker-2",
            "maker_orders":[
                {"order_id":"maker-1","owner":"other-key"},
                {"order_id":"maker-2","owner":"user-key-1"}
            ],
            "price":"0.61",
            "size":"50",
            "side":"sell",
            "timestamp":"1736000001000"
        }"#;
        ws.handle_message(msg).await;

        let ws_msg = timeout(Duration::from_millis(300), stream.next())
            .await
            .expect("expected fill event")
            .expect("stream unexpectedly closed")
            .expect("stream event should be ok");

        match ws_msg.data {
            ActivityEvent::Fill(fill) => {
                assert_eq!(fill.order_id.as_deref(), Some("maker-2"));
                assert_eq!(fill.liquidity_role, Some(LiquidityRole::Maker));
            }
            _ => panic!("expected fill event"),
        }
    }

    #[tokio::test]
    async fn user_trade_taker_role_from_trade_owner() {
        let mut ws = PolymarketWebSocket::with_auth(
            "user-key-1".to_string(),
            "secret".to_string(),
            "passphrase".to_string(),
        );
        ws.subscribe("asset-3")
            .await
            .expect("subscribe should succeed");
        let mut stream = ws
            .activity_stream("asset-3")
            .await
            .expect("activity stream should exist");

        let msg = r#"{
            "event_type":"trade",
            "asset_id":"asset-3",
            "market":"market-3",
            "id":"fill-3",
            "taker_order_id":"taker-3",
            "trade_owner":"user-key-1",
            "maker_orders":[
                {"order_id":"maker-1","owner":"other-key"}
            ],
            "price":"0.55",
            "size":"25",
            "side":"buy",
            "timestamp":"1736000002000"
        }"#;
        ws.handle_message(msg).await;

        let ws_msg = timeout(Duration::from_millis(300), stream.next())
            .await
            .expect("expected fill event")
            .expect("stream unexpectedly closed")
            .expect("stream event should be ok");

        match ws_msg.data {
            ActivityEvent::Fill(fill) => {
                assert_eq!(fill.liquidity_role, Some(LiquidityRole::Taker));
                assert_eq!(fill.order_id.as_deref(), Some("taker-3"));
            }
            _ => panic!("expected fill event"),
        }
    }

    #[tokio::test]
    async fn user_trade_no_auth_has_no_liquidity_role() {
        let mut ws = PolymarketWebSocket::new();
        ws.subscribe("asset-4")
            .await
            .expect("subscribe should succeed");
        let mut stream = ws
            .activity_stream("asset-4")
            .await
            .expect("activity stream should exist");

        let msg = r#"{
            "event_type":"trade",
            "asset_id":"asset-4",
            "market":"market-4",
            "id":"fill-4",
            "taker_order_id":"taker-4",
            "trade_owner":"some-other-key",
            "maker_orders":[
                {"order_id":"maker-1","owner":"another-key"}
            ],
            "price":"0.70",
            "size":"10",
            "side":"buy",
            "timestamp":"1736000003000"
        }"#;
        ws.handle_message(msg).await;

        let ws_msg = timeout(Duration::from_millis(300), stream.next())
            .await
            .expect("expected fill event")
            .expect("stream unexpectedly closed")
            .expect("stream event should be ok");

        match ws_msg.data {
            ActivityEvent::Fill(fill) => {
                assert!(fill.liquidity_role.is_none());
            }
            _ => panic!("expected fill event"),
        }
    }

    #[tokio::test]
    async fn user_order_event_does_not_emit_fill_activity() {
        let mut ws = PolymarketWebSocket::new();
        ws.subscribe("asset-1")
            .await
            .expect("subscribe should succeed");
        let mut stream = ws
            .activity_stream("asset-1")
            .await
            .expect("activity stream should exist");

        let msg = r#"{
            "event_type":"order",
            "asset_id":"asset-1",
            "market":"market-1",
            "id":"order-event-1",
            "order_id":"order-1",
            "status":"matched",
            "price":"0.52",
            "matched_size":"100",
            "side":"buy",
            "timestamp":"1736000000000"
        }"#;
        ws.handle_message(msg).await;

        let maybe_event = timeout(Duration::from_millis(300), stream.next()).await;
        assert!(
            maybe_event.is_err(),
            "order lifecycle events should not emit fill activity"
        );
    }

    /// Simulates the ExchangeWsClient flow: orderbook_stream → subscribe → book event.
    /// This is the exact path used by the WS manager for Polymarket.
    #[tokio::test]
    async fn orderbook_stream_before_subscribe_receives_snapshot() {
        let mut ws = PolymarketWebSocket::with_config(false);

        // Step 1: Create receiver BEFORE subscribe (like ExchangeWsClient does)
        let mut stream = ws
            .orderbook_stream("token-yes")
            .await
            .expect("orderbook_stream should succeed");

        // Step 2: Subscribe (sends WS message in real code, here just registers)
        ws.subscribe("token-yes")
            .await
            .expect("subscribe should succeed");

        // Step 3: Simulate a book event arriving from Polymarket
        let book_msg = r#"{
            "event_type": "book",
            "asset_id": "token-yes",
            "market": "market-1",
            "bids": [{"price": "0.55", "size": "100"}, {"price": "0.54", "size": "200"}],
            "asks": [{"price": "0.56", "size": "150"}, {"price": "0.57", "size": "250"}]
        }"#;
        ws.handle_message(book_msg).await;

        // Step 4: Verify the stream receives the snapshot
        let ws_msg = timeout(Duration::from_millis(500), stream.next())
            .await
            .expect("should receive snapshot within timeout")
            .expect("stream should not be closed")
            .expect("event should be Ok");

        match ws_msg.data {
            OrderbookUpdate::Snapshot(ob) => {
                assert_eq!(ob.bids.len(), 2);
                assert_eq!(ob.asks.len(), 2);
                assert_eq!(ob.bids[0].price, FixedPrice::from_f64(0.55));
            }
            _ => panic!("expected Snapshot, got Delta"),
        }
    }

    /// Simulates subscribe_pair flow with companion token
    #[tokio::test]
    async fn subscribe_pair_delivers_snapshot_to_stream() {
        let mut ws = PolymarketWebSocket::with_config(false);

        // Step 1: Create receiver BEFORE subscribe_pair (like ExchangeWsClient does)
        let mut stream = ws
            .orderbook_stream("token-yes")
            .await
            .expect("orderbook_stream should succeed");

        // Step 2: Subscribe pair
        ws.subscribe_pair("token-yes", "token-no")
            .await
            .expect("subscribe_pair should succeed");

        // Step 3: Simulate book event
        let book_msg = r#"{
            "event_type": "book",
            "asset_id": "token-yes",
            "market": "market-1",
            "bids": [{"price": "0.55", "size": "100"}],
            "asks": [{"price": "0.56", "size": "150"}]
        }"#;
        ws.handle_message(book_msg).await;

        // Step 4: Verify snapshot arrives
        let ws_msg = timeout(Duration::from_millis(500), stream.next())
            .await
            .expect("should receive snapshot")
            .expect("stream not closed")
            .expect("event ok");

        match ws_msg.data {
            OrderbookUpdate::Snapshot(ob) => {
                assert_eq!(ob.bids.len(), 1);
                assert_eq!(ob.asks.len(), 1);
            }
            _ => panic!("expected Snapshot"),
        }
    }

    /// Integration test: connect to real Polymarket WS and verify snapshot delivery.
    /// Uses the ExchangeWsClient wrapper (same as production).
    #[tokio::test]
    #[ignore] // requires network access
    async fn live_polymarket_ws_delivers_snapshot() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use tokio::sync::Mutex;

        // tokio-tungstenite with rustls requires the crypto provider
        let _ = tokio_tungstenite::tungstenite::http::Uri::from_static("wss://example.com");

        struct Client {
            inner: Mutex<PolymarketWebSocket>,
            connected: AtomicBool,
        }

        let client = Arc::new(Client {
            inner: Mutex::new(PolymarketWebSocket::new()),
            connected: AtomicBool::new(false),
        });

        // Connect
        {
            let mut inner = client.inner.lock().await;
            inner.connect().await.expect("connect should succeed");
            client.connected.store(true, Ordering::Relaxed);
        }

        // Use a known active market token_id
        let token_id =
            "7648925155044397287047798308912067661131447591491670430094875295487039626662";

        // Create receiver BEFORE subscribe (same as ExchangeWsClient::subscribe)
        let mut stream;
        {
            let mut inner = client.inner.lock().await;
            stream = inner
                .orderbook_stream(token_id)
                .await
                .expect("orderbook_stream ok");
            inner.subscribe(token_id).await.expect("subscribe ok");
        }

        // Wait for snapshot
        let ws_msg = timeout(Duration::from_secs(10), stream.next())
            .await
            .expect("should receive snapshot within 10s")
            .expect("stream not closed")
            .expect("event ok");

        match ws_msg.data {
            OrderbookUpdate::Snapshot(ob) => {
                eprintln!(
                    "Received snapshot: {} bids, {} asks",
                    ob.bids.len(),
                    ob.asks.len()
                );
                assert!(!ob.bids.is_empty(), "expected non-empty bids");
                assert!(!ob.asks.is_empty(), "expected non-empty asks");
            }
            _ => panic!("expected Snapshot but got Delta"),
        }

        // Cleanup
        {
            let mut inner = client.inner.lock().await;
            let _ = inner.disconnect().await;
        }
    }
}
