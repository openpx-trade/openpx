//! Integration tests for the SDK layer (ExchangeInner enum dispatch).
//!
//! These tests verify constructor behavior, metadata dispatch, config parsing,
//! describe consistency, and error handling without requiring network access
//! or mock servers.

use px_sdk::ExchangeInner;
use serde_json::json;

// ---------------------------------------------------------------------------
// 1. Constructor tests
// ---------------------------------------------------------------------------

#[test]
fn construct_kalshi_with_empty_config() {
    let ex = ExchangeInner::new("kalshi", json!({}));
    assert!(ex.is_ok(), "kalshi should construct with empty config");
    assert_eq!(ex.unwrap().id(), "kalshi");
}

#[test]
fn construct_polymarket_with_empty_config() {
    let ex = ExchangeInner::new("polymarket", json!({}));
    assert!(ex.is_ok(), "polymarket should construct with empty config");
    assert_eq!(ex.unwrap().id(), "polymarket");
}

#[test]
fn construct_opinion_with_empty_config() {
    let ex = ExchangeInner::new("opinion", json!({}));
    assert!(ex.is_ok(), "opinion should construct with empty config");
    assert_eq!(ex.unwrap().id(), "opinion");
}

#[test]
fn construct_unknown_exchange_returns_error() {
    let result = ExchangeInner::new("unknown_exchange", json!({}));
    assert!(result.is_err(), "unknown exchange should return Err");
    let err_msg = result.err().expect("should be Err").to_string();
    assert!(
        err_msg.contains("unknown exchange"),
        "error message should mention 'unknown exchange', got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// 2. Metadata dispatch
// ---------------------------------------------------------------------------

#[test]
fn kalshi_id_returns_correct_string() {
    let ex = ExchangeInner::new("kalshi", json!({})).unwrap();
    assert_eq!(ex.id(), "kalshi");
}

#[test]
fn polymarket_id_returns_correct_string() {
    let ex = ExchangeInner::new("polymarket", json!({})).unwrap();
    assert_eq!(ex.id(), "polymarket");
}

#[test]
fn opinion_id_returns_correct_string() {
    let ex = ExchangeInner::new("opinion", json!({})).unwrap();
    assert_eq!(ex.id(), "opinion");
}

#[test]
fn kalshi_name_is_nonempty() {
    let ex = ExchangeInner::new("kalshi", json!({})).unwrap();
    let name = ex.name();
    assert!(!name.is_empty(), "kalshi name should not be empty");
    assert_eq!(name, "Kalshi");
}

#[test]
fn polymarket_name_is_nonempty() {
    let ex = ExchangeInner::new("polymarket", json!({})).unwrap();
    let name = ex.name();
    assert!(!name.is_empty(), "polymarket name should not be empty");
    assert_eq!(name, "Polymarket");
}

#[test]
fn opinion_name_is_nonempty() {
    let ex = ExchangeInner::new("opinion", json!({})).unwrap();
    let name = ex.name();
    assert!(!name.is_empty(), "opinion name should not be empty");
    assert_eq!(name, "Opinion");
}

#[test]
fn kalshi_describe_returns_valid_exchange_info() {
    let ex = ExchangeInner::new("kalshi", json!({})).unwrap();
    let info = ex.describe();
    assert_eq!(info.id, "kalshi");
    assert_eq!(info.name, "Kalshi");
}

#[test]
fn polymarket_describe_returns_valid_exchange_info() {
    let ex = ExchangeInner::new("polymarket", json!({})).unwrap();
    let info = ex.describe();
    assert_eq!(info.id, "polymarket");
    assert_eq!(info.name, "Polymarket");
}

#[test]
fn opinion_describe_returns_valid_exchange_info() {
    let ex = ExchangeInner::new("opinion", json!({})).unwrap();
    let info = ex.describe();
    assert_eq!(info.id, "opinion");
    assert_eq!(info.name, "Opinion");
}

// ---------------------------------------------------------------------------
// 3. Config parsing through SDK
// ---------------------------------------------------------------------------

#[test]
fn kalshi_with_verbose_config() {
    let ex = ExchangeInner::new("kalshi", json!({"verbose": true}));
    assert!(ex.is_ok(), "kalshi should accept verbose config");
    assert_eq!(ex.unwrap().id(), "kalshi");
}

#[test]
fn kalshi_with_demo_config() {
    let ex = ExchangeInner::new("kalshi", json!({"demo": true}));
    assert!(ex.is_ok(), "kalshi should accept demo config");
    assert_eq!(ex.unwrap().id(), "kalshi");
}

#[test]
fn kalshi_with_api_url_config() {
    let ex = ExchangeInner::new("kalshi", json!({"api_url": "https://custom.kalshi.com"}));
    assert!(ex.is_ok(), "kalshi should accept api_url config");
}

#[test]
fn polymarket_with_gamma_url_config() {
    let ex = ExchangeInner::new("polymarket", json!({"gamma_url": "http://test"}));
    assert!(ex.is_ok(), "polymarket should accept gamma_url config");
    assert_eq!(ex.unwrap().id(), "polymarket");
}

#[test]
fn polymarket_with_clob_url_config() {
    let ex = ExchangeInner::new("polymarket", json!({"clob_url": "http://test-clob"}));
    assert!(ex.is_ok(), "polymarket should accept clob_url config");
}

#[test]
fn polymarket_with_verbose_config() {
    let ex = ExchangeInner::new("polymarket", json!({"verbose": true}));
    assert!(ex.is_ok(), "polymarket should accept verbose config");
}

#[test]
fn opinion_with_api_url_config() {
    let ex = ExchangeInner::new("opinion", json!({"api_url": "http://test"}));
    assert!(ex.is_ok(), "opinion should accept api_url config");
    assert_eq!(ex.unwrap().id(), "opinion");
}

#[test]
fn opinion_with_verbose_config() {
    let ex = ExchangeInner::new("opinion", json!({"verbose": true}));
    assert!(ex.is_ok(), "opinion should accept verbose config");
}

#[test]
fn config_with_null_value_treated_as_empty() {
    // A JSON null should be handled gracefully (no object keys to extract).
    let ex = ExchangeInner::new("kalshi", json!(null));
    assert!(ex.is_ok(), "null config should be handled gracefully");
}

#[test]
fn config_with_array_value_treated_as_empty() {
    // A JSON array is not an object, so as_object() returns None and no config
    // fields are extracted. The exchange should still construct with defaults.
    let ex = ExchangeInner::new("kalshi", json!([]));
    assert!(ex.is_ok(), "array config should be handled gracefully");
}

#[test]
fn config_with_extra_unknown_fields_ignored() {
    // Unknown fields in the config object should be silently ignored.
    let ex = ExchangeInner::new(
        "kalshi",
        json!({"unknown_field": "value", "another": 42}),
    );
    assert!(ex.is_ok(), "unknown config fields should be ignored");
}

// ---------------------------------------------------------------------------
// 4. Describe consistency across exchanges
// ---------------------------------------------------------------------------

#[test]
fn all_exchanges_report_has_fetch_markets_true() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert!(
            info.has_fetch_markets,
            "{id} should report has_fetch_markets = true"
        );
    }
}

