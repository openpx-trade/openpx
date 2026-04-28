//! Auto-swap + wrap during Polymarket V2 onboarding.
//!
//! After the 2026-04-28 CLOB V2 cutover, Polymarket trades against **pUSD**
//! (`0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB`) — not USDC.e. Users hold one
//! of three forms of dollars on Polygon:
//!   1. Native USDC (`0x3c499c5...`) — what most Polygon on-ramps deliver.
//!   2. Bridged USDC.e (`0x2791Bca...`) — pre-V2 collateral, still accepted by
//!      `CollateralOnramp.wrap()` as input.
//!   3. pUSD (`0xC011a7E...`) — the V2 trading collateral.
//!
//! This module encodes the calldata for the full onboarding pipeline so a relayer
//! or wallet can execute the chain end-to-end:
//!
//! ```text
//! native USDC ── (Uniswap V3 0.01% pool) ──► USDC.e ── (CollateralOnramp.wrap) ──► pUSD
//! ```
//!
//! Each leg is exposed as a separate encoder. Callers are expected to send the
//! transactions in order with appropriate approvals between them. The encoders
//! never touch private keys; signing happens in the relayer or wallet layer.

use alloy::primitives::{Address, Uint, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::sol;
use alloy::sol_types::SolCall;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::approvals::PUSD_ADDRESS;
use crate::config::DEFAULT_POLYGON_RPC;

/// Native USDC on Polygon (what MetaMask shows as "USDC")
pub const NATIVE_USDC_ADDRESS: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";

/// Bridged USDC.e on Polygon (input to CollateralOnramp.wrap() in V2)
pub const BRIDGED_USDC_E_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

/// CollateralOnramp — wraps USDC.e into pUSD (V2 trading collateral).
/// API-only flow: approve USDC.e to this address, then call wrap(amount).
/// Source: https://docs.polymarket.com/resources/contracts
pub const COLLATERAL_ONRAMP: &str = "0x93070a847efEf7F70739046A929D47a521F5B8ee";

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

    /// Permissionless onramp that wraps USDC.e into pUSD 1:1. Required after the
    /// 2026-04-28 CLOB V2 cutover, since trading collateral is now pUSD rather
    /// than USDC.e. Source: https://docs.polymarket.com/concepts/pusd
    ///
    /// `wrap(_asset, _to, _amount)` — `_asset` MUST be USDC.e
    /// (`0x2791Bca...`); pUSD is minted to `_to` (recipient may differ from
    /// msg.sender). Caller must approve USDC.e to this contract first.
    /// Reverts with `OnlyUnpaused()` if the asset is admin-paused.
    interface ICollateralOnramp {
        function wrap(address _asset, address _to, uint256 _amount) external;
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

/// Encode `approve(CollateralOnramp, amount)` on USDC.e. Required before
/// `CollateralOnramp.wrap()` can transfer USDC.e from the caller.
/// Returns `(to, calldata)` targeting the USDC.e contract.
pub fn encode_usdc_e_approval_to_onramp(amount: U256) -> (String, String) {
    let onramp = Address::from_str(COLLATERAL_ONRAMP).expect("valid CollateralOnramp address");
    let call = ISwapERC20::approveCall {
        spender: onramp,
        amount,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (BRIDGED_USDC_E_ADDRESS.to_string(), format!("0x{calldata}"))
}

/// Encode `CollateralOnramp.wrap(USDC.e, recipient, amount)`. Wraps the caller's
/// USDC.e 1:1 into pUSD, minted to `recipient`. Caller must have approved
/// USDC.e to the Onramp first (see `encode_usdc_e_approval_to_onramp`).
/// Returns `(to, calldata)` targeting the CollateralOnramp contract.
pub fn encode_pusd_wrap_calldata(recipient: &str, amount: U256) -> (String, String) {
    let asset_addr =
        Address::from_str(BRIDGED_USDC_E_ADDRESS).expect("valid USDC.e address");
    let to_addr = Address::from_str(recipient).expect("valid recipient address");
    let call = ICollateralOnramp::wrapCall {
        _asset: asset_addr,
        _to: to_addr,
        _amount: amount,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (COLLATERAL_ONRAMP.to_string(), format!("0x{calldata}"))
}

/// Convenience wrapper: encode the USDC.e → Onramp approval from a u128 amount.
pub fn encode_usdc_e_approval_to_onramp_u128(amount: u128) -> (String, String) {
    encode_usdc_e_approval_to_onramp(U256::from(amount))
}

/// Convenience wrapper: encode the wrap call from a u128 amount.
pub fn encode_pusd_wrap_calldata_u128(recipient: &str, amount: u128) -> (String, String) {
    encode_pusd_wrap_calldata(recipient, U256::from(amount))
}

/// Check the caller's pUSD balance. Returns 0 on any error (fail-silent),
/// matching the convention used by `check_native_usdc_balance` and friends.
pub async fn check_pusd_balance(rpc_url: Option<&str>, owner: &str) -> u128 {
    check_erc20_balance(rpc_url, PUSD_ADDRESS, owner).await
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

    #[test]
    fn test_encode_usdc_e_approval_to_onramp() {
        let (to, data) = encode_usdc_e_approval_to_onramp(U256::from(1_000_000u64));
        assert_eq!(to, BRIDGED_USDC_E_ADDRESS);
        // approve(address,uint256) selector = 0x095ea7b3
        assert!(data.starts_with("0x095ea7b3"));
        // The lowercase Onramp address must appear in the encoded calldata
        // (32-byte left-padded). This guards against accidentally approving the
        // wrong contract — wrong spender = funds at risk.
        let onramp_no_0x = COLLATERAL_ONRAMP.trim_start_matches("0x").to_lowercase();
        assert!(
            data.to_lowercase().contains(&onramp_no_0x),
            "approval calldata must reference CollateralOnramp address {COLLATERAL_ONRAMP}"
        );
    }

    #[test]
    fn test_encode_pusd_wrap_calldata() {
        let recipient = "0x1234567890123456789012345678901234567890";
        let (to, data) = encode_pusd_wrap_calldata(recipient, U256::from(2_500_000u64));
        assert_eq!(to, COLLATERAL_ONRAMP);
        // Calldata must contain the USDC.e asset address (it's the only valid
        // input asset for wrap; passing anything else reverts on-chain).
        let usdc_e_no_0x = BRIDGED_USDC_E_ADDRESS.trim_start_matches("0x").to_lowercase();
        assert!(
            data.to_lowercase().contains(&usdc_e_no_0x),
            "wrap calldata must reference USDC.e {BRIDGED_USDC_E_ADDRESS}"
        );
        // Recipient address must appear in calldata — wrong recipient = funds at risk.
        let recipient_no_0x = recipient.trim_start_matches("0x").to_lowercase();
        assert!(
            data.to_lowercase().contains(&recipient_no_0x),
            "wrap calldata must reference recipient {recipient}"
        );
        // Amount appears as the third arg (last 32 bytes of calldata).
        assert!(data.ends_with(&format!("{:064x}", 2_500_000u64)));
    }

    #[test]
    fn test_pusd_wrap_calldata_distinct_from_swap_and_approve() {
        // Selector regression: wrap, exactInputSingle, and approve must all
        // have distinct 4-byte function selectors.
        let recipient = "0x0000000000000000000000000000000000000001";
        let (_, wrap_data) = encode_pusd_wrap_calldata(recipient, U256::from(1_000_000u64));
        let (_, swap_data) =
            encode_swap_calldata(U256::from(1_000_000u64), recipient, U256::from(995_000u64));
        let (_, approve_data) = encode_usdc_e_approval_to_onramp(U256::from(1_000_000u64));
        assert_ne!(&wrap_data[..10], &swap_data[..10]);
        assert_ne!(&wrap_data[..10], &approve_data[..10]);
        assert_ne!(&swap_data[..10], &approve_data[..10]);
    }

    #[test]
    fn test_u128_wrappers_round_trip() {
        // Verify u128 wrappers produce identical output to their U256 counterparts.
        let amount = 10_000_000u128;
        let recipient = "0xabcdef0123456789abcdef0123456789abcdef01";

        let (to_a, data_a) = encode_usdc_e_approval_to_onramp(U256::from(amount));
        let (to_b, data_b) = encode_usdc_e_approval_to_onramp_u128(amount);
        assert_eq!((to_a, data_a), (to_b, data_b));

        let (to_c, data_c) = encode_pusd_wrap_calldata(recipient, U256::from(amount));
        let (to_d, data_d) = encode_pusd_wrap_calldata_u128(recipient, amount);
        assert_eq!((to_c, data_c), (to_d, data_d));
    }
}
