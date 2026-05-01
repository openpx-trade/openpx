use px_core::{Exchange, FetchMarketsParams, MarketStatus, MarketStatusFilter};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Wrap a list of events in the keyset envelope shape.
fn keyset_envelope(events: serde_json::Value) -> serde_json::Value {
    json!({
        "events": events,
        "next_cursor": serde_json::Value::Null,
    })
}

fn sample_events_response() -> serde_json::Value {
    json!([
        {
            "id": "event-1",
            "title": "Weather Event",
            "markets": [
                {
                    "id": "123",
                    "conditionId": "cond-123",
                    "slug": "weather-rain-tomorrow",
                    "question": "Will it rain tomorrow?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.65\", \"0.35\"]",
                    "volumeNum": 50000.0,
                    "liquidityNum": 10000.0,
                    "orderPriceMinTickSize": 0.01,
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
                    "conditionId": "cond-456",
                    "slug": "btc-100k-eoy",
                    "question": "Bitcoin > $100k by EOY?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.42\", \"0.58\"]",
                    "volumeNum": 1000000.0,
                    "liquidityNum": 250000.0,
                    "orderPriceMinTickSize": 0.001,
                    "description": "Crypto price prediction"
                }
            ]
        }
    ])
}

fn sample_single_market_response() -> serde_json::Value {
    json!({
        "id": "789",
        "conditionId": "cond-789",
        "slug": "single-market-test",
        "question": "Single market test",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.80\", \"0.20\"]",
        "volumeNum": 75000.0,
        "liquidityNum": 15000.0,
        "orderPriceMinTickSize": 0.01,
        "description": "Test market description"
    })
}

#[tokio::test]
async fn test_fetch_markets_parses_response() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(keyset_envelope(sample_events_response())),
        )
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
    assert_eq!(first.ticker, "weather-rain-tomorrow");
    assert_eq!(first.condition_id.as_deref(), Some("cond-123"));
    assert_eq!(first.title, "Will it rain tomorrow?");
    assert_eq!(
        first
            .outcomes
            .iter()
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Yes", "No"]
    );
    assert_eq!(first.outcomes[0].price, Some(0.65));
    assert_eq!(first.outcomes[1].price, Some(0.35));
    assert_eq!(first.volume, 50000.0);
}

