use px_core::{Exchange, TradesRequest};
use px_exchange_kalshi::{Kalshi, KalshiConfig};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_markets_response() -> serde_json::Value {
    serde_json::json!({
        "markets": [
            {
                "ticker": "INXD-24DEC31-B5000",
                "title": "S&P 500 above 5000 on Dec 31?",
                "subtitle": "Market resolves Yes if S&P closes above 5000",
                "yes_ask": 65,
                "volume": 150000.0,
                "open_interest": 25000.0,
                "close_time": "2024-12-31T21:00:00Z",
                "status": "open"
            },
            {
                "ticker": "ELON-TWEET-2024",
                "title": "Will Elon tweet about crypto today?",
                "subtitle": "Any tweet mentioning BTC, ETH, or DOGE",
                "yes_ask": 42,
                "volume": 50000.0,
                "open_interest": 8000.0,
                "close_time": "2024-12-28T23:59:59Z",
                "status": "open"
            }
        ],
        "cursor": null
    })
}

fn sample_single_market_response() -> serde_json::Value {
    serde_json::json!({
        "market": {
            "ticker": "INXD-24DEC31-B5000",
            "title": "S&P 500 above 5000 on Dec 31?",
            "subtitle": "Market resolves Yes if S&P closes above 5000",
            "yes_ask": 65,
            "volume": 150000.0,
            "open_interest": 25000.0,
            "close_time": "2024-12-31T21:00:00Z",
            "status": "open"
        }
    })
}

#[tokio::test]
async fn test_fetch_markets_parses_response() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_markets_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let markets = exchange.fetch_markets().await.unwrap();

    // #then
    assert_eq!(markets.len(), 2);

    let first = &markets[0];
    assert_eq!(first.id, "INXD-24DEC31-B5000");
    assert_eq!(first.title, "S&P 500 above 5000 on Dec 31?");
    assert_eq!(first.outcomes, vec!["Yes", "No"]);
    assert_eq!(*first.outcome_prices.get("Yes").unwrap(), 0.65);
    assert_eq!(*first.outcome_prices.get("No").unwrap(), 0.35);
    assert_eq!(first.volume, 150000.0);
    assert_eq!(first.open_interest, Some(25000.0));
}

#[tokio::test]
async fn test_fetch_market_by_ticker() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/INXD-24DEC31-B5000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_single_market_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let market = exchange.fetch_market("INXD-24DEC31-B5000").await.unwrap();

    // #then
    assert_eq!(market.id, "INXD-24DEC31-B5000");
    assert_eq!(market.title, "S&P 500 above 5000 on Dec 31?");
    assert_eq!(*market.outcome_prices.get("Yes").unwrap(), 0.65);
    assert_eq!(*market.outcome_prices.get("No").unwrap(), 0.35);
}

#[tokio::test]
async fn test_exchange_info() {
    // #given
    let config = KalshiConfig::new();
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let info = exchange.describe();

    // #then
    assert_eq!(info.id, "kalshi");
    assert_eq!(info.name, "Kalshi");
    assert!(info.has_fetch_markets);
    assert!(!info.has_create_order);
    assert!(!info.has_websocket); // No auth = no websocket
}

#[tokio::test]
async fn test_exchange_id_and_name() {
    // #given
    let config = KalshiConfig::new();
    let exchange = Kalshi::new(config).unwrap();

    // #when / #then
    assert_eq!(exchange.id(), "kalshi");
    assert_eq!(exchange.name(), "Kalshi");
}

#[test]
fn test_config_builder() {
    // #given
    let api_key_id = "test-api-key-id";
    let private_key_path = "/path/to/private-key.pem";

    // #when
    let config = KalshiConfig::new()
        .with_api_key_id(api_key_id)
        .with_private_key_path(private_key_path)
        .with_verbose(true);

    // #then
    assert_eq!(config.api_key_id, Some(api_key_id.to_string()));
    assert_eq!(config.private_key_path, Some(private_key_path.to_string()));
    assert!(config.is_authenticated());
    assert!(config.base.verbose);
}

#[test]
fn test_demo_config() {
    // #given / #when
    let config = KalshiConfig::demo();

    // #then
    assert!(config.demo);
    assert!(config.api_url.contains("demo"));
    assert!(!config.is_authenticated());
}

#[test]
fn test_default_config_not_authenticated() {
    // #given / #when
    let config = KalshiConfig::default();

    // #then
    assert!(!config.is_authenticated());
    assert!(!config.demo);
}

// ---------------------------------------------------------------------------
// fetch_trades: no_price semantics (Fix 0)
// ---------------------------------------------------------------------------

