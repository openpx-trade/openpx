mod api;
mod state;

use std::env;

use axum::routing::{delete, get, post};
use axum::Router;
use px_sdk::ExchangeInner;
use serde_json::json;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use state::AppState;

/// Construct exchanges from .env, authenticate, verify balance, keep only verified.
async fn load_exchanges_from_env(state: &AppState) {
    let mut pending: Vec<(String, ExchangeInner)> = Vec::new();

    // --- Polymarket ---
    if let Ok(pk) = env::var("POLYMARKET_PRIVATE_KEY") {
        if !pk.is_empty() {
            let mut config = json!({ "private_key": pk });
            // Funder is only valid for GnosisSafe/Proxy signature types.
            // PolymarketConfig defaults to EOA, and passing funder with EOA causes a
            // validation error. Only pass funder when the user explicitly sets a
            // non-EOA signature type.
            let sig_type = env::var("POLYMARKET_SIGNATURE_TYPE")
                .unwrap_or_default()
                .to_lowercase();
            if sig_type == "gnosis" || sig_type == "proxy" {
                if let Ok(f) = env::var("POLYMARKET_FUNDER") {
                    if !f.is_empty() {
                        config["funder"] = json!(f);
                    }
                }
            }
            if let Ok(k) = env::var("POLY_BUILDER_API_KEY") {
                if !k.is_empty() {
                    config["api_key"] = json!(k);
                }
            }
            if let Ok(s) = env::var("POLY_BUILDER_SECRET") {
                if !s.is_empty() {
                    config["api_secret"] = json!(s);
                }
            }
            if let Ok(p) = env::var("POLY_BUILDER_PASSPHRASE") {
                if !p.is_empty() {
                    config["api_passphrase"] = json!(p);
                }
            }
            match ExchangeInner::new("polymarket", config) {
                Ok(mut ex) => {
                    if let ExchangeInner::Polymarket(ref mut poly) = ex {
                        match poly.init_trading().await {
                            Ok(_) => tracing::info!("Polymarket: trading initialized"),
                            Err(e) => {
                                tracing::warn!("Polymarket: init_trading failed: {e}");
                            }
                        }
                    }
                    pending.push(("polymarket".into(), ex));
                }
                Err(e) => tracing::warn!("Polymarket: construction failed: {e}"),
            }
        }
    }

    // --- Kalshi ---
    if let Ok(key_id) = env::var("KALSHI_API_KEY_ID") {
        if !key_id.is_empty() {
            let mut config = json!({ "api_key_id": key_id });
            if let Ok(pem) = env::var("KALSHI_PRIVATE_KEY_PEM") {
                if !pem.is_empty() {
                    config["private_key_pem"] = json!(pem);
                }
            } else if let Ok(path) = env::var("KALSHI_PRIVATE_KEY_PATH") {
                if !path.is_empty() {
                    match std::fs::read_to_string(&path) {
                        Ok(pem) => config["private_key_pem"] = json!(pem),
                        Err(e) => tracing::warn!("Kalshi: failed to read key file '{path}': {e}"),
                    }
                }
            }
            if let Ok(demo) = env::var("KALSHI_DEMO") {
                if demo == "true" || demo == "1" {
                    config["demo"] = json!(true);
                }
            }
            match ExchangeInner::new("kalshi", config) {
                Ok(ex) => pending.push(("kalshi".into(), ex)),
                Err(e) => tracing::warn!("Kalshi: construction failed: {e}"),
            }
        }
    }

    // --- Limitless ---
    if let Ok(pk) = env::var("LIMITLESS_PRIVATE_KEY") {
        if !pk.is_empty() {
            let config = json!({ "private_key": pk });
            match ExchangeInner::new("limitless", config) {
                Ok(ex) => {
                    // Limitless requires explicit authentication before balance/position calls
                    if let ExchangeInner::Limitless(ref lim) = ex {
                        match lim.authenticate().await {
                            Ok(_) => tracing::info!("Limitless: authenticated"),
                            Err(e) => tracing::warn!("Limitless: authentication failed: {e}"),
                        }
                    }
                    pending.push(("limitless".into(), ex));
                }
                Err(e) => tracing::warn!("Limitless: construction failed: {e}"),
            }
        }
    }

    // --- Opinion ---
    if let Ok(api_key) = env::var("OPINION_API_KEY") {
        if !api_key.is_empty() {
            let mut config = json!({ "api_key": api_key });
            if let Ok(pk) = env::var("OPINION_PRIVATE_KEY") {
                if !pk.is_empty() {
                    config["private_key"] = json!(pk);
                }
            }
            if let Ok(ms) = env::var("OPINION_MULTI_SIG_ADDR") {
                if !ms.is_empty() {
                    config["multi_sig_addr"] = json!(ms);
                }
            }
            match ExchangeInner::new("opinion", config) {
                Ok(ex) => pending.push(("opinion".into(), ex)),
                Err(e) => tracing::warn!("Opinion: construction failed: {e}"),
            }
        }
    }

    // --- PredictFun ---
    if let Ok(api_key) = env::var("PREDICTFUN_API_KEY") {
        if !api_key.is_empty() {
            let mut config = json!({ "api_key": api_key });
            if let Ok(pk) = env::var("PREDICTFUN_PRIVATE_KEY") {
                if !pk.is_empty() {
                    config["private_key"] = json!(pk);
                }
            }
            if let Ok(t) = env::var("PREDICTFUN_TESTNET") {
                if t == "true" || t == "1" {
                    config["testnet"] = json!(true);
                }
            }
            match ExchangeInner::new("predictfun", config) {
                Ok(ex) => {
                    // PredictFun requires explicit authentication
                    if let ExchangeInner::PredictFun(ref pf) = ex {
                        match pf.authenticate().await {
                            Ok(_) => tracing::info!("PredictFun: authenticated"),
                            Err(e) => tracing::warn!("PredictFun: authentication failed: {e}"),
                        }
                    }
                    pending.push(("predictfun".into(), ex));
                }
                Err(e) => tracing::warn!("PredictFun: construction failed: {e}"),
            }
        }
    }

    // --- Verify each exchange by fetching balance ---
    tracing::info!("Verifying {} exchange(s) by fetching balance...", pending.len());

    let mut exchanges = state.exchanges.write().await;
    let mut balances = state.balances.write().await;

    for (id, exchange) in pending {
        match exchange.fetch_balance().await {
            Ok(bal) => {
                let total: f64 = bal.values().sum();
                tracing::info!("{}: verified (balance: ${:.2})", id, total);
                balances.insert(id.clone(), bal);
                exchanges.insert(id, exchange);
            }
            Err(e) => {
                tracing::warn!("{}: balance check failed, skipping — {}", id, e);
            }
        }
    }

    tracing::info!(
        "{} exchange(s) verified and ready",
        exchanges.len()
    );
}

