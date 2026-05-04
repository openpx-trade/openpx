//! REST latency harness for OpenPX autoresearch.
//!
//! Drives the **real `Exchange` trait** (no mocks) against a live exchange,
//! captures p50/p99 wall-clock per operation via hdrhistogram, and emits a
//! single JSON blob to stdout for `autoresearch/oracle.py` to parse.
//!
//! Operations exercised, all on a single hand-picked liquid market:
//! - `fetch_market(ticker)`   — single-ticker variant of `fetch_markets`
//! - `fetch_orderbook(asset)` — full-depth L2 snapshot
//! - `fetch_trades(asset)`    — recent tape
//! - `create_order` + `cancel_order` (only with `--with-trading`; tiny limit
//!   far from market, immediately cancelled)
//!
//! Stub-free by construction: every iteration awaits a real network round-trip.

use std::env;
use std::time::Instant;

use clap::Parser;
use hdrhistogram::Histogram;
use openpx::ExchangeInner;
use px_core::{
    error::OpenPxError, CreateOrderRequest, FetchMarketsParams, MarketStatusFilter, OrderOutcome,
    OrderSide, OrderType, TradesRequest,
};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "rest_bench", about = "Live REST latency harness for OpenPX autoresearch")]
struct Cli {
    /// Which exchange to drive (`kalshi` or `polymarket`).
    #[arg(long)]
    exchange: String,
    /// Single market ticker for `fetch_market` / `fetch_orderbook` / `fetch_trades`.
    #[arg(long)]
    ticker: String,
    /// Asset id for orderbook + trades fetches. Defaults to `--ticker` (Kalshi
    /// uses ticker-as-asset-id; Polymarket needs an explicit token id).
    #[arg(long)]
    asset_id: Option<String>,
    /// Measured iterations per operation (default 50).
    #[arg(long, default_value_t = 50)]
    iterations: usize,
    /// Warmup iterations excluded from histograms (default 5).
    #[arg(long, default_value_t = 5)]
    warmup: usize,
    /// Also bench `create_order` + `cancel_order`. Requires demo/testnet creds
    /// in env. Off by default — guards against accidental real-money fills.
    #[arg(long)]
    with_trading: bool,
    /// Limit price for the trading bench, far from market (e.g. 0.01).
    #[arg(long, default_value_t = 0.01)]
    trade_price: f64,
    /// Order size in contracts for the trading bench.
    #[arg(long, default_value_t = 1.0)]
    trade_size: f64,
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
    // 1µs precision up to ~60s. Sigfig=3 keeps storage bounded; HFT wants µs
    // resolution, not ns, so this is the right scale for REST round-trips.
    Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).expect("histogram bounds")
}

#[derive(Serialize)]
struct OpStats {
    op: &'static str,
    count: u64,
    errors: u64,
    p50_us: u64,
    p99_us: u64,
    p999_us: u64,
    max_us: u64,
    mean_us: f64,
}

impl OpStats {
    fn from_hist(op: &'static str, h: &Histogram<u64>, errors: u64) -> Self {
        Self {
            op,
            count: h.len(),
            errors,
            p50_us: h.value_at_quantile(0.50),
            p99_us: h.value_at_quantile(0.99),
            p999_us: h.value_at_quantile(0.999),
            max_us: h.max(),
            mean_us: h.mean(),
        }
    }
}

