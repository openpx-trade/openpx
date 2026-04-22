//! openpx vs polyfill-rs vs baseline-reqwest vs py-clob-client — single process,
//! same machine / network / time. Mirrors polyfill-rs's side_by_side_benchmark.
//!
//! Run:
//!     cargo run --release -p px-bench --bin openpx-bench-network -- \
//!         --iterations 20 --delay-ms 100 --warmup 5
//!
//! With polyfill-rs comparison (requires ../../../polyfill-rs checkout):
//!     cargo run --release -p px-bench --bin openpx-bench-network \
//!         --features compare-polyfill -- \
//!         --targets openpx,polyfill,baseline-reqwest,py-clob

use clap::Parser;
use px_bench::{gather_metadata, print_table, stats, write_report};
use reqwest::Client;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const ENDPOINT: &str = "https://clob.polymarket.com/simplified-markets?next_cursor=MA==";

#[derive(Parser, Debug, Clone)]
#[command(about = "openpx cross-library network latency benchmark")]
struct Args {
    #[arg(long, default_value_t = 20)]
    iterations: usize,

    #[arg(long, default_value_t = 100)]
    delay_ms: u64,

    #[arg(long, default_value_t = 5)]
    warmup: usize,

    #[arg(long, value_delimiter = ',', default_value = "openpx,baseline-reqwest")]
    targets: Vec<String>,

    #[arg(long, default_value = "bench-results")]
    out_dir: PathBuf,

    #[arg(long, default_value = ENDPOINT)]
    endpoint: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("  openpx cross-library network benchmark");
    println!("  endpoint    {}", args.endpoint);
    println!("  iterations  {}", args.iterations);
    println!("  delay       {} ms", args.delay_ms);
    println!("  warmup      {}", args.warmup);
    println!("  targets     {}", args.targets.join(", "));
    println!("═══════════════════════════════════════════════════════════════════════════\n");

    let mut results: Vec<(String, stats::Summary)> = Vec::new();

    for target in &args.targets {
        let name = target.trim();
        println!("▶ Running: {name}");
        let samples = match name {
            "openpx" => run_openpx(&args).await?,
            "baseline-reqwest" => run_baseline(&args).await?,
            "polyfill" | "polyfill-rs" => {
                eprintln!(
                    "  (skipped — polyfill comparison requires editing engine/bench/Cargo.toml \
                     to uncomment the polyfill-rs path dep; left out of the committed file so CI \
                     builds without the sibling repo)"
                );
                continue;
            }
            "py-clob" | "py-clob-client" => run_py_clob(&args).await?,
            other => {
                eprintln!("  unknown target '{other}' — skipping");
                continue;
            }
        };
        let summary = stats::summarize(&samples);
        println!(
            "  done  mean={:.1} ms  stddev={:.1} ms  n={}",
            summary.mean_ms, summary.stddev_ms, summary.n
        );
        let display_name = if name == "polyfill" {
            "polyfill-rs".to_string()
        } else {
            name.to_string()
        };
        results.push((display_name, summary));
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    print_table(&results);

    let metadata = gather_metadata(&args.endpoint, args.iterations, args.delay_ms, args.warmup);
    match write_report(&args.out_dir, &metadata, &results) {
        Ok(path) => println!("\nWrote report to {}", path.display()),
        Err(e) => eprintln!("\nReport write failed: {e}"),
    }
    Ok(())
}

/// Mirrors engine/exchanges/polymarket/src/client.rs:HttpClient::new.
/// Keep in sync with production settings.
fn build_openpx_client() -> Client {
    Client::builder()
        .http2_adaptive_window(true)
        .http2_initial_stream_window_size(512 * 1024)
        .tcp_nodelay(true)
        .pool_max_idle_per_host(10)
        .http2_keep_alive_interval(Duration::from_secs(15))
        .timeout(Duration::from_secs(30))
        .no_proxy()
        .build()
        .expect("build openpx client")
}

async fn sample_loop<F, Fut>(args: &Args, make_req: F) -> Vec<Duration>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    for _ in 0..args.warmup {
        if let Err(e) = make_req().await {
            eprintln!("  warmup error: {e}");
        }
        tokio::time::sleep(Duration::from_millis(args.delay_ms)).await;
    }

    let mut samples = Vec::with_capacity(args.iterations);
    for i in 1..=args.iterations {
        let start = Instant::now();
        match make_req().await {
            Ok(()) => {
                let elapsed = start.elapsed();
                samples.push(elapsed);
                if i <= 3 || i > args.iterations.saturating_sub(3) {
                    println!(
                        "  request {i:>3}: {:.1} ms",
                        elapsed.as_micros() as f64 / 1000.0
                    );
                } else if i == 4 {
                    println!("  ...");
                }
            }
            Err(e) => eprintln!("  request {i:>3}: error {e}"),
        }
        tokio::time::sleep(Duration::from_millis(args.delay_ms)).await;
    }
    samples
}

async fn run_openpx(args: &Args) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let client = build_openpx_client();
    let url = args.endpoint.clone();
    let samples = sample_loop(args, || {
        let client = client.clone();
        let url = url.clone();
        async move {
            let resp = client.get(&url).send().await?;
            let _body: serde_json::Value = resp.json().await?;
            Ok(())
        }
    })
    .await;
    Ok(samples)
}

async fn run_baseline(args: &Args) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = args.endpoint.clone();
    let samples = sample_loop(args, || {
        let client = client.clone();
        let url = url.clone();
        async move {
            let resp = client.get(&url).send().await?;
            let _body: serde_json::Value = resp.json().await?;
            Ok(())
        }
    })
    .await;
    Ok(samples)
}

#[cfg(feature = "compare-polyfill")]
async fn run_polyfill(args: &Args) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let client = polyfill_rs::ClobClient::new("https://clob.polymarket.com");
    client.start_keepalive(Duration::from_secs(30)).await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    let url = args.endpoint.clone();
    let samples = sample_loop(args, || {
        let http = client.http_client.clone();
        let url = url.clone();
        async move {
            let resp = http.get(url).send().await?;
            let _body: serde_json::Value = resp.json().await?;
            Ok(())
        }
    })
    .await;
    client.stop_keepalive().await;
    Ok(samples)
}

async fn run_py_clob(args: &Args) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("py_clob_bench.py");
    if !script_path.exists() {
        return Err(format!("py_clob_bench.py missing at {}", script_path.display()).into());
    }
    let output = tokio::process::Command::new("python3")
        .arg(&script_path)
        .arg("--iterations")
        .arg(args.iterations.to_string())
        .arg("--delay-ms")
        .arg(args.delay_ms.to_string())
        .arg("--warmup")
        .arg(args.warmup.to_string())
        .arg("--endpoint")
        .arg(&args.endpoint)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("py_clob_bench.py failed: {stderr}").into());
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
        eprintln!("  (py-clob produced no samples — check py-clob-client is installed)");
    }
    Ok(samples)
}
