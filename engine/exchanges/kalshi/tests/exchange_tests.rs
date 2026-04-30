use px_core::{
    Exchange, FetchMarketsParams, MarketStatus, MarketStatusFilter, OrderbookRequest,
    PriceHistoryInterval, PriceHistoryRequest, TradesRequest,
};
use px_exchange_kalshi::{Kalshi, KalshiConfig};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn empty_markets_response() -> serde_json::Value {
    serde_json::json!({ "markets": [], "cursor": null })
}

fn sample_markets_response() -> serde_json::Value {
    serde_json::json!({
        "markets": [
            {
                "ticker": "INXD-24DEC31-B5000",
                "event_ticker": "INXD-24DEC31",
                "yes_sub_title": "S&P 500 above 5000 on Dec 31",
                "no_sub_title": "S&P 500 at or below 5000 on Dec 31",
                "yes_ask": 65,
                "volume": 150000.0,
                "open_interest": 25000.0,
                "close_time": "2024-12-31T21:00:00Z",
                "status": "active"
            },
            {
                "ticker": "ELON-TWEET-2024",
                "event_ticker": "ELON-TWEET",
                "yes_sub_title": "Elon tweets about crypto today",
                "no_sub_title": "Elon does not tweet about crypto today",
                "yes_ask": 42,
                "volume": 50000.0,
                "open_interest": 8000.0,
                "close_time": "2024-12-28T23:59:59Z",
                "status": "active"
            }
        ],
        "cursor": null
    })
}

