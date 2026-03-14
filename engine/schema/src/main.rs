#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use schemars::schema::RootSchema;
use schemars::schema_for;
use std::collections::BTreeMap;

fn main() {
    let mut definitions = BTreeMap::new();

    // Collect schemas from all exported types.
    // Each schema_for!() call returns a RootSchema whose `definitions` map
    // contains the type and any referenced types.
    let schemas: Vec<RootSchema> = vec![
        // Models — market
        schema_for!(px_core::Market),
        schema_for!(px_core::UnifiedMarket),
        schema_for!(px_core::OutcomeToken),
        schema_for!(px_core::MarketStatus),
        // Models — order
        schema_for!(px_core::Order),
        schema_for!(px_core::OrderType),
        schema_for!(px_core::OrderSide),
        schema_for!(px_core::OrderStatus),
        schema_for!(px_core::LiquidityRole),
        schema_for!(px_core::Fill),
        // Models — position
        schema_for!(px_core::Position),
        schema_for!(px_core::Nav),
        schema_for!(px_core::PositionBreakdown),
        schema_for!(px_core::DeltaInfo),
        // Models — orderbook
        schema_for!(px_core::Orderbook),
        schema_for!(px_core::OrderbookSnapshot),
        schema_for!(px_core::PriceLevel),
        schema_for!(px_core::PriceLevelChange),
        schema_for!(px_core::PriceLevelSide),
        // Models — trade
        schema_for!(px_core::MarketTrade),
        schema_for!(px_core::Candlestick),
        schema_for!(px_core::PriceHistoryInterval),
        // Exchange config params
        schema_for!(px_core::FetchMarketsParams),
        schema_for!(px_core::FetchOrdersParams),
        schema_for!(px_core::FetchUserActivityParams),
        // Exchange trait request types
        schema_for!(px_core::OrderbookRequest),
        schema_for!(px_core::PriceHistoryRequest),
        schema_for!(px_core::TradesRequest),
        schema_for!(px_core::OrderbookHistoryRequest),
        schema_for!(px_core::ExchangeInfo),
        // WebSocket activity types
        schema_for!(px_core::ActivityEvent),
        schema_for!(px_core::ActivityTrade),
        schema_for!(px_core::ActivityFill),
    ];

    for root in schemas {
        // Insert the root type's schema under its title
        if let Some(title) = root.schema.metadata.as_ref().and_then(|m| m.title.as_ref()) {
            definitions.insert(
                title.clone(),
                serde_json::to_value(&root.schema).expect("serialize root schema"),
            );
        }

        // Insert all referenced definitions
        for (name, schema) in root.definitions {
            definitions.insert(
                name,
                serde_json::to_value(&schema).expect("serialize definition"),
            );
        }
    }

    let combined = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "OpenPX",
        "description": "Auto-generated JSON Schema for OpenPX core types",
        "definitions": definitions,
    });

    let output = serde_json::to_string_pretty(&combined).expect("serialize combined schema");
    println!("{output}");
}
