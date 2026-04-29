use px_core::{Exchange, FetchMarketsParams, MarketStatus, MarketStatusFilter};
use px_exchange_polymarket::{Polymarket, PolymarketConfig};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// /markets/keyset returns `{markets: [...], next_cursor: ...}` (wrapped),
/// not bare events. Use this whenever the test exercises the general
/// `fetch_markets` path.
fn sample_markets_keyset() -> serde_json::Value {
    serde_json::json!({
        "markets": [
            {
                "id": "123",
                "conditionId": "123",
                "question": "Will it rain tomorrow?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.65\", \"0.35\"]",
                "volumeNum": 50000.0,
                "liquidityNum": 10000.0,
                "minimum_tick_size": 0.01,
                "description": "Weather prediction market"
            },
            {
                "id": "456",
                "conditionId": "456",
                "question": "Bitcoin > $100k by EOY?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.42\", \"0.58\"]",
                "volumeNum": 1000000.0,
                "liquidityNum": 250000.0,
                "minimum_tick_size": 0.001,
                "description": "Crypto price prediction"
            }
        ],
        "next_cursor": null
    })
}

/// /events/keyset returns `{events: [{id, markets:[...]}, ...], next_cursor}`.
/// Used for the series_id-filtered path only.
fn sample_events_keyset() -> serde_json::Value {
    serde_json::json!({
        "events": [
            {
                "id": "event-1",
                "title": "Weather Event",
                "markets": [
                    {
                        "id": "123",
                        "conditionId": "123",
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
                        "conditionId": "456",
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
        ],
        "next_cursor": null
    })
}

fn sample_single_market_response() -> serde_json::Value {
    serde_json::json!({
        "id": "789",
        "conditionId": "789",
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
        .and(path("/markets/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_markets_keyset()))
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
    // given — the limit is sent to the server as a query param. The mock
    // returns whatever it is configured to return regardless of limit; we
    // don't truncate client-side. Verifying that the param is passed through
    // and that all returned markets are emitted.
    let mock_server = MockServer::start().await;
    let payload = serde_json::json!({
        "markets": [
            {
                "id": "m1",
                "conditionId": "m1",
                "question": "Market 1?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.60\", \"0.40\"]",
                "volumeNum": 1000.0,
                "liquidityNum": 500.0,
                "minimum_tick_size": 0.01,
                "description": "First market"
            },
            {
                "id": "m2",
                "conditionId": "m2",
                "question": "Market 2?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.50\", \"0.50\"]",
                "volumeNum": 2000.0,
                "liquidityNum": 1000.0,
                "minimum_tick_size": 0.01,
                "description": "Second market"
            },
            {
                "id": "m3",
                "conditionId": "m3",
                "question": "Market 3?",
                "outcomes": "[\"Yes\", \"No\"]",
                "outcomePrices": "[\"0.70\", \"0.30\"]",
                "volumeNum": 3000.0,
                "liquidityNum": 1500.0,
                "minimum_tick_size": 0.01,
                "description": "Third market"
            }
        ],
        "next_cursor": null
    });

    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
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

    // then — mock matched on limit=2 (proves param was sent), returns 3 markets,
    // we surface all 3 (no client-side truncation).
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
        .and(path("/markets/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [],
            "next_cursor": null
        })))
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
    let payload = serde_json::json!({
        "markets": [{
            "id": "multi-1",
            "conditionId": "multi-1",
            "question": "Who will win the election?",
            "outcomes": "[\"Alice\", \"Bob\", \"Charlie\"]",
            "outcomePrices": "[\"0.45\", \"0.35\", \"0.20\"]",
            "volumeNum": 200000.0,
            "liquidityNum": 50000.0,
            "minimum_tick_size": 0.01,
            "description": "Election market with 3 candidates"
        }],
        "next_cursor": null
    });

    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
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
    let payload = serde_json::json!({
        "markets": [{
            "id": "market-1",
            "conditionId": "cond-123",
            "question": "Test market with tokens",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.75\", \"0.25\"]",
            "volumeNum": 100000.0,
            "liquidityNum": 25000.0,
            "minimum_tick_size": 0.01,
            "description": "Market with full fields",
            "clobTokenIds": "[\"token1\", \"token2\"]",
            "slug": "test-market-slug"
        }],
        "next_cursor": null
    });

    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
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
    assert_eq!(market.id, "cond-123"); // Market.id is condition_id on Polymarket
    assert_eq!(market.native_numeric_id, Some("market-1".to_string()));
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
    let payload = serde_json::json!({
        "markets": [{
            "id": "vol-market",
            "conditionId": "vol-market",
            "question": "Volume test market?",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.55\", \"0.45\"]",
            "volumeNum": 987654.32,
            "liquidityNum": 123456.78,
            "minimum_tick_size": 0.001,
            "description": "Market for volume/liquidity testing"
        }],
        "next_cursor": null
    });

    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
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
        "conditionId": "abc-123",
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

// ---------------------------------------------------------------------------
// fetch_markets: status filter (closed=false / closed=true / two-phase All)
// ---------------------------------------------------------------------------

fn active_market_payload() -> serde_json::Value {
    serde_json::json!({
        "markets": [{
            "id": "pm-active",
            "conditionId": "pm-active",
            "question": "Active market?",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.60\", \"0.40\"]",
            "volumeNum": 1000.0,
            "liquidityNum": 500.0,
            "minimum_tick_size": 0.01,
            "description": "Currently active",
            "active": true,
            "closed": false
        }],
        "next_cursor": null
    })
}

fn closed_market_payload() -> serde_json::Value {
    serde_json::json!({
        "markets": [{
            "id": "pm-closed",
            "conditionId": "pm-closed",
            "question": "Closed market?",
            "outcomes": "[\"Yes\", \"No\"]",
            "outcomePrices": "[\"0.90\", \"0.10\"]",
            "volumeNum": 5000.0,
            "liquidityNum": 0.0,
            "minimum_tick_size": 0.01,
            "description": "Already settled",
            "active": false,
            "closed": true
        }],
        "next_cursor": null
    })
}

#[tokio::test]
async fn test_fetch_markets_status_all_returns_all_statuses() {
    // given — All filter is implemented as a sequential two-phase drain:
    // closed=false page first, then closed=true. Caller paginates twice.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("closed", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(active_market_payload()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("closed", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(closed_market_payload()))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when — first page: drains closed=false, returns transition cursor
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::All),
        ..Default::default()
    };
    let (page1, cursor1) = exchange.fetch_markets(&params).await.unwrap();

    // then — only the active market on page 1, but cursor is set (transition).
    assert_eq!(page1.len(), 1);
    assert_eq!(page1[0].id, "pm-active");
    assert!(cursor1.is_some(), "All filter must emit transition cursor");

    // when — second page: drains closed=true with the transition cursor.
    let params2 = FetchMarketsParams {
        status: Some(MarketStatusFilter::All),
        cursor: cursor1,
        ..Default::default()
    };
    let (page2, cursor2) = exchange.fetch_markets(&params2).await.unwrap();

    // then — only the closed market on page 2; cursor is None (drained).
    assert_eq!(page2.len(), 1);
    assert_eq!(page2[0].id, "pm-closed");
    assert!(cursor2.is_none());
}

#[tokio::test]
async fn test_fetch_markets_status_active_filters_correctly() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("closed", "false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(active_market_payload()))
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
    assert_eq!(markets[0].id, "pm-active");
    assert_eq!(markets[0].status, MarketStatus::Active);
}

#[tokio::test]
async fn test_fetch_markets_status_resolved_filters_correctly() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/keyset"))
        .and(query_param("closed", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(closed_market_payload()))
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
    assert_eq!(markets[0].id, "pm-closed");
    assert_eq!(markets[0].status, MarketStatus::Resolved);
}

// ---------------------------------------------------------------------------
// fetch_markets: series_id routes through /events/keyset (events-nested)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_with_series_id() {
    // given — series_id queries route through /events/keyset because
    // /markets/keyset has no series filter.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/keyset"))
        .and(query_param("series_id", "10345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_events_keyset()))
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        series_id: Some("10345".to_string()),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — mock matched on series_id=10345
    assert_eq!(markets.len(), 2);
}

// ---------------------------------------------------------------------------
// fetch_markets: event_id fetches a single event's nested markets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_with_event_id() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/903"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "903",
            "title": "Presidential Election 2028",
            "markets": [
                {
                    "id": "market-a",
                    "conditionId": "market-a",
                    "question": "Will Candidate A win?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.55\", \"0.45\"]",
                    "volumeNum": 100000.0,
                    "liquidityNum": 50000.0,
                    "minimum_tick_size": 0.01,
                    "description": "Candidate A prediction",
                    "active": true,
                    "closed": false
                },
                {
                    "id": "market-b",
                    "conditionId": "market-b",
                    "question": "Will Candidate B win?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.30\", \"0.70\"]",
                    "volumeNum": 80000.0,
                    "liquidityNum": 40000.0,
                    "minimum_tick_size": 0.01,
                    "description": "Candidate B prediction",
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

    // when
    let params = FetchMarketsParams {
        event_id: Some("903".to_string()),
        ..Default::default()
    };
    let (markets, cursor) = exchange.fetch_markets(&params).await.unwrap();

    // then
    assert_eq!(markets.len(), 2);
    assert!(
        cursor.is_none(),
        "event_id fetch should not return a cursor"
    );
    assert_eq!(markets[0].id, "market-a");
    assert_eq!(markets[1].id, "market-b");
    assert_eq!(markets[0].group_id, Some("903".to_string()));
}

#[tokio::test]
async fn test_fetch_markets_with_event_slug() {
    // given — slug routes to /events/slug/{slug}
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/slug/will-trump-win-2024"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "555",
            "title": "Will Trump win 2024?",
            "markets": [
                {
                    "id": "mkt-yes",
                    "conditionId": "mkt-yes",
                    "question": "Trump wins?",
                    "outcomes": "[\"Yes\", \"No\"]",
                    "outcomePrices": "[\"0.60\", \"0.40\"]",
                    "volumeNum": 200000.0,
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
        event_id: Some("will-trump-win-2024".to_string()),
        ..Default::default()
    };
    let (markets, cursor) = exchange.fetch_markets(&params).await.unwrap();

    // then
    assert_eq!(markets.len(), 1);
    assert!(cursor.is_none());
    assert_eq!(markets[0].id, "mkt-yes");
    assert_eq!(markets[0].group_id, Some("555".to_string()));
}