fn sample_single_market_response() -> serde_json::Value {
    serde_json::json!({
        "market": {
            "ticker": "INXD-24DEC31-B5000",
            "yes_sub_title": "S&P 500 above 5000 on Dec 31",
            "no_sub_title": "S&P 500 at or below 5000 on Dec 31",
            "yes_ask": 65,
            "volume": 150000.0,
            "open_interest": 25000.0,
            "close_time": "2024-12-31T21:00:00Z",
            "status": "active"
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
    let (markets, cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // #then
    assert_eq!(markets.len(), 2);
    assert!(cursor.is_none());

    let first = &markets[0];
    assert_eq!(first.id, "INXD-24DEC31-B5000");
    assert_eq!(
        first.title,
        "S&P 500 above 5000 on Dec 31 | S&P 500 at or below 5000 on Dec 31"
    );
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
    assert_eq!(
        market.title,
        "S&P 500 above 5000 on Dec 31 | S&P 500 at or below 5000 on Dec 31"
    );
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

// ---------------------------------------------------------------------------
// fetch_price_history (no auth required)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_price_history_parses_candlesticks() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/candlesticks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [{
                "market_ticker": "TEST-TICKER",
                "candlesticks": [
                    {
                        "end_period_ts": 1719532800,
                        "price": { "open": 65, "high": 70, "low": 60, "close": 68 },
                        "volume": 1000,
                        "open_interest": 500.0
                    },
                    {
                        "end_period_ts": 1719536400,
                        "price": { "open": 68, "high": 75, "low": 66, "close": 72 },
                        "volume": 1500,
                        "open_interest": 600.0
                    }
                ]
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let req = PriceHistoryRequest {
        market_id: "TEST-TICKER".into(),
        outcome: None,
        token_id: None,
        condition_id: None,
        interval: PriceHistoryInterval::OneHour,
        start_ts: Some(1719500000),
        end_ts: Some(1719540000),
    };
    let candles = exchange.fetch_price_history(req).await.unwrap();

    // #then
    assert_eq!(candles.len(), 2);

    // Prices converted from cents to decimal
    let c0 = &candles[0];
    assert!((c0.open - 0.65).abs() < 1e-8);
    assert!((c0.high - 0.70).abs() < 1e-8);
    assert!((c0.low - 0.60).abs() < 1e-8);
    assert!((c0.close - 0.68).abs() < 1e-8);
    assert!((c0.volume - 1000.0).abs() < 1e-8);
    assert_eq!(c0.open_interest, Some(500.0));

    // OHLC invariants: high >= open, close, low; low <= open, close
    assert!(c0.high >= c0.open);
    assert!(c0.high >= c0.close);
    assert!(c0.high >= c0.low);
    assert!(c0.low <= c0.open);
    assert!(c0.low <= c0.close);

    let c1 = &candles[1];
    assert!((c1.open - 0.68).abs() < 1e-8);
    assert!((c1.high - 0.75).abs() < 1e-8);
    assert!((c1.low - 0.66).abs() < 1e-8);
    assert!((c1.close - 0.72).abs() < 1e-8);
    assert!((c1.volume - 1500.0).abs() < 1e-8);
    assert_eq!(c1.open_interest, Some(600.0));

    // Candles should be sorted by timestamp ascending
    assert!(candles[0].timestamp < candles[1].timestamp);

    // Verify end_period_ts is converted to start-of-period
    // period_interval=60 (1h) → interval_secs=3600
    // start = end_period_ts - 3600
    let expected_ts_0 = 1719532800 - 3600;
    assert_eq!(candles[0].timestamp.timestamp(), expected_ts_0);
    let expected_ts_1 = 1719536400 - 3600;
    assert_eq!(candles[1].timestamp.timestamp(), expected_ts_1);
}

#[tokio::test]
async fn test_fetch_price_history_skips_null_ohlc() {
    // #given - candlestick with null OHLC (padding candle from include_latest_before_start)
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/candlesticks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [{
                "market_ticker": "TEST-TICKER",
                "candlesticks": [
                    {
                        "end_period_ts": 1719529200,
                        "price": { "open": null, "high": null, "low": null, "close": null },
                        "volume": 0,
                        "open_interest": 0.0
                    },
                    {
                        "end_period_ts": 1719532800,
                        "price": { "open": 65, "high": 70, "low": 60, "close": 68 },
                        "volume": 1000,
                        "open_interest": 500.0
                    }
                ]
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let req = PriceHistoryRequest {
        market_id: "TEST-TICKER".into(),
        outcome: None,
        token_id: None,
        condition_id: None,
        interval: PriceHistoryInterval::OneHour,
        start_ts: Some(1719500000),
        end_ts: Some(1719540000),
    };
    let candles = exchange.fetch_price_history(req).await.unwrap();

    // #then - padding candle with null OHLC should be filtered out
    assert_eq!(candles.len(), 1);
    assert!((candles[0].open - 0.65).abs() < 1e-8);
}

// ---------------------------------------------------------------------------
// fetch_markets: pagination cursor pass-through
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_with_pagination_cursor() {
    // #given - first page returns cursor "abc123"
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [{
                "ticker": "EVT-1-MKT1",
                "event_ticker": "EVT-1",
                "title": "Market 1",
                "subtitle": "First market",
                "yes_ask": 50,
                "volume": 1000.0,
                "open_interest": 100.0,
                "close_time": "2024-12-31T21:00:00Z",
                "status": "active"
            }],
            "cursor": "abc123"
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let (markets, cursor) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // #then
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "EVT-1-MKT1");
    // Cursor is a compound JSON encoding live + historical pagination;
    // Active filter only paginates live (`r`).
    let cursor_str = cursor.unwrap();
    let cursor_val: serde_json::Value = serde_json::from_str(&cursor_str).unwrap();
    assert_eq!(cursor_val["r"], "abc123");
}

#[tokio::test]
async fn test_fetch_markets_with_cursor_passes_through() {
    // #given - second page request with cursor parameter
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("cursor", "abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [{
                "ticker": "EVT-2-MKT1",
                "event_ticker": "EVT-2",
                "title": "Market 2",
                "subtitle": "Second page market",
                "yes_ask": 30,
                "volume": 500.0,
                "open_interest": 50.0,
                "close_time": "2025-01-15T21:00:00Z",
                "status": "active"
            }],
            "cursor": null
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when - pass a compound cursor from a previous page (live-only).
    let prior_cursor = serde_json::json!({"r": "abc123"}).to_string();
    let params = FetchMarketsParams {
        cursor: Some(prior_cursor),
        ..Default::default()
    };
    let (markets, cursor) = exchange.fetch_markets(&params).await.unwrap();

    // #then
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "EVT-2-MKT1");
    assert!(cursor.is_none(), "last page should have no cursor");
}

// ---------------------------------------------------------------------------
// fetch_market: 404 not found
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_market_not_found_returns_error() {
    // #given
    let mock_server = MockServer::start().await;

    // Mock both /markets/{id} and /events/{id} to return 404
    // (fetch_market falls back to fetch_event_as_market on 404)
    Mock::given(method("GET"))
        .and(path("/markets/NONEXISTENT-TICKER"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "code": "not_found", "message": "market not found" }
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/events/NONEXISTENT-TICKER"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": { "code": "not_found", "message": "event not found" }
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_market("NONEXISTENT-TICKER").await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("not found") || err_str.contains("MarketNotFound"),
        "expected market not found error, got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Error handling: 401 authentication failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_http_401_returns_auth_error() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized: invalid API key"))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_markets(&FetchMarketsParams::default()).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_http_403_returns_auth_error() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(
            ResponseTemplate::new(403).set_body_string("Forbidden: insufficient permissions"),
        )
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_markets(&FetchMarketsParams::default()).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth error, got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Error handling: 429 rate limiting
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_http_429_returns_rate_limit_error() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Too Many Requests"))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_markets(&FetchMarketsParams::default()).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("rate")
            || err_str.to_lowercase().contains("limit")
            || err_str.to_lowercase().contains("429"),
        "expected rate limit error, got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Error handling: 500 server error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_http_500_returns_api_error() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": { "code": "internal_error", "message": "something broke" }
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_markets(&FetchMarketsParams::default()).await;

    // #then
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Auth-required endpoints: return AuthRequired when not authenticated
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_orderbook_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();
    assert!(!exchange.describe().has_create_order); // confirms not authed

    // #when
    let req = OrderbookRequest {
        market_id: "TEST-TICKER".into(),
        outcome: None,
        token_id: None,
    };
    let result = exchange.fetch_orderbook(req).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_fetch_balance_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_balance().await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_fetch_balance_raw_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_balance_raw().await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_fetch_positions_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_positions(None).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_fetch_open_orders_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_open_orders(None).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_fetch_order_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_order("ord1", None).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

#[tokio::test]
async fn test_fetch_fills_requires_auth() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_fills(None, None).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("auth"),
        "expected auth required error, got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// Error handling: structured Kalshi error responses
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_insufficient_balance_error_parsed() {
    // #given - Kalshi returns structured error with code "insufficient_balance"
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/TEST-TICKER"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "code": "insufficient_balance",
                "message": "not enough funds"
            }
        })))
        .mount(&mock_server)
        .await;

    // Also mock the event fallback (fetch_market falls back to /events/{id} on 404)
    // but 400 won't trigger the fallback — it goes straight to error parsing.

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let result = exchange.fetch_market("TEST-TICKER").await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("insufficient")
            || err_str.to_lowercase().contains("funds")
            || err_str.to_lowercase().contains("balance"),
        "expected insufficient balance error, got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// fetch_markets: status filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_filters_by_status() {
    // #given - /markets returns mixed statuses; Active filter must narrow
    // to the single "active" row even if the server echoes others.
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "markets": [
                {
                    "ticker": "MIX-OPEN",
                    "event_ticker": "MIX-EVT",
                    "title": "Open market",
                    "subtitle": "Still trading",
                    "yes_ask": 50,
                    "volume": 1000.0,
                    "open_interest": 100.0,
                    "close_time": "2025-12-31T21:00:00Z",
                    "status": "active"
                },
                {
                    "ticker": "MIX-CLOSED",
                    "event_ticker": "MIX-EVT",
                    "title": "Closed market",
                    "subtitle": "No longer trading",
                    "yes_ask": 90,
                    "volume": 5000.0,
                    "open_interest": 0.0,
                    "close_time": "2024-06-01T21:00:00Z",
                    "status": "closed"
                }
            ],
            "cursor": null
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when - default fetches Active markets
    let (markets, _) = exchange
        .fetch_markets(&FetchMarketsParams::default())
        .await
        .unwrap();

    // #then - only the "active" market should be returned
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "MIX-OPEN");
}