#[tokio::main]
async fn main() {
    // Load .env file (silent if missing)
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let state = AppState::new();

    // Auto-connect exchanges from .env credentials
    load_exchanges_from_env(&state).await;

    let api = Router::new()
        // Exchange management
        .route(
            "/exchanges",
            get(api::list_exchanges).post(api::add_exchange),
        )
        .route("/exchanges/:id", delete(api::remove_exchange))
        // Market data
        .route(
            "/exchanges/:exchange_id/markets/:market_id",
            get(api::fetch_market),
        )
        .route(
            "/exchanges/:exchange_id/events/:group_id",
            get(api::fetch_event_markets),
        )
        .route(
            "/exchanges/:exchange_id/markets/:market_id/orderbook",
            get(api::fetch_orderbook),
        )
        .route(
            "/exchanges/:exchange_id/markets/:market_id/history",
            get(api::fetch_price_history),
        )
        .route(
            "/exchanges/:exchange_id/markets/:market_id/trades",
            get(api::fetch_trades),
        )
        // Trading
        .route(
            "/exchanges/:exchange_id/orders",
            post(api::create_order).get(api::fetch_orders),
        )
        .route(
            "/exchanges/:exchange_id/orders/:order_id",
            delete(api::cancel_order),
        )
        // Portfolio
        .route(
            "/exchanges/:exchange_id/positions",
            get(api::fetch_positions),
        )
        .route("/exchanges/:exchange_id/balance", get(api::fetch_balance))
        .route(
            "/exchanges/:exchange_id/fills",
            get(api::fetch_fills_handler),
        )
        .route("/portfolio/positions", get(api::fetch_all_positions))
        .route("/portfolio/balances", get(api::fetch_all_balances));

    let static_dir = if std::path::Path::new("dashboard/static").exists() {
        "dashboard/static"
    } else {
        "static"
    };

    let app = Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new(static_dir).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("OpenPX Dashboard running at http://localhost:3000");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
