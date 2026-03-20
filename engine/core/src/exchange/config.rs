use std::time::Duration;

/// Filter for market status in fetch queries.
///
/// Unlike `MarketStatus` (which represents a market's actual status), this enum
/// includes an `All` variant for fetching markets regardless of status.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum MarketStatusFilter {
    Active,
    Closed,
    Resolved,
    All,
}

impl std::fmt::Display for MarketStatusFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketStatusFilter::Active => write!(f, "active"),
            MarketStatusFilter::Closed => write!(f, "closed"),
            MarketStatusFilter::Resolved => write!(f, "resolved"),
            MarketStatusFilter::All => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for MarketStatusFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" | "open" => Ok(MarketStatusFilter::Active),
            "closed" | "inactive" | "paused" => Ok(MarketStatusFilter::Closed),
            "resolved" | "settled" | "determined" | "finalized" => Ok(MarketStatusFilter::Resolved),
            "all" => Ok(MarketStatusFilter::All),
            _ => Err(format!("Unknown market status filter: {}", s)),
        }
    }
}

impl From<crate::models::MarketStatus> for MarketStatusFilter {
    fn from(s: crate::models::MarketStatus) -> Self {
        match s {
            crate::models::MarketStatus::Active => MarketStatusFilter::Active,
            crate::models::MarketStatus::Closed => MarketStatusFilter::Closed,
            crate::models::MarketStatus::Resolved => MarketStatusFilter::Resolved,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    pub timeout: Duration,
    pub rate_limit_per_second: u32,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub verbose: bool,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            rate_limit_per_second: 10,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            verbose: false,
        }
    }
}