// ---------------------------------------------------------------------------
// fetch_price_history: unsupported interval
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_price_history_unsupported_interval() {
    // #given
    let config = KalshiConfig::new().with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when - SixHours is not supported by Kalshi
    let req = PriceHistoryRequest {
        market_id: "TEST-TICKER".into(),
        outcome: None,
        token_id: None,
        condition_id: None,
        interval: PriceHistoryInterval::SixHours,
        start_ts: None,
        end_ts: None,
    };
    let result = exchange.fetch_price_history(req).await;

    // #then
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.to_lowercase().contains("not support"),
        "expected not supported error, got: {err_str}"
    );
}

// ---------------------------------------------------------------------------
// describe(): ExchangeInfo reflects auth state
// ---------------------------------------------------------------------------

#[test]
fn test_describe_unauthenticated() {
    // #given
    let config = KalshiConfig::new();
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let info = exchange.describe();

    // #then
    assert_eq!(info.id, "kalshi");
    assert_eq!(info.name, "Kalshi");
    assert!(info.has_fetch_markets);
    assert!(info.has_fetch_trades);
    assert!(info.has_fetch_price_history);
    assert!(info.has_fetch_orderbook);
    assert!(info.has_fetch_positions);
    assert!(info.has_fetch_balance);
    assert!(info.has_fetch_fills);
    // Auth-dependent capabilities
    assert!(
        !info.has_create_order,
        "should not have create_order without auth"
    );
    assert!(
        !info.has_cancel_order,
        "should not have cancel_order without auth"
    );
    assert!(
        !info.has_websocket,
        "should not have websocket without auth"
    );
    // Unsupported features
    assert!(!info.has_fetch_user_activity);
    assert!(!info.has_approvals);
    assert!(!info.has_refresh_balance);
    assert!(!info.has_fetch_orderbook_history);
}

