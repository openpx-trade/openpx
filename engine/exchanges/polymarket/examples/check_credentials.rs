//! Test stored Polymarket CLOB credentials against the live API.
//!
//! For each candidate (key, secret, passphrase) tuple supplied via env vars,
//! authenticate L2 with the configured EOA + funder + GnosisSafe sig type and
//! call `api_keys()` (proves L2 auth works for this EOA) and `balance_allowance()`
//! (returns the funder's USDC balance — confirms it points to the funded account).
//!
//! Usage (from repo root):
//!   POLYMARKET_PRIVATE_KEY=... \
//!   POLYMARKET_FUNDER=0x... \
//!   CRED_A_KEY=... CRED_A_SECRET=... CRED_A_PASSPHRASE=... \
//!   CRED_B_KEY=... CRED_B_SECRET=... CRED_B_PASSPHRASE=... \
//!   cargo run -p px-exchange-polymarket --example check_credentials

use std::str::FromStr;

use polymarket_client_sdk_v2::auth::{Credentials, LocalSigner, Signer};
use polymarket_client_sdk_v2::clob::types::request::BalanceAllowanceRequest;
use polymarket_client_sdk_v2::clob::types::{AssetType, SignatureType};
use polymarket_client_sdk_v2::clob::{Client, Config};
use polymarket_client_sdk_v2::types::Address;
use polymarket_client_sdk_v2::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("install rustls provider");
    let _ = dotenvy::dotenv();

    let host = std::env::var("CLOB_API_URL")
        .unwrap_or_else(|_| "https://clob-v2.polymarket.com".into());

    let private_key = std::env::var(PRIVATE_KEY_VAR)?;
    let key_hex = if private_key.starts_with("0x") {
        private_key
    } else {
        format!("0x{private_key}")
    };
    let signer = LocalSigner::from_str(&key_hex)?.with_chain_id(Some(POLYGON));
    let eoa = signer.address();

    let funder_str = std::env::var("POLYMARKET_FUNDER")?;
    let funder = Address::from_str(&funder_str)?;

    println!("EOA (from POLYMARKET_PRIVATE_KEY): {eoa:?}");
    println!("Funder (Gnosis Safe):              {funder:?}");
    println!("Host:                              {host}");
    println!();

    let candidates = collect_candidates();
    if candidates.is_empty() {
        eprintln!(
            "no candidate credentials supplied. Set CRED_<NAME>_KEY/SECRET/PASSPHRASE env vars."
        );
        std::process::exit(1);
    }

    for (label, key, secret, passphrase) in candidates {
        println!("=== Candidate {label} (key={key}) ===");
        let key_uuid: uuid::Uuid = match key.parse() {
            Ok(u) => u,
            Err(e) => {
                println!("  invalid UUID: {e}");
                println!();
                continue;
            }
        };
        let creds = Credentials::new(key_uuid, secret.clone(), passphrase.clone());

        let client = Client::new(&host, Config::default())?
            .authentication_builder(&signer)
            .credentials(creds)
            .funder(funder)
            .signature_type(SignatureType::GnosisSafe)
            .authenticate()
            .await;

        let client = match client {
            Ok(c) => c,
            Err(e) => {
                println!("  authenticate() failed: {e}");
                println!();
                continue;
            }
        };

        match client.api_keys().await {
            Ok(keys) => println!("  api_keys()        ok — keys for EOA: {keys:?}"),
            Err(e) => {
                println!("  api_keys()        FAILED: {e}");
                println!("  -> this credential set does NOT belong to the configured EOA");
                println!();
                continue;
            }
        }

        let req = BalanceAllowanceRequest::builder()
            .asset_type(AssetType::Collateral)
            .build();
        match client.balance_allowance(req).await {
            Ok(b) => println!(
                "  balance_allowance ok — USDC balance for funder: {} (allowances: {:?})",
                b.balance, b.allowances
            ),
            Err(e) => println!("  balance_allowance FAILED: {e}"),
        }
        println!();
    }

    Ok(())
}

fn collect_candidates() -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    // Primary slot — read the standard openpx env var names if set.
    if let (Ok(k), Ok(s), Ok(p)) = (
        std::env::var("POLYMARKET_CLOB_API_KEY"),
        std::env::var("POLYMARKET_CLOB_API_SECRET"),
        std::env::var("POLYMARKET_CLOB_API_PASSPHRASE"),
    ) {
        out.push(("env".to_string(), k, s, p));
    }
    // Additional candidate slots for ad-hoc comparisons.
    for label in ["A", "B", "C", "D"] {
        let k = std::env::var(format!("CRED_{label}_KEY")).ok();
        let s = std::env::var(format!("CRED_{label}_SECRET")).ok();
        let p = std::env::var(format!("CRED_{label}_PASSPHRASE")).ok();
        if let (Some(k), Some(s), Some(p)) = (k, s, p) {
            out.push((label.to_string(), k, s, p));
        }
    }
    out
}
