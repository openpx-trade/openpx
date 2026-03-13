use async_trait::async_trait;
use futures::{FutureExt, StreamExt};
use rust_socketio::{
    asynchronous::{Client, ClientBuilder},
    Payload,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use px_core::{
    sort_asks, sort_bids, ActivityEvent, ActivityFill, ActivityStream, AtomicWebSocketState,
    OrderBookWebSocket, Orderbook, OrderbookStream, OrderbookUpdate, PriceLevel, WebSocketError,
    WebSocketState, WS_MAX_RECONNECT_ATTEMPTS, WS_RECONNECT_BASE_DELAY, WS_RECONNECT_MAX_DELAY,
};

use crate::config::LimitlessConfig;

const NAMESPACE: &str = "/markets";

#[derive(Debug, Clone, Serialize)]
struct SubscribePayload {
    #[serde(rename = "marketSlugs", skip_serializing_if = "Vec::is_empty")]
    market_slugs: Vec<String>,
    #[serde(rename = "marketAddresses", skip_serializing_if = "Vec::is_empty")]
    market_addresses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OrderbookUpdateData {
    #[serde(rename = "marketSlug", alias = "slug")]
    market_slug: Option<String>,
    orderbook: Option<OrderbookData>,
    bids: Option<Vec<PriceLevelData>>,
    asks: Option<Vec<PriceLevelData>>,
    #[allow(dead_code)]
    timestamp: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct OrderbookData {
    bids: Option<Vec<PriceLevelData>>,
    asks: Option<Vec<PriceLevelData>>,
}

#[derive(Debug, Clone, Deserialize)]
struct PriceLevelData {
    price: serde_json::Value,
    size: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct PriceUpdateData {
    #[serde(rename = "marketAddress")]
    market_address: Option<String>,
    #[serde(rename = "updatedPrices")]
    updated_prices: Option<PriceData>,
    #[serde(rename = "blockNumber")]
    #[allow(dead_code)]
    block_number: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct PriceData {
    yes: Option<f64>,
    no: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct PositionUpdateData {
    positions: Option<Vec<PositionEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
struct PositionEntry {
    #[serde(rename = "marketSlug", alias = "market_slug")]
    market_slug: Option<String>,
    #[serde(rename = "tokenId", alias = "token_id")]
    token_id: Option<String>,
    outcome: Option<String>,
    size: Option<f64>,
    price: Option<f64>,
    side: Option<String>,
    #[serde(rename = "fillId", alias = "fill_id")]
    fill_id: Option<String>,
    timestamp: Option<String>,
}

type OrderbookSender = broadcast::Sender<Result<OrderbookUpdate, WebSocketError>>;
type ActivitySender = broadcast::Sender<Result<ActivityEvent, WebSocketError>>;

struct SharedState {
    subscribed_slugs: Vec<String>,
    subscribed_addresses: Vec<String>,
    orderbook_senders: HashMap<String, OrderbookSender>,
    orderbooks: HashMap<String, Orderbook>,
    position_subscribed: bool,
}

impl SharedState {
    fn new() -> Self {
        Self {
            subscribed_slugs: Vec::new(),
            subscribed_addresses: Vec::new(),
            orderbook_senders: HashMap::new(),
            orderbooks: HashMap::new(),
            position_subscribed: false,
        }
    }
}

pub struct LimitlessWebSocket {
    config: LimitlessConfig,
    ws_state: Arc<AtomicWebSocketState>,
    shared: Arc<RwLock<SharedState>>,
    client: Arc<RwLock<Option<Client>>>,
    activity_senders: Arc<RwLock<HashMap<String, ActivitySender>>>,
    auto_reconnect: bool,
}

impl LimitlessWebSocket {
    pub fn new(config: LimitlessConfig) -> Self {
        Self::with_config(config, true)
    }

    pub fn with_config(config: LimitlessConfig, auto_reconnect: bool) -> Self {
        Self {
            config,
            ws_state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            shared: Arc::new(RwLock::new(SharedState::new())),
            client: Arc::new(RwLock::new(None)),
            activity_senders: Arc::new(RwLock::new(HashMap::new())),
            auto_reconnect,
        }
    }

    fn set_state(&self, new_state: WebSocketState) {
        self.ws_state.store(new_state);
    }

    async fn send_subscription(&self) -> Result<(), WebSocketError> {
        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| WebSocketError::Connection("not connected".into()))?;

        let shared = self.shared.read().await;
        if shared.subscribed_slugs.is_empty() && shared.subscribed_addresses.is_empty() {
            return Ok(());
        }

        let payload = SubscribePayload {
            market_slugs: shared.subscribed_slugs.clone(),
            market_addresses: shared.subscribed_addresses.clone(),
        };

        let json =
            serde_json::to_value(&payload).map_err(|e| WebSocketError::Protocol(e.to_string()))?;

        client
            .emit("subscribe_market_prices", json)
            .await
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;

        Ok(())
    }

    async fn send_position_subscription(&self) -> Result<(), WebSocketError> {
        let client_guard = self.client.read().await;
        let client = client_guard
            .as_ref()
            .ok_or_else(|| WebSocketError::Connection("not connected".into()))?;

        client
            .emit("subscribe_positions", serde_json::json!({}))
            .await
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;

        Ok(())
    }

    fn parse_price_level(data: &PriceLevelData) -> Option<PriceLevel> {
        let price = match &data.price {
            serde_json::Value::Number(n) => n.as_f64()?,
            serde_json::Value::String(s) => s.parse::<f64>().ok()?,
            _ => return None,
        };

        let size = match &data.size {
            serde_json::Value::Number(n) => n.as_f64()?,
            serde_json::Value::String(s) => s.parse::<f64>().ok()?,
            _ => return None,
        };

        if price > 0.0 && size > 0.0 {
            Some(PriceLevel::new(price, size))
        } else {
            None
        }
    }

    async fn handle_orderbook_update(shared: Arc<RwLock<SharedState>>, data: OrderbookUpdateData) {
        let market_slug = match data.market_slug {
            Some(s) => s,
            None => return,
        };

        let (raw_bids, raw_asks) = if let Some(ob) = data.orderbook {
            (ob.bids.unwrap_or_default(), ob.asks.unwrap_or_default())
        } else {
            (data.bids.unwrap_or_default(), data.asks.unwrap_or_default())
        };

        let mut bids: Vec<PriceLevel> = raw_bids
            .iter()
            .filter_map(Self::parse_price_level)
            .collect();
        let mut asks: Vec<PriceLevel> = raw_asks
            .iter()
            .filter_map(Self::parse_price_level)
            .collect();

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        let orderbook = Orderbook {
            market_id: market_slug.clone(),
            asset_id: market_slug.clone(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
        };

        let mut shared = shared.write().await;
        shared
            .orderbooks
            .insert(market_slug.clone(), orderbook.clone());

        if let Some(sender) = shared.orderbook_senders.get(&market_slug) {
            let _ = sender.send(Ok(OrderbookUpdate::Snapshot(orderbook)));
        }
    }

    async fn handle_price_update(shared: Arc<RwLock<SharedState>>, data: PriceUpdateData) {
        let market_address = match data.market_address {
            Some(a) => a,
            None => return,
        };

        let prices = match data.updated_prices {
            Some(p) => p,
            None => return,
        };

        let yes_price = prices.yes.unwrap_or(0.0);
        let no_price = prices.no.unwrap_or(0.0);

        if yes_price <= 0.0 && no_price <= 0.0 {
            return;
        }

        let mut bids = Vec::new();
        let mut asks = Vec::new();

        if yes_price > 0.0 {
            bids.push(PriceLevel::new(yes_price, 1.0));
        }
        if no_price > 0.0 {
            asks.push(PriceLevel::new(1.0 - no_price, 1.0));
        }

        let orderbook = Orderbook {
            market_id: market_address.clone(),
            asset_id: market_address.clone(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
        };

        let mut shared = shared.write().await;
        shared
            .orderbooks
            .insert(market_address.clone(), orderbook.clone());

        if let Some(sender) = shared.orderbook_senders.get(&market_address) {
            let _ = sender.send(Ok(OrderbookUpdate::Snapshot(orderbook)));
        }
    }

    async fn handle_position_update(
        activity_senders: Arc<RwLock<HashMap<String, ActivitySender>>>,
        data: PositionUpdateData,
    ) {
        let entries = match data.positions {
            Some(p) => p,
            None => return,
        };

        let senders = activity_senders.read().await;
        for entry in entries {
            let market_id = entry.market_slug.unwrap_or_default();
            if market_id.is_empty() {
                continue;
            }

            let fill = ActivityFill {
                market_id: market_id.clone(),
                asset_id: entry.token_id.unwrap_or_default(),
                fill_id: entry.fill_id,
                order_id: None,
                price: entry.price.unwrap_or(0.0),
                size: entry.size.unwrap_or(0.0),
                side: entry.side,
                outcome: entry.outcome,
                timestamp: entry.timestamp.and_then(|t| {
                    chrono::DateTime::parse_from_rfc3339(&t)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                source_channel: "limitless_positions".into(),
                liquidity_role: None,
            };

            let event = ActivityEvent::Fill(fill);
            if let Some(sender) = senders.get(&market_id) {
                let _ = sender.send(Ok(event));
            }
        }
    }

    pub async fn subscribe_positions(&mut self) -> Result<(), WebSocketError> {
        {
            let mut shared = self.shared.write().await;
            shared.position_subscribed = true;
        }

        if self.ws_state.load() == WebSocketState::Connected {
            self.send_position_subscription().await?;
        }

        Ok(())
    }
}

impl Default for LimitlessWebSocket {
    fn default() -> Self {
        Self::new(LimitlessConfig::default())
    }
}

#[async_trait]
impl OrderBookWebSocket for LimitlessWebSocket {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Connecting);

        let shared_orderbook = self.shared.clone();
        let shared_price = self.shared.clone();
        let shared_connect = self.shared.clone();
        let activity_senders_positions = self.activity_senders.clone();
        let state_connect = self.ws_state.clone();
        let state_disconnect = self.ws_state.clone();

        let mut builder = ClientBuilder::new(&self.config.ws_url).namespace(NAMESPACE);

        if let Some(ref api_key) = self.config.api_key {
            builder = builder.opening_header("X-API-Key", api_key.as_str());
        }

        if self.auto_reconnect {
            builder = builder
                .reconnect(true)
                .reconnect_on_disconnect(true)
                .reconnect_delay(
                    WS_RECONNECT_BASE_DELAY.as_millis() as u64,
                    WS_RECONNECT_MAX_DELAY.as_millis() as u64,
                )
                .max_reconnect_attempts(WS_MAX_RECONNECT_ATTEMPTS as u8);
        }

        let client = builder
            .on("connect", move |_, client: Client| {
                let state = state_connect.clone();
                let shared = shared_connect.clone();
                async move {
                    state.store(WebSocketState::Connected);
                    tracing::debug!("Connected to Limitless WebSocket");

                    // Auto-resubscribe on reconnect
                    let shared = shared.read().await;
                    if !shared.subscribed_slugs.is_empty()
                        || !shared.subscribed_addresses.is_empty()
                    {
                        let payload = SubscribePayload {
                            market_slugs: shared.subscribed_slugs.clone(),
                            market_addresses: shared.subscribed_addresses.clone(),
                        };
                        if let Ok(json) = serde_json::to_value(&payload) {
                            let _ = client.emit("subscribe_market_prices", json).await;
                        }
                    }
                    if shared.position_subscribed {
                        let _ = client
                            .emit("subscribe_positions", serde_json::json!({}))
                            .await;
                    }
                }
                .boxed()
            })
            .on("disconnect", move |_, _| {
                let state = state_disconnect.clone();
                async move {
                    state.store(WebSocketState::Disconnected);
                    tracing::debug!("Disconnected from Limitless WebSocket");
                }
                .boxed()
            })
            .on("authenticated", |_, _| {
                async move {
                    tracing::info!("Limitless WebSocket authenticated");
                }
                .boxed()
            })
            .on("orderbookUpdate", move |payload, _| {
                let shared = shared_orderbook.clone();
                async move {
                    if let Payload::Text(values) = payload {
                        for value in values {
                            if let Ok(data) = serde_json::from_value::<OrderbookUpdateData>(value) {
                                Self::handle_orderbook_update(shared.clone(), data).await;
                            }
                        }
                    }
                }
                .boxed()
            })
            .on("newPriceData", move |payload, _| {
                let shared = shared_price.clone();
                async move {
                    if let Payload::Text(values) = payload {
                        for value in values {
                            if let Ok(data) = serde_json::from_value::<PriceUpdateData>(value) {
                                Self::handle_price_update(shared.clone(), data).await;
                            }
                        }
                    }
                }
                .boxed()
            })
            .on("positions", move |payload, _| {
                let senders = activity_senders_positions.clone();
                async move {
                    if let Payload::Text(values) = payload {
                        for value in values {
                            if let Ok(data) =
                                serde_json::from_value::<PositionUpdateData>(value)
                            {
                                Self::handle_position_update(senders.clone(), data).await;
                            }
                        }
                    }
                }
                .boxed()
            })
            .on("system", |payload, _| {
                async move {
                    if let Payload::Text(values) = payload {
                        for value in values {
                            let message = value
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            tracing::info!("Limitless system: {}", message);
                        }
                    }
                }
                .boxed()
            })
            .on("exception", |payload, _| {
                async move {
                    tracing::warn!("Limitless WebSocket exception: {:?}", payload);
                }
                .boxed()
            })
            .connect()
            .await
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;

        {
            let mut client_guard = self.client.write().await;
            *client_guard = Some(client);
        }

        self.set_state(WebSocketState::Connected);

        self.send_subscription().await?;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), WebSocketError> {
        self.set_state(WebSocketState::Closed);

        let mut client_guard = self.client.write().await;
        if let Some(client) = client_guard.take() {
            client
                .disconnect()
                .await
                .map_err(|e| WebSocketError::Connection(e.to_string()))?;
        }

        Ok(())
    }

    async fn subscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        {
            let mut shared = self.shared.write().await;
            if !shared.subscribed_slugs.contains(&market_id.to_string()) {
                shared.subscribed_slugs.push(market_id.to_string());
            }
            if !shared.orderbook_senders.contains_key(market_id) {
                let (tx, _) = broadcast::channel(16_384);
                shared.orderbook_senders.insert(market_id.to_string(), tx);
            }
        }

        if self.ws_state.load() == WebSocketState::Connected {
            self.send_subscription().await?;
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        {
            let mut shared = self.shared.write().await;
            shared.subscribed_slugs.retain(|s| s != market_id);
            shared.subscribed_addresses.retain(|s| s != market_id);
            shared.orderbook_senders.remove(market_id);
            shared.orderbooks.remove(market_id);
        }

        if self.ws_state.load() == WebSocketState::Connected {
            self.send_subscription().await?;
        }

        Ok(())
    }

    fn state(&self) -> WebSocketState {
        self.ws_state.load()
    }

    async fn orderbook_stream(
        &mut self,
        market_id: &str,
    ) -> Result<OrderbookStream, WebSocketError> {
        // Ensure sender exists — may be called before subscribe() to avoid
        // the race where the initial snapshot is broadcast with no receivers.
        {
            let mut shared = self.shared.write().await;
            if !shared.orderbook_senders.contains_key(market_id) {
                let (tx, _) = broadcast::channel(16_384);
                shared.orderbook_senders.insert(market_id.to_string(), tx);
            }
        }

        let shared = self.shared.read().await;
        let sender = shared.orderbook_senders.get(market_id).ok_or_else(|| {
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

impl LimitlessWebSocket {
    pub async fn subscribe_market_address(
        &mut self,
        market_address: &str,
    ) -> Result<(), WebSocketError> {
        {
            let mut shared = self.shared.write().await;
            if !shared
                .subscribed_addresses
                .contains(&market_address.to_string())
            {
                shared.subscribed_addresses.push(market_address.to_string());
            }
            if !shared.orderbook_senders.contains_key(market_address) {
                let (tx, _) = broadcast::channel(16_384);
                shared
                    .orderbook_senders
                    .insert(market_address.to_string(), tx);
            }
        }

        if self.ws_state.load() == WebSocketState::Connected {
            self.send_subscription().await?;
        }

        Ok(())
    }

    pub async fn get_orderbook(&self, market_id: &str) -> Option<Orderbook> {
        let shared = self.shared.read().await;
        shared.orderbooks.get(market_id).cloned()
    }
}