fn sample_trades_response_with_no_price() -> serde_json::Value {
    serde_json::json!({
        "cursor": null,
        "trades": [
            {
                "trade_id": "t1",
                "created_time": "2024-06-15T10:00:00Z",
                "price": 0.60,
                "no_price": 0.42,
                "count": 10.0,
                "taker_side": "yes"
            },
            {
                "trade_id": "t2",
                "created_time": "2024-06-15T10:01:00Z",
                "price": 0.55,
                "no_price": null,
                "count": 5.0,
                "taker_side": "no"
            },
            {
                "trade_id": "t3",
                "created_time": "2024-06-15T10:02:00Z",
                "price": 0.70,
                "no_price": 0.31,
                "count": 8.0,
                "taker_side": "yes"
            },
            {
                "trade_id": "t4",
                "created_time": "2024-06-15T10:03:00Z",
                "price": 1.0,
                "no_price": 99.0,
                "count": 3.0,
                "taker_side": "no"
            }
        ]
    })
}

#[tokio::test]
async fn fetch_trades_no_outcome_uses_real_no_price() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/trades"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(sample_trades_response_with_no_price()),
        )
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // Query with outcome=No
    let req = TradesRequest {
        market_id: "TEST-TICKER".into(),
        market_ref: None,
        outcome: Some("No".into()),
        token_id: None,
        start_ts: None,
        end_ts: None,
        limit: Some(100),
        cursor: None,
    };

    let (trades, _cursor) = exchange.fetch_trades(req).await.unwrap();

    // t2 has null no_price → should be SKIPPED
    // t1/t3/t4 have valid no_price (t4 is cents-like 99.0) → should be included
    assert_eq!(
        trades.len(),
        3,
        "trade with null no_price should be excluded"
    );

    // t1: no_price=0.42 (NOT 1.0 - 0.60 = 0.40)
    let t1 = trades
        .iter()
        .find(|t| t.id.as_deref() == Some("t1"))
        .unwrap();
    assert!(
        (t1.price - 0.42).abs() < 1e-8,
        "price should be real no_price (0.42), not 1-yes_price (0.40); got {}",
        t1.price
    );
    // aggressor_side should be flipped: taker_side=yes → "sell" for No perspective
    assert_eq!(t1.aggressor_side.as_deref(), Some("sell"));
    // yes_price and no_price are reference fields, NOT swapped
    assert_eq!(t1.yes_price, Some(0.60));
    assert_eq!(t1.no_price, Some(0.42));

    // t3: no_price=0.31 (NOT 1.0 - 0.70 = 0.30)
    let t3 = trades
        .iter()
        .find(|t| t.id.as_deref() == Some("t3"))
        .unwrap();
    assert!(
        (t3.price - 0.31).abs() < 1e-8,
        "price should be real no_price (0.31), not 1-yes_price (0.30); got {}",
        t3.price
    );

    // t4: no_price=99.0 (cents-like) should normalize to 0.99
    let t4 = trades
        .iter()
        .find(|t| t.id.as_deref() == Some("t4"))
        .unwrap();
    assert!(
        (t4.price - 0.99).abs() < 1e-8,
        "price should normalize no_price cents (99.0 -> 0.99); got {}",
        t4.price
    );
    assert_eq!(t4.yes_price, Some(0.01));
    assert_eq!(t4.no_price, Some(0.99));
    assert_eq!(t4.outcome.as_deref(), Some("No"));
}

#[tokio::test]
async fn fetch_trades_yes_outcome_uses_yes_price() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/trades"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(sample_trades_response_with_no_price()),
        )
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // Query with outcome=Yes (default)
    let req = TradesRequest {
        market_id: "TEST-TICKER".into(),
        market_ref: None,
        outcome: Some("Yes".into()),
        token_id: None,
        start_ts: None,
        end_ts: None,
        limit: Some(100),
        cursor: None,
    };

    let (trades, _cursor) = exchange.fetch_trades(req).await.unwrap();

    // All 4 trades should be included for Yes perspective (no null filtering)
    assert_eq!(
        trades.len(),
        4,
        "all trades should be included for Yes perspective"
    );

    // t1: price should be yes_price=0.60
    let t1 = trades
        .iter()
        .find(|t| t.id.as_deref() == Some("t1"))
        .unwrap();
    assert!((t1.price - 0.60).abs() < 1e-8);
    // aggressor_side: taker_side=yes → "buy" for Yes perspective
    assert_eq!(t1.aggressor_side.as_deref(), Some("buy"));
    assert_eq!(t1.outcome.as_deref(), Some("Yes"));

    // t4 yes_price=1.0 (cents-like) should normalize to 0.01 in yes perspective too
    let t4 = trades
        .iter()
        .find(|t| t.id.as_deref() == Some("t4"))
        .unwrap();
    assert!((t4.price - 0.01).abs() < 1e-8);
    assert_eq!(t4.yes_price, Some(0.01));
    assert_eq!(t4.no_price, Some(0.99));
}
