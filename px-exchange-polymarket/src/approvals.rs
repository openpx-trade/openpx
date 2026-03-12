//! Token approval functionality for Polymarket trading.
//!
//! Users must approve USDC (ERC-20) and CTF tokens (ERC-1155) for Polymarket
//! exchange contracts before trading.
//!
//! Supports both EOA wallets (direct approval) and Gnosis Safe wallets
//! (approval executed through Safe's execTransaction).

// Allow too_many_arguments for generated Gnosis Safe ABI (execTransaction has 10 params)
#![allow(clippy::too_many_arguments)]

use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use alloy::sol_types::SolCall;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::config::DEFAULT_POLYGON_RPC;
use crate::error::PolymarketError;

// Contract addresses on Polygon Mainnet
pub const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
pub const CTF_ADDRESS: &str = "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045";
pub const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
pub const NEG_RISK_CTF_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
pub const NEG_RISK_ADAPTER: &str = "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296";

// Max approval amount (2^256 - 1)
pub const MAX_APPROVAL: &str =
    "115792089237316195423570985008687907853269984665640564039457584007913129639935";

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC1155 {
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address account, address operator) external view returns (bool);
    }

    #[sol(rpc)]
    interface IGnosisSafe {
        function execTransaction(
            address to,
            uint256 value,
            bytes calldata data,
            uint8 operation,
            uint256 safeTxGas,
            uint256 baseGas,
            uint256 gasPrice,
            address gasToken,
            address payable refundReceiver,
            bytes memory signatures
        ) external payable returns (bool success);

        function nonce() external view returns (uint256);
    }
}

/// Target contract for approval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalTarget {
    CtfExchange,
    NegRiskCtfExchange,
    NegRiskAdapter,
}

impl ApprovalTarget {
    pub fn address(&self) -> &'static str {
        match self {
            Self::CtfExchange => CTF_EXCHANGE,
            Self::NegRiskCtfExchange => NEG_RISK_CTF_EXCHANGE,
            Self::NegRiskAdapter => NEG_RISK_ADAPTER,
        }
    }

    pub fn all() -> [Self; 3] {
        [
            Self::CtfExchange,
            Self::NegRiskCtfExchange,
            Self::NegRiskAdapter,
        ]
    }
}

impl std::fmt::Display for ApprovalTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CtfExchange => write!(f, "ctf_exchange"),
            Self::NegRiskCtfExchange => write!(f, "neg_risk_ctf_exchange"),
            Self::NegRiskAdapter => write!(f, "neg_risk_adapter"),
        }
    }
}

/// Token type for approval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Usdc,
    Ctf,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usdc => write!(f, "usdc"),
            Self::Ctf => write!(f, "ctf"),
        }
    }
}

/// Status of a single allowance check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowanceStatus {
    pub token: TokenType,
    pub target: ApprovalTarget,
    pub approved: bool,
    pub details: String,
}

/// Request for setting approvals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Approve all tokens for all contracts
    #[serde(default)]
    pub all: bool,
    /// Approve USDC for all contracts
    #[serde(default)]
    pub usdc: bool,
    /// Approve CTF for all contracts
    #[serde(default)]
    pub ctf: bool,
    /// Approve USDC only for neg-risk contracts
    #[serde(default)]
    pub usdc_neg_risk: bool,
    /// Approve CTF only for neg-risk contracts
    #[serde(default)]
    pub ctf_neg_risk: bool,
}

impl ApprovalRequest {
    pub fn all() -> Self {
        Self {
            all: true,
            ..Default::default()
        }
    }

