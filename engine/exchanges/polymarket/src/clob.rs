//! Simplified CLOB types and conversions for Polymarket SDK integration.
//!
//! This module provides type bridges between the OpenPX Exchange trait
//! and the official polymarket-client-sdk. The SDK handles all authentication,
//! signing, and API interactions.

use polymarket_client_sdk_v2::auth::{Credentials, ExposeSecret};
use polymarket_client_sdk_v2::clob::types::{Side, SignatureType};
use serde::{Deserialize, Serialize};

use crate::config::PolymarketSignatureType;

pub const CLOB_URL: &str = "https://clob.polymarket.com";

// ============================================================================
// Order Side/Type Enums with SDK Conversions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ClobOrderSide {
    Buy,
    Sell,
}

impl From<ClobOrderSide> for Side {
    fn from(s: ClobOrderSide) -> Self {
        match s {
            ClobOrderSide::Buy => Side::Buy,
            ClobOrderSide::Sell => Side::Sell,
        }
    }
}

impl From<&ClobOrderSide> for Side {
    fn from(s: &ClobOrderSide) -> Self {
        match s {
            ClobOrderSide::Buy => Side::Buy,
            ClobOrderSide::Sell => Side::Sell,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ClobOrderType {
    Gtc,
    Fok,
    Ioc,
}

impl From<ClobOrderType> for polymarket_client_sdk_v2::clob::types::OrderType {
    fn from(t: ClobOrderType) -> Self {
        match t {
            ClobOrderType::Gtc => polymarket_client_sdk_v2::clob::types::OrderType::GTC,
            ClobOrderType::Fok => polymarket_client_sdk_v2::clob::types::OrderType::FOK,
            ClobOrderType::Ioc => polymarket_client_sdk_v2::clob::types::OrderType::FAK,
        }
    }
}

// ============================================================================
// Signature Type Conversions
// ============================================================================

impl From<PolymarketSignatureType> for SignatureType {
    fn from(t: PolymarketSignatureType) -> Self {
        match t {
            PolymarketSignatureType::Eoa => SignatureType::Eoa,
            PolymarketSignatureType::Proxy => SignatureType::Proxy,
            PolymarketSignatureType::GnosisSafe => SignatureType::GnosisSafe,
        }
    }
}

// ============================================================================
// Order Arguments (Input to create_order)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct OrderArgs {
    pub token_id: String,
    pub price: f64,
    pub size: f64,
    pub side: ClobOrderSide,
    pub tick_size: f64,
    pub neg_risk: bool,
}

// ============================================================================
// API Credentials
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ApiCredentials {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

impl From<&Credentials> for ApiCredentials {
    fn from(c: &Credentials) -> Self {
        ApiCredentials {
            api_key: c.key().to_string(),
            secret: c.secret().expose_secret().to_string(),
            passphrase: c.passphrase().expose_secret().to_string(),
        }
    }
}

// ============================================================================
// Response Types (for parsing CLOB API responses)
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    #[serde(rename = "orderID")]
    pub order_id: Option<String>,
    pub status: Option<String>,
    pub error_msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClobOrderData {
    #[serde(rename = "id", alias = "orderID")]
    pub id: Option<String>,
    pub market: Option<String>,
    pub asset_id: Option<String>,
    pub side: Option<String>,
    pub original_size: Option<String>,
    pub size_matched: Option<String>,
    pub price: Option<String>,
    pub status: Option<String>,
    pub outcome: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceAllowance {
    pub balance: Option<String>,
    pub allowance: Option<String>,
}
