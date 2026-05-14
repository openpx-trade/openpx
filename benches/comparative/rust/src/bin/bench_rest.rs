//! Rust REST head-to-head: OpenPX vs polymarket_client_sdk_v2 (+ Kalshi
//! OpenPX-only, since Kalshi has no upstream Rust SDK).
//!
//! Same methodology as the Python + TS suites: 20 rounds × 100 ms gap,
//! live unauthenticated endpoint. Writes a `{tasks: [...]}` JSON in the
//! same shape as the tinybench output so `tools/render_bench_readme.py`
//! can read it the same way.
//!
//! Run: `cargo run -p px-bench-comparative --bin bench_rest --release`
//! Output:
//!   benches/comparative/results/rust_polymarket.json
//!   benches/comparative/results/rust_kalshi.json

use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

use openpx::ExchangeInner;
use polymarket_client_sdk_v2::clob::types::request::OrderBookSummaryRequest;
use polymarket_client_sdk_v2::clob::{Client as ClobClient, Config as ClobConfig};
use polymarket_client_sdk_v2::types::U256;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::time::sleep;

const ROUNDS: usize = 20;
const GAP: Duration = Duration::from_millis(100);
const POLY_HOST: &str = "https://clob.polymarket.com";

#[derive(Serialize)]
struct Task {
    name: String,
    mean_ns: f64,
    stddev_ns: f64,
    samples: usize,
}

fn stats(samples: &[u128]) -> (f64, f64) {
    let n = samples.len() as f64;
    let mean = samples.iter().map(|&s| s as f64).sum::<f64>() / n;
    let var = samples
        .iter()
        .map(|&s| (s as f64 - mean).powi(2))
        .sum::<f64>()
        / n;
    (mean, var.sqrt())
}

fn fixture_meta(name: &str) -> Value {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "..",
        "fixtures",
        name,
    ]
    .iter()
    .collect();
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("missing fixture {}: {e}", path.display()));
    serde_json::from_slice(&bytes).expect("parse fixture meta")
}

fn results_dir() -> PathBuf {
    let p: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "results"].iter().collect();
    std::fs::create_dir_all(&p).expect("create results dir");
    p
}

fn write_tasks(filename: &str, tasks: Vec<Task>) {
    let path = results_dir().join(filename);
    let body = json!({ "tasks": tasks });
    std::fs::write(&path, serde_json::to_vec_pretty(&body).unwrap())
        .unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    println!("wrote {}", path.display());
}

async fn time_round<F, Fut, T>(rounds: usize, mut f: F) -> Vec<u128>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let mut samples = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        sleep(GAP).await;
        let t0 = Instant::now();
        let _ = f().await;
        samples.push(t0.elapsed().as_nanos());
    }
    samples
}

async fn bench_polymarket() {
    let meta = fixture_meta("polymarket_book.meta.json");
    let asset_id = meta["asset_id"].as_str().expect("asset_id").to_owned();
    let token_u256 = U256::from_str(&asset_id).expect("parse token_id");

    let openpx = ExchangeInner::new("polymarket", json!({})).expect("openpx polymarket");
    let sdk = ClobClient::new(POLY_HOST, ClobConfig::default()).expect("sdk client");

    let req = OrderBookSummaryRequest::builder().token_id(token_u256).build();

    println!("polymarket: openpx vs polymarket_client_sdk_v2, {ROUNDS} rounds × {GAP:?} gap");

    let openpx_samples = time_round(ROUNDS, || async {
        let _ = openpx.fetch_orderbook(&asset_id).await;
    })
    .await;

    let sdk_samples = time_round(ROUNDS, || async {
        let _ = sdk.order_book(&req).await;
    })
    .await;

    let (op_mean, op_sd) = stats(&openpx_samples);
    let (sk_mean, sk_sd) = stats(&sdk_samples);

    write_tasks(
        "rust_polymarket.json",
        vec![
            Task {
                name: "openpx::polymarket::fetch_orderbook".into(),
                mean_ns: op_mean,
                stddev_ns: op_sd,
                samples: openpx_samples.len(),
            },
            Task {
                name: "polymarket_client_sdk_v2::polymarket::fetch_orderbook".into(),
                mean_ns: sk_mean,
                stddev_ns: sk_sd,
                samples: sdk_samples.len(),
            },
        ],
    );
}

async fn bench_kalshi() {
    let meta = fixture_meta("kalshi_orderbook.meta.json");
    let ticker = meta["ticker"].as_str().expect("ticker").to_owned();

    let openpx = ExchangeInner::new("kalshi", json!({})).expect("openpx kalshi");

    println!("kalshi: openpx-only ({ROUNDS} rounds — no official Rust SDK)");
    let samples = time_round(ROUNDS, || async {
        let _ = openpx.fetch_orderbook(&ticker).await;
    })
    .await;

    let (mean, sd) = stats(&samples);
    write_tasks(
        "rust_kalshi.json",
        vec![Task {
            name: "openpx::kalshi::fetch_orderbook".into(),
            mean_ns: mean,
            stddev_ns: sd,
            samples: samples.len(),
        }],
    );
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    bench_polymarket().await;
    bench_kalshi().await;
}
