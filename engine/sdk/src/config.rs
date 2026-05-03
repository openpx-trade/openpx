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
    // The Polymarket SDK rejects `funder + Eoa` at .authenticate() time with
    // a cryptic "Cannot have a funder address with a Eoa signature type" —
    // this combo is a common misconfiguration for MetaMask users whose
    // funder is an auto-deployed Safe. Override silently with a warning so
    // the user finds out at construction (visible) instead of at first
    // create_order (where it surfaces as a confusing post-failure).
    let sig_type = match (explicit_sig_type, cfg.funder.is_some()) {
        (Some(PolymarketSignatureType::Eoa), true) => {
            tracing::warn!(
                "POLYMARKET_SIGNATURE_TYPE=eoa is invalid when POLYMARKET_FUNDER \
                 is set (Polymarket rejects this combo); overriding to gnosis_safe"
            );
            PolymarketSignatureType::GnosisSafe
        }
        (Some(t), _) => t,
        (None, true) => PolymarketSignatureType::GnosisSafe,
        (None, false) => PolymarketSignatureType::Eoa,
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn polymarket_no_funder_no_explicit_sig_type_defaults_to_eoa() {
        let cfg = parse_polymarket(&json!({"private_key": "0xabc"})).unwrap();
        assert_eq!(cfg.signature_type, PolymarketSignatureType::Eoa);
    }

    #[test]
    fn polymarket_funder_no_explicit_sig_type_defaults_to_gnosis_safe() {
        let cfg = parse_polymarket(&json!({
            "private_key": "0xabc",
            "funder": "0xd0d996b2c32d147b3e6c9e140f5c10fde68afab2",
        }))
        .unwrap();
        assert_eq!(cfg.signature_type, PolymarketSignatureType::GnosisSafe);
    }

    #[test]
    fn polymarket_explicit_sig_type_honored_when_no_funder() {
        let cfg = parse_polymarket(&json!({
            "private_key": "0xabc",
            "signature_type": "eoa",
        }))
        .unwrap();
        assert_eq!(cfg.signature_type, PolymarketSignatureType::Eoa);

        let cfg = parse_polymarket(&json!({
            "private_key": "0xabc",
            "signature_type": "poly_proxy",
        }))
        .unwrap();
        assert_eq!(cfg.signature_type, PolymarketSignatureType::Proxy);
    }

    #[test]
    fn polymarket_explicit_eoa_with_funder_overridden_to_gnosis_safe() {
        // The Polymarket SDK rejects this combo at .authenticate() time
        // with a confusing error — parse_polymarket should silently fix it.
        let cfg = parse_polymarket(&json!({
            "private_key": "0xabc",
            "funder": "0xd0d996b2c32d147b3e6c9e140f5c10fde68afab2",
            "signature_type": "eoa",
        }))
        .unwrap();
        assert_eq!(cfg.signature_type, PolymarketSignatureType::GnosisSafe);
    }

    #[test]
    fn polymarket_explicit_proxy_with_funder_honored() {
        // Proxy + funder is valid — don't override.
        let cfg = parse_polymarket(&json!({
            "private_key": "0xabc",
            "funder": "0xd0d996b2c32d147b3e6c9e140f5c10fde68afab2",
            "signature_type": "poly_proxy",
        }))
        .unwrap();
        assert_eq!(cfg.signature_type, PolymarketSignatureType::Proxy);
    }
}
