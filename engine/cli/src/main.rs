use std::env;

use clap::{Parser, Subcommand, ValueEnum};
use futures::StreamExt;
use openpx::{ExchangeInner, WebSocketInner};
use px_core::models::CryptoPriceSource;
use px_core::websocket::OrderBookWebSocket;
use px_core::MarketStatusFilter;
use px_core::{FetchMarketsParams, TradesRequest};
use px_crypto::CryptoPriceWebSocket;
use px_sports::SportsWebSocket;

#[derive(Parser)]
#[command(
    name = "openpx",
    about = "OpenPX CLI — test exchange APIs & WebSocket streams"
)]
struct Cli {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    /// Stream live sports scores (Ctrl+C to stop)
    Sports {
        /// Filter by league abbreviation (e.g. nfl, nba, nhl)
        #[arg(long)]
        league: Option<String>,
        /// Only show live games
        #[arg(long)]
        live_only: bool,
    },
    /// Stream live crypto prices (Ctrl+C to stop)
    Crypto {
        /// Price source (binance or chainlink)
        #[arg(long, value_enum, default_value = "binance")]
        source: CryptoSourceArg,
        /// Comma-separated symbols to subscribe to (e.g. btcusdt,ethusdt)
        #[arg(long)]
        symbols: Option<String>,
    },
    /// Kalshi exchange commands
    Kalshi {
        #[command(subcommand)]
        command: Command,
    },
    /// Polymarket exchange commands
    Polymarket {
        #[command(subcommand)]
        command: Command,
    },
}

