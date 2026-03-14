use std::time::Duration;

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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FetchMarketsResult {
    pub markets: Vec<crate::Market>,
    /// Next cursor for this exchange (None if no more data)
    pub next_cursor: Option<String>,
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
