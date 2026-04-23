use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use std::sync::atomic::{AtomicU64, Ordering};

use px_core::{
    now_pair, sort_asks, sort_bids, stall_watchdog, ActivityFill, ActivityTrade,
    AtomicWebSocketState, ChangeVec, FixedPrice, InvalidationReason, LiquidityRole, Orderbook,
    OrderBookWebSocket, PriceLevel, PriceLevelChange, PriceLevelSide, SessionEvent, SessionStream,
    UpdateStream, WebSocketError, WebSocketState, WsDispatcher, WsDispatcherConfig, WsUpdate,
    WS_MAX_RECONNECT_ATTEMPTS, WS_RECONNECT_BASE_DELAY, WS_RECONNECT_MAX_DELAY,
};

use crate::config::OpinionConfig;

/// Opinion-specific heartbeat interval (30s per their WS protocol).
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Per-market monotonic sequence counter map.
type SeqMap = Arc<RwLock<HashMap<String, Arc<AtomicU64>>>>;

pub struct OpinionWebSocket {
    config: OpinionConfig,
    state: Arc<AtomicWebSocketState>,
    subscriptions: Arc<RwLock<HashSet<String>>>,
    /// Multiplexed dispatch handle.
    dispatcher: Arc<WsDispatcher>,
    /// Per-market monotonic sequence counters.
    seqs: SeqMap,
    /// Wall-clock of the last successfully received WS message.
    last_message_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    write_tx: Arc<Mutex<Option<futures::channel::mpsc::UnboundedSender<Message>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    auto_reconnect: bool,
    reconnect_attempts: Arc<Mutex<u32>>,
    /// REST client for snapshot-on-subscribe. Opinion's WS protocol delivers
    /// `market.depth.diff` only — no server-pushed initial snapshot — so we
    /// fetch the book once per subscribe to seed consumer caches.
    http: reqwest::Client,
}

impl OpinionWebSocket {
    pub fn new(config: OpinionConfig) -> Result<Self, WebSocketError> {
        if config.api_key.is_none() {
            return Err(WebSocketError::Connection(
                "api_key required for Opinion WebSocket".into(),
            ));
        }
        let http = px_core::http::tuned_client_builder()
            .user_agent("openpx/1.0")
            .timeout(config.base.timeout)
            .build()
            .map_err(|e| WebSocketError::Connection(format!("http client build: {e}")))?;
        Ok(Self {
            config,
            state: Arc::new(AtomicWebSocketState::new(WebSocketState::Disconnected)),
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
            dispatcher: Arc::new(WsDispatcher::new(WsDispatcherConfig::default())),
            seqs: Arc::new(RwLock::new(HashMap::new())),
            last_message_at: Arc::new(RwLock::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect: true,
            reconnect_attempts: Arc::new(Mutex::new(0)),
            http,
        })
    }

    /// Allocate-or-fetch the per-market sequence counter for the 0.2 dispatch
    /// path. Lazy on first emit.
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
    /// `SessionEvent::Lagged` + `BookInvalidated(Lag)` + best-effort
    /// `WsUpdate::Clear`.
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

    fn ws_url(&self) -> Result<String, WebSocketError> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| WebSocketError::Connection("api_key required".into()))?;
        let base = self.config.ws_url.trim_end_matches('/');
        Ok(format!("{base}/?apikey={api_key}"))
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

