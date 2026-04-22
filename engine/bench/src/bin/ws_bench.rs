//! WebSocket subscribe-to-first-book latency — the "time to live data" metric
//! every trader cares about when a bot spins up.
//!
//! Comparisons:
//!
//! - `openpx` — the integrated `PolymarketWebSocket`: connect + subscribe +
//!   typed `WsUpdate::Snapshot` ready for downstream consumption.
//! - `baseline-tungstenite` — raw `tokio-tungstenite` + `serde_json` parse,
//!   which is what you'd hand-roll in Rust without openpx.
//! - `baseline-python-ws` — raw `websockets` library + `json.loads`, which
//!   is what a Python bot would do (neither `py-clob-client` nor any
//!   Polymarket-published Python SDK exposes market-book WebSockets).
//!
//! Each iteration is a cold start: open a fresh WS connection, send the
//! subscribe, wait for the first book snapshot, record elapsed, close.
//! An asset_id is fetched once via REST so every iteration / target
//! exercises the same live market.
//!
//! Run:
//!     cargo run --release -p px-bench --bin openpx-bench-ws -- \
//!         --iterations 10 --delay-ms 500 --warmup 2

use clap::Parser;
use futures::{SinkExt, StreamExt};
use px_bench::{gather_metadata, print_table, stats, write_report};
use px_core::OrderBookWebSocket;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio_tungstenite::{connect_async, tungstenite::Message};

const WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";
const GAMMA_BASE: &str = "https://gamma-api.polymarket.com";

#[derive(Parser, Debug, Clone)]
#[command(about = "openpx WebSocket subscribe-to-first-book latency benchmark")]
struct Args {
    /// Number of timed iterations per target.
    #[arg(long, default_value_t = 10)]
    iterations: usize,

    /// Delay between timed iterations, ms.
    #[arg(long, default_value_t = 500)]
    delay_ms: u64,

    /// Throwaway warmup iterations per target (not in stats).
    #[arg(long, default_value_t = 2)]
    warmup: usize,

    /// Per-iteration timeout waiting for the first book snapshot.
    #[arg(long, default_value_t = 15)]
    timeout_secs: u64,

    /// Comma-separated target list. Valid: openpx, baseline-tungstenite,
    /// python-ws.
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "openpx,baseline-tungstenite,python-ws"
    )]
    targets: Vec<String>,

    /// Asset ID to subscribe to. If omitted, a liquid market is fetched
    /// from the REST `/simplified-markets` endpoint at startup.
    #[arg(long)]
    asset_id: Option<String>,

    #[arg(long, default_value = "bench-results")]
    out_dir: PathBuf,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // rustls 0.23 requires explicit provider selection at process init.
    // Matches the engine/cli + engine/recorder binaries.
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| "failed to install rustls crypto provider")?;

    let args = Args::parse();

    let asset_id = match &args.asset_id {
        Some(id) => id.clone(),
        None => fetch_liquid_asset_id().await?,
    };

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  openpx WebSocket first-book benchmark");
    println!("  asset_id    {asset_id}");
    println!("  iterations  {}", args.iterations);
    println!("  delay       {} ms", args.delay_ms);
    println!("  warmup      {}", args.warmup);
    println!("  timeout     {}s per iteration", args.timeout_secs);
    println!("  targets     {}", args.targets.join(", "));
    println!("═══════════════════════════════════════════════════════════════════════════\n");

    let mut results: Vec<(String, stats::Summary)> = Vec::new();
    let timeout = Duration::from_secs(args.timeout_secs);

    for target in &args.targets {
        let name = target.trim();
        println!("▶ Running: {name}");
        let samples: Vec<Duration> = match name {
            "openpx" => run_openpx(&args, &asset_id, timeout).await,
            "baseline-tungstenite" | "baseline-tung" => {
                run_tungstenite(&args, &asset_id, timeout).await
            }
            "python-ws" | "py-ws" => run_python_ws(&args, &asset_id, timeout).await?,
            other => {
                eprintln!("  unknown target '{other}' — skipping");
                continue;
            }
        };
        let summary = stats::summarize(&samples);
        println!(
            "  done  mean={:.1} ms  stddev={:.1} ms  median={:.1} ms  n={}",
            summary.mean_ms, summary.stddev_ms, summary.median_ms, summary.n
        );
        let display_name = match name {
            "baseline-tung" => "baseline-tungstenite".to_string(),
            "py-ws" => "python-ws".to_string(),
            other => other.to_string(),
        };
        results.push((display_name, summary));
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    print_table(&results);

    let metadata = gather_metadata("ws:first-book", args.iterations, args.delay_ms, args.warmup);
    match write_report(&args.out_dir, &metadata, &results) {
        Ok(path) => println!("\nWrote report to {}", path.display()),
        Err(e) => eprintln!("\nReport write failed: {e}"),
    }
    Ok(())
}

async fn fetch_liquid_asset_id() -> Result<String, Box<dyn std::error::Error>> {
    // gamma-api is Polymarket's indexed market service — supports filtering
    // and sort by volume so we land on a liquid, still-open market rather
    // than the oldest-first drip from /simplified-markets.
    let url = format!(
        "{GAMMA_BASE}/markets?active=true&closed=false&limit=20&order=volume&ascending=false"
    );
    let resp = reqwest::get(&url).await?;
    let body: serde_json::Value = resp.json().await?;
    let markets = body
        .as_array()
        .cloned()
        .or_else(|| body.get("data").and_then(|v| v.as_array()).cloned())
        .ok_or("no market array in gamma-api response")?;

    for m in markets {
        if m.get("closed").and_then(|v| v.as_bool()).unwrap_or(true) {
            continue;
        }
        let tokens_raw = m.get("clobTokenIds").and_then(|v| v.as_str());
        let tokens: Vec<String> = match tokens_raw {
            Some(s) => serde_json::from_str(s).unwrap_or_default(),
            None => continue,
        };
        let slug = m
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("(unknown)");
        if let Some(id) = tokens.first() {
            if !id.is_empty() {
                println!("  fetched asset_id {id}  (market slug: {slug})");
                return Ok(id.clone());
            }
        }
    }
    Err("no active + open market found in gamma-api top-volume list".into())
}

