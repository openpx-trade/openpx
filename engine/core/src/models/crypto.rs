use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum CryptoPriceSource {
    Binance,
    Chainlink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CryptoPrice {
    pub symbol: String,
    pub timestamp: u64,
    pub value: f64,
    pub source: CryptoPriceSource,
}