impl ExchangeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_rate_limit(mut self, requests_per_second: u32) -> Self {
        self.rate_limit_per_second = requests_per_second;
        self
    }

    pub fn with_retries(mut self, max_retries: u32, delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = delay;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FetchMarketsParams {
    pub limit: Option<usize>,
    /// Exchange-specific cursor (offset, page number, or cursor string)
    #[serde(default)]
    pub cursor: Option<String>,
    /// Filter by market status. Defaults to Active at the exchange level when None.
    /// Use `MarketStatusFilter::All` to fetch markets of any status.
    #[serde(default)]
    pub status: Option<MarketStatusFilter>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FetchOrdersParams {
    pub market_id: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct FetchUserActivityParams {
    pub address: String,
    pub limit: Option<usize>,
}

// ============================================================================
// Customer Credentials (for per-customer exchange authentication)
//
// These credential structs hold per-exchange authentication data.
// Users provide their own exchange credentials to trade directly.
// ============================================================================

// TODO(wallet-support): Current wallet support and planned improvements.
//
// **Supported today:**
// - Raw private key + optional funder: covers EOA (sig_type=0), Proxy (sig_type=1),
//   GnosisSafe (sig_type=2). Server signs orders with the private key.
// - CLOB API credentials (api_key, api_secret, api_passphrase): if provided alongside
//   the private key, skips the expensive init_trading() derivation step.
//
// **SDK-side helpers to add (no server changes needed):**
// 1. CLOB credential derivation helper — SDK method that takes a wallet signer, signs a
//    ClobAuth EIP-712 message, calls Polymarket's /auth/derive-api-key, and returns
//    {apiKey, apiSecret, apiPassphrase}. Runs client-side.
//    Useful for both direct traders (automate credential setup) and platform builders
//    (onboard end-users without manual Polymarket UI steps).
//
// 2. Approval/allowance helpers — SDK methods to check and set the 6 Polymarket token
//    approvals (USDC + CTF for CTF Exchange, NegRisk CTF Exchange, NegRisk Adapter).
//    Expose via client.approvals.check() and client.approvals.setAll(). Should also
//    surface clear errors when orders fail due to missing approvals.
//
// **Future server-side additions (lower priority):**
// 3. Pre-signed order endpoint — POST /orders/signed that accepts orders already signed
//    client-side (EIP-712). Enables browser wallets (MetaMask, WalletConnect), hardware
//    wallets (Ledger, Trezor), and Privy embedded wallets to trade without exposing
//    private keys to any server. The server just forwards the pre-signed order to the
//    exchange CLOB. Useful for both Mode A and Mode B.
#[derive(Debug, Clone)]
pub struct PolymarketCredentials {
    pub private_key: Option<String>,
    pub funder: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>,
    pub signature_type: String,
}

impl PolymarketCredentials {
    /// Create credentials from individual field values (e.g., from DynamoDB).
    ///
    /// Auto-detection: If signature_type is not provided:
    /// - funder present → GnosisSafe (type 2)
    /// - funder absent → EOA (type 0)
    pub fn from_fields(
        private_key: Option<String>,
        funder: Option<String>,
        api_key: Option<String>,
        api_secret: Option<String>,
        api_passphrase: Option<String>,
        signature_type: Option<String>,
    ) -> Self {
        // Auto-detect: funder present without explicit type → GnosisSafe
        let resolved_signature_type = signature_type.unwrap_or_else(|| {
            if funder.is_some() {
                "GnosisSafe".to_string()
            } else {
                "EOA".to_string()
            }
        });

        Self {
            private_key,
            funder,
            api_key,
            api_secret,
            api_passphrase,
            signature_type: resolved_signature_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MarketStatus;

    #[test]
    fn market_status_filter_from_str() {
        assert_eq!(
            "active".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::Active
        );
        assert_eq!(
            "open".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::Active
        );
        assert_eq!(
            "closed".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::Closed
        );
        assert_eq!(
            "resolved".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::Resolved
        );
        assert_eq!(
            "settled".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::Resolved
        );
        assert_eq!(
            "all".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::All
        );
        assert_eq!(
            "ALL".parse::<MarketStatusFilter>().unwrap(),
            MarketStatusFilter::All
        );
        assert!("invalid".parse::<MarketStatusFilter>().is_err());
    }

    #[test]
    fn market_status_filter_display() {
        assert_eq!(MarketStatusFilter::Active.to_string(), "active");
        assert_eq!(MarketStatusFilter::Closed.to_string(), "closed");
        assert_eq!(MarketStatusFilter::Resolved.to_string(), "resolved");
        assert_eq!(MarketStatusFilter::All.to_string(), "all");
    }

    #[test]
    fn market_status_filter_from_market_status() {
        assert_eq!(
            MarketStatusFilter::from(MarketStatus::Active),
            MarketStatusFilter::Active
        );
        assert_eq!(
            MarketStatusFilter::from(MarketStatus::Closed),
            MarketStatusFilter::Closed
        );
        assert_eq!(
            MarketStatusFilter::from(MarketStatus::Resolved),
            MarketStatusFilter::Resolved
        );
    }

    #[test]
    fn market_status_filter_serde_roundtrip() {
        let filter = MarketStatusFilter::All;
        let json = serde_json::to_string(&filter).unwrap();
        assert_eq!(json, "\"all\"");
        let parsed: MarketStatusFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MarketStatusFilter::All);
    }

    #[test]
    fn fetch_markets_params_default_status_is_none() {
        let params = FetchMarketsParams::default();
        assert!(params.status.is_none());
    }

    #[test]
    fn fetch_markets_params_serde_with_all_status() {
        let params = FetchMarketsParams {
            status: Some(MarketStatusFilter::All),
            ..Default::default()
        };
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["status"], "all");

        let parsed: FetchMarketsParams = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.status, Some(MarketStatusFilter::All));
    }
}

#[derive(Debug, Clone)]
pub struct KalshiCredentials {
    pub api_key_id: String,
    pub private_key: String,
}

#[derive(Debug, Clone)]
pub struct OpinionCredentials {
    pub api_key: String,
    pub private_key: String,
    pub multi_sig_addr: String,
}