#[derive(Subcommand)]
enum Command {
    // -- Market Data --
    /// Fetch markets — pass `--market-tickers` for an explicit lookup, otherwise pages the catalog
    FetchMarkets {
        /// Filter by status
        #[arg(long, value_enum)]
        status: Option<StatusArg>,
        /// Pagination cursor from a previous response
        #[arg(long)]
        cursor: Option<String>,
        /// Max markets to return
        #[arg(long)]
        limit: Option<usize>,
        /// Explicit market tickers (Kalshi market ticker or Polymarket market slug). Repeat or comma-separate.
        #[arg(long, value_delimiter = ',', num_args = 0..)]
        market_tickers: Vec<String>,
        /// Filter by series ticker (Kalshi series ticker; Polymarket support arrives with `series_numeric_id`)
        #[arg(long)]
        series_ticker: Option<String>,
        /// Fetch all markets within an event (Kalshi event ticker or Polymarket event slug)
        #[arg(long)]
        event_ticker: Option<String>,
    },
    /// Fetch a market plus its parent event and series in one call
    FetchMarketLineage { market_ticker: String },
    /// Fetch full-depth L2 orderbook (bids + asks) for an asset_id (Kalshi market ticker or Polymarket token id)
    FetchOrderbook { asset_id: String },
    /// Fetch full-depth L2 orderbooks for multiple asset_ids in one round-trip
    FetchOrderbooksBatch {
        /// Repeat or comma-separate (Kalshi cap: 100; Polymarket: no documented cap)
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        asset_ids: Vec<String>,
    },
    /// Top-of-book stats: best bid/ask, mid, spread, weighted-mid, imbalance, total depth
    FetchOrderbookStats { asset_id: String },
    /// Slippage curve at a single requested size — buy and sell sides
    FetchOrderbookImpact {
        asset_id: String,
        /// Size in contracts (must be > 0)
        size: f64,
    },
    /// Microstructure signals: depth tiers, slope, max gap, level counts
    FetchOrderbookMicrostructure { asset_id: String },
    /// Fetch recent trades — `asset_id` is the Kalshi ticker or Polymarket slug
    FetchTrades {
        asset_id: String,
        #[arg(long)]
        start_ts: Option<i64>,
        #[arg(long)]
        end_ts: Option<i64>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        cursor: Option<String>,
    },
    // -- Account (requires auth) --
    /// Fetch account balance
    FetchBalance,
    /// Fetch open positions
    FetchPositions {
        #[arg(long)]
        market_ticker: Option<String>,
    },
    /// Fetch open orders, optionally filtered by `--asset-id`
    FetchOpenOrders {
        #[arg(long)]
        asset_id: Option<String>,
    },
    /// Fetch a single order by ID
    FetchOrder { order_id: String },
    /// Submit a single limit order (price ∈ (0,1), size in contracts)
    CreateOrder {
        asset_id: String,
        /// `yes`, `no`, or a categorical label (Polymarket only)
        #[arg(long)]
        outcome: String,
        /// `buy` or `sell`
        #[arg(long)]
        side: String,
        /// Limit price as YES probability in (0,1)
        #[arg(long)]
        price: f64,
        /// Order size in contracts
        #[arg(long)]
        size: f64,
        /// Time-in-force: `gtc`, `ioc`, or `fok` (default `gtc`)
        #[arg(long, default_value = "gtc")]
        order_type: String,
    },
    /// Cancel one open order by ID
    CancelOrder { order_id: String },
    /// Cancel all open orders, optionally scoped to one `--asset-id`
    CancelAllOrders {
        #[arg(long)]
        asset_id: Option<String>,
    },
    /// Refresh cached balance + on-chain allowance (Polymarket only)
    RefreshBalance,
    /// Fetch fill history
    FetchFills {
        #[arg(long)]
        market_ticker: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Fetch the exchange's current wall-clock time (UTC)
    FetchServerTime,

    // -- WebSocket --
    /// Stream live orderbook updates (Ctrl+C to stop)
    WsOrderbook { market_ticker: String },
    /// Stream live trade/fill activity (Ctrl+C to stop)
    WsActivity { market_ticker: String },
}

#[derive(Clone, ValueEnum)]
enum StatusArg {
    Active,
    Closed,
    Resolved,
    All,
}

#[derive(Clone, ValueEnum)]
enum CryptoSourceArg {
    Binance,
    Chainlink,
}

impl From<CryptoSourceArg> for CryptoPriceSource {
    fn from(s: CryptoSourceArg) -> Self {
        match s {
            CryptoSourceArg::Binance => CryptoPriceSource::Binance,
            CryptoSourceArg::Chainlink => CryptoPriceSource::Chainlink,
        }
    }
}

impl From<StatusArg> for MarketStatusFilter {
    fn from(s: StatusArg) -> Self {
        match s {
            StatusArg::Active => MarketStatusFilter::Active,
            StatusArg::Closed => MarketStatusFilter::Closed,
            StatusArg::Resolved => MarketStatusFilter::Resolved,
            StatusArg::All => MarketStatusFilter::All,
        }
    }
}

/// Build config JSON from env vars for a given exchange.
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

/// Pretty-print a serializable value as JSON.
fn print_json(val: &impl serde::Serialize) {
    println!(
        "{}",
        serde_json::to_string_pretty(val).expect("failed to serialize")
    );
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match cli.command {
        TopCommand::Sports { league, live_only } => {
            ws_sports(league, live_only).await;
        }
        TopCommand::Crypto { source, symbols } => {
            let symbols: Vec<String> = symbols
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            ws_crypto(source.into(), symbols).await;
        }
        TopCommand::Kalshi { command } => {
            run_exchange("kalshi", command).await;
        }
        TopCommand::Polymarket { command } => {
            run_exchange("polymarket", command).await;
        }
    }
}

async fn run_exchange(id: &str, command: Command) {
    let config = make_exchange_config(id);

    match command {
        Command::WsOrderbook { market_ticker } => {
            ws_orderbook(id, config, &market_ticker).await;
        }
        Command::WsActivity { market_ticker } => {
            ws_activity(id, config, &market_ticker).await;
        }
        cmd => {
            let exchange = ExchangeInner::new(id, config).unwrap_or_else(|e| {
                eprintln!("error: failed to create {id} exchange: {e}");
                std::process::exit(1);
            });
            run_rest_command(&exchange, cmd).await;
        }
    }
}

async fn run_rest_command(exchange: &ExchangeInner, cmd: Command) {
    let result: Result<(), px_core::error::OpenPxError> = async {
        match cmd {
            Command::FetchMarkets {
                status,
                cursor,
                limit,
                market_tickers,
                series_ticker,
                event_ticker,
            } => {
                let params = FetchMarketsParams {
                    status: status.map(Into::into),
                    cursor,
                    limit,
                    market_tickers,
                    series_ticker,
                    event_ticker,
                };
                let (markets, next_cursor) = exchange.fetch_markets(&params).await?;
                print_json(&serde_json::json!({
                    "markets": markets,
                    "next_cursor": next_cursor,
                    "count": markets.len(),
                }));
            }
            Command::FetchMarketLineage { market_ticker } => {
                let lineage = exchange.fetch_market_lineage(&market_ticker).await?;
                print_json(&lineage);
            }
            Command::FetchOrderbook { asset_id } => {
                let ob = exchange.fetch_orderbook(&asset_id).await?;
                print_json(&ob);
            }
            Command::FetchOrderbooksBatch { asset_ids } => {
                let books = exchange.fetch_orderbooks_batch(asset_ids).await?;
                print_json(&serde_json::json!({
                    "orderbooks": books,
                    "count": books.len(),
                }));
            }
            Command::FetchOrderbookStats { asset_id } => {
                let stats = exchange.fetch_orderbook_stats(&asset_id).await?;
                print_json(&stats);
            }
            Command::FetchOrderbookImpact { asset_id, size } => {
                let impact = exchange.fetch_orderbook_impact(&asset_id, size).await?;
                print_json(&impact);
            }
            Command::FetchOrderbookMicrostructure { asset_id } => {
                let micro = exchange.fetch_orderbook_microstructure(&asset_id).await?;
                print_json(&micro);
            }
            Command::FetchTrades {
                asset_id,
                start_ts,
                end_ts,
                limit,
                cursor,
            } => {
                let req = TradesRequest {
                    asset_id,
                    start_ts,
                    end_ts,
                    limit,
                    cursor,
                };
                let (trades, next_cursor) = exchange.fetch_trades(req).await?;
                print_json(&serde_json::json!({
                    "trades": trades,
                    "next_cursor": next_cursor,
                    "count": trades.len(),
                }));
            }
            Command::FetchBalance => {
                let bal = exchange.fetch_balance().await?;
                print_json(&bal);
            }
            Command::FetchPositions { market_ticker } => {
                let positions = exchange.fetch_positions(market_ticker.as_deref()).await?;
                print_json(&positions);
            }
            Command::FetchOpenOrders { asset_id } => {
                let orders = exchange.fetch_open_orders(asset_id.as_deref()).await?;
                print_json(&orders);
            }
            Command::FetchOrder { order_id } => {
                let order = exchange.fetch_order(&order_id).await?;
                print_json(&order);
            }
            Command::CreateOrder {
                asset_id,
                outcome,
                side,
                price,
                size,
                order_type,
            } => {
                let order_outcome = match outcome.to_ascii_lowercase().as_str() {
                    "yes" => px_core::OrderOutcome::Yes,
                    "no" => px_core::OrderOutcome::No,
                    other => px_core::OrderOutcome::Label(other.into()),
                };
                let order_side = match side.to_ascii_lowercase().as_str() {
                    "buy" => px_core::OrderSide::Buy,
                    "sell" => px_core::OrderSide::Sell,
                    other => {
                        return Err(px_core::error::OpenPxError::InvalidInput(format!(
                            "side must be 'buy' or 'sell', got '{other}'"
                        )));
                    }
                };
                let order_type_enum = order_type
                    .parse::<px_core::OrderType>()
                    .map_err(px_core::error::OpenPxError::InvalidInput)?;
                let req = px_core::CreateOrderRequest {
                    asset_id,
                    outcome: order_outcome,
                    side: order_side,
                    price,
                    size,
                    order_type: order_type_enum,
                };
                let order = exchange.create_order(req).await?;
                print_json(&order);
            }
            Command::CancelOrder { order_id } => {
                let order = exchange.cancel_order(&order_id).await?;
                print_json(&order);
            }
            Command::CancelAllOrders { asset_id } => {
                let orders = exchange.cancel_all_orders(asset_id.as_deref()).await?;
                print_json(&serde_json::json!({
                    "cancelled": orders,
                    "count": orders.len(),
                }));
            }
            Command::RefreshBalance => {
                exchange.refresh_balance().await?;
                print_json(&serde_json::json!({ "ok": true }));
            }
            Command::FetchFills {
                market_ticker,
                limit,
            } => {
                let fills = exchange
                    .fetch_fills(market_ticker.as_deref(), limit)
                    .await?;
                print_json(&fills);
            }
            Command::FetchServerTime => {
                let ts = exchange.fetch_server_time().await?;
                print_json(&serde_json::json!({
                    "iso": ts.to_rfc3339(),
                    "unix_seconds": ts.timestamp(),
                }));
            }
            // WebSocket commands handled in run_exchange
            Command::WsOrderbook { .. } | Command::WsActivity { .. } => unreachable!(),
        }
        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn ws_sports(league: Option<String>, live_only: bool) {
    let mut ws = SportsWebSocket::new();
    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: sports websocket connect failed: {e}");
        std::process::exit(1);
    });

    let mut stream = ws.stream();
    let league_lower = league.map(|l| l.to_lowercase());

    eprintln!("streaming sports scores (Ctrl+C to stop)...");
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                if live_only && !data.live {
                    continue;
                }
                if let Some(ref league) = league_lower {
                    if data.league_abbreviation.to_lowercase() != *league {
                        continue;
                    }
                }
                print_json(&data);
            }
            Err(e) => eprintln!("error: {e}"),
        }
    }
}