// ---------------------------------------------------------------------------
// fetch_market: empty/missing fields handled gracefully
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_market_minimal_response() {
    // #given - market response with only required fields
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/MINIMAL-MKT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "market": {
                "ticker": "MINIMAL-MKT",
                "yes_sub_title": "Yes",
                "no_sub_title": "No",
                "yes_ask": 50,
                "status": "active"
            }
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let market = exchange.fetch_market("MINIMAL-MKT").await.unwrap();

    // #then
    assert_eq!(market.id, "MINIMAL-MKT");
    assert_eq!(market.title, "Yes | No");
    assert_eq!(*market.outcome_prices.get("Yes").unwrap(), 0.50);
    assert_eq!(*market.outcome_prices.get("No").unwrap(), 0.50);
    assert_eq!(market.volume, 0.0); // defaults to 0
    assert_eq!(market.outcomes, vec!["Yes", "No"]);
}

// ---------------------------------------------------------------------------
// fetch_market: dollar-string price fields take precedence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_market_dollar_string_fields_preferred() {
    // #given - market with both legacy cent fields and new dollar-string fields
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/DOLLAR-MKT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "market": {
                "ticker": "DOLLAR-MKT",
                "title": "Dollar string market",
                "yes_ask": 65,
                "yes_ask_dollars": "0.72",
                "yes_bid": 60,
                "yes_bid_dollars": "0.68",
                "last_price": 63,
                "last_price_dollars": "0.70",
                "volume": 10000.0,
                "volume_fp": "15000.5",
                "status": "active"
            }
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let market = exchange.fetch_market("DOLLAR-MKT").await.unwrap();

    // #then - dollar-string fields should be preferred
    assert!((market.outcome_prices["Yes"] - 0.72).abs() < 1e-8);
    assert!((market.outcome_prices["No"] - 0.28).abs() < 1e-8);
    assert_eq!(market.best_ask, Some(0.72));
    assert_eq!(market.best_bid, Some(0.68));
    assert_eq!(market.last_trade_price, Some(0.70));
    assert!((market.volume - 15000.5).abs() < 1e-8); // volume_fp preferred
}

// ---------------------------------------------------------------------------
// fetch_trades: empty response
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_trades_empty_response() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/trades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cursor": null,
            "trades": []
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let req = TradesRequest {
        market_id: "EMPTY-MKT".into(),
        ..Default::default()
    };
    let (trades, cursor) = exchange.fetch_trades(req).await.unwrap();

    // #then
    assert!(trades.is_empty());
    assert!(cursor.is_none());
}

