use std::env;
use std::sync::Arc;
use std::time::Duration;

use arrow::array::{Float64Array, StringBuilder, TimestampMicrosecondArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use chrono::Utc;
use clap::Parser;
use futures::StreamExt;
use openpx::{ExchangeInner, WebSocketInner};
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;
use px_core::models::{MarketStatus, OrderbookUpdate};
use px_core::websocket::{OrderBookWebSocket, OrderbookStream};
use px_core::FetchMarketsParams;
use tokio::sync::Mutex;
use tokio::time::sleep;

const RECORD_DURATION: Duration = Duration::from_secs(60);
const MARKET_FETCH_LIMIT: usize = 50;

#[derive(Parser)]
#[command(
    name = "px-recorder",
    about = "Record WebSocket orderbook data to Parquet (zstd)"
)]
struct Args {
    /// Exchange to record: kalshi, polymarket, or opinion
    #[arg(short, long)]
    exchange: String,
}

/// A single flattened orderbook row for Parquet output.
#[derive(Debug)]
struct ObRow {
    timestamp_us: i64,
    exchange: String,
    market_id: String,
    asset_id: String,
    update_type: String, // "snapshot" or "delta"
    side: String,        // "bid" or "ask"
    price: f64,
    size: f64,
    last_update_id: Option<u64>, // exchange sequence number
    sequence: u64,
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
            ("POLYMARKET_API_KEY", "api_key"),
            ("POLYMARKET_API_SECRET", "api_secret"),
            ("POLYMARKET_API_PASSPHRASE", "api_passphrase"),
        ],
        "opinion" => &[
            ("OPINION_API_KEY", "api_key"),
            ("OPINION_PRIVATE_KEY", "private_key"),
            ("OPINION_MULTI_SIG_ADDR", "multi_sig_addr"),
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

fn parquet_schema() -> Schema {
    Schema::new(vec![
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            false,
        ),
        Field::new("exchange", DataType::Utf8, false),
        Field::new("market_id", DataType::Utf8, false),
        Field::new("asset_id", DataType::Utf8, false),
        Field::new("update_type", DataType::Utf8, false),
        Field::new("side", DataType::Utf8, false),
        Field::new("price", DataType::Float64, false),
        Field::new("size", DataType::Float64, false),
        Field::new("last_update_id", DataType::UInt64, true),
        Field::new("sequence", DataType::UInt64, false),
    ])
}

fn rows_to_batch(rows: &[ObRow], schema: &Arc<Schema>) -> RecordBatch {
    let timestamps: Vec<i64> = rows.iter().map(|r| r.timestamp_us).collect();
    let exchanges: Vec<&str> = rows.iter().map(|r| r.exchange.as_str()).collect();
    let market_ids: Vec<&str> = rows.iter().map(|r| r.market_id.as_str()).collect();
    let asset_ids: Vec<&str> = rows.iter().map(|r| r.asset_id.as_str()).collect();
    let update_types: Vec<&str> = rows.iter().map(|r| r.update_type.as_str()).collect();
    let sides: Vec<&str> = rows.iter().map(|r| r.side.as_str()).collect();
    let prices: Vec<f64> = rows.iter().map(|r| r.price).collect();
    let sizes: Vec<f64> = rows.iter().map(|r| r.size).collect();
    let last_update_ids: Vec<Option<u64>> = rows.iter().map(|r| r.last_update_id).collect();
    let sequences: Vec<u64> = rows.iter().map(|r| r.sequence).collect();

    let mut exchange_builder = StringBuilder::new();
    let mut market_id_builder = StringBuilder::new();
    let mut asset_id_builder = StringBuilder::new();
    let mut update_type_builder = StringBuilder::new();
    let mut side_builder = StringBuilder::new();

    for s in &exchanges {
        exchange_builder.append_value(s);
    }
    for s in &market_ids {
        market_id_builder.append_value(s);
    }
    for s in &asset_ids {
        asset_id_builder.append_value(s);
    }
    for s in &update_types {
        update_type_builder.append_value(s);
    }
    for s in &sides {
        side_builder.append_value(s);
    }

    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(TimestampMicrosecondArray::from(timestamps).with_timezone("UTC")),
            Arc::new(exchange_builder.finish()),
            Arc::new(market_id_builder.finish()),
            Arc::new(asset_id_builder.finish()),
            Arc::new(update_type_builder.finish()),
            Arc::new(side_builder.finish()),
            Arc::new(Float64Array::from(prices)),
            Arc::new(Float64Array::from(sizes)),
            Arc::new(UInt64Array::from(last_update_ids)),
            Arc::new(UInt64Array::from(sequences)),
        ],
    )
    .expect("failed to build record batch")
}