#[test]
fn unauthenticated_exchanges_report_has_create_order_false() {
    // When no credentials are provided, all exchanges should report
    // has_create_order = false.
    let exchanges = ["kalshi", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert!(
            !info.has_create_order,
            "{id} without auth should report has_create_order = false"
        );
    }
}

#[test]
fn unauthenticated_polymarket_reports_has_create_order_false() {
    let ex = ExchangeInner::new("polymarket", json!({})).unwrap();
    let info = ex.describe();
    assert!(
        !info.has_create_order,
        "polymarket without auth should report has_create_order = false"
    );
}

#[test]
fn unauthenticated_exchanges_report_has_cancel_order_false() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert!(
            !info.has_cancel_order,
            "{id} without auth should report has_cancel_order = false"
        );
    }
}

#[test]
fn describe_id_matches_exchange_id() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert_eq!(
            info.id,
            ex.id(),
            "describe().id should match id() for {id}"
        );
    }
}

#[test]
fn describe_name_matches_exchange_name() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert_eq!(
            info.name,
            ex.name(),
            "describe().name should match name() for {id}"
        );
    }
}

#[test]
fn all_exchanges_report_has_fetch_positions_true() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert!(
            info.has_fetch_positions,
            "{id} should report has_fetch_positions = true"
        );
    }
}

#[test]
fn all_exchanges_report_has_fetch_balance_true() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        assert!(
            info.has_fetch_balance,
            "{id} should report has_fetch_balance = true"
        );
    }
}

#[test]
fn kalshi_describe_capabilities() {
    let ex = ExchangeInner::new("kalshi", json!({})).unwrap();
    let info = ex.describe();
    assert!(info.has_fetch_orderbook, "kalshi should have fetch_orderbook");
    assert!(info.has_fetch_price_history, "kalshi should have fetch_price_history");
    assert!(info.has_fetch_trades, "kalshi should have fetch_trades");
    assert!(info.has_fetch_fills, "kalshi should have fetch_fills");
    assert!(!info.has_approvals, "kalshi should not have approvals");
    assert!(!info.has_refresh_balance, "kalshi should not have refresh_balance");
    assert!(!info.has_fetch_user_activity, "kalshi should not have fetch_user_activity");
}

#[test]
fn polymarket_describe_capabilities() {
    let ex = ExchangeInner::new("polymarket", json!({})).unwrap();
    let info = ex.describe();
    assert!(info.has_fetch_orderbook, "polymarket should have fetch_orderbook");
    assert!(info.has_fetch_price_history, "polymarket should have fetch_price_history");
    assert!(info.has_fetch_trades, "polymarket should have fetch_trades");
    assert!(info.has_fetch_fills, "polymarket should have fetch_fills");
    assert!(info.has_approvals, "polymarket should have approvals");
    assert!(info.has_refresh_balance, "polymarket should have refresh_balance");
    assert!(info.has_fetch_user_activity, "polymarket should have fetch_user_activity");
    assert!(info.has_websocket, "polymarket should have websocket");
}