// ---------------------------------------------------------------------------
// fetch_trades: pagination cursor pass-through
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_trades_returns_cursor() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/trades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cursor": "next-page-token",
            "trades": [{
                "trade_id": "t1",
                "created_time": "2024-06-15T10:00:00Z",
                "price": 0.60,
                "no_price": 0.40,
                "count": 10.0,
                "taker_side": "yes"
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let req = TradesRequest {
        market_id: "TEST-MKT".into(),
        ..Default::default()
    };
    let (trades, cursor) = exchange.fetch_trades(req).await.unwrap();

    // #then
    assert_eq!(trades.len(), 1);
    assert_eq!(cursor, Some("next-page-token".to_string()));
}

// ---------------------------------------------------------------------------
// fetch_trades: zero-size trades are filtered
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_trades_filters_zero_size() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets/trades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cursor": null,
            "trades": [
                {
                    "trade_id": "t-valid",
                    "created_time": "2024-06-15T10:00:00Z",
                    "price": 0.60,
                    "no_price": 0.40,
                    "count": 5.0,
                    "taker_side": "yes"
                },
                {
                    "trade_id": "t-zero",
                    "created_time": "2024-06-15T10:01:00Z",
                    "price": 0.55,
                    "no_price": 0.45,
                    "count": 0.0,
                    "taker_side": "yes"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let req = TradesRequest {
        market_id: "TEST-MKT".into(),
        outcome: Some("Yes".into()),
        ..Default::default()
    };
    let (trades, _) = exchange.fetch_trades(req).await.unwrap();

    // #then - zero-size trade should be filtered out
    assert_eq!(trades.len(), 1);
    assert_eq!(trades[0].id.as_deref(), Some("t-valid"));
}

// ---------------------------------------------------------------------------
// fetch_markets: MarketStatusFilter::All returns all statuses
// ---------------------------------------------------------------------------

fn sample_mixed_status_markets() -> serde_json::Value {
    serde_json::json!({
        "markets": [
            {
                "ticker": "MIX-ACTIVE",
                "event_ticker": "MIX-EVT",
                "title": "Active market",
                "subtitle": "Currently trading",
                "yes_ask": 50,
                "volume": 1000.0,
                "close_time": "2025-12-31T21:00:00Z",
                "status": "active"
            },
            {
                "ticker": "MIX-CLOSED",
                "event_ticker": "MIX-EVT",
                "title": "Closed market",
                "subtitle": "No longer trading",
                "yes_ask": 90,
                "volume": 5000.0,
                "close_time": "2024-06-01T21:00:00Z",
                "status": "closed"
            },
            {
                "ticker": "MIX-SETTLED",
                "event_ticker": "MIX-EVT",
                "title": "Settled market",
                "subtitle": "Resolved",
                "yes_ask": 100,
                "volume": 8000.0,
                "close_time": "2024-01-01T21:00:00Z",
                "status": "determined"
            }
        ],
        "cursor": null
    })
}

#[tokio::test]
async fn test_fetch_markets_status_all_returns_all_statuses() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_mixed_status_markets()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/historical/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_markets_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::All),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // #then — all three markets should be returned regardless of status
    assert_eq!(markets.len(), 3);

    let statuses: Vec<MarketStatus> = markets.iter().map(|m| m.status).collect();
    assert!(statuses.contains(&MarketStatus::Active));
    assert!(statuses.contains(&MarketStatus::Closed));
    assert!(statuses.contains(&MarketStatus::Resolved));
}

#[tokio::test]
async fn test_fetch_markets_status_active_filters_correctly() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_mixed_status_markets()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/historical/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_markets_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Active),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // #then — only the active market
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "MIX-ACTIVE");
    assert_eq!(markets[0].status, MarketStatus::Active);
}

#[tokio::test]
async fn test_fetch_markets_status_resolved_filters_correctly() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_mixed_status_markets()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/historical/markets"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_markets_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let params = FetchMarketsParams {
        status: Some(MarketStatusFilter::Resolved),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // #then — only the settled/determined market
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "MIX-SETTLED");
    assert_eq!(markets[0].status, MarketStatus::Resolved);
}

