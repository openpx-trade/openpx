//! Derive Polymarket CLOB credentials from POLYMARKET_PRIVATE_KEY against the
//! V2 host (`https://clob.polymarket.com`).
//!
//! Mirrors the upstream `examples/clob/keys/create_or_derive_api_key.rs` from
//! https://github.com/Polymarket/rs-clob-client-v2: build an unauthenticated
//! `Client`, then attempt POST /auth/api-key (create) and fall back to
//! GET /auth/derive-api-key (derive existing) on any failure.
//!
//! Run from repo root:
//!   cargo run -p px-exchange-polymarket --example derive_credentials

use std::str::FromStr;

use polymarket_client_sdk_v2::auth::{ExposeSecret, LocalSigner, Signer};
use polymarket_client_sdk_v2::clob::{Client, Config};
use polymarket_client_sdk_v2::{POLYGON, PRIVATE_KEY_VAR};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("install rustls provider");
    let _ = dotenvy::dotenv();

    let host = std::env::var("CLOB_API_URL")
        .unwrap_or_else(|_| "https://clob.polymarket.com".into());

    let private_key = std::env::var(PRIVATE_KEY_VAR)
        .expect("POLYMARKET_PRIVATE_KEY not set in env or .env");
    let key_hex = if private_key.starts_with("0x") {
        private_key
    } else {
        format!("0x{private_key}")
    };

    let signer = LocalSigner::from_str(&key_hex)?.with_chain_id(Some(POLYGON));

    let client = Client::new(&host, Config::default())?;

    // Try POST /auth/api-key directly so we can see its real failure mode if it's
    // not a "key already exists" case. Fall back to DERIVE on any error.
    let creds = match client.create_api_key(&signer, None).await {
        Ok(c) => {
            eprintln!("created new API key");
            c
        }
        Err(e) => {
            eprintln!("create_api_key failed: {e}");
            eprintln!("falling back to derive_api_key...");
            client.derive_api_key(&signer, None).await?
        }
    };

    println!(
        "{}",
        serde_json::json!({
            "api_key":    creds.key().to_string(),
            "secret":     creds.secret().expose_secret(),
            "passphrase": creds.passphrase().expose_secret(),
        })
    );
    Ok(())
}
