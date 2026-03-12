use px_core::{Exchange, FetchMarketsParams};
use px_exchange_kalshi::{Kalshi, KalshiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unauthenticated: market data only
    // Authenticated: set KALSHI_API_KEY_ID and KALSHI_PRIVATE_KEY_PATH env vars
    let config = if let (Ok(key_id), Ok(key_path)) = (
        std::env::var("KALSHI_API_KEY_ID"),
        std::env::var("KALSHI_PRIVATE_KEY_PATH"),
    ) {
        println!("Authenticated mode (key: {}...)", &key_id[..8.min(key_id.len())]);
        KalshiConfig::new()
            .with_api_key_id(key_id)
            .with_private_key_path(key_path)
    } else {
        println!("Unauthenticated mode (market data only)");
        KalshiConfig::new()
    };

    let exchange = Kalshi::new(config)?;
    println!("Exchange: {} ({})\n", exchange.name(), exchange.id());

    let markets = exchange
        .fetch_markets(Some(FetchMarketsParams {
            limit: Some(5),
            cursor: None,
        }))
        .await?;

    for market in &markets {
        println!("Ticker: {}", market.id);
        println!("  Question: {}", market.question);
        println!("  Outcomes: {:?}", market.outcomes);
        println!(
            "  Volume: ${:.0} | Liquidity: ${:.0}",
            market.volume, market.liquidity
        );
        println!();
    }

    println!("Fetched {} markets", markets.len());
    Ok(())
}
