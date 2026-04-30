//! Shared test harness for the YAML mapping contract.
//!
//! Each exchange has a `tests/mapping_contract.rs` that loads the committed
//! mapping YAML, fetches a market against a mocked upstream, and asserts that
//! the parsed unified `Market` matches what the YAML's `<exchange>:` sources
//! claim. The walk + transform vocabulary is identical across exchanges; this
//! module is the single source of truth for both, so kalshi and polymarket
//! cannot drift on what `parse_datetime` (or any other transform) means.
//!
//! Enabled via the `test-support` feature, which exchanges turn on as a
//! dev-dependency only. Not compiled into release builds.
//!
//! Caveat — same as the per-exchange tests: this verifies *behavioral
//! equivalence* against a real fixture (the unified value at field `name`
//! equals the fixture value at the declared `$ref` after the declared
//! transform). It does not statically prove the code reads exactly that
//! `$ref` at runtime; sentinel values in the fixture make any source-path
//! swap detectable in practice.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

const REF_PREFIX: &str = "#/components/schemas/Market/properties/";

/// Walk up from `test_manifest_dir` until a `schema/mappings/<name>.yaml` is
/// found, then load and parse it. The argument is typically
/// `env!("CARGO_MANIFEST_DIR")` from the calling test crate.
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

fn lookup_in_fixture<'a>(fixture: &'a Value, reference: &str) -> Option<&'a Value> {
    fixture.get(reference.strip_prefix(REF_PREFIX)?)
}

fn check_field(
    transform: &str,
    source: Option<&Value>,
    unified: Option<&Value>,
) -> Result<(), String> {
    let source_present = source.is_some_and(|v| !v.is_null());
    let unified_present = unified.is_some_and(|v| !v.is_null());

    match transform {
        "direct" => {
            // Treat JSON null as absent on either side.
            if !source_present && !unified_present {
                return Ok(());
            }
            if source_present != unified_present {
                return Err(format!(
                    "direct presence mismatch: source={source:?} unified={unified:?}"
                ));
            }
            let s = source.unwrap();
            let u = unified.unwrap();
            if s == u {
                return Ok(());
            }
            match (s.as_f64(), u.as_f64()) {
                (Some(a), Some(b)) if (a - b).abs() < 1e-9 => Ok(()),
                _ => Err(format!("direct mismatch: source={s:?} unified={u:?}")),
            }
        }
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
                Err(format!("parse_datetime mismatch: source={s} unified={u}"))
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
        "tick_size" => {
            // Source is the kalshi `price_ranges` array of {start, end, step}
            // strings. Unified is a single number — the smallest step across
            // all tiers. Presence-only on absence; on presence, verify the
            // unified value matches min(parsed steps).
            let source_steps: Vec<f64> = match source {
                Some(Value::Array(a)) => a
                    .iter()
                    .filter_map(|r| r.get("step")?.as_str().and_then(|s| s.parse::<f64>().ok()))
                    .collect(),
                _ => vec![],
            };
            let source_meaningful = !source_steps.is_empty();
            if source_meaningful && !unified_present {
                return Err(format!(
                    "tick_size: source present but unified is null: source={source:?}"
                ));
            }
            if !source_meaningful && unified_present {
                return Err(format!(
                    "tick_size: source absent but unified set: unified={unified:?}"
                ));
            }
            if source_meaningful {
                let expected = source_steps.iter().copied().fold(f64::INFINITY, f64::min);
                let actual = unified
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| format!("tick_size: unified not f64: {unified:?}"))?;
                if (expected - actual).abs() > 1e-9 {
                    return Err(format!(
                        "tick_size value mismatch: expected_min_step={expected} unified={actual}"
                    ));
                }
            }
            Ok(())
        }
        other => Err(format!("unknown transform `{other}`")),
    }
}

/// Walk a mapping YAML and verify every `<exchange_key>:`-sourced field's
/// value in `unified` matches the value at the declared `$ref` in `fixture`,
/// modulo the declared transform.
///
/// Returns `(checked, violations)`. Use [`assert_mapping_contract`] if you
/// just want to panic on any drift.
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
        if field.get("synthetic").is_some() && field.get("sources").is_none() {
            continue;
        }
        let src = match field.get("sources").and_then(|s| s.get(exchange_key)) {
            Some(s) => s,
            None => continue,
        };
        if src.get("omitted").and_then(|v| v.as_bool()) == Some(true) {
            continue;
        }
        if src.get("synthetic").is_some() {
            continue;
        }
        // `ref_unspecced` = direct read of a live-response field the upstream
        // OpenAPI doesn't document on this schema. Look up the field at the
        // top level of the fixture and run the declared transform — same
        // behavioral check as `ref:`, just without spec resolution.
        let (lookup_path, source_value) =
            if let Some(field) = src.get("ref_unspecced").and_then(|v| v.as_str()) {
                (field.to_string(), fixture.get(field))
            } else {
                let reference = match src.get("ref").and_then(|v| v.as_str()) {
                    Some(r) => r,
                    None => {
                        violations.push(format!(
                            "[{name}] {exchange_key} entry has no `ref:`, \
                             `ref_unspecced:`, `synthetic:`, or `omitted:`"
                        ));
                        continue;
                    }
                };
                (
                    reference
                        .strip_prefix(REF_PREFIX)
                        .unwrap_or(reference)
                        .to_string(),
                    lookup_in_fixture(fixture, reference),
                )
            };
        let transform = src
            .get("transform")
            .and_then(|v| v.as_str())
            .unwrap_or("direct");

        let unified_value = unified.get(name);

        if let Err(reason) = check_field(transform, source_value, unified_value) {
            violations.push(format!(
                "[{name}] ref={lookup_path} transform={transform}: {reason}"
            ));
        }
        checked += 1;
    }

    (checked, violations)
}

/// Run [`verify_mapping_contract`] and panic with a structured error on any
/// violation. Also panics if no fields were exercised at all (which would
/// usually mean the fixture or mapping is wrong).
pub fn assert_mapping_contract(
    fixture: &Value,
    unified: &Value,
    mapping: &serde_yaml::Value,
    exchange_key: &str,
) {
    let (checked, violations) = verify_mapping_contract(fixture, unified, mapping, exchange_key);
    if !violations.is_empty() {
        panic!(
            "{} of {} sourced fields drifted between schema/mappings/market.yaml and the \
             actual {exchange_key} parse_market — fix the YAML or fix the code:\n  {}",
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
