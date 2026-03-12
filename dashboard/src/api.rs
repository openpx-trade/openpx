use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use px_core::error::OpenPxError;
use px_core::models::OrderSide;
use px_core::{OrderbookRequest, PriceHistoryInterval, PriceHistoryRequest, TradesRequest};
use px_sdk::ExchangeInner;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

pub struct ApiError(pub OpenPxError);

impl From<OpenPxError> for ApiError {
    fn from(err: OpenPxError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            OpenPxError::Exchange(e) => match e {
                px_core::error::ExchangeError::MarketNotFound(_) => StatusCode::NOT_FOUND,
                px_core::error::ExchangeError::Authentication(_) => StatusCode::UNAUTHORIZED,
                px_core::error::ExchangeError::InsufficientFunds(_) => StatusCode::BAD_REQUEST,
                px_core::error::ExchangeError::NotSupported(_) => StatusCode::NOT_IMPLEMENTED,
                _ => StatusCode::BAD_REQUEST,
            },
            OpenPxError::Config(_) | OpenPxError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            OpenPxError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.0.to_string() }))).into_response()
    }
}

fn not_found(msg: &str) -> ApiError {
    ApiError(OpenPxError::Config(msg.to_string()))
}

// ---------------------------------------------------------------------------
// Request / query types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct AddExchangeRequest {
    pub exchange: String,
    pub config: serde_json::Value,
}

#[derive(Deserialize)]
pub struct OrderbookParams {
    pub outcome: Option<String>,
    pub token_id: Option<String>,
}

#[derive(Deserialize)]
pub struct HistoryParams {
    pub outcome: Option<String>,
    pub token_id: Option<String>,
    pub condition_id: Option<String>,
    pub interval: Option<String>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
}

