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
        schema_for!(px_core::Outcome),
        schema_for!(px_core::MarketStatus),
        schema_for!(px_core::MarketType),
        // Models — order
        schema_for!(px_core::Order),
        schema_for!(px_core::OrderType),
        schema_for!(px_core::OrderSide),
        schema_for!(px_core::OrderStatus),
        schema_for!(px_core::LiquidityRole),
        schema_for!(px_core::Fill),
        // Models — position
        schema_for!(px_core::Position),
        // Models — orderbook
        schema_for!(px_core::Orderbook),
        schema_for!(px_core::PriceLevel),
        schema_for!(px_core::PriceLevelChange),
        schema_for!(px_core::PriceLevelSide),
        schema_for!(px_core::OrderbookStats),
        schema_for!(px_core::OrderbookImpact),
        schema_for!(px_core::OrderbookMicrostructure),
        schema_for!(px_core::DepthBuckets),
        schema_for!(px_core::MaxGap),
        schema_for!(px_core::LevelCount),
        // Models — trade
        schema_for!(px_core::MarketTrade),
        // Models — events / series
        schema_for!(px_core::Event),
        schema_for!(px_core::Series),
        schema_for!(px_core::SettlementSource),
        // Exchange config params
        schema_for!(px_core::MarketStatusFilter),
        schema_for!(px_core::FetchMarketsParams),
        // Exchange trait request types
        schema_for!(px_core::TradesRequest),
        schema_for!(px_core::MarketLineage),
        schema_for!(px_core::CreateOrderRequest),
        schema_for!(px_core::OrderOutcome),
        schema_for!(px_core::ExchangeInfo),
        // WebSocket types
        schema_for!(px_core::WsUpdate),
        schema_for!(px_core::SessionEvent),
        schema_for!(px_core::ActivityTrade),
        schema_for!(px_core::ActivityFill),
        // Crypto + sports streams
        schema_for!(px_core::CryptoPrice),
        schema_for!(px_core::CryptoPriceSource),
        schema_for!(px_core::SportResult),
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
