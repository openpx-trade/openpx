use px_core::{Exchange, FetchMarketsParams};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_events_response() -> serde_json::Value {
    serde_json::json!([
        {
            "id": "event-1",
            "title": "Weather Event",
            "markets": [
                {
                    "id": "123",
                    "question": "Will it rain tomorrow?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.65\", \"0.35\"]",
                    "volumeNum": 50000.0,
                    "liquidityNum": 10000.0,
                    "minimum_tick_size": 0.01,
                    "description": "Weather prediction market"
                }
            ]
        },
        {
            "id": "event-2",
            "title": "Crypto Event",
            "markets": [
                {
                    "id": "456",
                    "question": "Bitcoin > $100k by EOY?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.42\", \"0.58\"]",
                    "volumeNum": 1000000.0,
                    "liquidityNum": 250000.0,
                    "minimum_tick_size": 0.001,
                    "description": "Crypto price prediction"
                }
            ]
        }
    ])
}

fn sample_single_market_response() -> serde_json::Value {
    serde_json::json!({
        "id": "789",
        "question": "Single market test",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.80\", \"0.20\"]",
        "volumeNum": 75000.0,
        "liquidityNum": 15000.0,
        "minimum_tick_size": 0.01,
        "description": "Test market description"
    })
}

#[tokio::test]
async fn test_fetch_markets_parses_response() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_events_response()))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let (markets, cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // then
    assert_eq!(markets.len(), 2);
    assert!(cursor.is_none());

    let first = &markets[0];
    assert_eq!(first.id, "123");
    assert_eq!(first.title, "Will it rain tomorrow?");
    assert_eq!(first.outcomes, vec!["Yes", "No"]);
    assert_eq!(*first.outcome_prices.get("Yes").unwrap(), 0.65);
    assert_eq!(*first.outcome_prices.get("No").unwrap(), 0.35);
    assert_eq!(first.volume, 50000.0);
    assert_eq!(first.liquidity, Some(10000.0));
}

#[tokio::test]
async fn test_fetch_market_by_id() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("id", "789"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([sample_single_market_response()])),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let market = exchange.fetch_market("789").await.unwrap();

    // then
    assert_eq!(market.id, "789");
    assert_eq!(market.title, "Single market test");
    assert_eq!(*market.outcome_prices.get("Yes").unwrap(), 0.80);
    assert_eq!(*market.outcome_prices.get("No").unwrap(), 0.20);
}

#[tokio::test]
async fn test_exchange_info() {
    // given
    let config = PolymarketConfig::new();
    let exchange = Polymarket::new(config).unwrap();

    // when
    let info = exchange.describe();

    // then
    assert_eq!(info.id, "polymarket");
    assert_eq!(info.name, "Polymarket");
    assert!(info.has_fetch_markets);
    assert!(!info.has_create_order);
    assert!(info.has_websocket);
}

#[tokio::test]
async fn test_exchange_id_and_name() {
    // given
    let config = PolymarketConfig::new();
    let exchange = Polymarket::new(config).unwrap();

    // when/then
    assert_eq!(exchange.id(), "polymarket");
    assert_eq!(exchange.name(), "Polymarket");
}