        let channels = [
            "market.depth.diff",
            "market.last.trade",
            "market.last.price",
            "trade.order.update",
            "trade.record.new",
        ];
        for channel in channels {
            let msg = serde_json::json!({
                "action": "SUBSCRIBE",
                "channel": channel,
                "marketId": numeric_id
            });
            self.send_message(&msg.to_string()).await?;
        }
        Ok(())
    }

    async fn send_unsubscribe(&self, market_id: &str) -> Result<(), WebSocketError> {
        let numeric_id: i64 = market_id
            .parse()
            .map_err(|_| WebSocketError::Subscription(format!("invalid market_id: {market_id}")))?;

        let channels = [
            "market.depth.diff",
            "market.last.trade",
            "market.last.price",
            "trade.order.update",
            "trade.record.new",
        ];
        for channel in channels {
            let msg = serde_json::json!({
                "action": "UNSUBSCRIBE",
                "channel": channel,
                "marketId": numeric_id
            });
            self.send_message(&msg.to_string()).await?;
        }
        Ok(())
    }

    /// Back-compat entry point for tests; production read loops call
    /// `handle_message_at` with a monotonic receive timestamp.
    #[cfg(test)]
    pub(crate) async fn handle_message(&self, text: &str) {
        let (local_ts, local_ts_ms) = now_pair();
        self.handle_message_at(text, local_ts, local_ts_ms).await;
    }

    async fn handle_message_at(&self, text: &str, local_ts: Instant, local_ts_ms: u64) {
        // Shared SIMD-accelerated parse with size-based switching.
        let Some(value) = px_core::decode_value(text) else {
            return;
        };

        let msg_type = value.get("msgType").and_then(|v| v.as_str()).unwrap_or("");

        match msg_type {
            "market.depth.diff" => self.handle_depth_diff(&value, local_ts, local_ts_ms).await,
            "market.last.trade" | "market.last.price" => {
                self.handle_last_trade(&value, local_ts, local_ts_ms).await
            }
            "trade.order.update" => {
                self.handle_order_update(&value, local_ts, local_ts_ms)
                    .await
            }
            "trade.record.new" => {
                self.handle_trade_executed(&value, local_ts, local_ts_ms)
                    .await
            }
            _ => {}
        }
    }

    async fn handle_depth_diff(
        &self,
        value: &serde_json::Value,
        local_ts: Instant,
        local_ts_ms: u64,
    ) {
        let market_id = match value.get("marketId").and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(String::from))
        }) {
            Some(id) => id,
            None => return,
        };

        let outcome_side = value
            .get("outcomeSide")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let side_str = value.get("side").and_then(|v| v.as_str()).unwrap_or("");

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

        let dispatch_seq = self
            .dispatcher_seq(&market_id)
            .await
            .fetch_add(1, Ordering::Relaxed);
        // Opinion's market.depth.diff is market-level (yes-space book);
        // asset_id mirrors market_id.
        self.dispatch(WsUpdate::Delta {
            market_id: market_id.clone(),
            asset_id: market_id,
            changes,
            exchange_ts: timestamp.map(|t| t.timestamp_millis() as u64),
            local_ts,
            local_ts_ms,
            seq: dispatch_seq,
        })
        .await;
    }

    async fn handle_last_trade(
        &self,
        value: &serde_json::Value,
        local_ts: Instant,
        local_ts_ms: u64,
    ) {
        let market_id = match value.get("marketId").and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(String::from))
        }) {
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

        let exchange_ts_ms = value
            .get("timestamp")
            .or_else(|| value.get("ts"))
            .and_then(|v| v.as_i64())
            .and_then(|ts| u64::try_from(ts).ok());

        let trade = ActivityTrade {
            market_id: market_id.clone(),
            asset_id: token_id,
            trade_id: None,
            price,
            size,
            side,
            aggressor_side: None,
            outcome: None,
            fee_rate_bps: None,
            exchange_ts_ms,
            source_channel: Cow::Borrowed("market.last.trade"),
        };

        self.dispatch(WsUpdate::Trade {
            trade,
            local_ts,
            local_ts_ms,
        })
        .await;
    }

    /// Handle `trade.order.update` — user order fill notifications.
    /// Only emits for `orderFill` updates; other types (orderNew, orderCancel, orderConfirm)
    /// are informational and not mapped to a `WsUpdate`.
    async fn handle_order_update(
        &self,
        value: &serde_json::Value,
        local_ts: Instant,
        local_ts_ms: u64,
    ) {
        let update_type = value
            .get("orderUpdateType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if update_type != "orderFill" {
            return;
        }

        let market_id = match value.get("marketId").and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(String::from))
        }) {
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
            .get("filledShares")
            .or_else(|| value.get("shares"))
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let side = match value.get("side").and_then(|v| v.as_i64()) {
            Some(1) => Some("buy".to_string()),
            Some(2) => Some("sell".to_string()),
            _ => None,
        };

        let outcome = match value.get("outcomeSide").and_then(|v| v.as_i64()) {
            Some(1) => Some("Yes".to_string()),
            Some(2) => Some("No".to_string()),
            _ => None,
        };

        let order_id = value
            .get("orderId")
            .and_then(|v| v.as_str())
            .map(String::from);

        // createdAt may be seconds or milliseconds; if > 1e12, treat as millis
        let exchange_ts_ms = value
            .get("createdAt")
            .and_then(|v| v.as_i64())
            .and_then(|ts| {
                if ts > 1_000_000_000_000 {
                    u64::try_from(ts).ok()
                } else {
                    u64::try_from(ts).ok().and_then(|s| s.checked_mul(1000))
                }
            });

        let fill = ActivityFill {
            market_id: market_id.clone(),
            asset_id: String::new(),
            fill_id: None,
            order_id,
            price,
            size,
            side,
            outcome,
            tx_hash: None,
            fee: None,
            exchange_ts_ms,
            source_channel: Cow::Borrowed("trade.order.update"),
            liquidity_role: None,
        };

        self.dispatch(WsUpdate::Fill {
            fill,
            local_ts,
            local_ts_ms,
        })
        .await;
    }

    /// Handle `trade.record.new` — confirmed on-chain trade execution.
    /// Each message is a single fill with final on-chain amounts and fee.
    async fn handle_trade_executed(
        &self,
        value: &serde_json::Value,
        local_ts: Instant,
        local_ts_ms: u64,
    ) {
        let market_id = match value.get("marketId").and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(String::from))
        }) {
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
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let side_str = value
            .get("side")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase());

        // Map Opinion's side field: Buy/Sell are trading, Split/Merge are position operations
        let liquidity_role = match side_str.as_deref() {
            Some("buy" | "sell") => Some(LiquidityRole::Taker),
            _ => None,
        };

        let outcome = match value.get("outcomeSide").and_then(|v| v.as_i64()) {
            Some(1) => Some("Yes".to_string()),
            Some(2) => Some("No".to_string()),
            _ => None,
        };

        let order_id = value
            .get("orderId")
            .and_then(|v| v.as_str())
            .map(String::from);

        let fill_id = value
            .get("tradeNo")
            .and_then(|v| v.as_str())
            .map(String::from);

        let tx_hash = value
            .get("txHash")
            .and_then(|v| v.as_str())
            .map(String::from);

        let fee = value.get("fee").and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        });

        // createdAt may be seconds or milliseconds; if > 1e12, treat as millis
        let exchange_ts_ms = value
            .get("createdAt")
            .and_then(|v| v.as_i64())
            .and_then(|ts| {
                if ts > 1_000_000_000_000 {
                    u64::try_from(ts).ok()
                } else {
                    u64::try_from(ts).ok().and_then(|s| s.checked_mul(1000))
                }
            });

        let fill = ActivityFill {
            market_id: market_id.clone(),
            asset_id: String::new(),
            fill_id,
            order_id,
            price,
            size,
            side: side_str,
            outcome,
            tx_hash,
            fee,
            exchange_ts_ms,
            source_channel: Cow::Borrowed("trade.record.new"),
            liquidity_role,
        };

        self.dispatch(WsUpdate::Fill {
            fill,
            local_ts,
            local_ts_ms,
        })
        .await;
    }

    async fn resubscribe_all(&self) -> Result<(), WebSocketError> {
        let markets: Vec<String> = {
            let subs = self.subscriptions.read().await;
            subs.iter().cloned().collect()
        };

        for market in markets {
            self.send_subscribe(&market).await?;
            self.spawn_snapshot_fetch(market);
        }

        Ok(())
    }

    /// Fetch `(yes_token_id, no_token_id)` from `/openapi/market/{market_id}`.
    /// Returns `None` for non-binary markets, auth failures, or any parse
    /// error — snapshot fetch is best-effort; diffs still work.
    async fn fetch_market_tokens(&self, market_id: &str) -> Option<(String, String)> {
        let base = self.config.api_url.trim_end_matches('/');
        let url = format!("{base}/openapi/market/{market_id}");
        let mut req = self.http.get(&url);
        if let Some(ref key) = self.config.api_key {
            req = req.header("apikey", key);
        }
        let resp = req.send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let v: serde_json::Value = resp.json().await.ok()?;
        let code = v
            .get("code")
            .or_else(|| v.get("errno"))
            .and_then(|c| c.as_i64())
            .unwrap_or(-1);
        if code != 0 {
            return None;
        }
        let data = v.get("result").and_then(|r| r.get("data"))?;
        let yes = data
            .get("yesTokenId")
            .or_else(|| data.get("yes_token_id"))
            .and_then(|t| t.as_str())
            .filter(|s| !s.is_empty())?
            .to_string();
        let no = data
            .get("noTokenId")
            .or_else(|| data.get("no_token_id"))
            .and_then(|t| t.as_str())
            .filter(|s| !s.is_empty())?
            .to_string();
        Some((yes, no))
    }

    /// Fetch raw (bids, asks) for a single outcome token. Empty vectors on any
    /// parse failure — caller merges what is available.
    async fn fetch_token_book(&self, token_id: &str) -> (Vec<PriceLevel>, Vec<PriceLevel>) {
        let base = self.config.api_url.trim_end_matches('/');
        let url = format!("{base}/openapi/token/orderbook?token_id={token_id}");
        let mut req = self.http.get(&url);
        if let Some(ref key) = self.config.api_key {
            req = req.header("apikey", key);
        }
        let Ok(resp) = req.send().await else {
            return (Vec::new(), Vec::new());
        };
        if !resp.status().is_success() {
            return (Vec::new(), Vec::new());
        }
        let Ok(v) = resp.json::<serde_json::Value>().await else {
            return (Vec::new(), Vec::new());
        };
        let code = v
            .get("code")
            .or_else(|| v.get("errno"))
            .and_then(|c| c.as_i64())
            .unwrap_or(-1);
        if code != 0 {
            return (Vec::new(), Vec::new());
        }
        let Some(data) = v.get("result").and_then(|r| r.get("data")) else {
            return (Vec::new(), Vec::new());
        };
        let parse = |key: &str| -> Vec<PriceLevel> {
            data.get(key)
                .and_then(|arr| arr.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let price = item.get("price").and_then(|p| {
                                p.as_f64()
                                    .or_else(|| p.as_str().and_then(|s| s.parse().ok()))
                            })?;
                            let size = item.get("size").and_then(|s| {
                                s.as_f64()
                                    .or_else(|| s.as_str().and_then(|s| s.parse().ok()))
                            })?;
                            if price > 0.0 && size > 0.0 {
                                Some(PriceLevel::new(price, size))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        (parse("bids"), parse("asks"))
    }

    /// Fetch both yes/no books and dispatch a single `WsUpdate::Snapshot` in
    /// the same yes-space layout that `handle_depth_diff` produces, so
    /// consumers can apply subsequent deltas on top.
    async fn fetch_and_dispatch_snapshot(&self, market_id: String) {
        let Some((yes_token, no_token)) = self.fetch_market_tokens(&market_id).await else {
            tracing::debug!(
                exchange = "opinion",
                market_id = %market_id,
                "snapshot skipped: token ids unavailable"
            );
            return;
        };

        let (yes_book, no_book) = tokio::join!(
            self.fetch_token_book(&yes_token),
            self.fetch_token_book(&no_token),
        );

        // Match handle_depth_diff mapping:
        //   yes "bids" → Bid (as-is), yes "asks" → Ask (as-is)
        //   no  "bids" → Ask at 1-p,  no  "asks" → Bid at 1-p
        let mut bids: Vec<PriceLevel> = yes_book.0;
        bids.extend(
            no_book
                .1
                .into_iter()
                .map(|l| PriceLevel::with_fixed(l.price.complement(), l.size)),
        );
        let mut asks: Vec<PriceLevel> = yes_book.1;
        asks.extend(
            no_book
                .0
                .into_iter()
                .map(|l| PriceLevel::with_fixed(l.price.complement(), l.size)),
        );

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        let book = Orderbook {
            market_id: market_id.clone(),
            asset_id: market_id.clone(),
            bids,
            asks,
            last_update_id: None,
            timestamp: Some(chrono::Utc::now()),
            hash: None,
        };

        let (local_ts, local_ts_ms) = now_pair();
        let seq = self
            .dispatcher_seq(&market_id)
            .await
            .fetch_add(1, Ordering::Relaxed);
        self.dispatch(WsUpdate::Snapshot {
            market_id: market_id.clone(),
            asset_id: market_id,
            book: Arc::new(book),
            exchange_ts: None,
            local_ts,
            local_ts_ms,
            seq,
        })
        .await;
    }

    /// Fire-and-forget snapshot fetch. Runs REST + dispatch off the subscribe
    /// path so callers don't block on HTTP latency.
    fn spawn_snapshot_fetch(&self, market_id: String) {
        let http = self.http.clone();
        let api_url = self.config.api_url.clone();
        let api_key = self.config.api_key.clone();
        let dispatcher = self.dispatcher.clone();
        let seqs = self.seqs.clone();
        // Rebuild a minimal handle for the spawned task. Cheap: all inner fields
        // are `Arc` or small `Clone`-ables.
        let handle = OpinionWebSocket {
            config: {
                let mut c = self.config.clone();
                c.api_url = api_url;
                c.api_key = api_key;
                c
            },
            state: self.state.clone(),
            subscriptions: self.subscriptions.clone(),
            dispatcher,
            seqs,
            last_message_at: self.last_message_at.clone(),
            write_tx: self.write_tx.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect: self.auto_reconnect,
            reconnect_attempts: self.reconnect_attempts.clone(),
            http,
        };
        tokio::spawn(async move {
            handle.fetch_and_dispatch_snapshot(market_id).await;
        });
    }

    fn calculate_reconnect_delay(attempt: u32) -> Duration {
        let delay = WS_RECONNECT_BASE_DELAY.as_millis() as f64 * 1.5_f64.powi(attempt as i32);
        let delay = delay.min(WS_RECONNECT_MAX_DELAY.as_millis() as f64) as u64;
        Duration::from_millis(delay)
    }
}

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
        let subscriptions = self.subscriptions.clone();
        let write_tx_clone = self.write_tx.clone();
        let reconnect_attempts_clone = self.reconnect_attempts.clone();
        let auto_reconnect = self.auto_reconnect;
        let config = self.config.clone();

        let dispatcher = self.dispatcher.clone();
        let seqs = self.seqs.clone();
        let last_message_at = self.last_message_at.clone();
        let http = self.http.clone();

        let ws_handle = OpinionWebSocket {
            config: config.clone(),
            state: state.clone(),
            subscriptions: subscriptions.clone(),
            dispatcher: dispatcher.clone(),
            seqs: seqs.clone(),
            last_message_at: last_message_at.clone(),
            write_tx: write_tx_clone.clone(),
            shutdown_tx: Arc::new(Mutex::new(None)),
            auto_reconnect,
            reconnect_attempts: reconnect_attempts_clone.clone(),
            http: http.clone(),
        };

        tokio::spawn(async move {
            let write_future = rx.map(Ok).forward(write);
            let read_future = async {
                let mut read = read;
                while let Some(msg) = read.next().await {
                    let (local_ts, local_ts_ms) = now_pair();
                    match msg {
                        Ok(Message::Text(text)) => {
                            *ws_handle.last_message_at.write().await = Some(chrono::Utc::now());
                            ws_handle
                                .handle_message_at(&text, local_ts, local_ts_ms)
                                .await;
                        }
                        Ok(Message::Ping(data)) => {
                            if let Some(ref tx) = *ws_handle.write_tx.lock().await {
                                let _ = tx.unbounded_send(Message::Pong(data));
                            }
                        }
                        Ok(Message::Close(_)) => break,
                        Err(e) => {
                            ws_handle
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

            let stall_future = stall_watchdog(last_message_at.clone());

            tokio::select! {
                _ = write_future => {},
                _ = read_future => {},
                _ = heartbeat_future => {},
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
                    exchange = "opinion",
                    attempt,
                    max = WS_MAX_RECONNECT_ATTEMPTS,
                    "websocket connection lost, starting reconnect"
                );

                while attempt <= WS_MAX_RECONNECT_ATTEMPTS {
                    state.store(WebSocketState::Reconnecting);

                    let delay = OpinionWebSocket::calculate_reconnect_delay(attempt);
                    tracing::info!(
                        exchange = "opinion",
                        attempt,
                        delay_ms = delay.as_millis() as u64,
                        "reconnect attempt starting"
                    );
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

                            // ONE global Reconnected event with wall-clock gap,
                            // plus one BookInvalidated per subscribed market.
                            {
                                let now = chrono::Utc::now();
                                let gap = ws_handle
                                    .last_message_at
                                    .read()
                                    .await
                                    .and_then(|t| (now - t).to_std().ok())
                                    .unwrap_or_else(|| Duration::from_secs(0));
                                ws_handle
                                    .dispatcher
                                    .send_session(SessionEvent::reconnected(gap))
                                    .await;
                                let market_ids: Vec<String> = ws_handle
                                    .subscriptions
                                    .read()
                                    .await
                                    .iter()
                                    .cloned()
                                    .collect();
                                for market_id in market_ids {
                                    ws_handle
                                        .dispatcher
                                        .send_session(SessionEvent::BookInvalidated {
                                            market_id: market_id.clone(),
                                            reason: InvalidationReason::Reconnect,
                                        })
                                        .await;
                                    let (ts_mono, ts_ms) = now_pair();
                                    let seq = ws_handle
                                        .dispatcher_seq(&market_id)
                                        .await
                                        .fetch_add(1, Ordering::Relaxed);
                                    let _ = ws_handle.dispatcher.try_send_update(WsUpdate::Clear {
                                        market_id: market_id.clone(),
                                        asset_id: market_id,
                                        reason: InvalidationReason::Reconnect,
                                        local_ts: ts_mono,
                                        local_ts_ms: ts_ms,
                                        seq,
                                    });
                                }
                            }

                            match ws_handle.resubscribe_all().await {
                                Ok(()) => {
                                    let market_count = ws_handle.subscriptions.read().await.len();
                                    tracing::info!(
                                        exchange = "opinion",
                                        markets = market_count,
                                        "reconnected and resubscribed to all markets"
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(exchange = "opinion", error = %e, "resubscription failed after reconnect");
                                }
                            }

                            let write_future = new_rx.map(Ok).forward(new_write);
                            let read_future = async {
                                let mut read = new_read;
                                while let Some(msg) = read.next().await {
                                    let (local_ts, local_ts_ms) = now_pair();
                                    match msg {
                                        Ok(Message::Text(text)) => {
                                            *ws_handle.last_message_at.write().await =
                                                Some(chrono::Utc::now());
                                            ws_handle
                                                .handle_message_at(&text, local_ts, local_ts_ms)
                                                .await;
                                        }
                                        Ok(Message::Ping(data)) => {
                                            if let Some(ref tx) = *ws_handle.write_tx.lock().await {
                                                let _ = tx.unbounded_send(Message::Pong(data));
                                            }
                                        }
                                        Ok(Message::Close(_)) => break,
                                        Err(e) => {
                                            ws_handle
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

                            let stall_future = stall_watchdog(last_message_at.clone());

                            tokio::select! {
                                _ = write_future => {},
                                _ = read_future => {},
                                _ = heartbeat_future => {},
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
                            tracing::warn!(exchange = "opinion", attempt, error = %e, "reconnect attempt failed");
                            attempt = {
                                let mut a = reconnect_attempts_clone.lock().await;
                                *a += 1;
                                *a
                            };
                        }
                    }
                }

                tracing::error!(
                    exchange = "opinion",
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
        let market = market_id.to_string();
        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(market.clone());
        }
        if self.state.load() == WebSocketState::Connected {
            self.send_subscribe(&market).await?;
            self.spawn_snapshot_fetch(market);
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
    use super::*;
    use px_core::{FixedPrice, PriceLevelSide};
    use tokio::time::{timeout, Duration as TDuration};

    fn make_ws() -> OpinionWebSocket {
        let config = OpinionConfig::new().with_api_key("test_key");
        OpinionWebSocket::new(config).unwrap()
    }

    async fn next_update(stream: &UpdateStream) -> WsUpdate {
        timeout(TDuration::from_millis(300), stream.next())
            .await
            .expect("expected an update")
            .expect("stream closed")
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
        let updates = ws.updates().unwrap();
        let msg = serde_json::json!({
            "msgType": "market.depth.diff",
            "marketId": 123,
            "outcomeSide": 1,
            "side": "bids",
            "price": "0.65",
            "size": "100"
        });
        ws.handle_message(&msg.to_string()).await;

        match next_update(&updates).await {
            WsUpdate::Delta {
                changes, market_id, ..
            } => {
                assert_eq!(market_id, "123");
                assert_eq!(changes.len(), 1);
                assert_eq!(changes[0].side, PriceLevelSide::Bid);
                assert_eq!(changes[0].price, FixedPrice::from_f64(0.65));
                assert!((changes[0].size - 100.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Delta, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_depth_diff_no_bid_inverts() {
        let ws = make_ws();
        let updates = ws.updates().unwrap();
        // outcomeSide 2 (no), side "bids" -> Ask at 1.0 - price
        let msg = serde_json::json!({
            "msgType": "market.depth.diff",
            "marketId": 456,
            "outcomeSide": 2,
            "side": "bids",
            "price": 0.3,
            "size": 50
        });
        ws.handle_message(&msg.to_string()).await;

        match next_update(&updates).await {
            WsUpdate::Delta { changes, .. } => {
                assert_eq!(changes[0].side, PriceLevelSide::Ask);
                assert_eq!(changes[0].price, FixedPrice::from_f64(0.7));
                assert!((changes[0].size - 50.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Delta, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_last_trade_emits_trade() {
        let ws = make_ws();
        let updates = ws.updates().unwrap();
        let msg = serde_json::json!({
            "msgType": "market.last.trade",
            "marketId": 789,
            "price": "0.55",
            "shares": 25,
            "side": "buy",
            "tokenId": "token_yes_789"
        });
        ws.handle_message(&msg.to_string()).await;

        match next_update(&updates).await {
            WsUpdate::Trade { trade, .. } => {
                assert_eq!(trade.market_id, "789");
                assert_eq!(trade.asset_id, "token_yes_789");
                assert!((trade.price - 0.55).abs() < f64::EPSILON);
                assert!((trade.size - 25.0).abs() < f64::EPSILON);
                assert_eq!(trade.side.as_deref(), Some("buy"));
                assert_eq!(trade.source_channel, "market.last.trade");
            }
            other => panic!("expected Trade, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_order_update_fill() {
        let ws = make_ws();
        let updates = ws.updates().unwrap();
        let msg = serde_json::json!({
            "msgType": "trade.order.update",
            "orderUpdateType": "orderFill",
            "marketId": 111,
            "orderId": "order-abc",
            "side": 1,
            "outcomeSide": 1,
            "price": "0.65",
            "shares": "10",
            "filledShares": "5",
            "createdAt": 1700000000_i64
        });
        ws.handle_message(&msg.to_string()).await;

        match next_update(&updates).await {
            WsUpdate::Fill { fill, .. } => {
                assert_eq!(fill.market_id, "111");
                assert_eq!(fill.order_id.as_deref(), Some("order-abc"));
                assert!((fill.price - 0.65).abs() < f64::EPSILON);
                assert!((fill.size - 5.0).abs() < f64::EPSILON);
                assert_eq!(fill.side.as_deref(), Some("buy"));
                assert_eq!(fill.outcome.as_deref(), Some("Yes"));
                assert_eq!(fill.source_channel, "trade.order.update");
            }
            other => panic!("expected Fill, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn handle_order_update_ignores_non_fill() {
        let ws = make_ws();
        let updates = ws.updates().unwrap();
        let msg = serde_json::json!({
            "msgType": "trade.order.update",
            "orderUpdateType": "orderNew",
            "marketId": 222,
            "orderId": "order-xyz",
            "side": 1,
            "outcomeSide": 1,
            "price": "0.50",
            "shares": "10"
        });
        ws.handle_message(&msg.to_string()).await;

        let maybe = timeout(TDuration::from_millis(150), updates.next()).await;
        assert!(maybe.is_err(), "non-fill order updates should not emit");
    }

    #[tokio::test]
    async fn handle_trade_executed_emits_fill() {
        let ws = make_ws();
        let updates = ws.updates().unwrap();
        let msg = serde_json::json!({
            "msgType": "trade.record.new",
            "marketId": 333,
            "orderId": "order-def",
            "tradeNo": "trade-001",
            "side": "Buy",
            "outcomeSide": 2,
            "price": "0.30",
            "shares": "9.44",
            "fee": "0.01",
            "createdAt": 1700000100_i64
        });
        ws.handle_message(&msg.to_string()).await;

        match next_update(&updates).await {
            WsUpdate::Fill { fill, .. } => {
                assert_eq!(fill.market_id, "333");
                assert_eq!(fill.order_id.as_deref(), Some("order-def"));
                assert_eq!(fill.fill_id.as_deref(), Some("trade-001"));
                assert!((fill.price - 0.30).abs() < f64::EPSILON);
                assert!((fill.size - 9.44).abs() < f64::EPSILON);
                assert_eq!(fill.side.as_deref(), Some("buy"));
                assert_eq!(fill.outcome.as_deref(), Some("No"));
                assert_eq!(fill.source_channel, "trade.record.new");
                assert_eq!(fill.liquidity_role, Some(LiquidityRole::Taker));
            }
            other => panic!("expected Fill, got {other:?}"),
        }
    }

    #[test]
    fn reconnect_delay_increases_exponentially() {
        let d0 = OpinionWebSocket::calculate_reconnect_delay(0);
        let d1 = OpinionWebSocket::calculate_reconnect_delay(1);
        let d2 = OpinionWebSocket::calculate_reconnect_delay(2);
        assert!(d1 > d0);
        assert!(d2 > d1);
        let d_max = OpinionWebSocket::calculate_reconnect_delay(100);
        assert_eq!(d_max, WS_RECONNECT_MAX_DELAY);
    }
}
