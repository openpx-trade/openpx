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
        schema_for!(px_core::UserTrade),
        // Models — position
        schema_for!(px_core::Position),
        // Models — orderbook
        schema_for!(px_core::Orderbook),
        schema_for!(px_core::OrderbookSnapshot),
        schema_for!(px_core::Spread),
        schema_for!(px_core::PriceLevel),
        schema_for!(px_core::PriceLevelChange),
        schema_for!(px_core::PriceLevelSide),
        // Models — trade
        schema_for!(px_core::MarketTrade),
        schema_for!(px_core::LastTrade),
        schema_for!(px_core::Candlestick),
        schema_for!(px_core::PriceHistoryInterval),
        // Models — events / series / tags
        schema_for!(px_core::Event),
        schema_for!(px_core::Series),
        schema_for!(px_core::SettlementSource),
        schema_for!(px_core::Tag),
        // Exchange config params
        schema_for!(px_core::MarketStatusFilter),
        schema_for!(px_core::FetchMarketsParams),
        schema_for!(px_core::FetchOrdersParams),
        schema_for!(px_core::FetchUserActivityParams),
        // Exchange trait request types
        schema_for!(px_core::OrderbookRequest),
        schema_for!(px_core::PriceHistoryRequest),
        schema_for!(px_core::TradesRequest),
        schema_for!(px_core::OrderbookHistoryRequest),
        schema_for!(px_core::EventsRequest),
        schema_for!(px_core::SeriesRequest),
        schema_for!(px_core::MidpointRequest),
        schema_for!(px_core::UserTradesRequest),
        schema_for!(px_core::NewOrder),
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
