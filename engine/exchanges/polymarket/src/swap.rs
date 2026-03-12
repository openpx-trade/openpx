//! Auto-swap native USDC → USDC.e during Polymarket onboarding.
//!
//! Users fund Polygon wallets with native USDC (0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359)
//! but Polymarket and fee escrow use bridged USDC.e (0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174).
//! This module detects native USDC and swaps it to USDC.e via Uniswap V3 before approvals.

use alloy::primitives::{Address, Uint, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use alloy::sol_types::SolCall;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::config::DEFAULT_POLYGON_RPC;

/// Native USDC on Polygon (what MetaMask shows as "USDC")
pub const NATIVE_USDC_ADDRESS: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";

/// Bridged USDC.e on Polygon (what Polymarket uses)
pub const BRIDGED_USDC_E_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

/// Uniswap V3 SwapRouter on Polygon
const UNISWAP_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";

/// Fee tier: 0.01% (100 bps) — optimal for stablecoin pairs
const POOL_FEE: u32 = 100;

/// Minimum balance to trigger swap (0.01 USDC = 10_000 units at 6 decimals)
pub const MIN_SWAP_BALANCE: u128 = 10_000;

/// Slippage tolerance: 0.5% for a 1:1 stablecoin peg
pub const SLIPPAGE_BPS: u64 = 50;

/// Compute the swap amount and min_out from a raw native USDC balance.
/// Returns `(amount_in, min_out)` as `(u128, u128)` to avoid leaking alloy types.
pub fn compute_swap_amounts(balance: u128) -> (u128, u128) {
    let min_out = balance * (10_000 - SLIPPAGE_BPS as u128) / 10_000;
    (balance, min_out)
}

sol! {
    #[sol(rpc)]
    interface INativeUSDC {
        function balanceOf(address owner) external view returns (uint256);
    }

    interface ISwapERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address to, uint256 amount) external returns (bool);
    }

    interface ISwapRouter {
        struct ExactInputSingleParams {
            address tokenIn;
            address tokenOut;
            uint24 fee;
            address recipient;
            uint256 deadline;
            uint256 amountIn;
            uint256 amountOutMinimum;
            uint160 sqrtPriceLimitX96;
        }

        function exactInputSingle(ExactInputSingleParams calldata params) external payable returns (uint256 amountOut);
    }
}

/// Result of the native USDC → USDC.e swap attempt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SwapResult {
    /// Native USDC balance detected (raw units, 6 decimals)
    pub native_balance: u128,
    /// Whether a swap was executed successfully
    pub swapped: bool,
    /// Approval tx hash (if swap was attempted)
    pub approval_tx: Option<String>,
    /// Swap tx hash (if swap succeeded)
    pub swap_tx: Option<String>,
    /// Transfer tx hash (if USDC.e was forwarded from EOA → Safe)
    pub transfer_tx: Option<String>,
    /// Error message (if swap failed — non-fatal)
    pub error: Option<String>,
}

/// Check any ERC20 token balance for an address. Returns 0 on any error (fail-silent).
pub async fn check_erc20_balance(rpc_url: Option<&str>, token_address: &str, owner: &str) -> u128 {
    let Ok(owner_addr) = Address::from_str(owner) else {
        return 0;
    };
    let Ok(token_addr) = Address::from_str(token_address) else {
        return 0;
    };

    let rpc = rpc_url.unwrap_or(DEFAULT_POLYGON_RPC);
    let Ok(provider) = ProviderBuilder::new().connect(rpc).await else {
        return 0;
    };

    let token = INativeUSDC::new(token_addr, &provider);
    match token.balanceOf(owner_addr).call().await {
        Ok(bal) => bal.to::<u128>(),
        Err(_) => 0,
    }
}

/// Check native USDC balance for an address. Returns 0 on any error (fail-silent).
pub async fn check_native_usdc_balance(rpc_url: Option<&str>, owner: &str) -> u128 {
    check_erc20_balance(rpc_url, NATIVE_USDC_ADDRESS, owner).await
}