    /// Returns the list of (token, target) pairs to approve
    pub fn to_approval_pairs(&self) -> Vec<(TokenType, ApprovalTarget)> {
        let mut pairs = Vec::new();

        if self.all {
            for target in ApprovalTarget::all() {
                pairs.push((TokenType::Usdc, target));
                pairs.push((TokenType::Ctf, target));
            }
            return pairs;
        }

        if self.usdc {
            for target in ApprovalTarget::all() {
                pairs.push((TokenType::Usdc, target));
            }
        }

        if self.ctf {
            for target in ApprovalTarget::all() {
                pairs.push((TokenType::Ctf, target));
            }
        }

        if self.usdc_neg_risk {
            pairs.push((TokenType::Usdc, ApprovalTarget::NegRiskCtfExchange));
            pairs.push((TokenType::Usdc, ApprovalTarget::NegRiskAdapter));
        }

        if self.ctf_neg_risk {
            pairs.push((TokenType::Ctf, ApprovalTarget::NegRiskCtfExchange));
            pairs.push((TokenType::Ctf, ApprovalTarget::NegRiskAdapter));
        }

        // Deduplicate
        pairs.sort_by_key(|(t, a)| (format!("{t}"), format!("{a}")));
        pairs.dedup();
        pairs
    }
}

/// Build ABI-encoded calldata for an approval pair. Returns (to_address, hex_calldata).
/// Used by the link handler to build transactions for Privy-managed wallets.
pub fn encode_approval_calldata(token: &TokenType, target: &ApprovalTarget) -> (String, String) {
    let spender = Address::from_str(target.address()).expect("valid target address");

    match token {
        TokenType::Usdc => {
            let call = IERC20::approveCall {
                spender,
                amount: U256::MAX,
            };
            let calldata = alloy::primitives::hex::encode(call.abi_encode());
            (USDC_ADDRESS.to_string(), format!("0x{calldata}"))
        }
        TokenType::Ctf => {
            let call = IERC1155::setApprovalForAllCall {
                operator: spender,
                approved: true,
            };
            let calldata = alloy::primitives::hex::encode(call.abi_encode());
            (CTF_ADDRESS.to_string(), format!("0x{calldata}"))
        }
    }
}

/// Encode USDC ERC-20 approve(spender, MAX) for an arbitrary address.
/// Used to approve the fee escrow contract during onboarding.
pub fn encode_usdc_approval(spender_address: &str) -> (String, String) {
    let spender = Address::from_str(spender_address).expect("valid spender address");
    let call = IERC20::approveCall {
        spender,
        amount: U256::MAX,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (USDC_ADDRESS.to_string(), format!("0x{calldata}"))
}

/// Result of a single approval transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResult {
    pub token: TokenType,
    pub target: ApprovalTarget,
    pub tx_hash: Option<String>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response from approval operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub results: Vec<ApprovalResult>,
    pub all_succeeded: bool,
}

/// Token approver for Polymarket contracts
pub struct TokenApprover {
    rpc_url: String,
}

impl TokenApprover {
    pub fn new(rpc_url: Option<&str>) -> Self {
        Self {
            rpc_url: rpc_url.unwrap_or(DEFAULT_POLYGON_RPC).to_string(),
        }
    }

    /// Check all allowances for a wallet address
    pub async fn check_allowances(
        &self,
        owner: Address,
    ) -> Result<Vec<AllowanceStatus>, PolymarketError> {
        let provider = ProviderBuilder::new()
            .connect(&self.rpc_url)
            .await
            .map_err(|e| PolymarketError::Network(format!("failed to connect to RPC: {e}")))?;

        let usdc_addr = Address::from_str(USDC_ADDRESS)
            .map_err(|e| PolymarketError::Config(format!("invalid USDC address: {e}")))?;
        let ctf_addr = Address::from_str(CTF_ADDRESS)
            .map_err(|e| PolymarketError::Config(format!("invalid CTF address: {e}")))?;

        let usdc = IERC20::new(usdc_addr, &provider);
        let ctf = IERC1155::new(ctf_addr, &provider);

        let mut statuses = Vec::new();

        for target in ApprovalTarget::all() {
            let spender = Address::from_str(target.address())
                .map_err(|e| PolymarketError::Config(format!("invalid target address: {e}")))?;

            // Check USDC allowance
            let allowance_val = usdc.allowance(owner, spender).call().await.map_err(|e| {
                PolymarketError::Api(format!("failed to check USDC allowance: {e}"))
            })?;
            let usdc_approved = allowance_val > U256::ZERO;
            statuses.push(AllowanceStatus {
                token: TokenType::Usdc,
                target,
                approved: usdc_approved,
                details: allowance_val.to_string(),
            });

            // Check CTF approval
            let is_approved = ctf
                .isApprovedForAll(owner, spender)
                .call()
                .await
                .map_err(|e| PolymarketError::Api(format!("failed to check CTF approval: {e}")))?;
            statuses.push(AllowanceStatus {
                token: TokenType::Ctf,
                target,
                approved: is_approved,
                details: if is_approved {
                    "approved".to_string()
                } else {
                    "not approved".to_string()
                },
            });
        }

        Ok(statuses)
    }

