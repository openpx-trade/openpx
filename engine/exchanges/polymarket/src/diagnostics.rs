use alloy::primitives::{Address, Bytes};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, info};

use crate::approvals::PUSD_ADDRESS;
use crate::config::{PolymarketSignatureType, DEFAULT_POLYGON_RPC};
use crate::error::PolymarketError;

// Polymarket SDK utilities to derive proxy/safe wallets.
use polymarket_client_sdk_v2::{derive_proxy_wallet, derive_safe_wallet};

sol! {
    #[sol(rpc)]
    interface IERC20Balance {
        function balanceOf(address owner) external view returns (uint256);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDiagnostics {
    pub eoa: String,
    pub proxy: Option<String>,
    pub safe: Option<String>,
    pub eoa_usdc: Option<String>,
    pub proxy_usdc: Option<String>,
    pub safe_usdc: Option<String>,
    pub proxy_deployed: Option<bool>,
    pub safe_deployed: Option<bool>,
}

fn as_hex(addr: Address) -> String {
    format!("{addr:#x}")
}

fn is_deployed(code: &Bytes) -> bool {
    !code.is_empty()
}

/// Diagnose potential Polymarket wallet mismatches.
///
/// This derives the EOA, Proxy, and Safe wallet addresses and checks on-chain
/// USDC balances + contract deployment status on Polygon.
pub async fn diagnose_wallets(
    private_key: &str,
    chain_id: u64,
    rpc_url: Option<&str>,
) -> Result<WalletDiagnostics, PolymarketError> {
    let signer = PrivateKeySigner::from_str(private_key)
        .map_err(|e| PolymarketError::Config(format!("invalid private key: {e}")))?;
    let eoa = signer.address();

    let proxy = derive_proxy_wallet(eoa, chain_id);
    let safe = derive_safe_wallet(eoa, chain_id);

    let provider = ProviderBuilder::new()
        .connect(rpc_url.unwrap_or(DEFAULT_POLYGON_RPC))
        .await
        .map_err(|e| PolymarketError::Network(format!("failed to connect to RPC: {e}")))?;

    let usdc_addr = Address::from_str(PUSD_ADDRESS)
        .map_err(|e| PolymarketError::Config(format!("invalid pUSD address: {e}")))?;
    let usdc = IERC20Balance::new(usdc_addr, &provider);

    let eoa_usdc = usdc
        .balanceOf(eoa)
        .call()
        .await
        .map_err(|e| PolymarketError::Api(format!("failed to read USDC balance: {e}")))?;

    let proxy_usdc = match proxy {
        Some(addr) => Some(
            usdc.balanceOf(addr)
                .call()
                .await
                .map_err(|e| PolymarketError::Api(format!("failed to read proxy USDC: {e}")))?,
        ),
        None => None,
    };

    let safe_usdc = match safe {
        Some(addr) => Some(
            usdc.balanceOf(addr)
                .call()
                .await
                .map_err(|e| PolymarketError::Api(format!("failed to read safe USDC: {e}")))?,
        ),
        None => None,
    };

    let proxy_deployed = match proxy {
        Some(addr) => {
            let code = provider
                .get_code_at(addr)
                .await
                .map_err(|e| PolymarketError::Api(format!("failed to read proxy code: {e}")))?;
            Some(is_deployed(&code))
        }
        None => None,
    };

    let safe_deployed = match safe {
        Some(addr) => {
            let code = provider
                .get_code_at(addr)
                .await
                .map_err(|e| PolymarketError::Api(format!("failed to read safe code: {e}")))?;
            Some(is_deployed(&code))
        }
        None => None,
    };

    Ok(WalletDiagnostics {
        eoa: as_hex(eoa),
        proxy: proxy.map(as_hex),
        safe: safe.map(as_hex),
        eoa_usdc: Some(eoa_usdc.to_string()),
        proxy_usdc: proxy_usdc.map(|v| v.to_string()),
        safe_usdc: safe_usdc.map(|v| v.to_string()),
        proxy_deployed,
        safe_deployed,
    })
}

/// Result of auto-detecting the proxy wallet type from an EOA private key.
#[derive(Debug, Clone)]
pub struct DetectedWallet {
    pub funder: Option<String>,
    pub signature_type: PolymarketSignatureType,
}

/// Auto-detect the Polymarket proxy wallet from an EOA private key.
///
/// Derives Safe and Proxy wallet addresses via CREATE2, then checks on-chain
/// which is deployed. Returns the detected funder address and signature type.
/// Fault-tolerant: any error falls back to EOA defaults.
pub async fn detect_proxy_wallet(
    private_key: &str,
    chain_id: u64,
    rpc_url: Option<&str>,
) -> DetectedWallet {
    let eoa_default = DetectedWallet {
        funder: None,
        signature_type: PolymarketSignatureType::Eoa,
    };

    let signer = match PrivateKeySigner::from_str(private_key) {
        Ok(s) => s,
        Err(e) => {
            debug!("detect_proxy_wallet: invalid private key: {e}");
            return eoa_default;
        }
    };
    let eoa = signer.address();

    let provider = match ProviderBuilder::new()
        .connect(rpc_url.unwrap_or(DEFAULT_POLYGON_RPC))
        .await
    {
        Ok(p) => p,
        Err(e) => {
            debug!("detect_proxy_wallet: RPC connect failed: {e}");
            return eoa_default;
        }
    };

    // Check Gnosis Safe first (more common for browser-based Polymarket users)
    if let Some(safe_addr) = derive_safe_wallet(eoa, chain_id) {
        match provider.get_code_at(safe_addr).await {
            Ok(code) if is_deployed(&code) => {
                let funder = as_hex(safe_addr);
                info!(
                    eoa = %as_hex(eoa),
                    funder = %funder,
                    "Auto-detected Polymarket GnosisSafe wallet"
                );
                return DetectedWallet {
                    funder: Some(funder),
                    signature_type: PolymarketSignatureType::GnosisSafe,
                };
            }
            Ok(_) => debug!("detect_proxy_wallet: Safe {safe_addr:#x} not deployed"),
            Err(e) => debug!("detect_proxy_wallet: Safe code check failed: {e}"),
        }
    }

    // Check Proxy wallet
    if let Some(proxy_addr) = derive_proxy_wallet(eoa, chain_id) {
        match provider.get_code_at(proxy_addr).await {
            Ok(code) if is_deployed(&code) => {
                let funder = as_hex(proxy_addr);
                info!(
                    eoa = %as_hex(eoa),
                    funder = %funder,
                    "Auto-detected Polymarket Proxy wallet"
                );
                return DetectedWallet {
                    funder: Some(funder),
                    signature_type: PolymarketSignatureType::Proxy,
                };
            }
            Ok(_) => debug!("detect_proxy_wallet: Proxy {proxy_addr:#x} not deployed"),
            Err(e) => debug!("detect_proxy_wallet: Proxy code check failed: {e}"),
        }
    }

    debug!(
        eoa = %as_hex(eoa),
        "No proxy wallet detected, using EOA"
    );
    eoa_default
}