/// Check native POL (MATIC) balance for an address. Returns 0 on any error (fail-silent).
pub async fn check_pol_balance(rpc_url: Option<&str>, owner: &str) -> u128 {
    let Ok(owner_addr) = Address::from_str(owner) else {
        return 0;
    };

    let rpc = rpc_url.unwrap_or(DEFAULT_POLYGON_RPC);
    let Ok(provider) = ProviderBuilder::new().connect(rpc).await else {
        return 0;
    };

    match provider.get_balance(owner_addr).await {
        Ok(bal) => {
            let b: U256 = bal;
            b.to::<u128>()
        }
        Err(_) => 0,
    }
}

/// Format a raw token balance as a human-readable string (e.g., 2000000 with 6 decimals → "2.00").
pub fn format_token_balance(raw: u128, decimals: u8) -> String {
    if decimals == 0 {
        return raw.to_string();
    }
    let divisor = 10u128.pow(decimals as u32);
    let whole = raw / divisor;
    let frac = raw % divisor;
    let frac_str = format!("{:0>width$}", frac, width = decimals as usize);
    // Trim trailing zeros, keep at least 2 decimal places
    let min_frac = if decimals >= 2 { 2 } else { decimals as usize };
    let trimmed_len = frac_str.trim_end_matches('0').len().max(min_frac);
    format!("{}.{}", whole, &frac_str[..trimmed_len])
}

/// Encode `approve(router, amount)` on native USDC. Returns `(to, calldata)`.
pub fn encode_native_usdc_approval(amount: U256) -> (String, String) {
    let router = Address::from_str(UNISWAP_V3_ROUTER).expect("valid router address");
    let call = ISwapERC20::approveCall {
        spender: router,
        amount,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (NATIVE_USDC_ADDRESS.to_string(), format!("0x{calldata}"))
}

/// Encode Uniswap V3 `exactInputSingle` call. Returns `(to, calldata)`.
pub fn encode_swap_calldata(amount_in: U256, recipient: &str, min_out: U256) -> (String, String) {
    let recipient_addr = Address::from_str(recipient).expect("valid recipient address");
    let token_in = Address::from_str(NATIVE_USDC_ADDRESS).expect("valid native USDC address");
    let token_out = Address::from_str(BRIDGED_USDC_E_ADDRESS).expect("valid USDC.e address");

    // Deadline: 10 minutes from now
    let deadline = U256::from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid system time")
            .as_secs()
            + 600,
    );

    let call = ISwapRouter::exactInputSingleCall {
        params: ISwapRouter::ExactInputSingleParams {
            tokenIn: token_in,
            tokenOut: token_out,
            fee: Uint::<24, 1>::from(POOL_FEE),
            recipient: recipient_addr,
            deadline,
            amountIn: amount_in,
            amountOutMinimum: min_out,
            sqrtPriceLimitX96: Uint::<160, 3>::ZERO,
        },
    };

    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (UNISWAP_V3_ROUTER.to_string(), format!("0x{calldata}"))
}

/// Convenience wrapper: encode approval from a u128 amount.
pub fn encode_native_usdc_approval_u128(amount: u128) -> (String, String) {
    encode_native_usdc_approval(U256::from(amount))
}

/// Convenience wrapper: encode swap calldata from u128 amounts.
pub fn encode_swap_calldata_u128(
    amount_in: u128,
    recipient: &str,
    min_out: u128,
) -> (String, String) {
    encode_swap_calldata(U256::from(amount_in), recipient, U256::from(min_out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_native_usdc_approval() {
        let (to, data) = encode_native_usdc_approval(U256::from(1_000_000u64));
        assert_eq!(to, NATIVE_USDC_ADDRESS);
        // approve(address,uint256) selector = 0x095ea7b3
        assert!(data.starts_with("0x095ea7b3"));
    }

    #[test]
    fn test_encode_swap_calldata() {
        let amount = U256::from(1_000_000u64);
        let min_out = U256::from(995_000u64);
        let recipient = "0x1234567890123456789012345678901234567890";
        let (to, data) = encode_swap_calldata(amount, recipient, min_out);
        assert_eq!(to, UNISWAP_V3_ROUTER);
        // exactInputSingle selector = 0x414bf389
        assert!(data.starts_with("0x414bf389"));
    }

    #[test]
    fn test_swap_result_default() {
        let r = SwapResult::default();
        assert_eq!(r.native_balance, 0);
        assert!(!r.swapped);
        assert!(r.approval_tx.is_none());
        assert!(r.swap_tx.is_none());
        assert!(r.error.is_none());
    }
}
