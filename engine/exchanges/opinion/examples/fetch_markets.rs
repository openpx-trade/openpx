use px_core::{Exchange, FetchMarketsParams};
use px_exchange_opinion::{Opinion, OpinionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Authenticated: set OPINION_API_KEY env var
    let config = if let Ok(api_key) = std::env::var("OPINION_API_KEY") {
        println!("Authenticated mode");
        OpinionConfig::new().with_api_key(api_key)
    } else {
        println!("Unauthenticated mode (market data only)");
        OpinionConfig::new()
    };

    let exchange = Opinion::new(config)?;
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
