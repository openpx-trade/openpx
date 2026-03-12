use px_core::{Exchange, FetchMarketsParams};
use px_exchange_opinion::{Opinion, OpinionConfig};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_markets_response() -> serde_json::Value {
    serde_json::json!({
        "errno": 0,
        "errmsg": null,
        "result": {
            "list": [
                {
                    "market_id": "123",
                    "market_title": "Will BTC reach $100k?",
                    "yes_token_id": "token_yes_123",
                    "no_token_id": "token_no_123",
                    "yes_label": "Yes",
                    "no_label": "No",
                    "volume": 50000.0,
                    "liquidity": 10000.0,
                    "description": "Bitcoin price prediction"
                },
                {
                    "market_id": "456",
                    "market_title": "ETH > $5k?",
                    "yes_token_id": "token_yes_456",
                    "no_token_id": "token_no_456",
                    "yes_label": "Yes",
                    "no_label": "No",
                    "volume": 30000.0,
                    "liquidity": 5000.0,
                    "description": "Ethereum price prediction"
                }
            ]
        }
    })
}

fn sample_single_market_response() -> serde_json::Value {
    serde_json::json!({
        "errno": 0,
        "errmsg": null,
        "result": {
            "data": {
                "market_id": "789",
                "market_title": "Test market question",
                "yes_token_id": "token_yes_789",
                "no_token_id": "token_no_789",
                "yes_label": "Yes",
                "no_label": "No",
                "volume": 25000.0,
                "liquidity": 8000.0,
                "description": "Test market description",
                "cutoff_at": 1735689600
            }
        }
    })
}

fn sample_orderbook_response() -> serde_json::Value {
    serde_json::json!({
        "errno": 0,
        "errmsg": null,
        "result": {
            "data": {
                "bids": [
                    {"price": 0.65, "size": 100.0},
                    {"price": 0.64, "size": 200.0}
                ],
                "asks": [
                    {"price": 0.66, "size": 150.0},
                    {"price": 0.67, "size": 250.0}
                ]
            }
        }
    })
}

#[tokio::test]
async fn test_fetch_markets_parses_response() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_markets_response()))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let markets = exchange.fetch_markets(None).await.unwrap();

    // then
    assert_eq!(markets.len(), 2);

    let first = &markets[0];
    assert_eq!(first.id, "123");
    assert_eq!(first.question, "Will BTC reach $100k?");
    assert_eq!(first.outcomes, vec!["Yes", "No"]);
    assert_eq!(first.volume, 50000.0);
}

#[tokio::test]
async fn test_fetch_market_by_id() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market/789"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_single_market_response()))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let market = exchange.fetch_market("789").await.unwrap();

    // then
    assert_eq!(market.id, "789");
    assert_eq!(market.question, "Test market question");
    assert_eq!(market.volume, 25000.0);
}

#[tokio::test]
async fn test_get_orderbook() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/token/orderbook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_orderbook_response()))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let orderbook = exchange.get_orderbook("token_yes_123").await.unwrap();

    // then
    assert_eq!(orderbook.bids.len(), 2);
    assert_eq!(orderbook.asks.len(), 2);
    assert_eq!(orderbook.bids[0].price, 0.65);
    assert_eq!(orderbook.asks[0].price, 0.66);
}

#[tokio::test]
async fn test_exchange_info() {
    // given
    let config = OpinionConfig::new();
    let exchange = Opinion::new(config).unwrap();

    // when
    let info = exchange.describe();

    // then
    assert_eq!(info.id, "opinion");
    assert_eq!(info.name, "Opinion");
    assert!(info.has_fetch_markets);
    assert!(!info.has_create_order);
    assert!(!info.has_websocket);
}

#[tokio::test]
async fn test_exchange_id_and_name() {
    // given
    let config = OpinionConfig::new();
    let exchange = Opinion::new(config).unwrap();

    // when/then
    assert_eq!(exchange.id(), "opinion");
    assert_eq!(exchange.name(), "Opinion");
}

#[tokio::test]
async fn test_market_tick_size() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market/789"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_single_market_response()))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let market = exchange.fetch_market("789").await.unwrap();

    // then
    assert_eq!(market.tick_size, 0.001);
}

#[tokio::test]
async fn test_parse_token_ids_from_market() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market/789"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_single_market_response()))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let market = exchange.fetch_market("789").await.unwrap();
    let token_ids = market.get_token_ids();

    // then
    assert_eq!(token_ids.len(), 2);
    assert_eq!(token_ids[0], "token_yes_789");
    assert_eq!(token_ids[1], "token_no_789");
}

#[tokio::test]
async fn test_pagination_offset_to_page_conversion() {
    // Opinion uses page-number pagination (1-indexed, limit=20).
    // The default fetch_all_unified_markets() passes offsets as cursor strings.
    // Verify the offset→page conversion: offset 0 → page 1, offset 20 → page 2, etc.
    let mock_server = MockServer::start().await;

    // Page 1 (offset 0 or no cursor): returns 2 markets (less than limit=20, so pagination stops)
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_markets_response()))
        .expect(2) // called twice: once with no cursor, once with cursor="0"
        .mount(&mock_server)
        .await;

    // Page 2 (offset 20): returns empty list
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": { "list": [] }
        })))
        .expect(1) // called once with cursor="20"
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // Test 1: no cursor → page 1
    let markets = exchange.fetch_markets(None).await.unwrap();
    assert_eq!(markets.len(), 2);

    // Test 2: cursor "0" → page 1
    let params = FetchMarketsParams {
        limit: Some(20),
        cursor: Some("0".to_string()),
    };
    let markets = exchange.fetch_markets(Some(params)).await.unwrap();
    assert_eq!(markets.len(), 2);

    // Test 3: cursor "20" → page 2 (verifies (20 / 20) + 1 = 2)
    let params = FetchMarketsParams {
        limit: Some(20),
        cursor: Some("20".to_string()),
    };
    let _markets = exchange.fetch_markets(Some(params)).await.unwrap();
    // page 2 returns empty, so 0 markets
    assert_eq!(_markets.len(), 0);
}