#[tokio::test]
async fn test_fetch_market_by_id() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("id", "789"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([sample_single_market_response()])),
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
    assert_eq!(market.ticker, "single-market-test");
    assert_eq!(market.title, "Single market test");
    assert_eq!(market.outcomes[0].price, Some(0.80));
    assert_eq!(market.outcomes[1].price, Some(0.20));
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

    let events = json!([
        {
            "id": "event-1",
            "title": "Event 1",
            "markets": [{
                "id": "m1",
                "conditionId": "cond-m1",
                "slug": "m1",
                "question": "Market 1?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.60\", \"0.40\"]",
                "volumeNum": 1000.0,
                "liquidityNum": 500.0,
                "orderPriceMinTickSize": 0.01,
                "description": "First market"
            }]
        },
        {
            "id": "event-2",
            "title": "Event 2",
            "markets": [{
                "id": "m2",
                "conditionId": "cond-m2",
                "slug": "m2",
                "question": "Market 2?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.50\", \"0.50\"]",
                "volumeNum": 2000.0,
                "liquidityNum": 1000.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Second market"
            }]
        },
        {
            "id": "event-3",
            "title": "Event 3",
            "markets": [{
                "id": "m3",
                "conditionId": "cond-m3",
                "slug": "m3",
                "question": "Market 3?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.70\", \"0.30\"]",
                "volumeNum": 3000.0,
                "liquidityNum": 1500.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Third market"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        limit: Some(2),
        ..Default::default()
    };
    let (markets, _cursor) = exchange.fetch_markets(&params).await.unwrap();

    // then — Polymarket fetch_markets does not truncate by limit client-side;
    // it returns whatever a single keyset page produces.
    assert_eq!(markets.len(), 3);
    assert_eq!(markets[0].ticker, "m1");
    assert_eq!(markets[1].ticker, "m2");
    assert_eq!(markets[2].ticker, "m3");
}

#[tokio::test]
async fn test_fetch_market_not_found() {
    // given — slug-shaped lookup goes to /markets/slug/{slug}; 404 maps to MarketNotFound
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/slug/nonexistent"))
        .respond_with(ResponseTemplate::new(404))
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
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(json!([]))))
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
    let events = json!([
        {
            "id": "event-multi",
            "title": "Multi-outcome Event",
            "markets": [{
                "id": "multi-1",
                "conditionId": "cond-multi-1",
                "slug": "multi-1",
                "question": "Who will win the election?",
                "outcomes": "[\"Alice\", \"Bob\", \"Charlie\"]",
                "outcomePrices": "[\"0.45\", \"0.35\", \"0.20\"]",
                "volumeNum": 200000.0,
                "liquidityNum": 50000.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Election market with 3 candidates"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
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
    assert_eq!(market.ticker, "multi-1");
    assert_eq!(
        market
            .outcomes
            .iter()
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>(),
        vec!["Alice", "Bob", "Charlie"]
    );
    assert_eq!(market.outcomes[0].price, Some(0.45));
    assert_eq!(market.outcomes[1].price, Some(0.35));
    assert_eq!(market.outcomes[2].price, Some(0.20));
}

#[tokio::test]
async fn test_fetch_markets_ticker_is_slug_and_condition_id_separate() {
    // given — every Polymarket market exposes both slug and conditionId; the
    // unified Market.ticker reads slug, Market.condition_id reads conditionId,
    // Market.numeric_id reads the REST-only numeric id.
    let mock_server = MockServer::start().await;
    let events = json!([
        {
            "id": "event-full",
            "title": "Full Fields Event",
            "markets": [{
                "id": "market-1",
                "conditionId": "cond-123",
                "slug": "test-market-slug",
                "question": "Test market with tokens",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.75\", \"0.25\"]",
                "volumeNum": 100000.0,
                "liquidityNum": 25000.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Market with full fields",
                "clobTokenIds": "[\"token1\", \"token2\"]"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
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
    assert_eq!(market.ticker, "test-market-slug");
    assert_eq!(market.numeric_id, Some("market-1".to_string()));
    assert_eq!(market.condition_id, Some("cond-123".to_string()));
    assert_eq!(market.openpx_id, "polymarket:test-market-slug");

    // Verify token IDs are parsed from clobTokenIds and zipped onto outcomes.
    let token_ids = market.token_ids();
    assert_eq!(token_ids, vec!["token1", "token2"]);
    assert_eq!(market.token_id_yes(), Some("token1"));
    assert_eq!(market.token_id_no(), Some("token2"));

    assert_eq!(market.outcomes.len(), 2);
    assert_eq!(market.outcomes[0].label, "Yes");
    assert_eq!(market.outcomes[0].token_id.as_deref(), Some("token1"));
    assert_eq!(market.outcomes[1].label, "No");
    assert_eq!(market.outcomes[1].token_id.as_deref(), Some("token2"));
}

#[tokio::test]
async fn test_polymarket_tick_size_reads_order_price_min_tick_size() {
    // given — assert the parser reads the spec field (orderPriceMinTickSize),
    // not the legacy `minimum_tick_size` name that earlier code referenced.
    let mock_server = MockServer::start().await;
    let events = json!([
        {
            "id": "event-tick",
            "title": "Tick Event",
            "markets": [{
                "id": "tick-market",
                "conditionId": "cond-tick",
                "slug": "tick-market",
                "question": "Tick test?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.50\", \"0.50\"]",
                "volumeNum": 100.0,
                "orderPriceMinTickSize": 0.001,
                "description": "Tick test"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    let (markets, _) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].tick_size, Some(0.001));
}

#[tokio::test]
async fn test_polymarket_settlement_time_from_closed_time() {
    // given — settlement_time should read closedTime, not endDate
    let mock_server = MockServer::start().await;
    let events = json!([
        {
            "id": "event-settle",
            "title": "Settlement Event",
            "markets": [{
                "id": "settle-market",
                "conditionId": "cond-settle",
                "slug": "settle-market",
                "question": "Settled?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"1.0\", \"0.0\"]",
                "volumeNum": 100.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Settled market",
                "active": false,
                "closed": true,
                "endDate": "2026-04-01T00:00:00Z",
                "closedTime": "2026-04-15T12:34:56Z"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    let (markets, _) = exchange
        .fetch_markets(&FetchMarketsParams {
            status: Some(MarketStatusFilter::All),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    let m = &markets[0];
    assert_eq!(m.status, MarketStatus::Resolved);
    let settlement = m.settlement_time.unwrap();
    assert_eq!(settlement.to_rfc3339(), "2026-04-15T12:34:56+00:00");
    // close_time should still come from endDate
    let close = m.close_time.unwrap();
    assert_eq!(close.to_rfc3339(), "2026-04-01T00:00:00+00:00");
}

#[tokio::test]
async fn test_polymarket_result_derived_from_winning_outcome() {
    // given — resolved binary market where Yes won (price 1.0)
    let mock_server = MockServer::start().await;
    let events = json!([
        {
            "id": "event-result",
            "title": "Resolved",
            "markets": [{
                "id": "yes-wins",
                "conditionId": "cond-yes",
                "slug": "yes-wins",
                "question": "Did Yes win?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"1.0\", \"0.0\"]",
                "volumeNum": 100.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Yes wins",
                "active": false,
                "closed": true
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    let (markets, _) = exchange
        .fetch_markets(&FetchMarketsParams {
            status: Some(MarketStatusFilter::All),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].result.as_deref(), Some("Yes"));
}

#[tokio::test]
async fn test_polymarket_result_none_for_unresolved() {
    // given — active market: result should be None even with prices set
    let mock_server = MockServer::start().await;
    let events = json!([
        {
            "id": "event-unresolved",
            "title": "Active",
            "markets": [{
                "id": "active-market",
                "conditionId": "cond-active",
                "slug": "active-market",
                "question": "In flight?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.65\", \"0.35\"]",
                "volumeNum": 100.0,
                "orderPriceMinTickSize": 0.01,
                "description": "Still trading",
                "active": true,
                "closed": false
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    let (markets, _) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].result, None);
}

#[tokio::test]
async fn test_market_volume() {
    // given
    let mock_server = MockServer::start().await;
    let events = json!([
        {
            "id": "event-vol",
            "title": "Volume Event",
            "markets": [{
                "id": "vol-market",
                "conditionId": "cond-vol",
                "slug": "vol-market",
                "question": "Volume test market?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.55\", \"0.45\"]",
                "volumeNum": 987654.32,
                "liquidityNum": 123456.78,
                "orderPriceMinTickSize": 0.001,
                "description": "Market for volume testing"
            }]
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyset_envelope(events)))
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
    assert_eq!(market.tick_size, Some(0.001));
}

#[tokio::test]
async fn test_fetch_market_single_returns_correct_exchange_field() {
    // given
    let mock_server = MockServer::start().await;

    let market_response = json!({
        "id": "abc-123",
        "conditionId": "cond-abc",
        "slug": "abc-123",
        "question": "Exchange field test",
        "outcomes": "[\"Yes\", \"No\"]",
        "outcomePrices": "[\"0.90\", \"0.10\"]",
        "volumeNum": 5000.0,
        "liquidityNum": 1000.0,
        "orderPriceMinTickSize": 0.01,
        "description": "Verify exchange and openpx_id fields",
        "clobTokenIds": "[\"tok-yes\", \"tok-no\"]"
    });

    Mock::given(method("GET"))
        .and(path("/markets/slug/abc-123"))
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
    assert_eq!(market.openpx_id, "polymarket:abc-123");
    assert_eq!(market.ticker, "abc-123");
    assert_eq!(market.condition_id.as_deref(), Some("cond-abc"));
}

// ---------------------------------------------------------------------------
// fetch_markets: MarketStatusFilter
// ---------------------------------------------------------------------------

fn sample_mixed_status_events() -> serde_json::Value {
    json!([
        {
            "id": "event-mixed",
            "title": "Mixed Status Event",
            "markets": [
                {
                    "id": "pm-active",
                    "conditionId": "cond-pm-active",
                    "slug": "pm-active",
                    "question": "Active market?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.60\", \"0.40\"]",
                    "volumeNum": 1000.0,
                    "liquidityNum": 500.0,
                    "orderPriceMinTickSize": 0.01,
                    "description": "Currently active",
                    "active": true,
                    "closed": false
                },
                {
                    "id": "pm-closed",
                    "conditionId": "cond-pm-closed",
                    "slug": "pm-closed",
                    "question": "Closed market?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.90\", \"0.10\"]",
                    "volumeNum": 5000.0,
                    "liquidityNum": 0.0,
                    "orderPriceMinTickSize": 0.01,
                    "description": "Already settled",
                    "active": false,
                    "closed": true
                }
            ]
        }
    ])
}

#[tokio::test]
async fn test_fetch_markets_status_all_returns_all_statuses() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(keyset_envelope(sample_mixed_status_events())),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::All),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — both active and closed markets returned
    assert_eq!(markets.len(), 2);

    let market_tickers: Vec<&str> = markets.iter().map(|m| m.ticker.as_str()).collect();
    assert!(market_tickers.contains(&"pm-active"));
    assert!(market_tickers.contains(&"pm-closed"));
}

#[tokio::test]
async fn test_fetch_markets_status_active_filters_correctly() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(keyset_envelope(sample_mixed_status_events())),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Active),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — only active market
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].ticker, "pm-active");
    assert_eq!(markets[0].status, MarketStatus::Active);
}

#[tokio::test]
async fn test_fetch_markets_status_resolved_filters_correctly() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(keyset_envelope(sample_mixed_status_events())),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Resolved),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — only the closed/resolved market
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].ticker, "pm-closed");
    assert_eq!(markets[0].status, MarketStatus::Resolved);
}

// ---------------------------------------------------------------------------
// fetch_markets: series_ticker is ignored on Polymarket (slug-semantic;
// numeric series id will be added later as `series_numeric_id`).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_series_ticker_ignored_on_polymarket() {
    // given — mock the bare /events/keyset call without any series_id query
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(keyset_envelope(sample_events_response())),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when — series_ticker is set but should be ignored upstream
    let params = FetchMarketsParams {
        series_ticker: Some("some-series".to_string()),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — the unfiltered keyset response is returned
    assert_eq!(markets.len(), 2);
}

// ---------------------------------------------------------------------------
// fetch_markets: event_ticker fetches a single event's nested markets via slug
// (numeric event ids are intentionally not supported here — coming on a future
// `event_numeric_id` field).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_with_event_slug() {
    // given — slug routes to /events/slug/{slug}
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/slug/will-trump-win-2024"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "555",
            "title": "Will Trump win 2024?",
            "markets": [
                {
                    "id": "mkt-yes",
                    "conditionId": "cond-mkt-yes",
                    "slug": "mkt-yes",
                    "question": "Trump wins?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.60\", \"0.40\"]",
                    "volumeNum": 200000.0,
                    "orderPriceMinTickSize": 0.01,
                    "description": "Presidential election market",
                    "active": true,
                    "closed": false
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when — non-numeric event_id is treated as a slug
    let params = FetchMarketsParams {
        event_ticker: Some("will-trump-win-2024".to_string()),
        ..Default::default()
    };
    let (markets, cursor) = exchange.fetch_markets(&params).await.unwrap();

    // then
    assert_eq!(markets.len(), 1);
    assert!(cursor.is_none());
    assert_eq!(markets[0].ticker, "mkt-yes");
    assert!(markets[0].event_ticker.is_some());
}

// ---------------------------------------------------------------------------
// fetch_market_lineage: market → event (with embedded series) — two round-trips
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_market_lineage_full_chain() {
    let mock_server = MockServer::start().await;

    // 1) Market lookup by slug
    Mock::given(method("GET"))
        .and(path("/markets/slug/will-trump-win-2024"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "253591",
            "slug": "will-trump-win-2024",
            "conditionId": "0xdd22472e552920b8438158ea7238bfadfa4f736aa4cee91a6b86c39ead110917",
            "question": "Will Donald Trump win the 2024 US Presidential Election?",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"1.0\", \"0.0\"]",
            "events": [{ "slug": "presidential-election-winner-2024" }],
            "volumeNum": 3672842910.0,
            "orderPriceMinTickSize": 0.001,
            "active": false,
            "closed": true,
            "negRisk": true
        })))
        .mount(&mock_server)
        .await;

    // 2) Event lookup with embedded series
    Mock::given(method("GET"))
        .and(path("/events/slug/presidential-election-winner-2024"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "12585",
            "slug": "presidential-election-winner-2024",
            "title": "Presidential Election Winner 2024",
            "category": "Politics",
            "negRisk": true,
            "closed": true,
            "volume": 3742190044.0,
            "markets": [
                { "slug": "will-trump-win-2024" },
                { "slug": "will-kamala-harris-win-2024" }
            ],
            "series": [{
                "id": 10345,
                "ticker": "us-presidential-elections",
                "slug": "us-presidential-elections",
                "title": "US Presidential Elections",
                "category": "Politics",
                "volume": 3742190044.0
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    let lineage = exchange
        .fetch_market_lineage("will-trump-win-2024")
        .await
        .unwrap();

    assert_eq!(lineage.market.ticker, "will-trump-win-2024");
    assert_eq!(lineage.market.numeric_id.as_deref(), Some("253591"));
    let event = lineage.event.as_ref().expect("event present");
    assert_eq!(event.ticker, "presidential-election-winner-2024");
    assert_eq!(event.numeric_id.as_deref(), Some("12585"));
    assert_eq!(
        event.series_ticker.as_deref(),
        Some("us-presidential-elections")
    );
    assert_eq!(event.mutually_exclusive, Some(true));
    assert_eq!(event.market_tickers.len(), 2);
    let series = lineage.series.as_ref().expect("series present");
    assert_eq!(series.ticker, "us-presidential-elections");
    assert_eq!(series.numeric_id.as_deref(), Some("10345"));
    assert_eq!(series.title, "US Presidential Elections");
}

#[tokio::test]
async fn test_fetch_market_lineage_no_event_returns_none() {
    let mock_server = MockServer::start().await;
    // Market with no event_ticker — lineage skips event/series fetches entirely.
    Mock::given(method("GET"))
        .and(path("/markets/slug/orphan-market"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "999",
            "slug": "orphan-market",
            "conditionId": "0xCID",
            "question": "Orphan?",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.5\", \"0.5\"]",
            "volumeNum": 0.0,
            "orderPriceMinTickSize": 0.01,
            "active": true,
            "closed": false
        })))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    let lineage = exchange
        .fetch_market_lineage("orphan-market")
        .await
        .unwrap();
    assert_eq!(lineage.market.ticker, "orphan-market");
    assert!(lineage.event.is_none());
    assert!(lineage.series.is_none());
}
