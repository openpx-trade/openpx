use std::env;

use clap::{Parser, Subcommand, ValueEnum};
use futures::StreamExt;
use openpx::{ExchangeInner, WebSocketInner};
use px_core::models::{CryptoPriceSource, PriceHistoryInterval};
use px_core::websocket::OrderBookWebSocket;
use px_core::MarketStatusFilter;
use px_core::{
    FetchMarketsParams, FetchOrdersParams, OrderbookHistoryRequest, OrderbookRequest,
    PriceHistoryRequest, TradesRequest,
};
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
    /// Fetch a page of markets
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
        /// Filter by series (Kalshi series ticker or Polymarket series ID)
        #[arg(long)]
        series_id: Option<String>,
        /// Fetch all markets within an event (Kalshi event ticker or Polymarket event slug/id)
        #[arg(long, alias = "event_id")]
        event_ticker: Option<String>,
    },
    /// Fetch a single market by ID
    FetchMarket { market_id: String },
    /// Fetch L2 orderbook
    FetchOrderbook {
        market_id: String,
        #[arg(long)]
        outcome: Option<String>,
        #[arg(long)]
        token_id: Option<String>,
    },
    /// Fetch OHLCV price history
    FetchPriceHistory {
        market_id: String,
        /// Interval: 1m, 1h, 6h, 1d, 1w, max
        #[arg(value_enum)]
        interval: IntervalArg,
        #[arg(long)]
        outcome: Option<String>,
        #[arg(long)]
        token_id: Option<String>,
        /// Start timestamp (unix seconds)
        #[arg(long)]
        start_ts: Option<i64>,
        /// End timestamp (unix seconds)
        #[arg(long)]
        end_ts: Option<i64>,
    },
    /// Fetch recent trades
    FetchTrades {
        market_id: String,
        #[arg(long)]
        outcome: Option<String>,
        #[arg(long)]
        token_id: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Fetch historical orderbook snapshots
    FetchOrderbookHistory {
        market_id: String,
        #[arg(long)]
        token_id: Option<String>,
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
        market_id: Option<String>,
    },
    /// Fetch open orders
    FetchOpenOrders {
        #[arg(long)]
        market_id: Option<String>,
    },
    /// Fetch a single order by ID
    FetchOrder {
        order_id: String,
        #[arg(long)]
        market_id: Option<String>,
    },
    /// Fetch fill history
    FetchFills {
        #[arg(long)]
        market_id: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },

    // -- WebSocket --
    /// Stream live orderbook updates (Ctrl+C to stop)
    WsOrderbook { market_id: String },
    /// Stream live trade/fill activity (Ctrl+C to stop)
    WsActivity { market_id: String },
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

#[derive(Clone, ValueEnum)]
enum IntervalArg {
    #[value(name = "1m")]
    OneMinute,
    #[value(name = "1h")]
    OneHour,
    #[value(name = "6h")]
    SixHours,
    #[value(name = "1d")]
    OneDay,
    #[value(name = "1w")]
    OneWeek,
    #[value(name = "max")]
    Max,
}

impl From<IntervalArg> for PriceHistoryInterval {
    fn from(i: IntervalArg) -> Self {
        match i {
            IntervalArg::OneMinute => PriceHistoryInterval::OneMinute,
            IntervalArg::OneHour => PriceHistoryInterval::OneHour,
            IntervalArg::SixHours => PriceHistoryInterval::SixHours,
            IntervalArg::OneDay => PriceHistoryInterval::OneDay,
            IntervalArg::OneWeek => PriceHistoryInterval::OneWeek,
            IntervalArg::Max => PriceHistoryInterval::Max,
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
        Command::WsOrderbook { market_id } => {
            ws_orderbook(id, config, &market_id).await;
        }
        Command::WsActivity { market_id } => {
            ws_activity(id, config, &market_id).await;
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
                series_id,
                event_ticker,
            } => {
                let params = FetchMarketsParams {
                    status: status.map(Into::into),
                    cursor,
                    limit,
                    series_id,
                    event_ticker,
                };
                let (markets, next_cursor) = exchange.fetch_markets(&params).await?;
                print_json(&serde_json::json!({
                    "markets": markets,
                    "next_cursor": next_cursor,
                    "count": markets.len(),
                }));
            }
            Command::FetchMarket { market_id } => {
                let market = exchange.fetch_market(&market_id).await?;
                print_json(&market);
            }
            Command::FetchOrderbook {
                market_id,
                outcome,
                token_id,
            } => {
                let req = OrderbookRequest {
                    market_id,
                    outcome,
                    token_id,
                };
                let ob = exchange.fetch_orderbook(req).await?;
                print_json(&ob);
            }
            Command::FetchPriceHistory {
                market_id,
                interval,
                outcome,
                token_id,
                start_ts,
                end_ts,
            } => {
                let req = PriceHistoryRequest {
                    market_id,
                    interval: interval.into(),
                    outcome,
                    token_id,
                    condition_id: None,
                    start_ts,
                    end_ts,
                };
                let candles = exchange.fetch_price_history(req).await?;
                print_json(&candles);
            }
            Command::FetchTrades {
                market_id,
                outcome,
                token_id,
                limit,
                cursor,
            } => {
                let req = TradesRequest {
                    market_id,
                    market_ref: None,
                    outcome,
                    token_id,
                    start_ts: None,
                    end_ts: None,
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
            Command::FetchOrderbookHistory {
                market_id,
                token_id,
                start_ts,
                end_ts,
                limit,
                cursor,
            } => {
                let req = OrderbookHistoryRequest {
                    market_id,
                    token_id,
                    start_ts,
                    end_ts,
                    limit,
                    cursor,
                };
                let (snapshots, next_cursor) = exchange.fetch_orderbook_history(req).await?;
                print_json(&serde_json::json!({
                    "snapshots": snapshots,
                    "next_cursor": next_cursor,
                    "count": snapshots.len(),
                }));
            }
            Command::FetchBalance => {
                let bal = exchange.fetch_balance().await?;
                print_json(&bal);
            }
            Command::FetchPositions { market_id } => {
                let positions = exchange.fetch_positions(market_id.as_deref()).await?;
                print_json(&positions);
            }
            Command::FetchOpenOrders { market_id } => {
                let params = market_id.map(|id| FetchOrdersParams {
                    market_id: Some(id),
                });
                let orders = exchange.fetch_open_orders(params).await?;
                print_json(&orders);
            }
            Command::FetchOrder {
                order_id,
                market_id,
            } => {
                let order = exchange
                    .fetch_order(&order_id, market_id.as_deref())
                    .await?;
                print_json(&order);
            }
            Command::FetchFills { market_id, limit } => {
                let fills = exchange.fetch_fills(market_id.as_deref(), limit).await?;
                print_json(&fills);
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

async fn ws_orderbook(id: &str, config: serde_json::Value, market_id: &str) {
    let mut ws = WebSocketInner::new(id, config).unwrap_or_else(|e| {
        eprintln!("error: failed to create {id} websocket: {e}");
        std::process::exit(1);
    });
    let updates = ws.updates().expect("updates() taken twice");
    let target = market_id.to_string();
    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: websocket connect failed: {e}");
        std::process::exit(1);
    });
    ws.subscribe(market_id).await.unwrap_or_else(|e| {
        eprintln!("error: failed to subscribe to market: {e}");
        std::process::exit(1);
    });
    eprintln!("streaming orderbook for {market_id} (Ctrl+C to stop)...");
    while let Some(update) = updates.next().await {
        // Filter to snapshots/deltas for the requested market_id.
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

async fn ws_activity(id: &str, config: serde_json::Value, market_id: &str) {
    let mut ws = WebSocketInner::new(id, config.clone()).unwrap_or_else(|e| {
        eprintln!("error: failed to create {id} websocket: {e}");
        std::process::exit(1);
    });

    // Auto-register outcome tokens so activity events include "Yes"/"No".
    // Best-effort: if the REST fetch fails, we continue without outcomes.
    if let Ok(exchange) = ExchangeInner::new(id, config) {
        match exchange.fetch_market(market_id).await {
            Ok(market) => {
                let yes = market.token_id_yes();
                let no = market.token_id_no();
                if let (Some(y), Some(n)) = (yes, no) {
                    ws.register_outcomes(y, n).await;
                }
            }
            Err(e) => {
                eprintln!("warning: could not fetch market metadata for outcomes: {e}");
            }
        }
    }

    let updates = ws.updates().expect("updates() taken twice");
    let target = market_id.to_string();
    ws.connect().await.unwrap_or_else(|e| {
        eprintln!("error: websocket connect failed: {e}");
        std::process::exit(1);
    });
    ws.subscribe(market_id).await.unwrap_or_else(|e| {
        eprintln!("error: failed to subscribe to market: {e}");
        std::process::exit(1);
    });
    eprintln!("streaming activity for {market_id} (Ctrl+C to stop)...");
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
