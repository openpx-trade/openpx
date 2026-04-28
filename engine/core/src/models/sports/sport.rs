use serde::{Deserialize, Serialize};

/// A top-level sport category (e.g., football, basketball, hockey).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Sport {
    /// Canonical sport id (lowercase, no spaces). Stable across providers.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
}

/// A specific competition within a sport (e.g., NFL, NBA, English Premier League).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct League {
    /// Canonical league id (lowercase, no spaces). Stable across providers.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Back-reference to the parent `Sport::id`.
    pub sport_id: String,
    /// Common abbreviation (e.g., "NFL", "NBA"). Optional — not every league has one.
    pub abbreviation: Option<String>,
}
