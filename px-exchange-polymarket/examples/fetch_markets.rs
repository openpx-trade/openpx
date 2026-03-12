use px_core::{Exchange, FetchMarketsParams};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unauthenticated: market data only (Gamma API is public)
    // Authenticated: set POLYMARKET_PRIVATE_KEY env var for trading
    let config = if let Ok(pk) = std::env::var("POLYMARKET_PRIVATE_KEY") {
        println!("Authenticated mode");
        PolymarketConfig::new().with_private_key(pk)
    } else {
        println!("Unauthenticated mode (market data only)");
        PolymarketConfig::new()
    };

    let exchange = Polymarket::new(config)?;
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
        if let Some(spread) = market.spread() {
            println!("  Spread: {:.4}", spread);
        }
        println!();
    }

    println!("Fetched {} markets", markets.len());
    Ok(())
}
