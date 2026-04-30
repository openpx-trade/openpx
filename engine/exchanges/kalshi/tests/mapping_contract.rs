//! Mapping contract test for Kalshi → OpenPX `Market`.
//!
//! Mocks Kalshi's `/markets/<id>` endpoint with the committed fixture, calls
//! `fetch_market`, and hands the result to the shared harness in
//! `px_core::test_support` which walks `schema/mappings/market.yaml` and
//! verifies every `kalshi:` source matches the parsed unified value.

use std::fs;
use std::path::PathBuf;

use px_core::{
    test_support::{assert_mapping_contract, load_mapping},
    Exchange,
};
use px_exchange_kalshi::{Kalshi, KalshiConfig};
use serde_json::Value;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn load_fixture() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/market.json");
    serde_json::from_str(&fs::read_to_string(&p).expect("read fixture")).expect("parse fixture")
}

#[tokio::test]
async fn yaml_contract_for_kalshi_market() {
    let fixture = load_fixture();
    let ticker = fixture
        .get("ticker")
        .and_then(|v| v.as_str())
        .expect("fixture missing ticker")
        .to_string();

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/markets/{ticker}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"market": fixture.clone()})),
        )
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();
    let market = exchange
        .fetch_market(&ticker)
        .await
        .expect("fetch_market should succeed against mock");

    let unified = serde_json::to_value(&market).expect("serialize Market");
    let mapping = load_mapping(env!("CARGO_MANIFEST_DIR"), "market");
    assert_mapping_contract(&fixture, &unified, &mapping, "kalshi");
}
