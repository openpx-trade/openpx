use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message},
};

use px_core::{
    now_pair, sort_asks, sort_bids, stall_watchdog, ActivityFill, ActivityTrade,
    AtomicWebSocketState, ChangeVec, FixedPrice, InvalidationReason, LiquidityRole,
    OrderBookWebSocket, Orderbook, PriceLevel, PriceLevelChange, PriceLevelSide, SessionEvent,
    SessionStream, UpdateStream, WebSocketError, WebSocketState, WsDispatcher, WsDispatcherConfig,
    WsUpdate, WS_MAX_RECONNECT_ATTEMPTS, WS_PING_INTERVAL, WS_RECONNECT_BASE_DELAY,
    WS_RECONNECT_MAX_DELAY,
};

use crate::{auth::KalshiAuth, KalshiConfig, KalshiError};

const WS_PATH: &str = "/trade-api/ws/v2";

/// Per-market monotonic sequence counter map. The dispatcher multiplexes all
/// markets onto one channel; sequencing stays scoped to the emitting market.
type SeqMap = Arc<RwLock<HashMap<String, Arc<AtomicU64>>>>;

#[derive(Debug, Deserialize)]
struct SnapshotPayload {
    market_ticker: String,
    #[allow(dead_code)]
    market_id: String,
    #[serde(default)]
    yes_dollars_fp: Option<Vec<[String; 2]>>,
    #[serde(default)]
    no_dollars_fp: Option<Vec<[String; 2]>>,
}

#[derive(Debug, Deserialize)]
struct DeltaPayload {
    market_ticker: String,
    #[allow(dead_code)]
    market_id: String,
    price_dollars: String,
    delta_fp: String,
    side: String,
    #[serde(default)]
    ts: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorPayload {
    code: i64,
    msg: String,
}

pub struct KalshiWebSocket {
    config: KalshiConfig,
    auth: Option<Arc<KalshiAuth>>,
    api_key_id: Option<String>,
    state: Arc<AtomicWebSocketState>,
    subscriptions: Arc<RwLock<HashMap<String, Option<u64>>>>,
    pending: Arc<RwLock<HashMap<u64, String>>>,
    orderbooks: Arc<RwLock<HashMap<String, Orderbook>>>,
    /// Multiplexed dispatch handle. Owns both halves of the bounded channels
    /// backing `updates()` / `session_events()`.
    dispatcher: Arc<WsDispatcher>,
    /// Per-market monotonic sequence counters.
    seqs: SeqMap,
    /// Wall-clock of the last successfully received WS message; powers the
    /// `gap_ms` field on `SessionEvent::Reconnected`.
    last_message_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    auto_reconnect: bool,
    reconnect_attempts: Arc<Mutex<u32>>,
    command_id: Arc<AtomicU64>,
    enable_user_fills: bool,
}

impl KalshiWebSocket {
    pub fn new(config: KalshiConfig) -> Result<Self, KalshiError> {
        Self::with_config(config, true, false)
    }

    pub fn with_user_fills(config: KalshiConfig) -> Result<Self, KalshiError> {
        Self::with_config(config, true, true)
    }

    pub fn with_config(
        config: KalshiConfig,
        auto_reconnect: bool,
        enable_user_fills: bool,
    ) -> Result<Self, KalshiError> {
        let auth = if config.is_authenticated() {
            let auth = if let Some(ref path) = config.private_key_path {
                KalshiAuth::from_file(path)?
            } else if let Some(ref pem) = config.private_key_pem {
                KalshiAuth::from_pem(pem)?
            } else {
                return Err(KalshiError::AuthRequired);
            };
            Some(Arc::new(auth))
        } else {
            None
        };

        Ok(Self {
            api_key_id: config.api_key_id.clone(),
            config,
            auth,
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(HashMap::new())),
            orderbooks: Arc::new(RwLock::new(HashMap::new())),
            dispatcher: Arc::new(WsDispatcher::new(WsDispatcherConfig::default())),
            seqs: Arc::new(RwLock::new(HashMap::new())),
            last_message_at: Arc::new(RwLock::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect,
            reconnect_attempts: Arc::new(Mutex::new(0)),
            command_id: Arc::new(AtomicU64::new(0)),
            enable_user_fills,
        })
    }

