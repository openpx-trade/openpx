//! Shared test harness for the YAML mapping contract.
//!
//! Each exchange has a `tests/mapping_contract.rs` that loads the committed
//! mapping YAML, fetches a market against a mocked upstream, and asserts that
//! the parsed unified `Market` matches what the YAML's `<exchange>:` sources
//! claim.
//!
//! Each per-exchange source declares one of three entry types:
//!   * `type: direct`    — value is taken from upstream at the given `ref`.
//!     The unified value must equal the upstream value (with permissive
//!     numeric coercion: stringified numbers parse to f64, and tiered Kalshi
//!     `price_ranges` arrays collapse to their smallest `step`).
//!   * `type: synthetic` — computed by OpenPX, skipped here (verified by
//!     parser unit tests).
//!   * `type: omitted`   — not exposed upstream, skipped here.
//!
//! Enabled via the `test-support` feature.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

const REF_PREFIX: &str = "#/components/schemas/";

pub fn load_mapping(test_manifest_dir: &str, mapping_name: &str) -> serde_yaml::Value {
    let start = PathBuf::from(test_manifest_dir);
    let mut cur = start.clone();
    loop {
        let candidate = cur.join(format!("schema/mappings/{mapping_name}.yaml"));
        if candidate.is_file() {
            let body = fs::read_to_string(&candidate).expect("read mapping yaml");
            return serde_yaml::from_str(&body).expect("parse mapping yaml");
        }
        if !cur.pop() {
            panic!("no schema/mappings/{mapping_name}.yaml found above {start:?}");
        }
    }
}

/// Resolve a `ref` against the fixture. Pointer-shaped refs (`#/components/...`)
/// are walked as `Type.field` after stripping the prefix; bare names look up
/// `fixture[name]` directly (used for spec-gap fields).
fn lookup_in_fixture<'a>(fixture: &'a Value, reference: &str) -> Option<&'a Value> {
    if let Some(rest) = reference.strip_prefix(REF_PREFIX) {
        // The fixture is a single object whose keys are the upstream
        // schema's property names. We strip "<Schema>/properties/" leaving
        // just the property name.
        let parts: Vec<&str> = rest.split('/').collect();
        // parts: [SchemaName, "properties", field_name, ...]
        if parts.len() >= 3 && parts[1] == "properties" {
            return fixture.get(parts[2]);
        }
        return None;
    }
    fixture.get(reference)
}

fn coerce_to_f64(v: &Value) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        return Some(n);
    }
    if let Some(s) = v.as_str() {
        return s.parse::<f64>().ok();
    }
    None
}

fn check_direct(source: Option<&Value>, unified: Option<&Value>) -> Result<(), String> {
    let source_present = source.is_some_and(|v| !v.is_null());
    let unified_present = unified.is_some_and(|v| !v.is_null());

    if !source_present && !unified_present {
        return Ok(());
    }

    // Tiered Kalshi `price_ranges`: array of {start, end, step} → min step.
    if let Some(Value::Array(rows)) = source {
        let steps: Vec<f64> = rows
            .iter()
            .filter_map(|r| r.get("step")?.as_str().and_then(|s| s.parse::<f64>().ok()))
            .collect();
        if !steps.is_empty() {
            let expected = steps.iter().copied().fold(f64::INFINITY, f64::min);
            let actual = unified
                .and_then(coerce_to_f64)
                .ok_or_else(|| format!("price_ranges: unified not numeric: {unified:?}"))?;
            if (expected - actual).abs() < 1e-9 {
                return Ok(());
            }
            return Err(format!(
                "price_ranges min-step mismatch: expected={expected} unified={actual}"
            ));
        }
    }

    if source_present != unified_present {
        return Err(format!(
            "presence mismatch: source={source:?} unified={unified:?}"
        ));
    }

    let s = source.unwrap();
    let u = unified.unwrap();

    if s == u {
        return Ok(());
    }

    // Numeric coercion: stringified numbers, integer/float, etc.
    if let (Some(a), Some(b)) = (coerce_to_f64(s), coerce_to_f64(u)) {
        if (a - b).abs() < 1e-9 {
            return Ok(());
        }
    }

    // Datetime equivalence (RFC3339 vs unix-seconds, etc.)
    if let (Some(a), Some(b)) = (s.as_str(), u.as_str()) {
        if let (Ok(pa), Ok(pb)) = (
            chrono::DateTime::parse_from_rfc3339(a),
            chrono::DateTime::parse_from_rfc3339(b),
        ) {
            if pa.timestamp() == pb.timestamp() {
                return Ok(());
            }
        }
    }

    Err(format!("direct mismatch: source={s:?} unified={u:?}"))
}

pub fn verify_mapping_contract(
    fixture: &Value,
    unified: &Value,
    mapping: &serde_yaml::Value,
    exchange_key: &str,
) -> (usize, Vec<String>) {
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
        let src = match field.get("sources").and_then(|s| s.get(exchange_key)) {
            Some(s) => s,
            None => continue,
        };
        let ty = src.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match ty {
            "synthetic" | "omitted" => continue,
            "direct" => {}
            other => {
                violations.push(format!("[{name}] {exchange_key} unknown type `{other}`"));
                continue;
            }
        }

        let reference = match src.get("ref").and_then(|v| v.as_str()) {
            Some(r) => r,
            None => {
                violations.push(format!(
                    "[{name}] {exchange_key} type=direct but missing `ref:`"
                ));
                continue;
            }
        };

        let source_value = lookup_in_fixture(fixture, reference);
        let unified_value = unified.get(name);

        if let Err(reason) = check_direct(source_value, unified_value) {
            violations.push(format!("[{name}] ref={reference}: {reason}"));
        }
        checked += 1;
    }

    (checked, violations)
}

pub fn assert_mapping_contract(
    fixture: &Value,
    unified: &Value,
    mapping: &serde_yaml::Value,
    exchange_key: &str,
) {
    let (checked, violations) = verify_mapping_contract(fixture, unified, mapping, exchange_key);
    if !violations.is_empty() {
        panic!(
            "{} of {} direct fields drifted between schema/mappings/ and the actual \
             {exchange_key} parser — fix the YAML or fix the code:\n  {}",
            violations.len(),
            checked,
            violations.join("\n  ")
        );
    }
    assert!(
        checked > 0,
        "no {exchange_key}-sourced fields exercised — fixture or mapping wrong"
    );
}