async fn ws_crypto(source: CryptoPriceSource, symbols: Vec<String>) {
    let mut ws = CryptoPriceWebSocket::new();
    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: crypto websocket connect failed: {e}");
        std::process::exit(1);
    });

    ws.subscribe(source, &symbols).await.unwrap_or_else(|e| {
        eprintln!("error: crypto subscribe failed: {e}");
        std::process::exit(1);
    });

    let mut stream = ws.stream();
    eprintln!("streaming crypto prices (Ctrl+C to stop)...");
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => print_json(&data),
            Err(e) => eprintln!("error: {e}"),
        }
    }
}

async fn ws_orderbook(id: &str, config: serde_json::Value, market_ticker: &str) {
    let mut ws = WebSocketInner::new(id, config).unwrap_or_else(|e| {
        eprintln!("error: failed to create {id} websocket: {e}");
        std::process::exit(1);
    });
    let updates = ws.updates().expect("updates() taken twice");
    let target = market_ticker.to_string();
    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: websocket connect failed: {e}");
        std::process::exit(1);
    });
    ws.subscribe(market_ticker).await.unwrap_or_else(|e| {
        eprintln!("error: failed to subscribe to market: {e}");
        std::process::exit(1);
    });
    eprintln!("streaming orderbook for {market_ticker} (Ctrl+C to stop)...");
    while let Some(update) = updates.next().await {
        // Filter to snapshots/deltas for the requested market_ticker.
        match &update {
            px_core::WsUpdate::Snapshot { market_id: m, .. }
            | px_core::WsUpdate::Delta { market_id: m, .. }
                if m == &target =>
            {
                print_json(&update);
            }
            _ => {}
        }
    }
}

