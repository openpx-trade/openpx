//! Mapping contract test for Polymarket → OpenPX `Market`.
//!
//! Loads `schema/mappings/market.yaml` from the workspace root and the
//! committed Polymarket gamma fixture, mocks `/markets?id=<id>` on the gamma
//! API, calls `fetch_market`, then walks the YAML and asserts the actual
//! code's output matches what each `polymarket:` source claims.
//!
//! The same caveat as Kalshi: this is *behavioral equivalence* against the
//! fixture — sentinel values surface any source-path swap, but the test does
//! not statically prove the code reads a specific path at runtime.

use std::fs;
use std::path::PathBuf;

use px_core::Exchange;
use px_exchange_polymarket::{Polymarket, PolymarketConfig};
use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const REF_PREFIX: &str = "#/components/schemas/Market/properties/";

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.pop();
    p
}

fn load_mapping() -> serde_yaml::Value {
    let p = workspace_root().join("schema/mappings/market.yaml");
    serde_yaml::from_str(&fs::read_to_string(&p).expect("read mapping yaml"))
        .expect("parse mapping yaml")
}

fn load_fixture() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/market.json");
    serde_json::from_str(&fs::read_to_string(&p).expect("read fixture"))
        .expect("parse fixture")
}

fn lookup_in_fixture<'a>(fixture: &'a Value, reference: &str) -> Option<&'a Value> {
    fixture.get(reference.strip_prefix(REF_PREFIX)?)
}

/// Behavioral equivalence between an upstream value (from the fixture) and a
/// unified value (from the serialized openpx Market), modulo the declared
/// transform. Returns `Ok(())` on match or `Err(reason)` on mismatch.
fn check_field(
    transform: &str,
    source: Option<&Value>,
    unified: Option<&Value>,
) -> Result<(), String> {
    let source_present = source.is_some_and(|v| !v.is_null());
    let unified_present = unified.is_some_and(|v| !v.is_null());

    match transform {
        "direct" => match (source, unified) {
            (Some(s), Some(u)) if s == u => Ok(()),
            (Some(s), Some(u)) => match (s.as_f64(), u.as_f64()) {
                (Some(a), Some(b)) if (a - b).abs() < 1e-9 => Ok(()),
                _ => Err(format!("direct mismatch: source={s:?} unified={u:?}")),
            },
            (None, None) => Ok(()),
            _ => Err(format!(
                "direct presence mismatch: source={source:?} unified={unified:?}"
            )),
        },
        "fixed_point_dollars" | "fixed_point_count" | "string_to_f64" => {
            if source_present != unified_present {
                return Err(format!(
                    "{transform} presence mismatch: source={source:?} unified={unified:?}"
                ));
            }
            if !source_present {
                return Ok(());
            }
            let parsed = source
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| source.and_then(|v| v.as_f64()))
                .ok_or_else(|| format!("{transform}: source not parseable: {source:?}"))?;
            let actual = unified
                .and_then(|v| v.as_f64())
                .ok_or_else(|| format!("{transform}: unified not f64: {unified:?}"))?;
            if (parsed - actual).abs() < 1e-9 {
                Ok(())
            } else {
                Err(format!(
                    "{transform} value mismatch: parsed_source={parsed} unified={actual}"
                ))
            }
        }
        "parse_datetime" => {
            if source_present != unified_present {
                return Err(format!(
                    "parse_datetime presence mismatch: source={source:?} unified={unified:?}"
                ));
            }
            if !source_present {
                return Ok(());
            }
            let s = source
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("parse_datetime: source not string: {source:?}"))?;
            let u = unified
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("parse_datetime: unified not string: {unified:?}"))?;
            let parsed_s = chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|e| format!("parse_datetime: source not RFC3339: {e}"))?;
            let parsed_u = chrono::DateTime::parse_from_rfc3339(u)
                .map_err(|e| format!("parse_datetime: unified not RFC3339: {e}"))?;
            if parsed_s.timestamp() == parsed_u.timestamp() {
                Ok(())
            } else {
                Err(format!(
                    "parse_datetime mismatch: source={s} unified={u}"
                ))
            }
        }
        "enum_remap" | "first_non_null" => {
            if source_present && !unified_present {
                Err(format!(
                    "{transform}: source present but unified is null: source={source:?}"
                ))
            } else {
                Ok(())
            }
        }
        other => Err(format!("unknown transform `{other}`")),
    }
}

#[tokio::test]
async fn yaml_contract_for_polymarket_market() {
    let fixture = load_fixture();
    let market_id = fixture
        .get("id")
        .and_then(|v| v.as_str())
        .expect("fixture missing id")
        .to_string();

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/markets"))
        .and(query_param("id", market_id.as_str()))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([fixture.clone()])),
        )
        .mount(&mock_server)
        .await;

    let config = PolymarketConfig::new()
        .with_gamma_url(mock_server.uri())
        .with_verbose(false);
    let exchange = Polymarket::new(config).unwrap();
    let market = exchange
        .fetch_market(&market_id)
        .await
        .expect("fetch_market should succeed against mock");

    let unified = serde_json::to_value(&market).expect("serialize Market");

    let mapping = load_mapping();
    let fields = mapping
        .get("fields")
        .and_then(|v| v.as_sequence())
        .expect("mapping has fields[]");

    let mut violations: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for field in fields {
        let name = match field.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };
        if field.get("synthetic").is_some() && field.get("sources").is_none() {
            continue;
        }
        let src = match field
            .get("sources")
            .and_then(|s| s.get("polymarket"))
        {
            Some(s) => s,
            None => continue,
        };
        if src.get("omitted").and_then(|v| v.as_bool()) == Some(true) {
            continue;
        }
        if src.get("synthetic").is_some() {
            continue;
        }
        let reference = match src.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => {
                violations.push(format!(
                    "[{name}] polymarket entry has no `ref:`, `synthetic:`, or `omitted:`"
                ));
                continue;
            }
        };
        let transform = src
            .get("transform")
            .and_then(|v| v.as_str())
            .unwrap_or("direct");

        let source_value = lookup_in_fixture(&fixture, reference);
        let unified_value = unified.get(name);

        match check_field(transform, source_value, unified_value) {
            Ok(()) => {}
            Err(reason) => {
                violations.push(format!(
                    "[{name}] ref={} transform={transform}: {reason}",
                    reference.strip_prefix(REF_PREFIX).unwrap_or(reference)
                ));
            }
        }
        checked += 1;
    }

    if !violations.is_empty() {
        panic!(
            "{} of {} sourced fields drifted between schema/mappings/market.yaml and the \
             actual Polymarket parse_market — fix the YAML or fix the code:\n  {}",
            violations.len(),
            checked,
            violations.join("\n  ")
        );
    }

    assert!(
        checked > 0,
        "no polymarket-sourced fields exercised — fixture or mapping wrong"
    );
}
