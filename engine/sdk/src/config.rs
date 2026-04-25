//! Single-source config parsers for the dict-shaped JSON config that
//! `Exchange("...", {...})` and `WebSocket("...", {...})` accept.
//!
//! Both `ExchangeInner::new` and `WebSocketInner::new` were hand-rolling the
//! same `obj.get("...").and_then(|v| v.as_str())` pattern, with two distinct
//! bugs as a result: `KalshiConfig::demo()` silently dropped `api_url` /
//! `private_key_path` because the demo branch rebuilt the config without
//! re-applying every field; and the Polymarket WebSocket path ignored
//! `private_key`, `funder`, and `signature_type` entirely.
//!
//! Going through one parser per exchange means new fields plumb to both
//! call sites at once and config-shape bugs surface at construction.

use serde_json::Value;

use px_core::error::OpenPxError;
use px_exchange_kalshi::KalshiConfig;
use px_exchange_polymarket::{PolymarketConfig, PolymarketSignatureType};

#[inline]
fn get_str<'a>(obj: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(Value::as_str)
}

#[inline]
fn get_bool(obj: &serde_json::Map<String, Value>, key: &str) -> Option<bool> {
    obj.get(key).and_then(Value::as_bool)
}

pub fn parse_kalshi(config: &Value) -> Result<KalshiConfig, OpenPxError> {
    let obj = match config.as_object() {
        Some(o) => o,
        None => return Ok(KalshiConfig::new()),
    };

    let mut cfg = if get_bool(obj, "demo").unwrap_or(false) {
        KalshiConfig::demo()
    } else {
        KalshiConfig::new()
    };

    if let Some(v) = get_str(obj, "api_url") {
        cfg = cfg.with_api_url(v);
    }
    if let Some(v) = get_str(obj, "api_key_id") {
        cfg = cfg.with_api_key_id(v);
    }
    if let Some(v) = get_str(obj, "private_key_pem") {
        cfg = cfg.with_private_key_pem(v);
    }
    if let Some(v) = get_str(obj, "private_key_path") {
        cfg = cfg.with_private_key_path(v);
    }
    if let Some(v) = get_bool(obj, "verbose") {
        cfg = cfg.with_verbose(v);
    }
    Ok(cfg)
}

pub fn parse_polymarket(config: &Value) -> Result<PolymarketConfig, OpenPxError> {
    let mut cfg = PolymarketConfig::new();
    let obj = match config.as_object() {
        Some(o) => o,
        None => return Ok(cfg),
    };

    if let Some(v) = get_str(obj, "private_key") {
        cfg = cfg.with_private_key(v);
    }
    if let Some(v) = get_str(obj, "funder") {
        cfg = cfg.with_funder(v);
    }

    let explicit_sig_type = obj.get("signature_type").and_then(|v| {
        v.as_str()
            .map(PolymarketSignatureType::from)
            .or_else(|| match v.as_u64()? {
                0 => Some(PolymarketSignatureType::Eoa),
                1 => Some(PolymarketSignatureType::Proxy),
                2 => Some(PolymarketSignatureType::GnosisSafe),
                _ => None,
            })
    });
    let sig_type = explicit_sig_type.unwrap_or_else(|| {
        if cfg.funder.is_some() {
            PolymarketSignatureType::GnosisSafe
        } else {
            PolymarketSignatureType::Eoa
        }
    });
    cfg = cfg.with_signature_type(sig_type);

    if let (Some(key), Some(secret), Some(passphrase)) = (
        get_str(obj, "api_key"),
        get_str(obj, "api_secret"),
        get_str(obj, "api_passphrase"),
    ) {
        cfg = cfg.with_api_credentials(key, secret, passphrase);
    }
    if let Some(v) = get_str(obj, "gamma_url") {
        cfg = cfg.with_gamma_url(v);
    }
    if let Some(v) = get_str(obj, "clob_url") {
        cfg = cfg.with_clob_url(v);
    }
    if let Some(v) = get_str(obj, "polygon_rpc_url") {
        cfg = cfg.with_polygon_rpc(v);
    }
    if let Some(v) = get_bool(obj, "verbose") {
        cfg = cfg.with_verbose(v);
    }
    Ok(cfg)
}