#[derive(Deserialize)]
pub struct TradesParams {
    pub outcome: Option<String>,
    pub token_id: Option<String>,
    pub market_ref: Option<String>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub market_id: String,
    pub outcome: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CancelParams {
    pub market_id: Option<String>,
}

#[derive(Deserialize)]
pub struct FillsParams {
    pub market_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct PositionsParams {
    pub market_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Exchange management
// ---------------------------------------------------------------------------

pub async fn add_exchange(
    State(state): State<AppState>,
    Json(req): Json<AddExchangeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchange = ExchangeInner::new(&req.exchange, req.config)?;
    // Verify by fetching balance
    let bal = exchange.fetch_balance().await?;
    let balance_total: f64 = bal.values().sum();
    let info = exchange.describe();
    let id = exchange.id().to_string();
    state.balances.write().await.insert(id.clone(), bal);
    state.exchanges.write().await.insert(id.clone(), exchange);
    Ok(Json(json!({
        "id": id,
        "name": info.name,
        "balance": balance_total,
        "capabilities": {
            "has_fetch_markets": info.has_fetch_markets,
            "has_create_order": info.has_create_order,
            "has_cancel_order": info.has_cancel_order,
            "has_fetch_positions": info.has_fetch_positions,
            "has_fetch_balance": info.has_fetch_balance,
            "has_fetch_orderbook": info.has_fetch_orderbook,
            "has_fetch_price_history": info.has_fetch_price_history,
            "has_fetch_trades": info.has_fetch_trades,
            "has_fetch_events": info.has_fetch_events,
            "has_fetch_fills": info.has_fetch_fills,
        }
    })))
}

pub async fn list_exchanges(State(state): State<AppState>) -> Json<serde_json::Value> {
    let exchanges = state.exchanges.read().await;
    let balances = state.balances.read().await;
    let list: Vec<serde_json::Value> = exchanges
        .values()
        .map(|e| {
            let info = e.describe();
            let id = e.id();
            let bal = balances.get(id);
            let balance_total: f64 = bal.map(|b| b.values().sum()).unwrap_or(0.0);
            json!({
                "id": id,
                "name": info.name,
                "balance": balance_total,
                "balance_detail": bal,
                "capabilities": {
                    "has_fetch_markets": info.has_fetch_markets,
                    "has_create_order": info.has_create_order,
                    "has_cancel_order": info.has_cancel_order,
                    "has_fetch_positions": info.has_fetch_positions,
                    "has_fetch_balance": info.has_fetch_balance,
                    "has_fetch_orderbook": info.has_fetch_orderbook,
                    "has_fetch_price_history": info.has_fetch_price_history,
                    "has_fetch_trades": info.has_fetch_trades,
                    "has_fetch_events": info.has_fetch_events,
                    "has_fetch_fills": info.has_fetch_fills,
                }
            })
        })
        .collect();
    Json(json!({ "exchanges": list }))
}

pub async fn remove_exchange(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut exchanges = state.exchanges.write().await;
    if exchanges.remove(&id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found(&format!("exchange '{id}' not connected")))
    }
}

// ---------------------------------------------------------------------------
// Market data
// ---------------------------------------------------------------------------

pub async fn fetch_market(
    State(state): State<AppState>,
    Path((exchange_id, market_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let market = exchange.fetch_market(&market_id).await?;
    Ok(Json(serde_json::to_value(market).unwrap()))
}

pub async fn fetch_event_markets(
    State(state): State<AppState>,
    Path((exchange_id, group_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let markets = exchange.fetch_event_markets(&group_id).await?;
    Ok(Json(serde_json::to_value(markets).unwrap()))
}

pub async fn fetch_orderbook(
    State(state): State<AppState>,
    Path((exchange_id, market_id)): Path<(String, String)>,
    Query(params): Query<OrderbookParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let orderbook = exchange
        .fetch_orderbook(OrderbookRequest {
            market_id,
            outcome: params.outcome,
            token_id: params.token_id,
        })
        .await?;
    Ok(Json(serde_json::to_value(orderbook).unwrap()))
}

pub async fn fetch_price_history(
    State(state): State<AppState>,
    Path((exchange_id, market_id)): Path<(String, String)>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let interval = params
        .interval
        .as_deref()
        .unwrap_or("1d")
        .parse::<PriceHistoryInterval>()
        .unwrap_or(PriceHistoryInterval::OneDay);
    let candles = exchange
        .fetch_price_history(PriceHistoryRequest {
            market_id,
            outcome: params.outcome,
            token_id: params.token_id,
            condition_id: params.condition_id,
            interval,
            start_ts: params.start_ts,
            end_ts: params.end_ts,
        })
        .await?;
    Ok(Json(serde_json::to_value(candles).unwrap()))
}

pub async fn fetch_trades(
    State(state): State<AppState>,
    Path((exchange_id, market_id)): Path<(String, String)>,
    Query(params): Query<TradesParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let (trades, cursor) = exchange
        .fetch_trades(TradesRequest {
            market_id,
            market_ref: params.market_ref,
            outcome: params.outcome,
            token_id: params.token_id,
            start_ts: None,
            end_ts: None,
            limit: params.limit,
            cursor: params.cursor,
        })
        .await?;
    Ok(Json(json!({ "trades": trades, "next_cursor": cursor })))
}

// ---------------------------------------------------------------------------
// Trading
// ---------------------------------------------------------------------------

pub async fn create_order(
    State(state): State<AppState>,
    Path(exchange_id): Path<String>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let order = exchange
        .create_order(
            &req.market_id,
            &req.outcome,
            req.side,
            req.price,
            req.size,
            req.params,
        )
        .await?;
    Ok(Json(serde_json::to_value(order).unwrap()))
}

pub async fn fetch_orders(
    State(state): State<AppState>,
    Path(exchange_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let orders = exchange.fetch_open_orders(None).await?;
    Ok(Json(serde_json::to_value(orders).unwrap()))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Path((exchange_id, order_id)): Path<(String, String)>,
    Query(params): Query<CancelParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let order = exchange
        .cancel_order(&order_id, params.market_id.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(order).unwrap()))
}

// ---------------------------------------------------------------------------
// Portfolio
// ---------------------------------------------------------------------------

pub async fn fetch_positions(
    State(state): State<AppState>,
    Path(exchange_id): Path<String>,
    Query(params): Query<PositionsParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let positions = exchange
        .fetch_positions(params.market_id.as_deref())
        .await?;
    Ok(Json(serde_json::to_value(positions).unwrap()))
}

pub async fn fetch_balance(
    State(state): State<AppState>,
    Path(exchange_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let balance = exchange.fetch_balance().await?;
    // Update cached balance
    state
        .balances
        .write()
        .await
        .insert(exchange_id, balance.clone());
    Ok(Json(serde_json::to_value(balance).unwrap()))
}

pub async fn fetch_fills_handler(
    State(state): State<AppState>,
    Path(exchange_id): Path<String>,
    Query(params): Query<FillsParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let exchange = exchanges
        .get(&exchange_id)
        .ok_or_else(|| not_found(&format!("exchange '{exchange_id}' not connected")))?;
    let fills = exchange
        .fetch_fills(params.market_id.as_deref(), params.limit)
        .await?;
    Ok(Json(serde_json::to_value(fills).unwrap()))
}

pub async fn fetch_all_positions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let mut all = Vec::new();
    for (id, exchange) in exchanges.iter() {
        match exchange.fetch_positions(None).await {
            Ok(positions) => {
                for p in positions {
                    all.push(json!({
                        "exchange": id,
                        "market_id": p.market_id,
                        "outcome": p.outcome,
                        "size": p.size,
                        "average_price": p.average_price,
                        "current_price": p.current_price,
                        "unrealized_pnl": p.unrealized_pnl(),
                        "unrealized_pnl_percent": p.unrealized_pnl_percent(),
                        "cost_basis": p.cost_basis(),
                        "current_value": p.current_value(),
                    }));
                }
            }
            Err(e) => {
                tracing::warn!(exchange = id, error = %e, "failed to fetch positions");
            }
        }
    }
    Ok(Json(json!({ "positions": all })))
}

pub async fn fetch_all_balances(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let exchanges = state.exchanges.read().await;
    let mut balances = serde_json::Map::new();
    for (id, exchange) in exchanges.iter() {
        match exchange.fetch_balance().await {
            Ok(bal) => {
                balances.insert(id.clone(), serde_json::to_value(bal).unwrap());
            }
            Err(e) => {
                tracing::warn!(exchange = id, error = %e, "failed to fetch balance");
            }
        }
    }
    Ok(Json(json!({ "balances": balances })))
}