async fn ws_activity(id: &str, config: serde_json::Value, market_ticker: &str) {
    let mut ws = WebSocketInner::new(id, config.clone()).unwrap_or_else(|e| {
        eprintln!("error: failed to create {id} websocket: {e}");
        std::process::exit(1);
    });

    // Auto-register outcome tokens so activity events include "Yes"/"No".
    // Best-effort: if the REST fetch fails, we continue without outcomes.
    if let Ok(exchange) = ExchangeInner::new(id, config) {
        let params = FetchMarketsParams {
            market_tickers: vec![market_ticker.to_string()],
            status: Some(px_core::MarketStatusFilter::All),
            ..Default::default()
        };
        match exchange.fetch_markets(&params).await {
            Ok((mut markets, _)) if !markets.is_empty() => {
                let market = markets.remove(0);
                let yes = market.token_id_yes();
                let no = market.token_id_no();
                if let (Some(y), Some(n)) = (yes, no) {
                    ws.register_outcomes(y, n).await;
                }
            }
            Ok(_) => {
                eprintln!("warning: market not found for outcomes: {market_ticker}");
            }
            Err(e) => {
                eprintln!("warning: could not fetch market metadata for outcomes: {e}");
            }
        }
    }

    let updates = ws.updates().expect("updates() taken twice");
    let target = market_ticker.to_string();
    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: websocket connect failed: {e}");
        std::process::exit(1);
    });
    ws.subscribe(market_ticker).await.unwrap_or_else(|e| {
        eprintln!("error: failed to subscribe to market: {e}");
        std::process::exit(1);
    });
    eprintln!("streaming activity for {market_ticker} (Ctrl+C to stop)...");
    while let Some(update) = updates.next().await {
        match &update {
            px_core::WsUpdate::Trade { trade, .. } if trade.market_id == target => {
                print_json(&update);
            }
            px_core::WsUpdate::Fill { fill, .. } if fill.market_id == target => {
                print_json(&update);
            }
            _ => {}
        }
    }
}
