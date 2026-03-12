//! CTF (Conditional Token Framework) calldata encoding for on-chain operations.
//!
//! Encodes ABI calldata for split, merge, and redeem operations against
//! Polymarket's ConditionalTokens and NegRiskAdapter contracts.

use alloy::primitives::{Address, B256, U256};
use alloy::sol;
use alloy::sol_types::SolCall;
use std::str::FromStr;

use crate::approvals::{CTF_ADDRESS, NEG_RISK_ADAPTER, USDC_ADDRESS};
use crate::error::PolymarketError;

sol! {
    interface IConditionalTokens {
        function splitPosition(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] partition,
            uint256 amount
        ) external;

        function mergePositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] partition,
            uint256 amount
        ) external;

        function redeemPositions(
            address collateralToken,
            bytes32 parentCollectionId,
            bytes32 conditionId,
            uint256[] indexSets
        ) external;
    }

    interface INegRiskAdapter {
        function redeemPositions(
            bytes32 conditionId,
            uint256[] amounts
        ) external;
    }
}

/// Binary market partition: [1 (YES), 2 (NO)]
const BINARY_PARTITION: [u64; 2] = [1, 2];

/// Encode splitPosition calldata for binary markets.
/// Returns (to_address, hex_calldata) targeting the CTF contract.
pub fn encode_split_calldata(condition_id: B256, amount: U256) -> (String, String) {
    let collateral = Address::from_str(USDC_ADDRESS).expect("USDC_ADDRESS constant must be valid");
    let partition: Vec<U256> = BINARY_PARTITION.iter().map(|&v| U256::from(v)).collect();

    let call = IConditionalTokens::splitPositionCall {
        collateralToken: collateral,
        parentCollectionId: B256::ZERO,
        conditionId: condition_id,
        partition,
        amount,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (CTF_ADDRESS.to_string(), format!("0x{calldata}"))
}

/// Encode mergePositions calldata for binary markets.
/// Returns (to_address, hex_calldata) targeting the CTF contract.
pub fn encode_merge_calldata(condition_id: B256, amount: U256) -> (String, String) {
    let collateral = Address::from_str(USDC_ADDRESS).expect("USDC_ADDRESS constant must be valid");
    let partition: Vec<U256> = BINARY_PARTITION.iter().map(|&v| U256::from(v)).collect();

    let call = IConditionalTokens::mergePositionsCall {
        collateralToken: collateral,
        parentCollectionId: B256::ZERO,
        conditionId: condition_id,
        partition,
        amount,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (CTF_ADDRESS.to_string(), format!("0x{calldata}"))
}

/// Encode redeemPositions calldata for standard (non-neg-risk) markets.
/// Returns (to_address, hex_calldata) targeting the CTF contract.
pub fn encode_redeem_calldata(condition_id: B256) -> (String, String) {
    let collateral = Address::from_str(USDC_ADDRESS).expect("USDC_ADDRESS constant must be valid");
    let index_sets: Vec<U256> = BINARY_PARTITION.iter().map(|&v| U256::from(v)).collect();

    let call = IConditionalTokens::redeemPositionsCall {
        collateralToken: collateral,
        parentCollectionId: B256::ZERO,
        conditionId: condition_id,
        indexSets: index_sets,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (CTF_ADDRESS.to_string(), format!("0x{calldata}"))
}

/// Encode redeemPositions calldata for neg-risk markets via NegRiskAdapter.
/// Returns (to_address, hex_calldata) targeting the NegRiskAdapter contract.
pub fn encode_redeem_neg_risk_calldata(condition_id: B256, amounts: Vec<U256>) -> (String, String) {
    let call = INegRiskAdapter::redeemPositionsCall {
        conditionId: condition_id,
        amounts,
    };
    let calldata = alloy::primitives::hex::encode(call.abi_encode());
    (NEG_RISK_ADAPTER.to_string(), format!("0x{calldata}"))
}

// ── String-accepting wrappers (for callers without alloy dependency) ──────────

fn parse_condition_id(hex: &str) -> Result<B256, PolymarketError> {
    B256::from_str(hex).map_err(|e| PolymarketError::Api(format!("invalid condition_id: {e}")))
}

fn parse_amount(s: &str) -> Result<U256, PolymarketError> {
    U256::from_str(s).map_err(|e| PolymarketError::Api(format!("invalid amount: {e}")))
}

/// String-accepting split calldata encoder.
pub fn encode_split(condition_id: &str, amount: &str) -> Result<(String, String), PolymarketError> {
    Ok(encode_split_calldata(
        parse_condition_id(condition_id)?,
        parse_amount(amount)?,
    ))
}

/// String-accepting merge calldata encoder.
pub fn encode_merge(condition_id: &str, amount: &str) -> Result<(String, String), PolymarketError> {
    Ok(encode_merge_calldata(
        parse_condition_id(condition_id)?,
        parse_amount(amount)?,
    ))
}

/// String-accepting redeem calldata encoder.
pub fn encode_redeem(condition_id: &str) -> Result<(String, String), PolymarketError> {
    Ok(encode_redeem_calldata(parse_condition_id(condition_id)?))
}

/// String-accepting neg-risk redeem calldata encoder.
pub fn encode_redeem_neg_risk(
    condition_id: &str,
    amounts: &[String],
) -> Result<(String, String), PolymarketError> {
    let parsed: Vec<U256> = amounts
        .iter()
        .map(|s| parse_amount(s))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(encode_redeem_neg_risk_calldata(
        parse_condition_id(condition_id)?,
        parsed,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CONDITION_ID: &str =
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

    fn test_condition_id() -> B256 {
        B256::from_str(TEST_CONDITION_ID).unwrap()
    }

    #[test]
    fn split_calldata_targets_ctf_contract() {
        let (to, data) = encode_split_calldata(test_condition_id(), U256::from(1_000_000));
        assert_eq!(to, CTF_ADDRESS);
        assert!(data.starts_with("0x"));
        // splitPosition selector: first 4 bytes of keccak256
        assert!(data.len() > 10);
    }

    #[test]
    fn merge_calldata_targets_ctf_contract() {
        let (to, data) = encode_merge_calldata(test_condition_id(), U256::from(1_000_000));
        assert_eq!(to, CTF_ADDRESS);
        assert!(data.starts_with("0x"));
    }

    #[test]
    fn redeem_calldata_targets_ctf_contract() {
        let (to, data) = encode_redeem_calldata(test_condition_id());
        assert_eq!(to, CTF_ADDRESS);
        assert!(data.starts_with("0x"));
    }

    #[test]
    fn redeem_neg_risk_targets_adapter() {
        let amounts = vec![U256::from(500_000), U256::from(500_000)];
        let (to, data) = encode_redeem_neg_risk_calldata(test_condition_id(), amounts);
        assert_eq!(to, NEG_RISK_ADAPTER);
        assert!(data.starts_with("0x"));
    }

    #[test]
    fn split_and_merge_produce_different_calldata() {
        let cid = test_condition_id();
        let amount = U256::from(1_000_000);
        let (_, split_data) = encode_split_calldata(cid, amount);
        let (_, merge_data) = encode_merge_calldata(cid, amount);
        // Different function selectors
        assert_ne!(&split_data[..10], &merge_data[..10]);
    }
}