    /// Execute approvals based on request.
    ///
    /// For EOA wallets (safe_address = None), approvals are executed directly.
    /// For Safe wallets (safe_address = Some), approvals are executed through
    /// the Safe's `execTransaction()` function.
    pub async fn execute_approvals(
        &self,
        private_key: &str,
        safe_address: Option<Address>,
        request: &ApprovalRequest,
    ) -> Result<ApprovalResponse, PolymarketError> {
        let pairs = request.to_approval_pairs();
        if pairs.is_empty() {
            return Ok(ApprovalResponse {
                results: vec![],
                all_succeeded: true,
            });
        }

        // Parse private key
        let key_hex = if private_key.starts_with("0x") {
            private_key.to_string()
        } else {
            format!("0x{private_key}")
        };
        let signer: PrivateKeySigner = key_hex
            .parse()
            .map_err(|e| PolymarketError::Config(format!("invalid private key: {e}")))?;

        // Build provider with signer
        let provider = ProviderBuilder::new()
            .wallet(alloy::network::EthereumWallet::from(signer.clone()))
            .connect(&self.rpc_url)
            .await
            .map_err(|e| PolymarketError::Network(format!("failed to connect to RPC: {e}")))?;

        let usdc_addr = Address::from_str(USDC_ADDRESS)
            .map_err(|e| PolymarketError::Config(format!("invalid USDC address: {e}")))?;
        let ctf_addr = Address::from_str(CTF_ADDRESS)
            .map_err(|e| PolymarketError::Config(format!("invalid CTF address: {e}")))?;

        let max_amount = U256::from_str(MAX_APPROVAL)
            .map_err(|e| PolymarketError::Config(format!("invalid max approval: {e}")))?;

        let mut results = Vec::new();

        // Route based on wallet type
        if let Some(safe) = safe_address {
            // Execute approvals through Gnosis Safe
            let safe_contract = IGnosisSafe::new(safe, &provider);
            let signatures = Self::build_prevalidated_signature(signer.address());

            for (token, target) in pairs {
                let spender = Address::from_str(target.address())
                    .map_err(|e| PolymarketError::Config(format!("invalid target address: {e}")))?;

                let (to, call_data) = match token {
                    TokenType::Usdc => {
                        let call = IERC20::approveCall {
                            spender,
                            amount: max_amount,
                        };
                        (usdc_addr, Bytes::from(call.abi_encode()))
                    }
                    TokenType::Ctf => {
                        let call = IERC1155::setApprovalForAllCall {
                            operator: spender,
                            approved: true,
                        };
                        (ctf_addr, Bytes::from(call.abi_encode()))
                    }
                };

                // Execute through Safe's execTransaction
                // Parameters: to, value=0, data, operation=0 (Call),
                // safeTxGas=0, baseGas=0, gasPrice=0, gasToken=0, refundReceiver=0, signatures
                let result = match safe_contract
                    .execTransaction(
                        to,
                        U256::ZERO,
                        call_data,
                        0,
                        U256::ZERO,
                        U256::ZERO,
                        U256::ZERO,
                        Address::ZERO,
                        Address::ZERO,
                        signatures.clone(),
                    )
                    .send()
                    .await
                {
                    Ok(pending) => match pending.watch().await {
                        Ok(tx_hash) => ApprovalResult {
                            token,
                            target,
                            tx_hash: Some(format!("{tx_hash:#x}")),
                            success: true,
                            error: None,
                        },
                        Err(e) => ApprovalResult {
                            token,
                            target,
                            tx_hash: None,
                            success: false,
                            error: Some(format!("Safe tx confirmation failed: {e}")),
                        },
                    },
                    Err(e) => ApprovalResult {
                        token,
                        target,
                        tx_hash: None,
                        success: false,
                        error: Some(format!("Safe execTransaction failed: {e}")),
                    },
                };

                results.push(result);
            }
        } else {
            // Direct EOA execution
            let usdc = IERC20::new(usdc_addr, &provider);
            let ctf = IERC1155::new(ctf_addr, &provider);

            for (token, target) in pairs {
                let spender = Address::from_str(target.address())
                    .map_err(|e| PolymarketError::Config(format!("invalid target address: {e}")))?;

                let result = match token {
                    TokenType::Usdc => match usdc.approve(spender, max_amount).send().await {
                        Ok(pending) => match pending.watch().await {
                            Ok(tx_hash) => ApprovalResult {
                                token,
                                target,
                                tx_hash: Some(format!("{tx_hash:#x}")),
                                success: true,
                                error: None,
                            },
                            Err(e) => ApprovalResult {
                                token,
                                target,
                                tx_hash: None,
                                success: false,
                                error: Some(format!("tx confirmation failed: {e}")),
                            },
                        },
                        Err(e) => ApprovalResult {
                            token,
                            target,
                            tx_hash: None,
                            success: false,
                            error: Some(format!("tx send failed: {e}")),
                        },
                    },
                    TokenType::Ctf => match ctf.setApprovalForAll(spender, true).send().await {
                        Ok(pending) => match pending.watch().await {
                            Ok(tx_hash) => ApprovalResult {
                                token,
                                target,
                                tx_hash: Some(format!("{tx_hash:#x}")),
                                success: true,
                                error: None,
                            },
                            Err(e) => ApprovalResult {
                                token,
                                target,
                                tx_hash: None,
                                success: false,
                                error: Some(format!("tx confirmation failed: {e}")),
                            },
                        },
                        Err(e) => ApprovalResult {
                            token,
                            target,
                            tx_hash: None,
                            success: false,
                            error: Some(format!("tx send failed: {e}")),
                        },
                    },
                };

                results.push(result);
            }
        }

        let all_succeeded = results.iter().all(|r| r.success);

        Ok(ApprovalResponse {
            results,
            all_succeeded,
        })
    }