/// Fetch all active market IDs for an exchange, paginating until exhausted.
async fn fetch_active_markets(exchange: &ExchangeInner) -> Vec<(String, Vec<String>)> {
    let mut all = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let params = FetchMarketsParams {
            status: Some(MarketStatus::Active),
            cursor: cursor.clone(),
            limit: Some(MARKET_FETCH_LIMIT),
        };
        match exchange.fetch_markets(&params).await {
            Ok((markets, next_cursor)) => {
                for m in &markets {
                    // Collect token IDs for markets that need them (polymarket)
                    let token_ids: Vec<String> = m
                        .outcome_tokens
                        .iter()
                        .map(|t| t.token_id.clone())
                        .collect();
                    all.push((m.id.clone(), token_ids));
                }
                eprintln!("  fetched {} markets (total: {})", markets.len(), all.len());
                if next_cursor.is_none() || markets.is_empty() {
                    break;
                }
                cursor = next_cursor;
            }
            Err(e) => {
                eprintln!("  error fetching markets: {e}");
                break;
            }
        }
    }
    all
}

/// Read orderbook updates from a pre-created stream for a single market.
async fn record_from_stream(
    exchange_id: &str,
    market_id: &str,
    mut stream: OrderbookStream,
    rows: Arc<Mutex<Vec<ObRow>>>,
    seq: Arc<std::sync::atomic::AtomicU64>,
) {
    // Track asset_id from the most recent snapshot so deltas can carry it
    let mut last_asset_id = String::new();

    while let Some(result) = stream.next().await {
        let now = Utc::now().timestamp_micros();
        let sequence = seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        match result {
            Ok(OrderbookUpdate::Snapshot(ob)) => {
                last_asset_id = ob.asset_id.clone();
                let ts = ob.timestamp.map(|t| t.timestamp_micros()).unwrap_or(now);

                let mut buf = rows.lock().await;
                for level in &ob.bids {
                    buf.push(ObRow {
                        timestamp_us: ts,
                        exchange: exchange_id.to_string(),
                        market_id: ob.market_id.clone(),
                        asset_id: ob.asset_id.clone(),
                        update_type: "snapshot".into(),
                        side: "bid".into(),
                        price: level.price.to_f64(),
                        size: level.size,
                        last_update_id: ob.last_update_id,
                        sequence,
                    });
                }
                for level in &ob.asks {
                    buf.push(ObRow {
                        timestamp_us: ts,
                        exchange: exchange_id.to_string(),
                        market_id: ob.market_id.clone(),
                        asset_id: ob.asset_id.clone(),
                        update_type: "snapshot".into(),
                        side: "ask".into(),
                        price: level.price.to_f64(),
                        size: level.size,
                        last_update_id: ob.last_update_id,
                        sequence,
                    });
                }
            }
            Ok(OrderbookUpdate::Delta { changes, timestamp }) => {
                let ts = timestamp.map(|t| t.timestamp_micros()).unwrap_or(now);
                let mut buf = rows.lock().await;
                for change in changes.iter() {
                    buf.push(ObRow {
                        timestamp_us: ts,
                        exchange: exchange_id.to_string(),
                        market_id: market_id.to_string(),
                        asset_id: last_asset_id.clone(),
                        update_type: "delta".into(),
                        side: match change.side {
                            px_core::models::PriceLevelSide::Bid => "bid".into(),
                            px_core::models::PriceLevelSide::Ask => "ask".into(),
                        },
                        price: change.price.to_f64(),
                        size: change.size,
                        last_update_id: None,
                        sequence,
                    });
                }
            }
            Err(e) => {
                eprintln!("[{exchange_id}] ws error on {market_id}: {e}");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
    let _ = dotenvy::dotenv();
    let args = Args::parse();

    let exchange_id = args.exchange.to_lowercase();
    if !matches!(exchange_id.as_str(), "kalshi" | "polymarket" | "opinion") {
        eprintln!(
            "error: unknown exchange '{exchange_id}' — must be kalshi, polymarket, or opinion"
        );
        std::process::exit(1);
    }

    let config = make_exchange_config(&exchange_id);
    let exchange = ExchangeInner::new(&exchange_id, config).unwrap_or_else(|e| {
        eprintln!("[{exchange_id}] failed to create exchange: {e}");
        std::process::exit(1);
    });

    eprintln!("[{exchange_id}] fetching active markets...");
    let markets = fetch_active_markets(&exchange).await;
    eprintln!("[{exchange_id}] {} active markets found", markets.len());

    if markets.is_empty() {
        eprintln!("no active markets found — nothing to record.");
        return;
    }

    eprintln!(
        "\nrecording orderbooks for {} markets on {exchange_id} for 60s...\n",
        markets.len()
    );

    let rows: Arc<Mutex<Vec<ObRow>>> = Arc::new(Mutex::new(Vec::new()));
    let seq = Arc::new(std::sync::atomic::AtomicU64::new(0));

    // Create a single shared WebSocket connection for all markets
    let ws_config = make_exchange_config(&exchange_id);
    let mut ws = WebSocketInner::new(&exchange_id, ws_config).unwrap_or_else(|e| {
        eprintln!("[{exchange_id}] failed to create websocket: {e}");
        std::process::exit(1);
    });

    if let Err(e) = ws.connect().await {
        eprintln!("[{exchange_id}] failed to connect websocket: {e}");
        std::process::exit(1);
    }
    eprintln!(
        "[{exchange_id}] websocket connected, subscribing to {} markets...",
        markets.len()
    );

    // Sequentially create streams and subscribe (requires &mut self)
    let mut market_streams = Vec::new();
    for (market_id, _token_ids) in &markets {
        match ws.orderbook_stream(market_id).await {
            Ok(stream) => {
                if let Err(e) = ws.subscribe(market_id).await {
                    eprintln!("[{exchange_id}] subscribe error for {market_id}: {e}");
                    continue;
                }
                market_streams.push((market_id.clone(), stream));
            }
            Err(e) => {
                eprintln!("[{exchange_id}] stream error for {market_id}: {e}");
            }
        }
    }
    eprintln!(
        "[{exchange_id}] subscribed to {} markets",
        market_streams.len()
    );

    // Spawn one read task per market, each owning its stream
    let mut handles = Vec::new();
    for (market_id, stream) in market_streams {
        let rows = rows.clone();
        let seq = seq.clone();
        let eid = exchange_id.clone();

        handles.push(tokio::spawn(async move {
            record_from_stream(&eid, &market_id, stream, rows, seq).await;
        }));
    }

    // Keep ws alive during recording, then clean up
    sleep(RECORD_DURATION + Duration::from_secs(2)).await;
    ws.disconnect().await.ok();

    for h in &handles {
        h.abort();
    }
    for h in handles {
        let _ = h.await;
    }

    // Write parquet
    let rows = Arc::try_unwrap(rows).unwrap().into_inner();
    let row_count = rows.len();

    if row_count == 0 {
        eprintln!("no data recorded — check your exchange credentials and network.");
        return;
    }

    let schema = Arc::new(parquet_schema());
    let batch = rows_to_batch(&rows, &schema);

    let filename = format!(
        "orderbook_{exchange_id}_{}.parquet.zst",
        Utc::now().format("%Y%m%d_%H%M%S")
    );
    let file = std::fs::File::create(&filename).expect("failed to create output file");

    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).unwrap()))
        .build();

    let mut writer =
        ArrowWriter::try_new(file, schema, Some(props)).expect("failed to create parquet writer");
    writer.write(&batch).expect("failed to write batch");
    writer.close().expect("failed to close writer");

    eprintln!("\nwrote {row_count} rows to {filename}");
}
