use serde::{Deserialize, Serialize};

use super::{Event, Market, Series};

/// A market plus its parent event and series.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarketLineage {
    /// The market itself.
    pub market: Market,
    /// Parent event; `null` if the market is standalone or the parent is missing upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<Event>,
    /// Parent series; `null` if the event is standalone or the parent is missing upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<Series>,
}
