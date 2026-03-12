use px_core::{Exchange, FetchMarketsParams};
use px_exchange_limitless::{Limitless, LimitlessConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unauthenticated: market data only
    // Authenticated: set LIMITLESS_PRIVATE_KEY env var for trading
    let config = if let Ok(pk) = std::env::var("LIMITLESS_PRIVATE_KEY") {
        println!("Authenticated mode");
        LimitlessConfig::new().with_private_key(pk)
    } else {
        println!("Unauthenticated mode (market data only)");
        LimitlessConfig::new()
    };

    let exchange = Limitless::new(config)?;
    println!("Exchange: {} ({})\n", exchange.name(), exchange.id());

    let markets = exchange
        .fetch_markets(Some(FetchMarketsParams {
            limit: Some(5),
            cursor: None,
        }))
        .await?;

    for market in &markets {
        println!("ID: {}", market.id);
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
