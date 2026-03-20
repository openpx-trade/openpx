use px_core::{Exchange, FetchMarketsParams, FixedPrice, MarketStatus, MarketStatusFilter};
use px_exchange_opinion::{Opinion, OpinionConfig};
use wiremock::matchers::{method, path};
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
    let (markets, cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // then
    assert_eq!(markets.len(), 2);
    assert!(cursor.is_none()); // less than page size → no more pages

    let first = &markets[0];
    assert_eq!(first.id, "123");
    assert_eq!(first.title, "Will BTC reach $100k?");
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
    assert_eq!(market.title, "Test market question");
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
    assert_eq!(orderbook.bids[0].price, FixedPrice::from_f64(0.65));
    assert_eq!(orderbook.asks[0].price, FixedPrice::from_f64(0.66));
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
    assert_eq!(market.tick_size, Some(0.001));
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
async fn test_fetch_markets_returns_cursor_none_when_incomplete_page() {
    // Opinion returns 2 markets (less than page_size 20), so cursor should be None.
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

    let (markets, cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();
    assert_eq!(markets.len(), 2);
    assert!(cursor.is_none());
    assert_eq!(markets[0].id, "123");
    assert_eq!(markets[1].id, "456");
}

#[test]
fn test_config_builder() {
    let config = OpinionConfig::new()
        .with_api_key("test-key")
        .with_private_key("test-pk")
        .with_multi_sig("0x123")
        .with_api_url("http://test.example")
        .with_verbose(true);

    assert_eq!(config.api_key, Some("test-key".to_string()));
    assert_eq!(config.private_key, Some("test-pk".to_string()));
    assert_eq!(config.multi_sig_addr, Some("0x123".to_string()));
    assert_eq!(config.api_url, "http://test.example");
    assert!(config.base.verbose);
    assert!(config.is_authenticated());
}

#[test]
fn test_default_config_not_authenticated() {
    let config = OpinionConfig::new();

    assert!(config.api_key.is_none());
    assert!(config.private_key.is_none());
    assert!(config.multi_sig_addr.is_none());
    assert!(!config.is_authenticated());
}

#[tokio::test]
async fn test_fetch_markets_empty_response() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": {
                "list": []
            }
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

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
async fn test_fetch_market_not_found() {
    // given
    let mock_server = MockServer::start().await;

    // Binary endpoint returns error
    Mock::given(method("GET"))
        .and(path("/openapi/market/nonexistent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 1,
            "errmsg": "market not found",
            "result": null
        })))
        .mount(&mock_server)
        .await;

    // Categorical fallback also returns error
    Mock::given(method("GET"))
        .and(path("/openapi/market/categorical/nonexistent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 1,
            "errmsg": "market not found",
            "result": null
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let result = exchange.fetch_market("nonexistent").await;

    // then
    assert!(result.is_err());
}

#[tokio::test]
async fn test_orderbook_empty() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/token/orderbook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": {
                "data": {
                    "bids": [],
                    "asks": []
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let orderbook = exchange.get_orderbook("token_xyz").await.unwrap();

    // then
    assert!(orderbook.bids.is_empty());
    assert!(orderbook.asks.is_empty());
    assert_eq!(orderbook.asset_id, "token_xyz");
}

#[tokio::test]
async fn test_orderbook_sorted_correctly() {
    // given — bids and asks deliberately unsorted
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/token/orderbook"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": {
                "data": {
                    "bids": [
                        {"price": 0.50, "size": 100.0},
                        {"price": 0.60, "size": 200.0},
                        {"price": 0.55, "size": 150.0}
                    ],
                    "asks": [
                        {"price": 0.75, "size": 300.0},
                        {"price": 0.65, "size": 100.0},
                        {"price": 0.70, "size": 200.0}
                    ]
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let orderbook = exchange.get_orderbook("token_sort").await.unwrap();

    // then — bids should be sorted descending by price
    assert_eq!(orderbook.bids.len(), 3);
    assert_eq!(orderbook.bids[0].price, FixedPrice::from_f64(0.60));
    assert_eq!(orderbook.bids[1].price, FixedPrice::from_f64(0.55));
    assert_eq!(orderbook.bids[2].price, FixedPrice::from_f64(0.50));

    // then — asks should be sorted ascending by price
    assert_eq!(orderbook.asks.len(), 3);
    assert_eq!(orderbook.asks[0].price, FixedPrice::from_f64(0.65));
    assert_eq!(orderbook.asks[1].price, FixedPrice::from_f64(0.70));
    assert_eq!(orderbook.asks[2].price, FixedPrice::from_f64(0.75));
}

#[tokio::test]
async fn test_fetch_market_close_time() {
    // given — cutoff_at is 1735689600 (2025-01-01 00:00:00 UTC)
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market/ct_test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": {
                "data": {
                    "market_id": "ct_test",
                    "market_title": "Close time test",
                    "yes_token_id": "token_yes_ct",
                    "no_token_id": "token_no_ct",
                    "yes_label": "Yes",
                    "no_label": "No",
                    "volume": 1000.0,
                    "liquidity": 500.0,
                    "description": "Testing close time parsing",
                    "cutoff_at": 1735689600
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let market = exchange.fetch_market("ct_test").await.unwrap();

    // then
    let close_time = market.close_time.expect("close_time should be set");
    assert_eq!(close_time.timestamp(), 1735689600);
    assert_eq!(
        close_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        "2025-01-01 00:00:00"
    );
}

#[tokio::test]
async fn test_exchange_describe_capabilities() {
    // given — not authenticated (no api_key)
    let config = OpinionConfig::new();
    let exchange = Opinion::new(config).unwrap();

    // when
    let info = exchange.describe();

    // then
    assert!(info.has_fetch_markets);
    assert!(!info.has_create_order);
    assert!(!info.has_cancel_order);
    assert!(!info.has_websocket);
    assert!(info.has_fetch_positions);
    assert!(info.has_fetch_balance);
    assert!(info.has_fetch_orderbook);

    // given — authenticated
    let auth_config = OpinionConfig::new().with_api_key("my-key");
    let auth_exchange = Opinion::new(auth_config).unwrap();

    // when
    let auth_info = auth_exchange.describe();

    // then — authenticated enables order and websocket capabilities
    assert!(auth_info.has_create_order);
    assert!(auth_info.has_cancel_order);
    assert!(auth_info.has_websocket);
}

#[tokio::test]
async fn test_market_exchange_field() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market/ex_test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": {
                "data": {
                    "market_id": "ex_test",
                    "market_title": "Exchange field test",
                    "yes_token_id": "token_yes_ex",
                    "no_token_id": "token_no_ex",
                    "yes_label": "Yes",
                    "no_label": "No",
                    "volume": 0.0,
                    "liquidity": 0.0,
                    "description": "Verify exchange field"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let market = exchange.fetch_market("ex_test").await.unwrap();

    // then
    assert_eq!(market.exchange, "opinion");
    assert!(
        market.openpx_id.starts_with("opinion:"),
        "openpx_id should start with 'opinion:' but was '{}'",
        market.openpx_id
    );
    assert_eq!(market.openpx_id, "opinion:ex_test");
}

#[tokio::test]
async fn test_fetch_markets_pagination_full_page() {
    // given — return exactly 20 markets (Opinion page_size)
    let mut market_list = Vec::new();
    for i in 0..20 {
        market_list.push(serde_json::json!({
            "market_id": format!("mkt_{}", i),
            "market_title": format!("Market {}", i),
            "yes_token_id": format!("yes_{}", i),
            "no_token_id": format!("no_{}", i),
            "yes_label": "Yes",
            "no_label": "No",
            "volume": 1000.0,
            "liquidity": 500.0,
            "description": format!("Description {}", i)
        }));
    }

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "errno": 0,
            "errmsg": null,
            "result": {
                "list": market_list
            }
        })))
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let (markets, cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // then — full page means there could be more, cursor should be Some
    assert_eq!(markets.len(), 20);
    assert!(
        cursor.is_some(),
        "cursor should be Some when a full page of 20 markets is returned"
    );
}

// ---------------------------------------------------------------------------
// fetch_markets: MarketStatusFilter::All returns all statuses
// ---------------------------------------------------------------------------

fn sample_mixed_status_markets_response() -> serde_json::Value {
    serde_json::json!({
        "errno": 0,
        "errmsg": null,
        "result": {
            "list": [
                {
                    "market_id": "op-active",
                    "market_title": "Active market",
                    "yes_token_id": "yes_active",
                    "no_token_id": "no_active",
                    "yes_label": "Yes",
                    "no_label": "No",
                    "volume": 1000.0,
                    "liquidity": 500.0,
                    "description": "Currently active",
                    "statusEnum": "Activated"
                },
                {
                    "market_id": "op-resolved",
                    "market_title": "Resolved market",
                    "yes_token_id": "yes_resolved",
                    "no_token_id": "no_resolved",
                    "yes_label": "Yes",
                    "no_label": "No",
                    "volume": 5000.0,
                    "liquidity": 0.0,
                    "description": "Already resolved",
                    "statusEnum": "Resolved"
                }
            ]
        }
    })
}

#[tokio::test]
async fn test_fetch_markets_status_all_returns_all_statuses() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(sample_mixed_status_markets_response()),
        )
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::All),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — both active and resolved markets returned
    assert_eq!(markets.len(), 2);

    let ids: Vec<&str> = markets.iter().map(|m| m.id.as_str()).collect();
    assert!(ids.contains(&"op-active"));
    assert!(ids.contains(&"op-resolved"));
}

#[tokio::test]
async fn test_fetch_markets_status_active_filters_correctly() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(sample_mixed_status_markets_response()),
        )
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Active),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — only the active market
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "op-active");
    assert_eq!(markets[0].status, MarketStatus::Active);
}

#[tokio::test]
async fn test_fetch_markets_status_resolved_filters_correctly() {
    // given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi/market"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(sample_mixed_status_markets_response()),
        )
        .mount(&mock_server)
        .await;

    let config = OpinionConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Opinion::new(config).unwrap();

    // when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Resolved),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // then — only the resolved market
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "op-resolved");
    assert_eq!(markets[0].status, MarketStatus::Resolved);
}
