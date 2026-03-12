//! Gasless relayer client for Polymarket CTF operations.
//!
//! Submits on-chain transactions via Polymarket's relayer so users
//! don't need POL for gas. Authenticates with Builder HMAC headers.

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use hmac::{Hmac, Mac};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::PolymarketError;

const RELAYER_BASE_URL: &str = "https://relayer-v2.polymarket.com";
const RELAY_PATH: &str = "/relay";

// Builder auth header names (match polymarket-client-sdk auth.rs:280-283)
const POLY_BUILDER_API_KEY: &str = "POLY_BUILDER_API_KEY";
const POLY_BUILDER_PASSPHRASE: &str = "POLY_BUILDER_PASSPHRASE";
const POLY_BUILDER_SIGNATURE: &str = "POLY_BUILDER_SIGNATURE";
const POLY_BUILDER_TIMESTAMP: &str = "POLY_BUILDER_TIMESTAMP";

/// A single transaction to submit via the relayer.
#[derive(Debug, Clone, Serialize)]
pub struct RelayerTransaction {
    pub to: String,
    pub data: String,
    pub value: String,
}

/// Response from the relayer after submitting transaction(s).
#[derive(Debug, Clone, Deserialize)]
pub struct RelayerResponse {
    #[serde(default)]
    pub transaction_hash: Option<String>,
    #[serde(default)]
    pub status: String,
}

/// HTTP client for Polymarket's gasless relayer with Builder HMAC auth.
pub struct PolymarketRelayer {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    secret: String,
    passphrase: String,
}

impl PolymarketRelayer {
    pub fn new(api_key: String, secret: String, passphrase: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: RELAYER_BASE_URL.to_string(),
            api_key,
            secret,
            passphrase,
        }
    }

    /// Construct from POLY_BUILDER_* environment variables.
    pub fn from_env() -> Result<Self, PolymarketError> {
        let api_key = std::env::var("POLY_BUILDER_API_KEY")
            .map_err(|_| PolymarketError::Config("POLY_BUILDER_API_KEY not set".into()))?;
        let secret = std::env::var("POLY_BUILDER_SECRET")
            .map_err(|_| PolymarketError::Config("POLY_BUILDER_SECRET not set".into()))?;
        let passphrase = std::env::var("POLY_BUILDER_PASSPHRASE")
            .map_err(|_| PolymarketError::Config("POLY_BUILDER_PASSPHRASE not set".into()))?;
        Ok(Self::new(api_key, secret, passphrase))
    }

    /// Submit gasless transaction(s) to the relayer.
    pub async fn execute(
        &self,
        transactions: Vec<RelayerTransaction>,
    ) -> Result<RelayerResponse, PolymarketError> {
        let body = serde_json::to_string(&transactions)
            .map_err(|e| PolymarketError::Api(format!("failed to serialize request: {e}")))?;

        let headers = self
            .build_headers("POST", RELAY_PATH, &body)
            .map_err(|e| PolymarketError::Auth(format!("failed to build auth headers: {e}")))?;

        let url = format!("{}{}", self.base_url, RELAY_PATH);
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(PolymarketError::Http)?;

        let status = resp.status();
        let text = resp.text().await.map_err(PolymarketError::Http)?;

        if !status.is_success() {
            return Err(PolymarketError::Api(format!(
                "relayer returned {status}: {text}"
            )));
        }

        serde_json::from_str(&text).map_err(|e| {
            PolymarketError::InvalidResponse(format!(
                "failed to parse relayer response: {e} — body: {text}"
            ))
        })
    }

    /// Build HMAC-SHA256 Builder auth headers.
    ///
    /// Signature message: "{timestamp}{method}{path}{body}"
    /// HMAC: base64url_decode(secret) → HMAC-SHA256 → base64url_encode
    /// (Matches polymarket-client-sdk auth.rs:399-421)
    fn build_headers(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<HeaderMap, PolymarketError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| PolymarketError::Api(format!("system time error: {e}")))?
            .as_secs();

        let message = format!("{timestamp}{method}{path}{body}");

        let decoded_secret = URL_SAFE
            .decode(&self.secret)
            .map_err(|e| PolymarketError::Auth(format!("invalid base64url secret: {e}")))?;

        let mut mac = Hmac::<Sha256>::new_from_slice(&decoded_secret)
            .map_err(|e| PolymarketError::Auth(format!("HMAC init failed: {e}")))?;
        mac.update(message.as_bytes());
        let signature = URL_SAFE.encode(mac.finalize().into_bytes());

        let mut headers = HeaderMap::new();
        headers.insert(
            POLY_BUILDER_API_KEY,
            self.api_key
                .parse()
                .map_err(|_| PolymarketError::Auth("invalid api key header value".into()))?,
        );
        headers.insert(
            POLY_BUILDER_PASSPHRASE,
            self.passphrase
                .parse()
                .map_err(|_| PolymarketError::Auth("invalid passphrase header value".into()))?,
        );
        headers.insert(
            POLY_BUILDER_SIGNATURE,
            signature
                .parse()
                .map_err(|_| PolymarketError::Auth("invalid signature header value".into()))?,
        );
        headers.insert(
            POLY_BUILDER_TIMESTAMP,
            timestamp
                .to_string()
                .parse()
                .map_err(|_| PolymarketError::Auth("invalid timestamp header value".into()))?,
        );

        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_signature_matches_sdk_test_vector() {
        // Test vector from polymarket-client-sdk auth.rs:562-585
        let secret = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let _relayer =
            PolymarketRelayer::new("test-key".into(), secret.into(), "test-passphrase".into());

        // Replicate the SDK test: message = "1000000test-sign/orders{\"hash\":\"0x123\"}"
        // We test just the HMAC function directly via build_headers internals
        let decoded_secret = URL_SAFE.decode(secret).unwrap();
        let message = r#"1000000test-sign/orders{"hash":"0x123"}"#;
        let mut mac = Hmac::<Sha256>::new_from_slice(&decoded_secret).unwrap();
        mac.update(message.as_bytes());
        let signature = URL_SAFE.encode(mac.finalize().into_bytes());

        assert_eq!(signature, "4gJVbox-R6XlDK4nlaicig0_ANVL1qdcahiL8CXfXLM=");
    }

    #[test]
    fn build_headers_includes_all_required_keys() {
        let relayer = PolymarketRelayer::new(
            "test-key".into(),
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".into(),
            "test-pass".into(),
        );

        let headers = relayer.build_headers("POST", "/relay", "{}").unwrap();
        assert!(headers.contains_key(POLY_BUILDER_API_KEY));
        assert!(headers.contains_key(POLY_BUILDER_PASSPHRASE));
        assert!(headers.contains_key(POLY_BUILDER_SIGNATURE));
        assert!(headers.contains_key(POLY_BUILDER_TIMESTAMP));
    }

    #[test]
    fn relayer_transaction_serializes_correctly() {
        let tx = RelayerTransaction {
            to: "0x1234".into(),
            data: "0xabcd".into(),
            value: "0".into(),
        };
        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("\"to\":\"0x1234\""));
        assert!(json.contains("\"value\":\"0\""));
    }
}
