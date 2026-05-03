//! Bypass Cloudflare's WAF on POST /auth/api-key by replaying a `cf_clearance`
//! cookie that your browser earned from solving the CF JS challenge.
//!
//! Background: Cloudflare's bot rule on /auth/api-key blocks SDK clients on
//! flagged IPs (datacenter, commercial VPN). A real browser on the SAME IP
//! passes — Cloudflare drops a `cf_clearance` cookie scoped to that IP+UA.
//! Replaying the cookie + matching User-Agent from any HTTP client clears
//! the WAF; Polymarket's app then handles the L1-signed request normally.
//!
//! Setup (do this once, in the SAME network session you'll run the call from):
//!   1. Open https://polymarket.com in Chrome/Firefox.
//!   2. Solve any Cloudflare challenge (Turnstile box, etc.).
//!   3. DevTools → Application → Cookies → polymarket.com → copy `cf_clearance`.
//!   4. DevTools → Network → any request → Headers → copy your `User-Agent`.
//!
//! Run:
//!   CF_CLEARANCE='<paste>' \
//!   CF_USER_AGENT='<paste>' \
//!   cargo run -p px-exchange-polymarket --example derive_credentials_with_cookie
//!
//! On success, copy the printed api_key/secret/passphrase into .env as
//! POLYMARKET_API_KEY/_SECRET/_PASSPHRASE — derive-from-anywhere then works.

use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::dyn_abi::Eip712Domain;
use alloy::primitives::U256;
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use alloy::sol;
use alloy::sol_types::SolStruct;

const HOST: &str = "https://clob.polymarket.com";
const POLYGON_CHAIN_ID: u64 = 137;
const ATTEST_MSG: &str = "This message attests that I control the given wallet";

sol! {
    struct ClobAuth {
        address address;
        string  timestamp;
        uint256 nonce;
        string  message;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("install rustls provider");
    let _ = dotenvy::dotenv();

    let cf_clearance = std::env::var("CF_CLEARANCE")
        .map_err(|_| anyhow::anyhow!("CF_CLEARANCE not set — see file header"))?;
    let user_agent = std::env::var("CF_USER_AGENT")
        .map_err(|_| anyhow::anyhow!("CF_USER_AGENT not set — see file header"))?;

    let private_key = std::env::var("POLYMARKET_PRIVATE_KEY")?;
    let key_hex = if private_key.starts_with("0x") {
        private_key
    } else {
        format!("0x{private_key}")
    };
    let signer = PrivateKeySigner::from_str(&key_hex)?.with_chain_id(Some(POLYGON_CHAIN_ID));
    let address = signer.address();

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let nonce: u32 = 0;

    let auth = ClobAuth {
        address,
        timestamp: timestamp.to_string(),
        nonce: U256::from(nonce),
        message: ATTEST_MSG.to_owned(),
    };

    let domain = Eip712Domain {
        name: Some("ClobAuthDomain".into()),
        version: Some("1".into()),
        chain_id: Some(U256::from(POLYGON_CHAIN_ID)),
        ..Eip712Domain::default()
    };

    let hash = auth.eip712_signing_hash(&domain);
    let signature = signer.sign_hash(&hash).await?;
    let sig_hex = signature.to_string();

    eprintln!("EOA:       {address:?}");
    eprintln!("timestamp: {timestamp}");
    eprintln!("nonce:     {nonce}");
    eprintln!("UA:        {user_agent}");
    eprintln!(
        "cookie:    cf_clearance={}...",
        &cf_clearance[..cf_clearance.len().min(20)]
    );
    eprintln!();

    let client = reqwest::Client::builder().user_agent(&user_agent).build()?;

    let resp = client
        .post(format!("{HOST}/auth/api-key"))
        .header("Cookie", format!("cf_clearance={cf_clearance}"))
        .header("POLY_ADDRESS", format!("{address:?}"))
        .header("POLY_SIGNATURE", &sig_hex)
        .header("POLY_TIMESTAMP", timestamp.to_string())
        .header("POLY_NONCE", nonce.to_string())
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if status.is_success() {
        println!("HTTP {status}");
        println!("{body}");
    } else {
        eprintln!("HTTP {status}");
        let preview: String = body.chars().take(500).collect();
        eprintln!("{preview}");
        std::process::exit(1);
    }

    Ok(())
}
