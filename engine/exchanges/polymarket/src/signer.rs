use async_trait::async_trait;

use crate::error::PolymarketError;

/// External signer trait for signing EIP-712 typed data via remote services (e.g., Privy).
/// This allows Polymarket order signing without exposing raw private keys.
/// Uses `async_trait` because it is used as `dyn ExternalSigner`.
#[async_trait]
pub trait ExternalSigner: Send + Sync {
    /// Sign EIP-712 typed data, returning hex-encoded signature.
    async fn sign_typed_data(
        &self,
        typed_data: &serde_json::Value,
    ) -> Result<String, PolymarketError>;

    /// The wallet address this signer controls.
    fn address(&self) -> &str;
}
