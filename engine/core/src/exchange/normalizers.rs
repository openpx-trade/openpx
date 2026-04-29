use chrono::{DateTime, Utc};
use serde_json::Value;

/// Safe type coercion to i64 with null fallback (never panic).
pub fn coerce_to_int(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s
            .trim()
            .parse::<i64>()
            .ok()
            .or_else(|| s.trim().parse::<f64>().ok().map(|f| f as i64)),
        Value::Bool(b) => Some(if *b { 1 } else { 0 }),
        _ => None,
    }
}

/// Safe type coercion to f64 with null fallback (never panic).
pub fn coerce_to_float(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Safe type coercion to string.
pub fn coerce_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Parse an ISO8601 / RFC3339 string into `DateTime<Utc>`.
/// Falls back through a few common variants seen in the wild.
pub fn coerce_iso8601_datetime(value: &Value) -> Option<DateTime<Utc>> {
    let s = value.as_str()?;
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|ndt| ndt.and_utc())
        })
}

/// Parse a Unix-seconds integer into `DateTime<Utc>`.
pub fn coerce_unix_secs_datetime(value: &Value) -> Option<DateTime<Utc>> {
    let ts = coerce_to_int(value)?;
    DateTime::from_timestamp(ts, 0)
}

/// Parse a Unix-milliseconds integer into `DateTime<Utc>`.
pub fn coerce_unix_millis_datetime(value: &Value) -> Option<DateTime<Utc>> {
    let ts = coerce_to_int(value)?;
    DateTime::from_timestamp_millis(ts)
}

/// Extract value from JSON using a dot-notation path.
/// Supports array indexing with numeric indices (e.g., "events.0.id").
pub fn get_nested<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }

    let mut current = value;
    for part in path.split('.') {
        current = if let Ok(index) = part.parse::<usize>() {
            current.get(index)?
        } else {
            current.get(part)?
        };
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use serde_json::json;

    #[test]
    fn test_coerce_to_int() {
        assert_eq!(coerce_to_int(&json!(42)), Some(42));
        assert_eq!(coerce_to_int(&json!(42.7)), Some(42));
        assert_eq!(coerce_to_int(&json!("123")), Some(123));
        assert_eq!(coerce_to_int(&json!("123.5")), Some(123));
        assert_eq!(coerce_to_int(&json!(true)), Some(1));
        assert_eq!(coerce_to_int(&json!(null)), None);
        assert_eq!(coerce_to_int(&json!("not a number")), None);
    }

    #[test]
    fn test_coerce_to_float() {
        assert_eq!(coerce_to_float(&json!(42.5)), Some(42.5));
        assert_eq!(coerce_to_float(&json!(42)), Some(42.0));
        assert_eq!(coerce_to_float(&json!("123.5")), Some(123.5));
        assert_eq!(coerce_to_float(&json!(null)), None);
    }

    #[test]
    fn test_get_nested() {
        let data = json!({
            "events": [
                {"id": "event1"},
                {"id": "event2"}
            ],
            "metadata": {
                "status": "active"
            }
        });

        assert_eq!(
            get_nested(&data, "events.0.id").and_then(|v| v.as_str()),
            Some("event1")
        );
        assert_eq!(
            get_nested(&data, "metadata.status").and_then(|v| v.as_str()),
            Some("active")
        );
        assert!(get_nested(&data, "nonexistent").is_none());
    }

    #[test]
    fn test_coerce_iso8601_datetime() {
        let value = json!("2024-12-31T23:59:59Z");
        let dt = coerce_iso8601_datetime(&value);
        assert!(dt.is_some());
        assert_eq!(dt.unwrap().year(), 2024);
    }

    #[test]
    fn test_coerce_unix_secs_datetime() {
        let value = json!(1704067199);
        let dt = coerce_unix_secs_datetime(&value);
        assert!(dt.is_some());
    }

    #[test]
    fn test_coerce_unix_millis_datetime() {
        let value = json!(1704067199000_i64);
        let dt = coerce_unix_millis_datetime(&value);
        assert!(dt.is_some());
    }
}