#[test]
fn opinion_describe_capabilities() {
    let ex = ExchangeInner::new("opinion", json!({})).unwrap();
    let info = ex.describe();
    assert!(info.has_fetch_orderbook, "opinion should have fetch_orderbook");
    assert!(!info.has_fetch_price_history, "opinion should not have fetch_price_history");
    assert!(!info.has_fetch_trades, "opinion should not have fetch_trades");
    assert!(!info.has_fetch_fills, "opinion should not have fetch_fills");
    assert!(!info.has_approvals, "opinion should not have approvals");
    assert!(!info.has_refresh_balance, "opinion should not have refresh_balance");
    assert!(info.has_fetch_user_activity, "opinion should have fetch_user_activity");
}

#[test]
fn unauthenticated_kalshi_has_no_websocket() {
    let ex = ExchangeInner::new("kalshi", json!({})).unwrap();
    let info = ex.describe();
    assert!(
        !info.has_websocket,
        "kalshi without auth should not have websocket"
    );
}

#[test]
fn unauthenticated_opinion_has_no_websocket() {
    let ex = ExchangeInner::new("opinion", json!({})).unwrap();
    let info = ex.describe();
    assert!(
        !info.has_websocket,
        "opinion without auth should not have websocket"
    );
}

// ---------------------------------------------------------------------------
// 5. Error handling
// ---------------------------------------------------------------------------

#[test]
fn empty_string_exchange_id_returns_error() {
    let result = ExchangeInner::new("", json!({}));
    assert!(result.is_err(), "empty string exchange ID should return Err");
}

#[test]
fn whitespace_exchange_id_returns_error() {
    let result = ExchangeInner::new("  ", json!({}));
    assert!(result.is_err(), "whitespace exchange ID should return Err");
}

#[test]
fn case_sensitive_exchange_id() {
    // Exchange IDs should be case-sensitive: "Kalshi" is not "kalshi".
    let result = ExchangeInner::new("Kalshi", json!({}));
    assert!(result.is_err(), "exchange ID should be case-sensitive");

    let result = ExchangeInner::new("POLYMARKET", json!({}));
    assert!(result.is_err(), "exchange ID should be case-sensitive");
}

#[test]
fn numeric_exchange_id_returns_error() {
    let result = ExchangeInner::new("12345", json!({}));
    assert!(result.is_err(), "numeric exchange ID should return Err");
}

#[test]
fn config_with_wrong_type_for_verbose_is_ignored() {
    // verbose expects a bool; passing a string should be silently ignored
    // (the .as_bool() call returns None).
    let ex = ExchangeInner::new("kalshi", json!({"verbose": "yes"}));
    assert!(
        ex.is_ok(),
        "non-bool verbose should be ignored, not cause an error"
    );
}

#[test]
fn config_with_wrong_type_for_api_url_is_ignored() {
    // api_url expects a string; passing a number should be silently ignored
    // (the .as_str() call returns None).
    let ex = ExchangeInner::new("kalshi", json!({"api_url": 42}));
    assert!(
        ex.is_ok(),
        "non-string api_url should be ignored, not cause an error"
    );
}

// ---------------------------------------------------------------------------
// 6. Multiple exchanges can coexist
// ---------------------------------------------------------------------------

#[test]
fn multiple_exchange_instances_are_independent() {
    let kalshi = ExchangeInner::new("kalshi", json!({})).unwrap();
    let polymarket = ExchangeInner::new("polymarket", json!({})).unwrap();
    let opinion = ExchangeInner::new("opinion", json!({})).unwrap();

    // Each instance reports its own identity.
    assert_eq!(kalshi.id(), "kalshi");
    assert_eq!(polymarket.id(), "polymarket");
    assert_eq!(opinion.id(), "opinion");

    // Each instance's describe() is consistent with its own identity.
    assert_eq!(kalshi.describe().id, "kalshi");
    assert_eq!(polymarket.describe().id, "polymarket");
    assert_eq!(opinion.describe().id, "opinion");
}

#[test]
fn describe_is_serializable_to_json() {
    let exchanges = ["kalshi", "polymarket", "opinion"];
    for id in &exchanges {
        let ex = ExchangeInner::new(id, json!({})).unwrap();
        let info = ex.describe();
        let json_result = serde_json::to_value(&info);
        assert!(
            json_result.is_ok(),
            "ExchangeInfo for {id} should be serializable to JSON"
        );
        let json_val = json_result.unwrap();
        assert_eq!(
            json_val["id"].as_str().unwrap(),
            *id,
            "serialized id should match for {id}"
        );
        assert!(
            json_val["has_fetch_markets"].as_bool().unwrap(),
            "serialized has_fetch_markets should be true for {id}"
        );
    }
}