async fn time_op<F, Fut, T>(h: &mut Histogram<u64>, errors: &mut u64, mut op: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, OpenPxError>>,
{
    let start = Instant::now();
    match op().await {
        Ok(_) => {
            let elapsed_us = start.elapsed().as_micros() as u64;
            // Record clamped into histogram bounds; better than panic on outliers.
            let _ = h.record(elapsed_us.clamp(1, 60_000_000));
        }
        Err(_) => {
            *errors += 1;
        }
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("rustls crypto provider");
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    let exchange = ExchangeInner::new(&cli.exchange, make_exchange_config(&cli.exchange))
        .unwrap_or_else(|e| {
            eprintln!("error: failed to create {} exchange: {e}", cli.exchange);
            std::process::exit(1);
        });

    // Resolve --ticker via `next_active_market_in_series` so a Kalshi series
    // ticker (e.g. "KXBTC15M") or a Polymarket event slug
    // (e.g. "btc-updown-5m-1777931700") both end up pointing at a concrete
    // currently-open market. Falls through unchanged when the input is
    // already a specific market_ticker / asset_id.
    let (ticker, asset_id) = match exchange.next_active_market_in_series(&cli.ticker).await {
        Ok(Some(market)) => {
            let resolved_asset = match cli.exchange.as_str() {
                "polymarket" => market
                    .outcomes
                    .iter()
                    .find_map(|o| o.token_id.clone())
                    .unwrap_or_else(|| cli.asset_id.clone().unwrap_or_else(|| market.ticker.clone())),
                _ => cli.asset_id.clone().unwrap_or_else(|| market.ticker.clone()),
            };
            (market.ticker, resolved_asset)
        }
        _ => (
            cli.ticker.clone(),
            cli.asset_id.clone().unwrap_or_else(|| cli.ticker.clone()),
        ),
    };

    let mut h_fetch_market = new_histogram();
    let mut h_fetch_orderbook = new_histogram();
    let mut h_fetch_trades = new_histogram();
    let mut h_create_cancel = new_histogram();

    let mut e_fetch_market = 0u64;
    let mut e_fetch_orderbook = 0u64;
    let mut e_fetch_trades = 0u64;
    let mut e_create_cancel = 0u64;

    let total = cli.warmup + cli.iterations;

    eprintln!(
        "rest_bench: exchange={} input={} ticker={} asset_id={} iterations={} warmup={} trading={}",
        cli.exchange, cli.ticker, ticker, asset_id, cli.iterations, cli.warmup, cli.with_trading
    );

    for i in 0..total {
        let measured = i >= cli.warmup;

        // fetch_market(ticker) — unified single-ticker fetch, the closest
        // analog to each exchange's native single-market endpoint.
        let mut throwaway_h = new_histogram();
        let mut throwaway_e = 0u64;
        let (h, e) = if measured {
            (&mut h_fetch_market, &mut e_fetch_market)
        } else {
            (&mut throwaway_h, &mut throwaway_e)
        };
        let params = FetchMarketsParams {
            market_tickers: vec![ticker.clone()],
            status: Some(MarketStatusFilter::All),
            ..Default::default()
        };
        time_op(h, e, || async {
            exchange.fetch_markets(&params).await.map(|_| ())
        })
        .await;

        // fetch_orderbook(asset)
        let mut throwaway_h = new_histogram();
        let mut throwaway_e = 0u64;
        let (h, e) = if measured {
            (&mut h_fetch_orderbook, &mut e_fetch_orderbook)
        } else {
            (&mut throwaway_h, &mut throwaway_e)
        };
        time_op(h, e, || async {
            exchange.fetch_orderbook(&asset_id).await.map(|_| ())
        })
        .await;

        // fetch_trades(asset)
        let mut throwaway_h = new_histogram();
        let mut throwaway_e = 0u64;
        let (h, e) = if measured {
            (&mut h_fetch_trades, &mut e_fetch_trades)
        } else {
            (&mut throwaway_h, &mut throwaway_e)
        };
        let req = TradesRequest {
            asset_id: asset_id.clone(),
            limit: Some(50),
            ..Default::default()
        };
        time_op(h, e, || async {
            exchange.fetch_trades(req.clone()).await.map(|_| ())
        })
        .await;

        // Optional: create + cancel an order on a demo/testnet endpoint.
        if cli.with_trading {
            let mut throwaway_h = new_histogram();
            let mut throwaway_e = 0u64;
            let (h, e) = if measured {
                (&mut h_create_cancel, &mut e_create_cancel)
            } else {
                (&mut throwaway_h, &mut throwaway_e)
            };
            let create_req = CreateOrderRequest {
                asset_id: asset_id.clone(),
                outcome: OrderOutcome::Yes,
                side: OrderSide::Buy,
                price: cli.trade_price,
                size: cli.trade_size,
                order_type: OrderType::Gtc,
            };
            let start = Instant::now();
            let result = async {
                let order = exchange.create_order(create_req.clone()).await?;
                exchange.cancel_order(&order.id).await?;
                Ok::<(), OpenPxError>(())
            }
            .await;
            match result {
                Ok(()) => {
                    let us = start.elapsed().as_micros() as u64;
                    let _ = h.record(us.clamp(1, 60_000_000));
                }
                Err(_) => *e += 1,
            }
        }
    }

    let mut ops = vec![
        OpStats::from_hist("fetch_market", &h_fetch_market, e_fetch_market),
        OpStats::from_hist("fetch_orderbook", &h_fetch_orderbook, e_fetch_orderbook),
        OpStats::from_hist("fetch_trades", &h_fetch_trades, e_fetch_trades),
    ];
    if cli.with_trading {
        ops.push(OpStats::from_hist(
            "create_cancel_order",
            &h_create_cancel,
            e_create_cancel,
        ));
    }

    let report = serde_json::json!({
        "exchange": cli.exchange,
        "input": cli.ticker,
        "ticker": ticker,
        "asset_id": asset_id,
        "iterations": cli.iterations,
        "warmup": cli.warmup,
        "ops": ops,
    });
    println!("{}", serde_json::to_string(&report).expect("serialize report"));
}
