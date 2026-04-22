//! Standalone latency probe for `polymarket-rs-client`.
//!
//! Prints one JSON line per timed iteration on stdout, e.g.
//!     {"i": 1, "elapsed_ms": 412.3}
//!
//! The main engine/bench/src/bin/network_bench.rs harness subprocesses this
//! binary and consumes those lines. It lives outside the main openpx
//! workspace because `polymarket-rs-client`'s pkcs8 pin conflicts with the
//! alloy stack we use in kalshi.

use clap::Parser;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(about = "polymarket-rs-client latency probe (stdout = JSON lines)")]
struct Args {
    #[arg(long, default_value_t = 20)]
    iterations: usize,

    #[arg(long, default_value_t = 100)]
    delay_ms: u64,

    #[arg(long, default_value_t = 5)]
    warmup: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // polymarket-rs-client 0.1 exposes `ClobClient::new` for unauthenticated
    // reads. The simplified-markets endpoint is anonymous so we can exercise
    // the full HTTP + JSON parse + response-construction path without keys.
    let client = polymarket_rs_client::ClobClient::new("https://clob.polymarket.com");

    // Warm up so DNS / TLS / HTTP/2 are primed — discarded.
    for _ in 0..args.warmup {
        let _ = client.get_simplified_markets(None).await;
        tokio::time::sleep(Duration::from_millis(args.delay_ms)).await;
    }

    for i in 1..=args.iterations {
        let start = Instant::now();
        match client.get_simplified_markets(None).await {
            Ok(_) => {
                let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                println!("{{\"i\":{i},\"elapsed_ms\":{elapsed_ms}}}");
            }
            Err(e) => {
                eprintln!("iter {i} error: {e}");
            }
        }
        tokio::time::sleep(Duration::from_millis(args.delay_ms)).await;
    }
    Ok(())
}
