use px_core::ExchangeConfig;

pub const GAMMA_API_URL: &str = "https://gamma-api.polymarket.com";
pub const CLOB_API_URL: &str = "https://clob.polymarket.com";
pub const DATA_API_URL: &str = "https://data-api.polymarket.com";
pub const DEFAULT_POLYGON_RPC: &str = "https://polygon-bor-rpc.publicnode.com";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PolymarketSignatureType {
    #[default]
    Eoa = 0,
    Proxy = 1,
    GnosisSafe = 2,
}

impl From<PolymarketSignatureType> for u8 {
    fn from(t: PolymarketSignatureType) -> u8 {
        t as u8
    }
}

impl From<&str> for PolymarketSignatureType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "proxy" | "browser" | "poly_proxy" | "1" => Self::Proxy,
            "gnosis" | "gnosissafe" | "gnosis_safe" | "safe" | "poly_gnosis_safe" | "2" => {
                Self::GnosisSafe
            }
            _ => Self::Eoa,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolymarketConfig {
    pub base: ExchangeConfig,
    pub gamma_url: String,
    pub clob_url: String,
    pub data_api_url: String,
    pub private_key: Option<String>,
    /// The funder address (Safe or Proxy wallet). Can be auto-detected from EOA.
    pub funder: Option<String>,
    /// Signature type: Eoa (0), Proxy (1), or GnosisSafe (2). Can be auto-detected.
    pub signature_type: PolymarketSignatureType,
    pub chain_id: u64,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>,
    pub polygon_rpc_url: Option<String>,
    /// Builder API key for affiliate attribution (POLY_BUILDER_API_KEY header)
    pub builder_api_key: Option<String>,
    /// Builder API secret for signing builder headers
    pub builder_secret: Option<String>,
    /// Builder API passphrase (POLY_BUILDER_PASSPHRASE header)
    pub builder_passphrase: Option<String>,
}

impl Default for PolymarketConfig {
    fn default() -> Self {
        Self {
            base: ExchangeConfig {
                rate_limit_per_second: 50, // Polymarket allows ~60/s sustained for orders
                ..ExchangeConfig::default()
            },
            gamma_url: GAMMA_API_URL.into(),
            clob_url: CLOB_API_URL.into(),
            data_api_url: DATA_API_URL.into(),
            private_key: None,
            funder: None,
            signature_type: PolymarketSignatureType::default(),
            chain_id: 137,
            api_key: None,
            api_secret: None,
            api_passphrase: None,
            polygon_rpc_url: None,
            builder_api_key: None,
            builder_secret: None,
            builder_passphrase: None,
        }
    }
}

impl PolymarketConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_private_key(mut self, key: impl Into<String>) -> Self {
        self.private_key = Some(key.into());
        self
    }

    pub fn with_funder(mut self, funder: impl Into<String>) -> Self {
        self.funder = Some(funder.into());
        self
    }

    pub fn with_signature_type(mut self, sig_type: PolymarketSignatureType) -> Self {
        self.signature_type = sig_type;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.base = self.base.with_verbose(verbose);
        self
    }

    pub fn with_gamma_url(mut self, url: impl Into<String>) -> Self {
        self.gamma_url = url.into();
        self
    }

    pub fn with_clob_url(mut self, url: impl Into<String>) -> Self {
        self.clob_url = url.into();
        self
    }

    pub fn with_api_credentials(
        mut self,
        key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        self.api_key = Some(key.into());
        self.api_secret = Some(secret.into());
        self.api_passphrase = Some(passphrase.into());
        self
    }

    pub fn with_polygon_rpc(mut self, url: impl Into<String>) -> Self {
        self.polygon_rpc_url = Some(url.into());
        self
    }

    pub fn with_builder_credentials(
        mut self,
        key: impl Into<String>,
        secret: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        self.builder_api_key = Some(key.into());
        self.builder_secret = Some(secret.into());
        self.builder_passphrase = Some(passphrase.into());
        self
    }

    /// Load builder credentials from POLY_BUILDER_* environment variables if set.
    pub fn with_builder_credentials_from_env(self) -> Self {
        if let (Ok(key), Ok(secret), Ok(passphrase)) = (
            std::env::var("POLY_BUILDER_API_KEY"),
            std::env::var("POLY_BUILDER_SECRET"),
            std::env::var("POLY_BUILDER_PASSPHRASE"),
        ) {
            self.with_builder_credentials(key, secret, passphrase)
        } else {
            self
        }
    }

    pub fn has_api_credentials(&self) -> bool {
        self.api_key.is_some() && self.api_secret.is_some() && self.api_passphrase.is_some()
    }

    pub fn has_builder_credentials(&self) -> bool {
        self.builder_api_key.is_some()
            && self.builder_secret.is_some()
            && self.builder_passphrase.is_some()
    }

    pub fn is_authenticated(&self) -> bool {
        self.private_key.is_some() || self.has_api_credentials()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_type_parses_gnosis_safe_variants() {
        for s in [
            "gnosis",
            "gnosissafe",
            "gnosis_safe",
            "safe",
            "poly_gnosis_safe",
            "2",
            "GNOSIS_SAFE",
        ] {
            assert_eq!(
                PolymarketSignatureType::from(s),
                PolymarketSignatureType::GnosisSafe,
                "expected {s:?} → GnosisSafe"
            );
        }
    }

    #[test]
    fn signature_type_parses_proxy_variants() {
        for s in ["proxy", "browser", "poly_proxy", "1"] {
            assert_eq!(
                PolymarketSignatureType::from(s),
                PolymarketSignatureType::Proxy,
                "expected {s:?} → Proxy"
            );
        }
    }

    #[test]
    fn signature_type_defaults_to_eoa() {
        for s in ["eoa", "0", "", "nonsense"] {
            assert_eq!(
                PolymarketSignatureType::from(s),
                PolymarketSignatureType::Eoa,
                "expected {s:?} → Eoa"
            );
        }
    }
}
