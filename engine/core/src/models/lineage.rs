use serde::{Deserialize, Serialize};

use super::{Event, Market, Series};

/// A market plus the parent event and series that contextualize it.
///
/// `event` and `series` are `Option` because:
///   - some markets have no parent event surface (Kalshi standalone markets);
///   - the upstream lookup for the parent may legitimately 404 on a dangling
///     reference. A dangling parent should not fail the whole call — the
///     caller still gets the `Market`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarketLineage {
    pub market: Market,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<Event>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<Series>,
}