    /// Allocate-or-fetch the per-market sequence counter for the 0.2 dispatch
    /// path. Lazy: created on first emit so subscribe stays a pure protocol op.
    async fn dispatcher_seq(&self, market_id: &str) -> Arc<AtomicU64> {
        {
            let map = self.seqs.read().await;
            if let Some(s) = map.get(market_id) {
                return s.clone();
            }
        }
        let mut map = self.seqs.write().await;
        map.entry(market_id.to_string())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone()
    }

    /// Emit a `WsUpdate` through the 0.2 dispatcher. On overflow raises
    /// `SessionEvent::Lagged` + `BookInvalidated(Lag)` + a best-effort
    /// `WsUpdate::Clear` on the update stream so consumers can invalidate
    /// without merging session + update streams — correctness fix vs.
    /// tokio broadcast's lag-and-skip.
    async fn dispatch(&self, update: WsUpdate) {
        let market = update.market_id().map(str::to_string);
        let asset = update.asset_id().map(str::to_string);
        if !self.dispatcher.try_send_update(update) {
            self.dispatcher
                .send_session(SessionEvent::Lagged {
                    dropped: 1,
                    first_seq: 0,
                    last_seq: 0,
                })
                .await;
            if let (Some(market_id), Some(asset_id)) = (market, asset) {
                self.dispatcher
                    .send_session(SessionEvent::BookInvalidated {
                        market_id: market_id.clone(),
                        reason: InvalidationReason::Lag,
                    })
                    .await;
                let (local_ts, local_ts_ms) = now_pair();
                let seq = self
                    .dispatcher_seq(&market_id)
                    .await
                    .fetch_add(1, Ordering::Relaxed);
                // Best-effort: channel may still be full. If it drops, the
                // session `BookInvalidated` already signaled the invalidation.
                let _ = self.dispatcher.try_send_update(WsUpdate::Clear {
                    market_id,
                    asset_id,
                    reason: InvalidationReason::Lag,
                    local_ts,
                    local_ts_ms,
                    seq,
                });
            }
        }
    }

    fn set_state(&self, new_state: WebSocketState) {
        self.state.store(new_state);
    }

    async fn reset_reconnect_attempts(&self) {
        let mut attempts = self.reconnect_attempts.lock().await;
        *attempts = 0;
    }

