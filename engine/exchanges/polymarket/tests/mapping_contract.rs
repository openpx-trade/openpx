//! Mapping contract test for Polymarket → OpenPX `Market`.
//!
//! Mocks the gamma `/markets?id=<id>` endpoint with the committed fixture,
//! calls `fetch_market`, and hands the result to the shared harness in
//! `px_core::test_support` which walks `schema/mappings/market.yaml` and
//! verifies every `polymarket:` source matches the parsed unified value.

use std::fs;
use std::path::PathBuf;

use px_core::{
    test_support::{assert_mapping_contract, load_mapping},
    Exchange,
};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};
use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn load_fixture() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/market.json");
    serde_json::from_str(&fs::read_to_string(&p).expect("read fixture")).expect("parse fixture")
}

#[tokio::test]
async fn yaml_contract_for_polymarket_market() {
    let fixture = load_fixture();
    let market_id = fixture
        .get("id")
        .and_then(|v| v.as_str())
        .expect("fixture missing id")
        .to_string();

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("id", market_id.as_str()))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([fixture.clone()])),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();
    let market = exchange
        .fetch_market(&market_id)
        .await
        .expect("fetch_market should succeed against mock");

    let unified = serde_json::to_value(&market).expect("serialize Market");
    let mapping = load_mapping(env!("CARGO_MANIFEST_DIR"), "market");
    assert_mapping_contract(&fixture, &unified, &mapping, "polymarket");
}
