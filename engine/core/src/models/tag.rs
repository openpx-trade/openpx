use serde::{Deserialize, Serialize};

/// A category or tag attached to a market or event. Used for filtering and
/// search. Kalshi exposes tags via category-keyed lookups; Polymarket exposes
/// per-market and per-event tag arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}