    fn next_command_id(&self) -> u64 {
        self.command_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn ws_url(&self) -> Result<reqwest::Url, WebSocketError> {
        let api_url = reqwest::Url::parse(&self.config.api_url)
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;
        let scheme = if api_url.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        let host = api_url
            .host_str()
            .ok_or_else(|| WebSocketError::Connection("missing host".into()))?;
        let port = api_url.port().map(|p| format!(":{p}")).unwrap_or_default();
        let url = format!("{scheme}://{host}{port}{WS_PATH}");
        reqwest::Url::parse(&url).map_err(|e| WebSocketError::Connection(e.to_string()))
    }

    fn build_request(
        &self,
    ) -> Result<tokio_tungstenite::tungstenite::http::Request<()>, WebSocketError> {
        let api_key_id = self
            .api_key_id
            .as_ref()
            .ok_or_else(|| WebSocketError::Connection("authentication required".into()))?;
        let auth = self
            .auth
            .as_ref()
            .ok_or_else(|| WebSocketError::Connection("authentication required".into()))?;

        let url = self.ws_url()?;
        // Build a valid websocket client request so tungstenite populates required
        // handshake headers (Sec-WebSocket-Key, Upgrade, Connection, etc.).
        let mut request = url
            .as_str()
            .into_client_request()
            .map_err(|e| WebSocketError::Connection(e.to_string()))?;

        let timestamp_ms = chrono::Utc::now().timestamp_millis();
        let signature = auth.sign(timestamp_ms, "GET", WS_PATH);

        let headers = request.headers_mut();
        headers.insert(
            "KALSHI-ACCESS-KEY",
            HeaderValue::from_str(api_key_id)
                .map_err(|e| WebSocketError::Protocol(e.to_string()))?,
        );
        headers.insert(
            "KALSHI-ACCESS-SIGNATURE",
            HeaderValue::from_str(&signature)
                .map_err(|e| WebSocketError::Protocol(e.to_string()))?,
        );
        headers.insert(
            "KALSHI-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp_ms.to_string())
                .map_err(|e| WebSocketError::Protocol(e.to_string()))?,
        );
        headers.insert("User-Agent", HeaderValue::from_static("openpx/1.0"));

        Ok(request)
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

    fn parse_levels(levels: Option<Vec<[String; 2]>>) -> Vec<PriceLevel> {
        // px_core::parse_level skips the f64 round-trip — scans decimal
        // strings directly into u32/i64 ticks.
        levels
            .unwrap_or_default()
            .into_iter()
            .filter_map(|[price_str, size_str]| px_core::parse_level(&price_str, &size_str))
            .collect()
    }

    /// Apply a Kalshi delta to a sorted level vec and return the resulting
    /// absolute size at `fp`. `0.0` means the level is no longer present
    /// (either removed by the delta or never existed). Binary search keeps
    /// the lookup at O(log n) on the typical 20-50 level book; the previous
    /// `iter().position(...)` was O(n) and was followed by another full
    /// scan in the caller to read the post-apply size.
    fn update_levels(
        levels: &mut Vec<PriceLevel>,
        fp: FixedPrice,
        delta: f64,
        descending: bool,
    ) -> f64 {
        let search = if descending {
            levels.binary_search_by(|l| fp.cmp(&l.price))
        } else {
            levels.binary_search_by(|l| l.price.cmp(&fp))
        };
        match search {
            Ok(idx) => {
                let new_size = levels[idx].size + delta;
                if new_size <= 0.0 {
                    levels.remove(idx);
                    0.0
                } else {
                    levels[idx].size = new_size;
                    new_size
                }
            }
            Err(idx) => {
                if delta > 0.0 {
                    levels.insert(idx, PriceLevel::with_fixed(fp, delta));
                    delta
                } else {
                    0.0
                }
            }
        }
    }

    async fn handle_message(&self, text: &str, local_ts: Instant, local_ts_ms: u64) {
        // Shared SIMD-accelerated parse with size-based switching (simd-json
        // above 512 B, serde_json below).
        let Some(value) = px_core::decode_value(text) else {
            return;
        };

        let msg_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match msg_type {
            "orderbook_snapshot" => self.handle_snapshot(&value, local_ts, local_ts_ms).await,
            "orderbook_delta" => self.handle_delta(&value, local_ts, local_ts_ms).await,
            "trade" => self.handle_trade(&value, local_ts, local_ts_ms).await,
            "fill" => self.handle_fill(&value, local_ts, local_ts_ms).await,
            "subscribed" => self.handle_subscribed(&value).await,
            "unsubscribed" => self.handle_unsubscribed(&value).await,
            "error" => self.handle_error(&value).await,
            _ => {}
        }
    }

    fn value_to_string(value: Option<&serde_json::Value>) -> Option<String> {
        match value {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(serde_json::Value::Number(n)) => Some(n.to_string()),
            _ => None,
        }
    }

    fn value_to_f64(value: Option<&serde_json::Value>) -> Option<f64> {
        match value {
            Some(serde_json::Value::Number(n)) => n.as_f64(),
            Some(serde_json::Value::String(s)) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Normalize Kalshi `ts` into exchange millis since epoch. Accepts seconds,
    /// millis, or RFC3339 strings.
    fn value_to_ts_ms(value: Option<&serde_json::Value>) -> Option<u64> {
        let value = value?;
        if let Some(s) = value.as_str() {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return u64::try_from(dt.timestamp_millis()).ok();
            }
            if let Ok(raw) = s.parse::<i64>() {
                return Self::raw_to_ms(raw);
            }
            return None;
        }
        value.as_i64().and_then(Self::raw_to_ms)
    }

    /// Convert a Kalshi timestamp (seconds or millis) to millis. Kalshi emits
    /// both; heuristic: a value < 10^12 is seconds, otherwise millis.
    fn raw_to_ms(raw: i64) -> Option<u64> {
        if raw < 0 {
            return None;
        }
        let v = if raw < 1_000_000_000_000 {
            raw.checked_mul(1000)?
        } else {
            raw
        };
        u64::try_from(v).ok()
    }

    async fn handle_snapshot(
        &self,
        value: &serde_json::Value,
        local_ts: Instant,
        local_ts_ms: u64,
    ) {
        let payload: SnapshotPayload = match value.get("msg") {
            Some(msg) => match serde_json::from_value(msg.clone()) {
                Ok(parsed) => parsed,
                Err(_) => return,
            },
            None => return,
        };

        let mut bids = Self::parse_levels(payload.yes_dollars_fp);
        let mut asks: Vec<PriceLevel> = Self::parse_levels(payload.no_dollars_fp)
            .into_iter()
            .map(|level| PriceLevel::with_fixed(level.price.complement(), level.size))
            .collect();

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        let orderbook = Orderbook {
            asset_id: payload.market_ticker.clone(),
            bids,
            asks,
            last_update_id: value.get("seq").and_then(|v| v.as_u64()),
            timestamp: Some(chrono::Utc::now()),
            hash: None,
        };

        {
            let mut obs = self.orderbooks.write().await;
            obs.insert(payload.market_ticker.clone(), orderbook.clone());
        }

        let exchange_time = orderbook.timestamp;

        let seq = self
            .dispatcher_seq(&payload.market_ticker)
            .await
            .fetch_add(1, Ordering::Relaxed);
        // Kalshi markets are binary, keyed by a single ticker; market_id and
        // asset_id are the same string.
        self.dispatch(WsUpdate::Snapshot {
            market_id: payload.market_ticker.clone(),
            asset_id: payload.market_ticker,
            book: Arc::new(orderbook),
            exchange_ts: exchange_time.map(|t| t.timestamp_millis() as u64),
            local_ts,
            local_ts_ms,
            seq,
        })
        .await;
    }

    async fn handle_delta(&self, value: &serde_json::Value, local_ts: Instant, local_ts_ms: u64) {
        let payload: DeltaPayload = match value.get("msg") {
            Some(msg) => match serde_json::from_value(msg.clone()) {
                Ok(parsed) => parsed,
                Err(_) => return,
            },
            None => return,
        };

        let price: f64 = match payload.price_dollars.parse() {
            Ok(v) => v,
            Err(_) => return,
        };
        let delta: f64 = match payload.delta_fp.parse() {
            Ok(v) => v,
            Err(_) => return,
        };
        let fp = FixedPrice::from_f64(price);
        let is_yes = payload.side.eq_ignore_ascii_case("yes");

        let mut obs = self.orderbooks.write().await;
        if let Some(ob) = obs.get_mut(&payload.market_ticker) {
            let (plc_side, plc_fp) = if is_yes {
                (PriceLevelSide::Bid, fp)
            } else {
                (PriceLevelSide::Ask, fp.complement())
            };

            let abs_size = if is_yes {
                Self::update_levels(&mut ob.bids, fp, delta, true)
            } else {
                Self::update_levels(&mut ob.asks, fp.complement(), delta, false)
            };

            let change = PriceLevelChange {
                side: plc_side,
                price: plc_fp,
                size: abs_size,
            };

            ob.last_update_id = value.get("seq").and_then(|v| v.as_u64());
            let ts = payload
                .ts
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            ob.timestamp = ts.or(ob.timestamp);
            let exchange_ts = ob
                .timestamp
                .and_then(|t| u64::try_from(t.timestamp_millis()).ok());
            drop(obs);

            let mut changes = ChangeVec::new();
            changes.push(change);

            let dispatch_seq = self
                .dispatcher_seq(&payload.market_ticker)
                .await
                .fetch_add(1, Ordering::Relaxed);
            self.dispatch(WsUpdate::Delta {
                market_id: payload.market_ticker.clone(),
                asset_id: payload.market_ticker,
                changes,
                exchange_ts,
                local_ts,
                local_ts_ms,
                seq: dispatch_seq,
            })
            .await;
        }
    }

    async fn handle_trade(&self, value: &serde_json::Value, local_ts: Instant, local_ts_ms: u64) {
        let msg = value.get("msg").unwrap_or(value);

        let market_id = msg
            .get("market_id")
            .or_else(|| msg.get("ticker"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let Some(market_id) = market_id else {
            return;
        };

        let price = Self::value_to_f64(msg.get("yes_price_dollars"));
        let size = Self::value_to_f64(msg.get("count_fp"));
        let Some(price) = price else {
            return;
        };
        let Some(size) = size else {
            return;
        };

        let aggressor_side = msg
            .get("taker_side")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let trade_id = Self::value_to_string(msg.get("trade_id"));
        let exchange_ts_ms = Self::value_to_ts_ms(msg.get("ts"));

        let trade = ActivityTrade {
            market_id: market_id.clone(),
            asset_id: market_id,
            trade_id,
            price,
            size,
            side: None,
            aggressor_side,
            outcome: None,
            fee_rate_bps: None,
            exchange_ts_ms,
            source_channel: Cow::Borrowed("kalshi_public_trade"),
        };

        self.dispatch(WsUpdate::Trade {
            trade,
            local_ts,
            local_ts_ms,
        })
        .await;
    }

    async fn handle_fill(&self, value: &serde_json::Value, local_ts: Instant, local_ts_ms: u64) {
        let msg = value.get("msg").unwrap_or(value);
        let market_id = msg
            .get("market_id")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let Some(market_id) = market_id else {
            return;
        };

        let price = Self::value_to_f64(msg.get("yes_price_dollars")).unwrap_or(0.0);
        let size = Self::value_to_f64(msg.get("count_fp")).unwrap_or(0.0);
        if price <= 0.0 || size <= 0.0 {
            return;
        }

        let side = msg
            .get("action")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let outcome = msg.get("side").and_then(|v| v.as_str()).map(str::to_string);
        let fill_id = Self::value_to_string(msg.get("trade_id"));
        let order_id = Self::value_to_string(msg.get("order_id"));
        let exchange_ts_ms = Self::value_to_ts_ms(msg.get("ts"));

        let liquidity_role = msg
            .get("is_taker")
            .and_then(|v| v.as_bool())
            .map(|is_taker| {
                if is_taker {
                    LiquidityRole::Taker
                } else {
                    LiquidityRole::Maker
                }
            });

        let fill = ActivityFill {
            market_id: market_id.clone(),
            asset_id: market_id,
            fill_id,
            order_id,
            price,
            size,
            side,
            outcome,
            tx_hash: None,
            fee: None,
            exchange_ts_ms,
            source_channel: Cow::Borrowed("kalshi_user_fill"),
            liquidity_role,
        };

        self.dispatch(WsUpdate::Fill {
            fill,
            local_ts,
            local_ts_ms,
        })
        .await;
    }

    async fn handle_subscribed(&self, value: &serde_json::Value) {
        let id = value.get("id").and_then(|v| v.as_u64());
        let sid = value
            .get("msg")
            .and_then(|msg| msg.get("sid"))
            .and_then(|v| v.as_u64());

        if let (Some(id), Some(sid)) = (id, sid) {
            let market = {
                let mut pending = self.pending.write().await;
                pending.remove(&id)
            };

            if let Some(market) = market {
                let mut subs = self.subscriptions.write().await;
                subs.insert(market, Some(sid));
            }
        }
    }

    async fn handle_unsubscribed(&self, value: &serde_json::Value) {
        let sid = value.get("sid").and_then(|v| v.as_u64());
        if let Some(sid) = sid {
            let mut subs = self.subscriptions.write().await;
            let remove_key = subs.iter().find_map(|(k, v)| {
                if v == &Some(sid) {
                    Some(k.clone())
                } else {
                    None
                }
            });
            if let Some(key) = remove_key {
                subs.remove(&key);
            }
        }
    }

    async fn handle_error(&self, value: &serde_json::Value) {
        let id = value.get("id").and_then(|v| v.as_u64());
        let payload: ErrorPayload = match value.get("msg") {
            Some(msg) => match serde_json::from_value(msg.clone()) {
                Ok(parsed) => parsed,
                Err(_) => return,
            },
            None => return,
        };

        if let Some(id) = id {
            let market = {
                let mut pending = self.pending.write().await;
                pending.remove(&id)
            };
            if let Some(market_id) = market {
                let err = WebSocketError::Subscription(format!(
                    "kalshi ws error {}: {}",
                    payload.code, payload.msg
                ));
                self.dispatcher.send_session(SessionEvent::error(err)).await;
                self.dispatcher
                    .send_session(SessionEvent::BookInvalidated {
                        market_id: market_id.clone(),
                        reason: InvalidationReason::ExchangeReset,
                    })
                    .await;
                let (local_ts, local_ts_ms) = now_pair();
                let seq = self
                    .dispatcher_seq(&market_id)
                    .await
                    .fetch_add(1, Ordering::Relaxed);
                let _ = self.dispatcher.try_send_update(WsUpdate::Clear {
                    market_id: market_id.clone(),
                    asset_id: market_id,
                    reason: InvalidationReason::ExchangeReset,
                    local_ts,
                    local_ts_ms,
                    seq,
                });
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
            let _ = self.send_subscribe(&market).await?;
        }

        Ok(())
    }

    async fn send_subscribe(&self, market_id: &str) -> Result<u64, WebSocketError> {
        let id = self.next_command_id();
        let mut channels = vec!["orderbook_delta", "trade"];
        if self.enable_user_fills {
            channels.push("fill");
        }
        let payload = serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": channels,
                "market_id": market_id
            }
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

    async fn send_unsubscribe(&self, sid: u64) -> Result<(), WebSocketError> {
        let id = self.next_command_id();
        let payload = serde_json::json!({
            "id": id,
            "cmd": "unsubscribe",
            "params": { "sids": [sid] }
        });
        let json =
            serde_json::to_string(&payload).map_err(|e| WebSocketError::Protocol(e.to_string()))?;
        self.send_message(&json).await?;
        Ok(())
    }

    fn calculate_reconnect_delay(attempt: u32) -> Duration {
        let delay = WS_RECONNECT_BASE_DELAY.as_millis() as f64 * 1.5_f64.powi(attempt as i32);
        let delay = delay.min(WS_RECONNECT_MAX_DELAY.as_millis() as f64) as u64;
        Duration::from_millis(delay)
    }
}

impl OrderBookWebSocket for KalshiWebSocket {
    async fn connect(&mut self) -> Result<(), WebSocketError> {
        if let Some(ref api_key_id) = self.config.api_key_id {
            self.api_key_id = Some(api_key_id.clone());
        }

        if self.api_key_id.is_none() || self.auth.is_none() {
            return Err(WebSocketError::Connection("authentication required".into()));
        }

        self.set_state(WebSocketState::Connecting);

        let request = self.build_request()?;
        let (ws_stream, _) = connect_async(request)
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
        let orderbooks = self.orderbooks.clone();
        let subscriptions = self.subscriptions.clone();
        let pending = self.pending.clone();
        let write_tx_clone = self.write_tx.clone();
        let reconnect_attempts_clone = self.reconnect_attempts.clone();
        let auto_reconnect = self.auto_reconnect;
        let config = self.config.clone();
        let auth = self.auth.clone();
        let command_id = self.command_id.clone();

        let dispatcher = self.dispatcher.clone();
        let seqs = self.seqs.clone();
        let last_message_at = self.last_message_at.clone();

        let ws_self = KalshiWebSocket {
            config,
            auth,
            api_key_id: self.api_key_id.clone(),
            state: state.clone(),
            subscriptions: subscriptions.clone(),
            pending: pending.clone(),
            orderbooks: orderbooks.clone(),
            dispatcher: dispatcher.clone(),
            seqs: seqs.clone(),
            last_message_at: last_message_at.clone(),
            write_tx: write_tx_clone.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect,
            reconnect_attempts: reconnect_attempts_clone.clone(),
            command_id,
            enable_user_fills: self.enable_user_fills,
        };

        tokio::spawn(async move {
            let write_future = rx.map(Ok).forward(write);
            let read_future = async {
                let mut read = read;
                while let Some(msg) = read.next().await {
                    // Monotonic + wall-clock pair captured the moment the socket
                    // returns, before parse — this is what makes local_ts
                    // honest under NTP adjustments and what FFI consumers serialize.
                    let (local_ts, local_ts_ms) = now_pair();
                    match msg {
                        Ok(Message::Text(text)) => {
                            *ws_self.last_message_at.write().await = Some(chrono::Utc::now());
                            ws_self.handle_message(&text, local_ts, local_ts_ms).await;
                        }
                        Ok(Message::Ping(data)) => {
                            if let Some(ref tx) = *ws_self.write_tx.lock().await {
                                let _ = tx.unbounded_send(Message::Pong(data));
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            ws_self
                                .dispatcher
                                .send_session(SessionEvent::error(WebSocketError::Connection(
                                    e.to_string(),
                                )))
                                .await;
                            break;
                        }
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

            let stall_future = stall_watchdog(last_message_at.clone());

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = ping_future => {},
                _ = stall_future => {},
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
                    exchange = "kalshi",
                    attempt,
                    max = WS_MAX_RECONNECT_ATTEMPTS,
                    "websocket connection lost, starting reconnect"
                );

                while attempt <= WS_MAX_RECONNECT_ATTEMPTS {
                    state.store(WebSocketState::Reconnecting);

                    let delay = KalshiWebSocket::calculate_reconnect_delay(attempt);
                    tracing::info!(
                        exchange = "kalshi",
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        "reconnect attempt starting"
                    );
                    tokio::time::sleep(delay).await;

                    let request = match ws_self.build_request() {
                        Ok(req) => req,
                        Err(_) => break,
                    };

                    match connect_async(request).await {
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

                            // ONE Reconnected event globally (with wall-clock gap)
                            // plus one BookInvalidated per subscribed market so
                            // callers can reset caches.
                            {
                                let now = chrono::Utc::now();
                                let gap = ws_self
                                    .last_message_at
                                    .read()
                                    .await
                                    .and_then(|t| (now - t).to_std().ok())
                                    .unwrap_or_else(|| Duration::from_secs(0));
                                ws_self
                                    .dispatcher
                                    .send_session(SessionEvent::reconnected(gap))
                                    .await;
                                let subs = ws_self.subscriptions.read().await;
                                for market_id in subs.keys() {
                                    ws_self
                                        .dispatcher
                                        .send_session(SessionEvent::BookInvalidated {
                                            market_id: market_id.clone(),
                                            reason: InvalidationReason::Reconnect,
                                        })
                                        .await;
                                    let (ts_mono, ts_ms) = now_pair();
                                    let seq = ws_self
                                        .dispatcher_seq(market_id)
                                        .await
                                        .fetch_add(1, Ordering::Relaxed);
                                    let _ = ws_self.dispatcher.try_send_update(WsUpdate::Clear {
                                        market_id: market_id.clone(),
                                        asset_id: market_id.clone(),
                                        reason: InvalidationReason::Reconnect,
                                        local_ts: ts_mono,
                                        local_ts_ms: ts_ms,
                                        seq,
                                    });
                                }
                            }

                            match ws_self.resubscribe_all().await {
                                Ok(()) => {
                                    let market_count = ws_self.subscriptions.read().await.len();
                                    tracing::info!(
                                        exchange = "kalshi",
                                        markets = market_count,
                                        "reconnected and resubscribed to all markets"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(exchange = "kalshi", error = %e, "resubscription failed after reconnect");
                                }
                            }

                            let write_future = new_rx.map(Ok).forward(new_write);
                            let read_future = async {
                                let mut read = new_read;
                                while let Some(msg) = read.next().await {
                                    let (local_ts, local_ts_ms) = now_pair();
                                    match msg {
                                        Ok(Message::Text(text)) => {
                                            *ws_self.last_message_at.write().await =
                                                Some(chrono::Utc::now());
                                            ws_self
                                                .handle_message(&text, local_ts, local_ts_ms)
                                                .await;
                                        }
                                        Ok(Message::Ping(data)) => {
                                            if let Some(ref tx) = *ws_self.write_tx.lock().await {
                                                let _ = tx.unbounded_send(Message::Pong(data));
                                            }
                                        }
                                        Ok(Message::Close(_)) => break,
                                        Err(e) => {
                                            ws_self
                                                .dispatcher
                                                .send_session(SessionEvent::error(
                                                    WebSocketError::Connection(e.to_string()),
                                                ))
                                                .await;
                                            break;
                                        }
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

                            let stall_future = stall_watchdog(last_message_at.clone());

                            tokio::select! {
                                _ = write_future => {},
                                _ = read_future => {},
                                _ = ping_future => {},
                                _ = stall_future => {},
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
                            tracing::warn!(exchange = "kalshi", attempt, error = %e, "reconnect attempt failed");
                            attempt = {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a += 1;
                                *a
                            };
                        }
                    }
                }

                tracing::error!(
                    exchange = "kalshi",
                    max = WS_MAX_RECONNECT_ATTEMPTS,
                    "max reconnect attempts exhausted, giving up"
                );
                state.store(WebSocketState::Disconnected);
            }
        });

        self.set_state(WebSocketState::Connected);
        self.reset_reconnect_attempts().await;
        *self.last_message_at.write().await = Some(chrono::Utc::now());
        self.dispatcher.send_session(SessionEvent::Connected).await;
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
        let market_id = market_id.to_string();

        {
            let mut subs = self.subscriptions.write().await;
            subs.entry(market_id.clone()).or_insert(None);
        }

        if self.state.load() == WebSocketState::Connected {
            let _ = self.send_subscribe(&market_id).await?;
        }

        Ok(())
    }

    async fn unsubscribe(&mut self, market_id: &str) -> Result<(), WebSocketError> {
        let market_id = market_id.to_string();
        let sid = {
            let mut subs = self.subscriptions.write().await;
            subs.remove(&market_id).and_then(|v| v)
        };

        if let Some(sid) = sid {
            let _ = self.send_unsubscribe(sid).await;
        }

        {
            let mut obs = self.orderbooks.write().await;
            obs.remove(&market_id);
        }

        Ok(())
    }

    fn state(&self) -> WebSocketState {
        self.state.load()
    }

    fn updates(&self) -> Option<UpdateStream> {
        self.dispatcher.take_updates()
    }

    fn session_events(&self) -> Option<SessionStream> {
        self.dispatcher.take_session_events()
    }
}

#[cfg(test)]
mod tests {
    use super::KalshiWebSocket;
    use px_core::LiquidityRole;
    use serde_json::json;

    #[test]
    fn value_to_ts_ms_parses_seconds_epoch() {
        let ts = KalshiWebSocket::value_to_ts_ms(Some(&json!(1_770_241_151_i64)))
            .expect("timestamp should parse");
        assert_eq!(ts, 1_770_241_151_000_u64);
    }

    #[test]
    fn value_to_ts_ms_parses_rfc3339_string() {
        let ts = KalshiWebSocket::value_to_ts_ms(Some(&json!("2026-02-05T18:19:11Z")))
            .expect("timestamp should parse");
        assert_eq!(ts, 1_770_315_551_000_u64);
    }

    #[test]
    fn value_to_ts_ms_passes_millis_through() {
        let ts = KalshiWebSocket::value_to_ts_ms(Some(&json!(1_770_241_151_000_i64)))
            .expect("timestamp should parse");
        assert_eq!(ts, 1_770_241_151_000_u64);
    }

    /// Mirrors the liquidity_role extraction in handle_fill().
    fn extract_liquidity_role(msg: &serde_json::Value) -> Option<LiquidityRole> {
        msg.get("is_taker")
            .and_then(|v| v.as_bool())
            .map(|is_taker| {
                if is_taker {
                    LiquidityRole::Taker
                } else {
                    LiquidityRole::Maker
                }
            })
    }

    #[test]
    fn fill_is_taker_true_yields_taker() {
        let msg = json!({ "is_taker": true });
        assert_eq!(extract_liquidity_role(&msg), Some(LiquidityRole::Taker));
    }

    #[test]
    fn fill_is_taker_false_yields_maker() {
        let msg = json!({ "is_taker": false });
        assert_eq!(extract_liquidity_role(&msg), Some(LiquidityRole::Maker));
    }

    #[test]
    fn fill_is_taker_absent_yields_none() {
        let msg = json!({ "market_id": "SOME-TICKER" });
        assert_eq!(extract_liquidity_role(&msg), None);
    }
}