    /// Build pre-validated signature for Gnosis Safe single-owner execution.
    ///
    /// Format: r=owner (32 bytes), s=0 (32 bytes), v=1 (1 byte)
    /// This tells the Safe "the owner at address r has pre-approved this tx".
    fn build_prevalidated_signature(owner: Address) -> Bytes {
        let mut signature = [0u8; 65];
        // r = owner address padded to 32 bytes (left-padded with zeros)
        signature[12..32].copy_from_slice(owner.as_slice());
        // s = 0 (bytes 32-64 are already zero)
        // v = 1 (pre-validated signature marker)
        signature[64] = 1;
        Bytes::from(signature.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_request_all() {
        let req = ApprovalRequest::all();
        let pairs = req.to_approval_pairs();
        assert_eq!(pairs.len(), 6); // 2 tokens * 3 targets
    }

    #[test]
    fn test_approval_request_usdc_only() {
        let req = ApprovalRequest {
            usdc: true,
            ..Default::default()
        };
        let pairs = req.to_approval_pairs();
        assert_eq!(pairs.len(), 3); // 1 token * 3 targets
        assert!(pairs.iter().all(|(t, _)| *t == TokenType::Usdc));
    }

    #[test]
    fn test_approval_request_neg_risk_only() {
        let req = ApprovalRequest {
            usdc_neg_risk: true,
            ctf_neg_risk: true,
            ..Default::default()
        };
        let pairs = req.to_approval_pairs();
        assert_eq!(pairs.len(), 4); // 2 tokens * 2 neg-risk targets
    }

    #[test]
    fn test_target_addresses() {
        assert_eq!(ApprovalTarget::CtfExchange.address(), CTF_EXCHANGE);
        assert_eq!(
            ApprovalTarget::NegRiskCtfExchange.address(),
            NEG_RISK_CTF_EXCHANGE
        );
        assert_eq!(ApprovalTarget::NegRiskAdapter.address(), NEG_RISK_ADAPTER);
    }
}
