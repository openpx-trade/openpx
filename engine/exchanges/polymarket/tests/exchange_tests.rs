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

#[tokio::test]
async fn test_fetch_markets_with_limit() {
    // given
    let mock_server = MockServer::start().await;

    // Return 3 events, each with 1 market
    let events = serde_json::json!([
        {
            "id": "event-1",
            "title": "Event 1",
            "markets": [{
                "id": "m1",
                "question": "Market 1?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.60\", \"0.40\"]",
                "volumeNum": 1000.0,
                "liquidityNum": 500.0,
                "minimum_tick_size": 0.01,
                "description": "First market"
            }]
        },
        {
            "id": "event-2",
            "title": "Event 2",
            "markets": [{
                "id": "m2",
                "question": "Market 2?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.50\", \"0.50\"]",
                "volumeNum": 2000.0,
                "liquidityNum": 1000.0,
                "minimum_tick_size": 0.01,
                "description": "Second market"
            }]
        },
        {
            "id": "event-3",
            "title": "Event 3",
            "markets": [{
                "id": "m3",
                "question": "Market 3?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.70\", \"0.30\"]",
                "volumeNum": 3000.0,
                "liquidityNum": 1500.0,
                "minimum_tick_size": 0.01,
                "description": "Third market"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(events))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when — pass limit=2 in params
    let params = FetchMarketsParams {
        limit: Some(2),
        ..Default::default()
    };
    let (markets, _cursor) = exchange.fetch_markets(&params).await.unwrap();

    // then — Polymarket fetch_markets does not truncate by limit client-side;
    // it always fetches a full page of 200 events. The mock returns 3 events
    // with 1 market each, so all 3 markets are returned.
    assert_eq!(markets.len(), 3);
    assert_eq!(markets[0].id, "m1");
    assert_eq!(markets[1].id, "m2");
    assert_eq!(markets[2].id, "m3");
}

#[tokio::test]
async fn test_fetch_market_not_found() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("id", "nonexistent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let result = exchange.fetch_market("nonexistent").await;

    // then
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{err}");
    assert!(
        err_msg.contains("not found")
            || err_msg.contains("NotFound")
            || err_msg.contains("nonexistent"),
        "expected market-not-found error, got: {err_msg}"
    );
}

#[test]
fn test_config_builder() {
    // given/when
    let config = PolymarketConfig::new()
        .with_gamma_url("http://test.example")
        .with_clob_url("http://clob.example")
        .with_verbose(true);

    // then
    assert_eq!(config.gamma_url, "http://test.example");
    assert_eq!(config.clob_url, "http://clob.example");
    assert!(config.base.verbose);
}

#[test]
fn test_default_config_not_authenticated() {
    // given/when
    let config = PolymarketConfig::new();

    // then
    assert!(config.private_key.is_none());
    assert!(config.api_key.is_none());
    assert!(config.api_secret.is_none());
    assert!(config.api_passphrase.is_none());
    assert!(!config.is_authenticated());
    assert!(!config.has_api_credentials());
}

#[tokio::test]
async fn test_exchange_describe_unauthenticated() {
    // given
    let config = PolymarketConfig::new();
    let exchange = Polymarket::new(config).unwrap();

    // when
    let info = exchange.describe();

    // then — unauthenticated exchange can fetch markets and stream websocket,
    // but cannot create or cancel orders
    assert_eq!(info.id, "polymarket");
    assert!(info.has_fetch_markets);
    assert!(!info.has_create_order);
    assert!(!info.has_cancel_order);
    assert!(info.has_websocket);
    assert!(info.has_fetch_orderbook);
    assert!(info.has_fetch_trades);
}

#[tokio::test]
async fn test_fetch_markets_empty_response() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
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
    assert!(markets.is_empty());
    assert!(cursor.is_none());
}

#[tokio::test]
async fn test_fetch_markets_multiple_outcomes() {
    // given — market with 3 outcomes (categorical)
    let mock_server = MockServer::start().await;
    let events = serde_json::json!([
        {
            "id": "event-multi",
            "title": "Multi-outcome Event",
            "markets": [{
                "id": "multi-1",
                "question": "Who will win the election?",
                "outcomes": "[\"Alice\", \"Bob\", \"Charlie\"]",
                "outcomePrices": "[\"0.45\", \"0.35\", \"0.20\"]",
                "volumeNum": 200000.0,
                "liquidityNum": 50000.0,
                "minimum_tick_size": 0.01,
                "description": "Election market with 3 candidates"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(events))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let (markets, _cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // then
    assert_eq!(markets.len(), 1);
    let market = &markets[0];
    assert_eq!(market.id, "multi-1");
    assert_eq!(market.outcomes, vec!["Alice", "Bob", "Charlie"]);
    assert_eq!(market.outcomes.len(), 3);
    assert_eq!(*market.outcome_prices.get("Alice").unwrap(), 0.45);
    assert_eq!(*market.outcome_prices.get("Bob").unwrap(), 0.35);
    assert_eq!(*market.outcome_prices.get("Charlie").unwrap(), 0.20);
}

#[tokio::test]
async fn test_fetch_markets_with_additional_fields() {
    // given — market with clobTokenIds, conditionId, and slug
    let mock_server = MockServer::start().await;
    let events = serde_json::json!([
        {
            "id": "event-full",
            "title": "Full Fields Event",
            "markets": [{
                "id": "market-1",
                "question": "Test market with tokens",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.75\", \"0.25\"]",
                "volumeNum": 100000.0,
                "liquidityNum": 25000.0,
                "minimum_tick_size": 0.01,
                "description": "Market with full fields",
                "clobTokenIds": "[\"token1\", \"token2\"]",
                "conditionId": "cond-123",
                "slug": "test-market-slug"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(events))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let (markets, _cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // then
    assert_eq!(markets.len(), 1);
    let market = &markets[0];
    assert_eq!(market.id, "market-1");
    assert_eq!(market.slug, Some("test-market-slug".to_string()));
    assert_eq!(market.condition_id, Some("cond-123".to_string()));

    // Verify token IDs are parsed from clobTokenIds
    let token_ids = market.get_token_ids();
    assert_eq!(token_ids, vec!["token1", "token2"]);
    assert_eq!(market.token_id_yes, Some("token1".to_string()));
    assert_eq!(market.token_id_no, Some("token2".to_string()));

    // Verify outcome_tokens are built from outcomes + clobTokenIds
    assert_eq!(market.outcome_tokens.len(), 2);
    assert_eq!(market.outcome_tokens[0].outcome, "Yes");
    assert_eq!(market.outcome_tokens[0].token_id, "token1");
    assert_eq!(market.outcome_tokens[1].outcome, "No");
    assert_eq!(market.outcome_tokens[1].token_id, "token2");
}

#[tokio::test]
async fn test_market_volume_and_liquidity() {
    // given
    let mock_server = MockServer::start().await;
    let events = serde_json::json!([
        {
            "id": "event-vol",
            "title": "Volume Event",
            "markets": [{
                "id": "vol-market",
                "question": "Volume test market?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.55\", \"0.45\"]",
                "volumeNum": 987654.32,
                "liquidityNum": 123456.78,
                "minimum_tick_size": 0.001,
                "description": "Market for volume/liquidity testing"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(events))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let (markets, _cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // then
    assert_eq!(markets.len(), 1);
    let market = &markets[0];
    assert_eq!(market.volume, 987654.32);
    assert_eq!(market.liquidity, Some(123456.78));
    assert_eq!(market.tick_size, Some(0.001));
}

#[tokio::test]
async fn test_fetch_market_single_returns_correct_exchange_field() {
    // given
    let mock_server = MockServer::start().await;

    // Include clobTokenIds so fetch_market skips the fetch_token_ids call
    // (which uses a hardcoded CLOB_URL). Omit conditionId to skip the
    // fetch_open_interest call (also uses a hardcoded URL).
    let market_response = serde_json::json!([{
        "id": "abc-123",
        "question": "Exchange field test",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.90\", \"0.10\"]",
        "volumeNum": 5000.0,
        "liquidityNum": 1000.0,
        "minimum_tick_size": 0.01,
        "description": "Verify exchange and openpx_id fields",
        "clobTokenIds": "[\"tok-yes\", \"tok-no\"]"
    }]);

    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("id", "abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(market_response))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let market = exchange.fetch_market("abc-123").await.unwrap();

    // then
    assert_eq!(market.exchange, "polymarket");
    assert!(
        market.openpx_id.starts_with("polymarket:"),
        "openpx_id should start with 'polymarket:', got: {}",
        market.openpx_id
    );
    assert_eq!(market.openpx_id, "polymarket:abc-123");
    assert_eq!(market.id, "abc-123");
}
