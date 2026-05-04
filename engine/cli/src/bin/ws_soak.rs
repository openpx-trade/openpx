//! WebSocket latency soak for OpenPX autoresearch.
//!
//! Connects via the **real `OrderBookWebSocket` trait** (no mocks) to a live
//! exchange, subscribes to a hand-picked set of liquid markets, and runs for
//! a fixed duration (default 60s — the user's hard requirement). Per message,
//! captures the in-process pipeline latency: wall-clock between socket-read
//! (`update.local_ts()`) and the moment the consumer receives the update.
//! That's the latency a consumer of the unified API actually observes.
//!
//! Emits a single JSON blob to stdout for `autoresearch/oracle.py` to parse.

use std::env;
use std::time::{Duration, Instant};

use clap::Parser;
use hdrhistogram::Histogram;
use openpx::WebSocketInner;
use px_core::websocket::OrderBookWebSocket;
use serde::Serialize;

#[derive(Parser)]
#[command(name = "ws_soak", about = "Live WebSocket latency soak for OpenPX autoresearch")]
struct Cli {
    /// Which exchange (`kalshi` or `polymarket`).
    #[arg(long)]
    exchange: String,
    /// Markets to subscribe to. Repeat or comma-separate.
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    markets: Vec<String>,
    /// Soak duration in seconds (default 60 — autoresearch oracle's hard requirement).
    #[arg(long, default_value_t = 60)]
    duration_secs: u64,
}

fn make_exchange_config(id: &str) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    let vars: &[(&str, &str)] = match id {
        "kalshi" => &[
            ("KALSHI_API_KEY_ID", "api_key_id"),
            ("KALSHI_PRIVATE_KEY_PEM", "private_key_pem"),
            ("KALSHI_PRIVATE_KEY_PATH", "private_key_path"),
        ],
        "polymarket" => &[
            ("POLYMARKET_PRIVATE_KEY", "private_key"),
            ("POLYMARKET_FUNDER", "funder"),
            ("POLYMARKET_SIGNATURE_TYPE", "signature_type"),
            ("POLYMARKET_API_KEY", "api_key"),
            ("POLYMARKET_API_SECRET", "api_secret"),
            ("POLYMARKET_API_PASSPHRASE", "api_passphrase"),
        ],
        _ => &[],
    };
    for (env_key, config_key) in vars {
        if let Ok(v) = env::var(env_key) {
            obj.insert((*config_key).into(), v.into());
        }
    }
    serde_json::Value::Object(obj)
}

fn new_histogram() -> Histogram<u64> {
    // µs precision up to ~1s — WS pipeline latency above 1s is broken anyway.
    Histogram::<u64>::new_with_bounds(1, 1_000_000, 3).expect("histogram bounds")
}

#[derive(Serialize)]
struct SoakReport {
    exchange: String,
    markets: Vec<String>,
    duration_secs: u64,
    total_messages: u64,
    msgs_per_sec: f64,
    snapshots: u64,
    deltas: u64,
    trades: u64,
    other: u64,
    pipeline_p50_us: u64,
    pipeline_p99_us: u64,
    pipeline_p999_us: u64,
    pipeline_max_us: u64,
    pipeline_mean_us: f64,
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("rustls crypto provider");
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    if cli.markets.is_empty() {
        eprintln!("error: --markets must list at least one ticker");
        std::process::exit(1);
    }

    let config = make_exchange_config(&cli.exchange);
    let mut ws = WebSocketInner::new(&cli.exchange, config).unwrap_or_else(|e| {
        eprintln!("error: failed to create {} websocket: {e}", cli.exchange);
        std::process::exit(1);
    });

    let updates = ws.updates().expect("updates() taken twice");

    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: websocket connect failed: {e}");
        std::process::exit(1);
    });

    for ticker in &cli.markets {
        ws.subscribe(ticker).await.unwrap_or_else(|e| {
            eprintln!("error: failed to subscribe to {ticker}: {e}");
            std::process::exit(1);
        });
    }

    eprintln!(
        "ws_soak: exchange={} markets={:?} duration={}s — soaking…",
        cli.exchange, cli.markets, cli.duration_secs
    );

    let mut hist = new_histogram();
    let mut total: u64 = 0;
    let mut snapshots: u64 = 0;
    let mut deltas: u64 = 0;
    let mut trades: u64 = 0;
    let mut other: u64 = 0;

    let deadline = Instant::now() + Duration::from_secs(cli.duration_secs);

    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline - now;

        match tokio::time::timeout(remaining, updates.next()).await {
            Ok(Some(update)) => {
                let arrival = Instant::now();
                let pipeline_us = arrival
                    .saturating_duration_since(update.local_ts())
                    .as_micros() as u64;
                let _ = hist.record(pipeline_us.clamp(1, 1_000_000));

                total += 1;
                match &update {
                    px_core::WsUpdate::Snapshot { .. } => snapshots += 1,
                    px_core::WsUpdate::Delta { .. } => deltas += 1,
                    px_core::WsUpdate::Trade { .. } => trades += 1,
                    _ => other += 1,
                }
            }
            Ok(None) => {
                eprintln!("ws_soak: stream ended early");
                break;
            }
            Err(_elapsed) => break,
        }
    }

    let secs = cli.duration_secs as f64;
    let report = SoakReport {
        exchange: cli.exchange,
        markets: cli.markets,
        duration_secs: cli.duration_secs,
        total_messages: total,
        msgs_per_sec: if secs > 0.0 { total as f64 / secs } else { 0.0 },
        snapshots,
        deltas,
        trades,
        other,
        pipeline_p50_us: hist.value_at_quantile(0.50),
        pipeline_p99_us: hist.value_at_quantile(0.99),
        pipeline_p999_us: hist.value_at_quantile(0.999),
        pipeline_max_us: hist.max(),
        pipeline_mean_us: hist.mean(),
    };

    println!(
        "{}",
        serde_json::to_string(&report).expect("serialize report")
    );
}
