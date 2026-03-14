/// Canonical event identity helpers.
///
/// Why this exists:
/// - Exchanges expose different event/group identifiers for the same real-world event.
/// - OpenPX keeps source-native `group_id` for transparency.
/// - OpenPX also exposes canonical `event_id` so SDK users can query events uniformly.
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventAlias {
    pub exchange: &'static str,
    pub group_id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventAliasOwned {
    pub exchange: String,
    pub group_id: String,
}

#[derive(Debug, Clone, Copy)]
struct CanonicalEventEntry {
    canonical_event_id: &'static str,
    aliases: &'static [EventAlias],
}

// Curated registry for explicit cross-exchange links.
// Keep this small and auditable; avoid fuzzy matching mistakes.
// TODO(openpx-events): Add verified Kalshi/Polymarket aliases for high-volume
// shared events as we complete event-by-event adjudication.
// TODO(openpx-market-aliases): `event_id` is event-level only. To power
// cross-exchange "same market" UX (logo stack with clickable jump + live price),
// add a second curated mapping layer for market-level equivalence within an event:
//   (canonical event_id, exchange, market_id|group_item) <-> canonical market key.
const CANONICAL_EVENT_REGISTRY: &[CanonicalEventEntry] = &[CanonicalEventEntry {
    canonical_event_id: "ev:us-pres-election-winner-2028",
    aliases: &[EventAlias {
        exchange: "polymarket",
        group_id: "31552",
    }],
}];

fn normalize_exchange(exchange: &str) -> Cow<'_, str> {
    let trimmed = exchange.trim();
    if trimmed.bytes().all(|b| !b.is_ascii_uppercase()) {
        Cow::Borrowed(trimmed)
    } else {
        Cow::Owned(trimmed.to_ascii_lowercase())
    }
}

/// Stable fallback when no explicit registry mapping exists.
///
/// This preserves a canonical OpenPX shape without inventing risky fuzzy links.
pub fn default_event_id(exchange: &str, group_id: &str) -> Option<String> {
    let exchange = normalize_exchange(exchange);
    let group_id = group_id.trim();
    if exchange.is_empty() || group_id.is_empty() {
        return None;
    }
    Some(format!("ev:{exchange}:{group_id}"))
}

/// Resolve canonical `event_id` from source-native identifiers.
pub fn canonical_event_id(exchange: &str, group_id: &str) -> Option<String> {
    let exchange_norm = normalize_exchange(exchange);
    let group_norm = group_id.trim();
    if exchange_norm.is_empty() || group_norm.is_empty() {
        return None;
    }

    for entry in CANONICAL_EVENT_REGISTRY {
        if entry
            .aliases
            .iter()
            .any(|alias| alias.exchange == exchange_norm && alias.group_id == group_norm)
        {
            return Some(entry.canonical_event_id.to_string());
        }
    }

    default_event_id(&exchange_norm, group_norm)
}

/// Expand canonical `event_id` into source aliases for exchange queries.
///
/// Supports:
/// - Registry-backed canonical IDs (cross-exchange).
/// - Deterministic fallback IDs (`ev:{exchange}:{group_id}`).
pub fn aliases_for_event_id(event_id: &str) -> Vec<EventAliasOwned> {
    let event_id = event_id.trim();
    if event_id.is_empty() {
        return Vec::new();
    }

    if let Some(entry) = CANONICAL_EVENT_REGISTRY
        .iter()
        .find(|entry| entry.canonical_event_id == event_id)
    {
        return entry
            .aliases
            .iter()
            .map(|alias| EventAliasOwned {
                exchange: alias.exchange.to_string(),
                group_id: alias.group_id.to_string(),
            })
            .collect();
    }

    let raw = match event_id.strip_prefix("ev:") {
        Some(v) => v,
        None => return Vec::new(),
    };

    if let Some((exchange, group_id)) = raw.split_once(':') {
        let exchange = normalize_exchange(exchange);
        let group_id = group_id.trim();
        if !exchange.is_empty() && !group_id.is_empty() {
            return vec![EventAliasOwned {
                exchange: exchange.into_owned(),
                group_id: group_id.to_string(),
            }];
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::{aliases_for_event_id, canonical_event_id, default_event_id};

    #[test]
    fn canonical_event_id_uses_registry() {
        let id = canonical_event_id("polymarket", "31552");
        assert_eq!(id.as_deref(), Some("ev:us-pres-election-winner-2028"));
    }

    #[test]
    fn canonical_event_id_falls_back_deterministically() {
        let id = canonical_event_id("kalshi", "KXABC-123");
        assert_eq!(id.as_deref(), Some("ev:kalshi:KXABC-123"));
    }

    #[test]
    fn aliases_expand_registry_and_fallback() {
        let mapped = aliases_for_event_id("ev:us-pres-election-winner-2028");
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].exchange, "polymarket");
        assert_eq!(mapped[0].group_id, "31552");

        let fallback = aliases_for_event_id("ev:kalshi:KXABC-123");
        assert_eq!(fallback.len(), 1);
        assert_eq!(fallback[0].exchange, "kalshi");
        assert_eq!(fallback[0].group_id, "KXABC-123");
    }

    #[test]
    fn default_event_id_validates_inputs() {
        assert_eq!(
            default_event_id("kalshi", "ABC").as_deref(),
            Some("ev:kalshi:ABC")
        );
        assert!(default_event_id("", "ABC").is_none());
        assert!(default_event_id("kalshi", "").is_none());
    }
}
