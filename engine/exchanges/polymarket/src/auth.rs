//! Polymarket Authentication Module
//!
//! Handles L1 (wallet signing) and L2 (API credentials) authentication.
//! Supports EOA wallets and Gnosis Safe proxy wallets.

use crate::config::PolymarketSignatureType;
use crate::error::PolymarketError;

/// Configuration for Polymarket authentication
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub private_key: String,
    pub funder: Option<String>,
    pub signature_type: PolymarketSignatureType,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>,
}

impl AuthConfig {
    /// Create auth config with auto-detection of signature type
    ///
    /// If signature_type is not provided:
    /// - funder present → GnosisSafe (type 2)
    /// - funder absent → EOA (type 0)
    pub fn new(
        private_key: String,
        funder: Option<String>,
        signature_type: Option<PolymarketSignatureType>,
    ) -> Self {
        let resolved_type = signature_type.unwrap_or_else(|| {
            if funder.is_some() {
                PolymarketSignatureType::GnosisSafe
            } else {
                PolymarketSignatureType::Eoa
            }
        });

        Self {
            private_key,
            funder,
            signature_type: resolved_type,
            api_key: None,
            api_secret: None,
            api_passphrase: None,
        }
    }

    /// Create with pre-derived API credentials
    pub fn with_api_credentials(mut self, key: String, secret: String, passphrase: String) -> Self {
        self.api_key = Some(key);
        self.api_secret = Some(secret);
        self.api_passphrase = Some(passphrase);
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), PolymarketError> {
        match self.signature_type {
            PolymarketSignatureType::Eoa if self.funder.is_some() => Err(PolymarketError::Config(
                "EOA signature type cannot have a funder. \
                 Use GnosisSafe (2) for proxy wallets, or remove the funder."
                    .into(),
            )),
            _ => Ok(()),
        }
    }
}

/// Detect signature type from environment variables
///
/// Priority:
/// 1. Explicit POLYMARKET_SIGNATURE_TYPE if set (non-empty)
/// 2. Auto-detect based on funder presence (POLYMARKET_FUNDER or PROXY_ADDRESS)
pub fn detect_signature_type_from_env() -> PolymarketSignatureType {
    use std::env;

    // Check for explicit signature type first (non-empty value)
    if let Ok(sig_type_str) = env::var("POLYMARKET_SIGNATURE_TYPE") {
        let trimmed = sig_type_str.trim();
        if !trimmed.is_empty() {
            return PolymarketSignatureType::from(trimmed);
        }
    }

    // Auto-detect based on funder presence
    let has_funder = env::var("POLYMARKET_FUNDER").is_ok() || env::var("PROXY_ADDRESS").is_ok();

    if has_funder {
        PolymarketSignatureType::GnosisSafe
    } else {
        PolymarketSignatureType::Eoa
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_auto_detect_eoa() {
        let config = AuthConfig::new("0xprivatekey".into(), None, None);
        assert_eq!(config.signature_type, PolymarketSignatureType::Eoa);
    }

    #[test]
    fn test_auth_config_auto_detect_gnosis_safe() {
        let config = AuthConfig::new("0xprivatekey".into(), Some("0xfunder".into()), None);
        assert_eq!(config.signature_type, PolymarketSignatureType::GnosisSafe);
    }

    #[test]
    fn test_auth_config_explicit_override() {
        let config = AuthConfig::new(
            "0xprivatekey".into(),
            Some("0xfunder".into()),
            Some(PolymarketSignatureType::Proxy),
        );
        assert_eq!(config.signature_type, PolymarketSignatureType::Proxy);
    }

    #[test]
    fn test_validate_eoa_with_funder_fails() {
        let config = AuthConfig {
            private_key: "0xkey".into(),
            funder: Some("0xfunder".into()),
            signature_type: PolymarketSignatureType::Eoa,
            api_key: None,
            api_secret: None,
            api_passphrase: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_gnosis_safe_with_funder_ok() {
        let config = AuthConfig::new("0xkey".into(), Some("0xfunder".into()), None);
        assert!(config.validate().is_ok());
    }
}