// ---------------------------------------------------------------------------
// fetch_markets: series_id filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_with_series_id() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("series_ticker", "INXD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_markets_response()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/historical/markets"))
        .and(query_param("series_ticker", "INXD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_markets_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let params = FetchMarketsParams {
        series_id: Some("INXD".to_string()),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // #then — mocks only match when series_ticker=INXD is in the query string
    assert_eq!(markets.len(), 2);
}

#[tokio::test]
async fn test_fetch_markets_with_series_id_and_status() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("series_ticker", "KXBTC"))
        .and(query_param("status", "settled"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_mixed_status_markets()))
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/historical/markets"))
        .and(query_param("series_ticker", "KXBTC"))
        .respond_with(ResponseTemplate::new(200).set_body_json(empty_markets_response()))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when
    let params = FetchMarketsParams {
        series_id: Some("KXBTC".to_string()),
        status: Some(MarketStatusFilter::Resolved),
        ..Default::default()
    };
    let (markets, _) = exchange.fetch_markets(&params).await.unwrap();

    // #then — both series_id and status passed; client-side filter keeps only resolved
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0].id, "MIX-SETTLED");
    assert_eq!(markets[0].status, MarketStatus::Resolved);
}

// ---------------------------------------------------------------------------
// fetch_markets: event_id fetches a single event's nested markets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fetch_markets_with_event_id() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/KXBTC-25MAR14"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event": {
                "event_ticker": "KXBTC-25MAR14",
                "title": "Bitcoin March 14"
            },
            "markets": [
                {
                    "ticker": "KXBTC-25MAR14-B85000",
                    "event_ticker": "KXBTC-25MAR14",
                    "title": "BTC above 85000?",
                    "subtitle": "Resolves Yes if BTC >= 85000",
                    "yes_ask": 60,
                    "volume": 10000.0,
                    "status": "active"
                },
                {
                    "ticker": "KXBTC-25MAR14-B90000",
                    "event_ticker": "KXBTC-25MAR14",
                    "title": "BTC above 90000?",
                    "subtitle": "Resolves Yes if BTC >= 90000",
                    "yes_ask": 35,
                    "volume": 8000.0,
                    "status": "active"
                },
                {
                    "ticker": "KXBTC-25MAR14-B95000",
                    "event_ticker": "KXBTC-25MAR14",
                    "title": "BTC above 95000?",
                    "subtitle": "Resolves Yes if BTC >= 95000",
                    "yes_ask": 15,
                    "volume": 5000.0,
                    "status": "determined"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when — default status=Active filters out the determined market
    let params = FetchMarketsParams {
        event_id: Some("KXBTC-25MAR14".to_string()),
        ..Default::default()
    };
    let (markets, cursor) = exchange.fetch_markets(&params).await.unwrap();

    // #then
    assert_eq!(markets.len(), 2);
    assert!(
        cursor.is_none(),
        "event_id fetch should not return a cursor"
    );
    assert_eq!(markets[0].id, "KXBTC-25MAR14-B85000");
    assert_eq!(markets[1].id, "KXBTC-25MAR14-B90000");
}

#[tokio::test]
async fn test_fetch_markets_with_event_id_status_all() {
    // #given
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/events/KXBTC-25MAR14"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event": {
                "event_ticker": "KXBTC-25MAR14",
                "title": "Bitcoin March 14"
            },
            "markets": [
                {
                    "ticker": "KXBTC-25MAR14-B85000",
                    "event_ticker": "KXBTC-25MAR14",
                    "title": "BTC above 85000?",
                    "yes_ask": 60,
                    "status": "active"
                },
                {
                    "ticker": "KXBTC-25MAR14-B95000",
                    "event_ticker": "KXBTC-25MAR14",
                    "title": "BTC above 95000?",
                    "yes_ask": 15,
                    "status": "determined"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let config = KalshiConfig::new()
        .with_api_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Kalshi::new(config).unwrap();

    // #when — status=All returns everything
    let params = FetchMarketsParams {
        event_id: Some("KXBTC-25MAR14".to_string()),
        status: Some(MarketStatusFilter::All),
        ..Default::default()
    };
    let (markets, cursor) = exchange.fetch_markets(&params).await.unwrap();

    // #then
    assert_eq!(markets.len(), 2);
    assert!(cursor.is_none());
}