async fn timed_iter<F, Fut>(args: &Args, label: &str, mut runner: F) -> Vec<Duration>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<Duration, String>>,
{
    for _ in 0..args.warmup {
        let _ = runner().await;
        tokio::time::sleep(Duration::from_millis(args.delay_ms)).await;
    }

    let mut samples = Vec::with_capacity(args.iterations);
    for i in 1..=args.iterations {
        match runner().await {
            Ok(elapsed) => {
                samples.push(elapsed);
                if i <= 3 || i > args.iterations.saturating_sub(3) {
                    println!(
                        "  iter {i:>3}: {:.1} ms",
                        elapsed.as_micros() as f64 / 1000.0
                    );
                } else if i == 4 {
                    println!("  ...");
                }
            }
            Err(e) => eprintln!("  iter {i:>3}: {label} error — {e}"),
        }
        tokio::time::sleep(Duration::from_millis(args.delay_ms)).await;
    }
    samples
}

/// openpx path: build the integrated `PolymarketWebSocket`, connect, take
/// the updates stream, subscribe, wait for the first `WsUpdate::Snapshot`.
async fn run_openpx(args: &Args, asset_id: &str, timeout: Duration) -> Vec<Duration> {
    let asset_id = asset_id.to_string();
    timed_iter(args, "openpx", || {
        let asset_id = asset_id.clone();
        async move {
            let start = Instant::now();
            let mut ws = px_exchange_polymarket::PolymarketWebSocket::with_config(false);
            ws.connect().await.map_err(|e| e.to_string())?;
            let updates = ws.updates().ok_or("updates stream already taken")?;
            ws.subscribe(&asset_id).await.map_err(|e| e.to_string())?;

            let deadline = tokio::time::timeout(timeout, async {
                while let Some(ev) = updates.next().await {
                    if matches!(ev, px_core::WsUpdate::Snapshot { .. }) {
                        return Ok::<_, String>(());
                    }
                }
                Err("updates stream ended before first snapshot".into())
            })
            .await
            .map_err(|_| "timeout waiting for snapshot".to_string())?;
            deadline?;

            let elapsed = start.elapsed();
            let _ = ws.disconnect().await;
            Ok(elapsed)
        }
    })
    .await
}

/// What a Rust dev would write without openpx: tokio-tungstenite +
/// serde_json, hand-built subscribe payload, manual walk for the first
/// `event_type == "book"` message.
async fn run_tungstenite(args: &Args, asset_id: &str, timeout: Duration) -> Vec<Duration> {
    let asset_id = asset_id.to_string();
    timed_iter(args, "baseline-tungstenite", || {
        let asset_id = asset_id.clone();
        async move {
            let start = Instant::now();
            let (mut ws, _resp) = connect_async(WS_URL).await.map_err(|e| e.to_string())?;

            let subscribe = serde_json::json!({
                "type": "market",
                "assets_ids": [asset_id],
                "markets": []
            });
            ws.send(Message::Text(subscribe.to_string()))
                .await
                .map_err(|e| e.to_string())?;

            let found = tokio::time::timeout(timeout, async {
                while let Some(frame) = ws.next().await {
                    let msg = frame.map_err(|e| e.to_string())?;
                    if let Message::Text(text) = msg {
                        let s = text.as_str();
                        let parsed = serde_json::from_str::<serde_json::Value>(s)
                            .map_err(|e| e.to_string())?;
                        let items: Vec<&serde_json::Value> = if let Some(arr) = parsed.as_array() {
                            arr.iter().collect()
                        } else {
                            vec![&parsed]
                        };
                        for item in items {
                            if item.get("event_type").and_then(|v| v.as_str()) == Some("book") {
                                return Ok::<_, String>(());
                            }
                        }
                    }
                }
                Err("stream closed before first book event".into())
            })
            .await
            .map_err(|_| "timeout".to_string())?;
            found?;

            let elapsed = start.elapsed();
            let _ = ws.close(None).await;
            Ok(elapsed)
        }
    })
    .await
}

async fn run_python_ws(
    args: &Args,
    asset_id: &str,
    timeout: Duration,
) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("ws_baseline_py.py");
    if !script_path.exists() {
        return Err(format!("ws_baseline_py.py missing at {}", script_path.display()).into());
    }

    let output = tokio::process::Command::new("python3")
        .arg(&script_path)
        .arg("--iterations")
        .arg((args.iterations + args.warmup).to_string())
        .arg("--warmup")
        .arg(args.warmup.to_string())
        .arg("--delay-ms")
        .arg(args.delay_ms.to_string())
        .arg("--timeout-secs")
        .arg(timeout.as_secs().to_string())
        .arg("--asset-id")
        .arg(asset_id)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ws_baseline_py.py failed: {stderr}").into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut samples = Vec::with_capacity(args.iterations);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(ms) = value.get("elapsed_ms").and_then(|v| v.as_f64()) {
                samples.push(Duration::from_micros((ms * 1000.0) as u64));
            }
        }
    }
    if samples.is_empty() {
        eprintln!("  (python-ws produced no samples — check `websockets` is installed)");
    }
    Ok(samples)
}
