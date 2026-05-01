use alloy::primitives::{Address, ChainId, Signature as AlloySig, B256};
use k256::ecdsa::SigningKey;
use metrics::histogram;
use polymarket_client_sdk_v2::auth::state::Authenticated;
use polymarket_client_sdk_v2::auth::{LocalSigner, Normal, Signer};
use polymarket_client_sdk_v2::clob::types::request::{BalanceAllowanceRequest, OrdersRequest};
use polymarket_client_sdk_v2::clob::types::{AssetType, OrderType, Side, SignedOrder};
use polymarket_client_sdk_v2::clob::{Client as SdkClient, Config as SdkConfig};
use polymarket_client_sdk_v2::contract_config;
use polymarket_client_sdk_v2::types::{Decimal, U256};
use polymarket_client_sdk_v2::POLYGON;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

use px_core::{
    manifests::POLYMARKET_MANIFEST, sort_asks, sort_bids, Event, Exchange, ExchangeInfo,
    ExchangeManifest, FetchMarketsParams, FetchOrdersParams, Fill, Market, MarketLineage,
    MarketStatus, MarketStatusFilter, MarketTrade, MarketType, NewOrder, OpenPxError, Order,
    OrderSide, OrderStatus, OrderType as UnifiedOrderType, Orderbook, Outcome, Position,
    PriceLevel, PublicTrade, RateLimiter, Series, SettlementSource, TradesRequest,
};

use crate::approvals::{AllowanceStatus, ApprovalRequest, ApprovalResponse, TokenApprover};
use crate::client::HttpClient;
use crate::clob::{ApiCredentials, CLOB_URL};
use crate::config::PolymarketConfig;
use crate::error::PolymarketError;

/// Type alias for the SDK's LocalSigner with k256 SigningKey
type PrivateKeySigner = LocalSigner<SigningKey>;

/// Stub signer that reports a fixed address but never signs.
/// Used to satisfy the SDK's type system when we already have CLOB API credentials
/// and the real signing is done by an external service (Privy).
struct AddressOnlySigner {
    addr: Address,
    chain_id: Option<ChainId>,
}

// Signer trait from alloy uses async_trait internally, so the impl must match.
#[async_trait::async_trait]
impl Signer for AddressOnlySigner {
    async fn sign_hash(&self, _hash: &B256) -> alloy::signers::Result<AlloySig> {
        Err(alloy::signers::Error::UnsupportedOperation(
            alloy::signers::UnsupportedSignerOperation::SignHash,
        ))
    }

    fn address(&self) -> Address {
        self.addr
    }

    fn chain_id(&self) -> Option<ChainId> {
        self.chain_id
    }

    fn set_chain_id(&mut self, chain_id: Option<ChainId>) {
        self.chain_id = chain_id;
    }
}

/// Wraps the authenticated SDK client. In V1 this enum had a `Builder` variant
/// that promoted the client to attach `POLY_BUILDER_*` HMAC headers for builder
/// attribution. In V2 (CLOB V2 cutover 2026-04-28) the SDK builder-promotion
/// flow is removed — builder attribution is now per-order via the `builder` field
/// (bytes32 builder code) on the signed order itself, configurable via
/// `SdkConfig::builder().builder_code(...)`. The wrapper is preserved as a
/// transparent struct so the `sdk_dispatch!` macro and existing call sites can
/// stay unchanged.
pub(crate) struct AuthenticatedSdkClient(SdkClient<Authenticated<Normal>>);

/// Dispatches a method call chain to the inner SDK client.
macro_rules! sdk_dispatch {
    ($client:expr, $($rest:tt)*) => {
        $client.0.$($rest)*
    };
}

/// Lazily-initialized SDK state: authenticated client + derived CLOB credentials.
struct SdkState {
    client: Arc<RwLock<AuthenticatedSdkClient>>,
    creds: ApiCredentials,
}

pub struct Polymarket {
    config: PolymarketConfig,
    client: HttpClient,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    /// SDK state - lazily initialized on first authenticated call.
    /// Derives CLOB credentials automatically from the private key.
    sdk_state: tokio::sync::OnceCell<SdkState>,
    /// Local signer for signing orders
    signer: Option<PrivateKeySigner>,
    /// Pre-configured API credentials (from config or set_api_credentials).
    /// Consumed during lazy SDK initialization.
    preset_api_creds: Option<ApiCredentials>,
    /// External signer for Privy-managed wallets (bypasses local k256 signing)
    external_signer: Option<Arc<dyn crate::signer::ExternalSigner>>,
    /// Background heartbeat task handle. Polymarket auto-cancels all open orders
    /// if heartbeats stop arriving. Posts `POST /heartbeats` every ~10 seconds.
    heartbeat_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Polymarket {
    pub fn new(config: PolymarketConfig) -> Result<Self, PolymarketError> {
        let client = HttpClient::new(&config)?;
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(
            config.base.rate_limit_per_second,
        )));

        // Create signer from private key if provided
        let signer = if let Some(ref private_key) = config.private_key {
            let key_hex = if private_key.starts_with("0x") {
                private_key.clone()
            } else {
                format!("0x{private_key}")
            };
            let signer = PrivateKeySigner::from_str(&key_hex)
                .map_err(|e| PolymarketError::Config(format!("invalid private key: {e}")))?
                .with_chain_id(Some(POLYGON));
            Some(signer)
        } else {
            None
        };

        // Pre-set API credentials if provided in config
        let preset_api_creds = if let (Some(key), Some(secret), Some(pass)) =
            (&config.api_key, &config.api_secret, &config.api_passphrase)
        {
            Some(ApiCredentials {
                api_key: key.clone(),
                secret: secret.clone(),
                passphrase: pass.clone(),
            })
        } else {
            None
        };

        Ok(Self {
            config,
            client,
            rate_limiter,
            sdk_state: tokio::sync::OnceCell::new(),
            signer,
            preset_api_creds,
            external_signer: None,
            heartbeat_handle: Arc::new(Mutex::new(None)),
        })
    }

    pub fn with_default_config() -> Result<Self, PolymarketError> {
        Self::new(PolymarketConfig::default())
    }

    /// Attach an external signer (e.g., Privy) for signing orders without local keys.
    pub fn with_external_signer(mut self, signer: Arc<dyn crate::signer::ExternalSigner>) -> Self {
        self.external_signer = Some(signer);
        self
    }

    /// Whether an external signer is configured.
    pub fn has_external_signer(&self) -> bool {
        self.external_signer.is_some()
    }

    pub fn has_private_key(&self) -> bool {
        self.config.private_key.is_some()
    }

    pub fn has_api_credentials(&self) -> bool {
        self.config.has_api_credentials()
    }

    pub fn api_credentials(&self) -> Option<ApiCredentials> {
        self.sdk_state
            .get()
            .map(|s| s.creds.clone())
            .or_else(|| self.preset_api_creds.clone())
    }

    /// Eagerly derive CLOB credentials and initialize the SDK client.
    /// This is optional — all authenticated methods lazily initialize via `ensure_sdk_client`.
    pub async fn init_trading(&self) -> Result<ApiCredentials, PolymarketError> {
        let state = self.ensure_sdk_client().await?;
        Ok(state.creds.clone())
    }

    /// Start the background heartbeat task. Polymarket auto-cancels all open orders
    /// if heartbeats stop arriving. Posts `POST /heartbeats` every ~10 seconds.
    /// Requires the SDK client to be initialized (call `init_trading` first).
    pub async fn start_heartbeat(&self) -> Result<(), PolymarketError> {
        let state = self.ensure_sdk_client().await?;
        let client = Arc::clone(&state.client);
        let handle_lock = Arc::clone(&self.heartbeat_handle);

        // Cancel existing heartbeat if running
        if let Some(h) = handle_lock.lock().await.take() {
            h.abort();
        }

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let guard = client.read().await;
                let result = sdk_dispatch!(&*guard, post_heartbeat(None).await);
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("Polymarket heartbeat failed: {e}");
                    }
                }
            }
        });

        *handle_lock.lock().await = Some(handle);
        Ok(())
    }

    /// Stop the background heartbeat task.
    pub async fn stop_heartbeat(&self) {
        if let Some(h) = self.heartbeat_handle.lock().await.take() {
            h.abort();
        }
    }

    /// Lazily initialize the SDK client, deriving CLOB credentials from the private key
    /// if needed. Returns a reference to the cached SDK state. Thread-safe and idempotent:
    /// concurrent callers wait for the first initialization to complete.
    async fn ensure_sdk_client(&self) -> Result<&SdkState, PolymarketError> {
        self.sdk_state
            .get_or_try_init(|| self.init_sdk_state_inner())
            .await
    }

    /// Core initialization: derive CLOB credentials + authenticate the SDK client.
    async fn init_sdk_state_inner(&self) -> Result<SdkState, PolymarketError> {
        use polymarket_client_sdk_v2::auth::{Credentials, ExposeSecret};

        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| PolymarketError::Auth("private key required for trading".into()))?;

        let unauth_client = SdkClient::new(CLOB_URL, SdkConfig::builder().build())
            .map_err(PolymarketError::from)?;

        // Use pre-set API credentials if available, otherwise derive from Polymarket
        let (sdk_creds, creds) = if let Some(ref existing) = self.preset_api_creds {
            let key_uuid: uuid::Uuid = existing
                .api_key
                .parse()
                .map_err(|e| PolymarketError::Auth(format!("invalid CLOB api_key UUID: {e}")))?;
            let sdk_creds = Credentials::new(
                key_uuid,
                existing.secret.clone(),
                existing.passphrase.clone(),
            );
            let api_key_prefix: String = existing.api_key.chars().take(6).collect();
            info!("Polymarket using stored API key: {}...", api_key_prefix);
            (sdk_creds, existing.clone())
        } else {
            let sdk_creds: Credentials = unauth_client
                .create_or_derive_api_key(signer, None)
                .await
                .map_err(PolymarketError::from)?;
            let creds = ApiCredentials {
                api_key: sdk_creds.key().to_string(),
                secret: sdk_creds.secret().expose_secret().to_string(),
                passphrase: sdk_creds.passphrase().expose_secret().to_string(),
            };
            let api_key_prefix: String = creds.api_key.chars().take(6).collect();
            info!("Polymarket API key derived: {}...", api_key_prefix);
            (sdk_creds, creds)
        };

        let mut builder = unauth_client
            .authentication_builder(signer)
            .signature_type(self.config.signature_type.into())
            .credentials(sdk_creds);

        if let Some(ref funder) = self.config.funder {
            let funder_addr = funder
                .parse()
                .map_err(|e| PolymarketError::Config(format!("invalid funder address: {e}")))?;
            builder = builder.funder(funder_addr);
        }

        let sdk_client = builder
            .authenticate()
            .await
            .map_err(PolymarketError::from)?;

        Ok(SdkState {
            client: Arc::new(RwLock::new(AuthenticatedSdkClient(sdk_client))),
            creds,
        })
    }

    pub async fn set_api_credentials(
        &mut self,
        creds: ApiCredentials,
    ) -> Result<(), PolymarketError> {
        self.preset_api_creds = Some(creds);
        // Reset SDK state so next call re-initializes with the new credentials
        self.sdk_state = tokio::sync::OnceCell::new();
        Ok(())
    }

    /// Initialize the SDK client from pre-existing CLOB API credentials.
    ///
    /// Used for Privy-managed wallets where credentials are derived during the
    /// link flow and the local process has no private key. The `wallet_address`
    /// must be the Privy wallet's EOA address so the SDK sends the correct
    /// `POLY_ADDRESS` header in L2 auth.
    pub async fn init_sdk_client_from_creds(
        &mut self,
        wallet_address: &str,
    ) -> Result<(), PolymarketError> {
        use polymarket_client_sdk_v2::auth::Credentials;
        use uuid::Uuid;

        if self.sdk_state.initialized() {
            return Ok(());
        }

        let creds = self
            .preset_api_creds
            .as_ref()
            .ok_or_else(|| PolymarketError::Auth("no API credentials set".into()))?;

        // Build SDK Credentials from our stored values
        let key_uuid: Uuid = creds
            .api_key
            .parse()
            .map_err(|e| PolymarketError::Auth(format!("invalid CLOB api_key UUID: {e}")))?;
        let sdk_creds = Credentials::new(key_uuid, creds.secret.clone(), creds.passphrase.clone());

        // Stub signer that reports the real Privy wallet address.
        // The SDK uses signer.address() for the POLY_ADDRESS L2 header —
        // it must match the address that derived the CLOB API credentials.
        let addr: Address = wallet_address
            .parse()
            .map_err(|e| PolymarketError::Config(format!("invalid wallet address: {e}")))?;
        let stub_signer = AddressOnlySigner {
            addr,
            chain_id: Some(POLYGON),
        };

        let unauth_client = SdkClient::new(CLOB_URL, SdkConfig::builder().build())
            .map_err(PolymarketError::from)?;

        let mut builder = unauth_client
            .authentication_builder(&stub_signer)
            .signature_type(self.config.signature_type.into())
            .credentials(sdk_creds);

        if let Some(ref funder) = self.config.funder {
            let funder_addr = funder
                .parse()
                .map_err(|e| PolymarketError::Config(format!("invalid funder address: {e}")))?;
            builder = builder.funder(funder_addr);
        }

        let sdk_client = builder
            .authenticate()
            .await
            .map_err(PolymarketError::from)?;

        let state = SdkState {
            client: Arc::new(RwLock::new(AuthenticatedSdkClient(sdk_client))),
            creds: creds.clone(),
        };
        let _ = self.sdk_state.set(state);

        // Store wallet address so owner_address() works without a private key
        if self.config.funder.is_none() && self.signer.is_none() {
            self.config.funder = Some(wallet_address.to_lowercase());
        }

        Ok(())
    }

    fn get_signer(&self) -> Result<&PrivateKeySigner, OpenPxError> {
        self.signer.as_ref().ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Authentication(
                "private key required".into(),
            ))
        })
    }

    async fn rate_limit(&self) {
        self.rate_limiter.lock().await.wait().await;
    }

    /// Fetch a single Polymarket event by slug; the upstream payload embeds
    /// the parent series, so we return both in one round-trip. Used by
    /// `fetch_market_lineage`; not exposed on the unified `Exchange` trait.
    pub async fn fetch_event_with_series(
        &self,
        event_ticker: &str,
    ) -> Result<(Event, Option<Series>), OpenPxError> {
        self.rate_limit().await;
        let endpoint = format!("/events/slug/{event_ticker}");
        let value: serde_json::Value = self
            .client
            .get_gamma(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let event = parse_polymarket_event(&value).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "could not parse event: {event_ticker}"
            )))
        })?;

        // Polymarket events embed `series: Series[]`; pick the primary one
        // when present (matches `Event.series_ticker` semantics — the unified
        // model is single-parent).
        let series = value
            .get("series")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(parse_polymarket_series);

        Ok((event, series))
    }

    /// Fetch a single market by slug, numeric id, or condition id. Used
    /// internally by Polymarket's own trait methods (orderbook lookup,
    /// position resolution, fills, …) and by mapping/contract tests; not
    /// exposed on the unified `Exchange` trait — callers use `fetch_markets`
    /// with `market_tickers: vec![ticker]` instead.
    pub async fn fetch_market(&self, market_ticker: &str) -> Result<Market, OpenPxError> {
        self.rate_limit().await;

        // The unified Market.ticker for Polymarket is the slug, so the
        // canonical lookup is `/markets/slug/{slug}`. Numeric ids and condition
        // ids fall back to the filtered `/markets?id=...` query.
        let looks_like_slug = market_ticker
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            && !market_ticker.chars().all(|c| c.is_ascii_digit());
        let data: serde_json::Value = if looks_like_slug {
            let endpoint = format!("/markets/slug/{market_ticker}");
            match self.client.get_gamma::<serde_json::Value>(&endpoint).await {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!("{e}");
                    if msg.contains("404") {
                        return Err(OpenPxError::Exchange(
                            px_core::ExchangeError::MarketNotFound(market_ticker.into()),
                        ));
                    }
                    return Err(OpenPxError::Exchange(e.into()));
                }
            }
        } else {
            let endpoint = format!("/markets?id={market_ticker}");
            let mut list: Vec<serde_json::Value> = self
                .client
                .get_gamma(&endpoint)
                .await
                .map_err(|e| OpenPxError::Exchange(e.into()))?;
            list.pop().ok_or_else(|| {
                OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_ticker.into()))
            })?
        };

        let map_start = Instant::now();
        let mut parsed = self.parse_market(data).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_ticker.into()))
        })?;
        let map_us = map_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.mapping_us",
            "exchange" => "polymarket",
            "operation" => "fetch_market"
        )
        .record(map_us);

        if parsed.token_ids().is_empty() {
            let condition_id = parsed.condition_id.as_deref().unwrap_or("");

            if let Ok(token_ids) = self.fetch_token_ids(condition_id).await {
                if !token_ids.is_empty() {
                    for (i, outcome) in parsed.outcomes.iter_mut().enumerate() {
                        if let Some(tid) = token_ids.get(i) {
                            outcome.token_id = Some(tid.clone());
                        }
                    }
                }
            }
        }

        Ok(parsed)
    }

    /// Get the owner address (funder if set, otherwise signer address)
    fn owner_address(&self) -> Result<String, PolymarketError> {
        if let Some(ref funder) = self.config.funder {
            return Ok(funder.clone());
        }
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| PolymarketError::Auth("private key required".into()))?;
        Ok(format!("{:#x}", signer.address()))
    }

    pub async fn get_orderbook(&self, token_id: &str) -> Result<Orderbook, PolymarketError> {
        self.rate_limit().await;

        let url = format!("{}/book?token_id={}", CLOB_URL, token_id);
        let response = reqwest::get(&url)
            .await
            .map_err(|e| PolymarketError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            // Return MarketNotFound for 404s (e.g., "No orderbook exists for the requested token id")
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(PolymarketError::MarketNotFound(format!(
                    "no orderbook for token: {token_id}"
                )));
            }
            return Err(PolymarketError::Api(format!(
                "get orderbook failed: {status} - {text}"
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PolymarketError::Api(e.to_string()))?;

        let parse_levels = |key: &str| -> Vec<PriceLevel> {
            data.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let price = item
                                .get("price")
                                .and_then(|p| {
                                    p.as_str()
                                        .and_then(|s| s.parse().ok())
                                        .or_else(|| p.as_f64())
                                })
                                .unwrap_or(0.0);
                            let size = item
                                .get("size")
                                .and_then(|s| {
                                    s.as_str()
                                        .and_then(|s| s.parse().ok())
                                        .or_else(|| s.as_f64())
                                })
                                .unwrap_or(0.0);
                            if price > 0.0 && size > 0.0 {
                                Some(PriceLevel::new(price, size))
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let mut bids = parse_levels("bids");
        let mut asks = parse_levels("asks");

        sort_bids(&mut bids);
        sort_asks(&mut asks);

        // Prefer server timestamp, fall back to local time
        let timestamp = data
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .or_else(|| Some(chrono::Utc::now()));

        Ok(Orderbook {
            asset_id: token_id.to_string(),
            bids,
            asks,
            last_update_id: None,
            timestamp,
            hash: None,
        })
    }

    fn parse_sdk_order(
        &self,
        resp: &polymarket_client_sdk_v2::clob::types::response::OpenOrderResponse,
    ) -> Order {
        let side = match resp.side {
            Side::Buy => OrderSide::Buy,
            Side::Sell => OrderSide::Sell,
            Side::Unknown => OrderSide::Buy, // default
            _ => OrderSide::Buy,             // Handle non-exhaustive enum
        };

        let status = match &resp.status {
            polymarket_client_sdk_v2::clob::types::OrderStatusType::Live => OrderStatus::Open,
            polymarket_client_sdk_v2::clob::types::OrderStatusType::Matched => OrderStatus::Filled,
            polymarket_client_sdk_v2::clob::types::OrderStatusType::Canceled => {
                OrderStatus::Cancelled
            }
            polymarket_client_sdk_v2::clob::types::OrderStatusType::Delayed => OrderStatus::Open,
            polymarket_client_sdk_v2::clob::types::OrderStatusType::Unmatched => {
                OrderStatus::Cancelled
            }
            polymarket_client_sdk_v2::clob::types::OrderStatusType::Unknown(_) => OrderStatus::Open,
            _ => OrderStatus::Open, // Handle non-exhaustive enum
        };

        let price = f64::try_from(resp.price).unwrap_or(0.0);
        let size = f64::try_from(resp.original_size).unwrap_or(0.0);
        let filled = f64::try_from(resp.size_matched).unwrap_or(0.0);

        Order {
            id: resp.id.clone(),
            market_ticker: format!("{:#x}", resp.market),
            outcome: resp.outcome.clone(),
            side,
            price,
            size,
            filled,
            status,
            created_at: resp.created_at,
            updated_at: None, // SDK doesn't have updated_at, using None
        }
    }

    pub async fn fetch_token_ids(
        &self,
        condition_id: &str,
    ) -> Result<Vec<String>, PolymarketError> {
        self.rate_limit().await;

        let url = format!("{}/simplified-markets", CLOB_URL);
        let response = reqwest::get(&url)
            .await
            .map_err(|e| PolymarketError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PolymarketError::Api("failed to fetch markets".into()));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PolymarketError::Api(e.to_string()))?;

        let markets_list = data
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_else(|| data.as_array().cloned().unwrap_or_default());

        for market in markets_list {
            let market_ticker = market
                .get("condition_id")
                .or_else(|| market.get("id"))
                .and_then(|v| v.as_str());

            if market_ticker == Some(condition_id) {
                if let Some(tokens) = market.get("tokens").and_then(|v| v.as_array()) {
                    let token_ids: Vec<String> = tokens
                        .iter()
                        .filter_map(|t| {
                            if let Some(obj) = t.as_object() {
                                obj.get("token_id")
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                            } else {
                                t.as_str().map(|s| s.to_string())
                            }
                        })
                        .collect();

                    if !token_ids.is_empty() {
                        return Ok(token_ids);
                    }
                }

                if let Some(clob_tokens) = market.get("clobTokenIds").and_then(|v| v.as_array()) {
                    let token_ids: Vec<String> = clob_tokens
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    if !token_ids.is_empty() {
                        return Ok(token_ids);
                    }
                }
            }
        }

        Err(PolymarketError::Api(format!(
            "token IDs not found for market {condition_id}"
        )))
    }

    pub async fn fetch_public_trades(
        &self,
        market: Option<&str>,
        limit: Option<usize>,
        offset: Option<usize>,
        user: Option<&str>,
        side: Option<&str>,
        taker_only: Option<bool>,
    ) -> Result<Vec<PublicTrade>, PolymarketError> {
        self.rate_limit().await;

        let data_api_url = &self.config.data_api_url;
        const PAGE_SIZE: usize = 500;

        let total_limit = limit.unwrap_or(100);
        let initial_offset = offset.unwrap_or(0);
        let taker = taker_only.unwrap_or(true);

        let mut all_trades: Vec<PublicTrade> = Vec::new();
        let mut current_offset = initial_offset;

        loop {
            let page_limit = PAGE_SIZE.min(total_limit - all_trades.len());
            if page_limit == 0 {
                break;
            }

            let mut url = format!(
                "{data_api_url}/trades?limit={page_limit}&offset={current_offset}&takerOnly={taker}"
            );

            if let Some(m) = market {
                url.push_str(&format!("&market={m}"));
            }
            if let Some(u) = user {
                url.push_str(&format!("&user={u}"));
            }
            if let Some(s) = side {
                url.push_str(&format!("&side={s}"));
            }

            let response = reqwest::get(&url)
                .await
                .map_err(|e| PolymarketError::Network(e.to_string()))?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(PolymarketError::Api(format!(
                    "fetch public trades failed: {status} - {text}"
                )));
            }

            let data: Vec<serde_json::Value> = response
                .json()
                .await
                .map_err(|e| PolymarketError::Api(e.to_string()))?;

            if data.is_empty() {
                break;
            }

            for item in &data {
                if let Some(trade) = self.parse_public_trade(item) {
                    all_trades.push(trade);
                }
            }

            if data.len() < page_limit {
                break;
            }

            current_offset += data.len();

            if all_trades.len() >= total_limit {
                break;
            }
        }

        all_trades.truncate(total_limit);
        Ok(all_trades)
    }

    fn parse_public_trade(&self, data: &serde_json::Value) -> Option<PublicTrade> {
        let obj = data.as_object()?;

        let proxy_wallet = obj
            .get("proxyWallet")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let side = obj
            .get("side")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let asset = obj
            .get("asset")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let condition_id = obj
            .get("conditionId")
            .or_else(|| obj.get("market"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let size = obj
            .get("size")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let price = obj
            .get("price")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0.0);

        let timestamp = obj
            .get("timestamp")
            .or_else(|| obj.get("matchTime"))
            .and_then(|v| {
                v.as_i64().and_then(normalize_timestamp).or_else(|| {
                    v.as_str().and_then(|s| {
                        chrono::DateTime::parse_from_rfc3339(s)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .or_else(|| s.parse::<i64>().ok().and_then(normalize_timestamp))
                    })
                })
            })
            .unwrap_or_else(chrono::Utc::now);

        Some(PublicTrade {
            proxy_wallet,
            side,
            asset,
            condition_id,
            size,
            price,
            timestamp,
            title: obj.get("title").and_then(|v| v.as_str()).map(String::from),
            slug: obj.get("slug").and_then(|v| v.as_str()).map(String::from),
            icon: obj.get("icon").and_then(|v| v.as_str()).map(String::from),
            event_slug: obj
                .get("eventSlug")
                .and_then(|v| v.as_str())
                .map(String::from),
            outcome: obj
                .get("outcome")
                .and_then(|v| v.as_str())
                .map(String::from),
            outcome_index: obj
                .get("outcomeIndex")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32),
            name: obj.get("name").and_then(|v| v.as_str()).map(String::from),
            pseudonym: obj
                .get("pseudonym")
                .and_then(|v| v.as_str())
                .map(String::from),
            bio: obj.get("bio").and_then(|v| v.as_str()).map(String::from),
            profile_image: obj
                .get("profileImage")
                .and_then(|v| v.as_str())
                .map(String::from),
            profile_image_optimized: obj
                .get("profileImageOptimized")
                .and_then(|v| v.as_str())
                .map(String::from),
            transaction_hash: obj
                .get("transactionHash")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }

    /// Convert a Data API trade object into a unified `Fill`.
    /// Data API lacks per-fill maker/taker — `is_taker` defaults to false.
    /// Real-time WS fills already carry `liquidity_role` via ws-liquidity-role.
    fn parse_poly_fill(&self, trade: &PublicTrade) -> Fill {
        let side = match trade.side.to_uppercase().as_str() {
            "BUY" => OrderSide::Buy,
            _ => OrderSide::Sell,
        };
        let outcome = trade.outcome.clone().unwrap_or_else(|| "Yes".to_string());

        Fill {
            fill_id: trade.transaction_hash.clone().unwrap_or_default(),
            order_id: String::new(), // Data API doesn't expose order IDs
            market_ticker: trade.condition_id.clone(),
            outcome,
            side,
            price: trade.price,
            size: trade.size,
            is_taker: false, // Data API has no maker/taker field; WS fills have liquidity_role
            fee: 0.0,        // Data API doesn't expose fees
            created_at: trade.timestamp,
        }
    }

    async fn fetch_all_positions(&self, user: &str) -> Result<Vec<Position>, OpenPxError> {
        let data_api_url = &self.config.data_api_url;

        let url = format!("{data_api_url}/positions?user={user}&sizeThreshold=0.01&limit=500");
        let response = reqwest::get(&url)
            .await
            .map_err(|e| OpenPxError::Network(px_core::NetworkError::Http(e.to_string())))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "fetch positions failed: {status} - {text}"
            ))));
        }

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;

        let positions = data
            .iter()
            .filter_map(|item| {
                let obj = item.as_object()?;

                let market_ticker = obj.get("conditionId").and_then(|v| v.as_str())?.to_string();

                let outcome = obj
                    .get("outcome")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let size = obj
                    .get("size")
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or(v.as_f64()))
                    .unwrap_or(0.0);

                let average_price = obj
                    .get("avgPrice")
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or(v.as_f64()))
                    .unwrap_or(0.0);

                let current_price = obj
                    .get("curPrice")
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or(v.as_f64()))
                    .unwrap_or(0.0);

                if size <= 0.0 {
                    return None;
                }

                Some(Position {
                    market_ticker,
                    outcome,
                    size,
                    average_price,
                    current_price,
                })
            })
            .collect();

        Ok(positions)
    }

    pub async fn fetch_positions_for_market(
        &self,
        market: &Market,
    ) -> Result<Vec<Position>, PolymarketError> {
        let condition_id = market
            .condition_id
            .as_deref()
            .ok_or_else(|| PolymarketError::Api("market missing condition_id".into()))?;
        self.fetch_positions(Some(condition_id))
            .await
            .map_err(|e| PolymarketError::Api(format!("{e}")))
    }

    fn parse_market(&self, data: serde_json::Value) -> Option<Market> {
        let obj = data.as_object()?;

        fn parse_f64(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<f64> {
            obj.get(key).and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
        }

        fn parse_str(
            obj: &serde_json::Map<String, serde_json::Value>,
            key: &str,
        ) -> Option<String> {
            obj.get(key).and_then(|v| v.as_str()).map(String::from)
        }

        fn parse_datetime(
            obj: &serde_json::Map<String, serde_json::Value>,
            key: &str,
        ) -> Option<chrono::DateTime<chrono::Utc>> {
            obj.get(key).and_then(|v| {
                v.as_str().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                })
            })
        }

        // ticker = Polymarket slug. Required — refuse to construct a Market
        // with no slug (the spec types it nullable but every live market has
        // one). conditionId stays in `condition_id` for on-chain lookups.
        let ticker = obj
            .get("slug")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())?
            .to_string();
        let numeric_id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let title = obj
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let outcome_labels: Vec<String> = obj
            .get("outcomes")
            .and_then(|v| {
                if let Some(arr) = v.as_array() {
                    Some(
                        arr.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect(),
                    )
                } else if let Some(s) = v.as_str() {
                    serde_json::from_str(s).ok()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| vec!["Yes".into(), "No".into()]);

        let parsed_prices: Vec<f64> = if let Some(prices_val) = obj.get("outcomePrices") {
            if let Some(arr) = prices_val.as_array() {
                arr.iter()
                    .filter_map(|v| {
                        v.as_str()
                            .and_then(|s| s.parse().ok())
                            .or_else(|| v.as_f64())
                    })
                    .collect()
            } else if let Some(s) = prices_val.as_str() {
                serde_json::from_str::<Vec<String>>(s)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|p| p.parse::<f64>().ok())
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let clob_token_ids: Vec<String> = obj
            .get("clobTokenIds")
            .and_then(|v| {
                if let Some(arr) = v.as_array() {
                    Some(
                        arr.iter()
                            .filter_map(|x| x.as_str().map(String::from))
                            .collect(),
                    )
                } else if let Some(s) = v.as_str() {
                    serde_json::from_str(s).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        // Zip labels + prices + token_ids by index into the unified Outcome list.
        let outcomes: Vec<Outcome> = outcome_labels
            .iter()
            .enumerate()
            .map(|(i, label)| Outcome {
                label: label.clone(),
                price: parsed_prices.get(i).copied().filter(|p| *p > 0.0),
                token_id: clob_token_ids.get(i).cloned(),
            })
            .collect();

        let volume = parse_f64(obj, "volumeNum")
            .or_else(|| parse_f64(obj, "volume"))
            .unwrap_or(0.0);

        // Spec field is `orderPriceMinTickSize` — the earlier code read
        // `minimum_tick_size`, which doesn't exist, so every market silently
        // fell back to 0.01.
        let tick_size = parse_f64(obj, "orderPriceMinTickSize");

        let rules = parse_str(obj, "description");

        // event_ticker = parent event slug. The earlier code used
        // `events[0].id` (numeric DB id), but the slug is the human-readable
        // identifier matching `FetchMarketsParams.event_ticker` filter values
        // and Kalshi's `event_ticker` semantics.
        let event_ticker = obj
            .get("events")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|e| e.get("slug"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        // Derive status — Polymarket uses boolean flags on each market object.
        // closed=true means the market has settled (no separate "resolved" state).
        let is_closed = obj.get("closed").and_then(|v| v.as_bool()).unwrap_or(false);
        let is_active = obj.get("active").and_then(|v| v.as_bool()).unwrap_or(true);
        let status = if is_closed {
            MarketStatus::Resolved
        } else if is_active {
            MarketStatus::Active
        } else {
            MarketStatus::Closed
        };

        let market_type = if outcomes.len() == 2 {
            MarketType::Binary
        } else {
            MarketType::Categorical
        };

        let best_bid = parse_f64(obj, "bestBid");
        let best_ask = parse_f64(obj, "bestAsk");

        // Polymarket has no single result field; for resolved markets, the
        // winning outcome is the one whose outcomePrice has settled to 1.0.
        let result = if status == MarketStatus::Resolved {
            parsed_prices
                .iter()
                .position(|p| (*p - 1.0).abs() < 1e-9)
                .and_then(|i| outcome_labels.get(i).cloned())
        } else {
            None
        };

        Some(Market {
            openpx_id: Market::make_openpx_id("polymarket", &ticker),
            exchange: "polymarket".into(),
            ticker,
            event_ticker,
            numeric_id,
            title,
            rules,
            status,
            market_type,
            outcomes,
            condition_id: parse_str(obj, "conditionId"),
            volume,
            volume_24h: parse_f64(obj, "volume24hr"),
            last_trade_price: parse_f64(obj, "lastTradePrice"),
            best_bid,
            best_ask,
            tick_size,
            min_order_size: parse_f64(obj, "orderMinSize"),
            close_time: parse_datetime(obj, "endDate"),
            open_time: parse_datetime(obj, "startDate"),
            created_at: parse_datetime(obj, "createdAt"),
            settlement_time: parse_datetime(obj, "closedTime"),
            neg_risk: obj.get("negRisk").and_then(|v| v.as_bool()),
            neg_risk_market_id: parse_str(obj, "negRiskMarketID"),
            result,
        })
    }

    /// Check current token allowances for all Polymarket contracts.
    /// Returns the approval status for USDC and CTF tokens across all 3 contracts.
    pub async fn check_allowances(&self) -> Result<Vec<AllowanceStatus>, PolymarketError> {
        let owner_str = self.owner_address()?;
        let owner = alloy::primitives::Address::from_str(&owner_str)
            .map_err(|e| PolymarketError::Config(format!("invalid owner address: {e}")))?;

        let approver = TokenApprover::new(self.config.polygon_rpc_url.as_deref());

        approver.check_allowances(owner).await
    }

    /// Set token approvals based on the request.
    /// Requires a private key to sign the approval transactions.
    ///
    /// For EOA wallets (no funder), approvals are executed directly from the signer.
    /// For Safe wallets (funder is set), approvals are executed through the Safe's
    /// `execTransaction()` function, ensuring allowances are set on the Safe address.
    pub async fn set_approvals(
        &self,
        request: &ApprovalRequest,
    ) -> Result<ApprovalResponse, PolymarketError> {
        let private_key =
            self.config.private_key.as_ref().ok_or_else(|| {
                PolymarketError::Auth("private key required for approvals".into())
            })?;

        // Determine if this is a Safe wallet (funder address set)
        let safe_address = self
            .config
            .funder
            .as_ref()
            .map(|f| alloy::primitives::Address::from_str(f))
            .transpose()
            .map_err(|e| PolymarketError::Config(format!("invalid funder address: {e}")))?;

        let approver = TokenApprover::new(self.config.polygon_rpc_url.as_deref());

        approver
            .execute_approvals(private_key, safe_address, request)
            .await
    }

    /// Convenience method to approve all tokens for all Polymarket contracts.
    /// This sets up USDC and CTF approvals for CTF Exchange, Neg Risk CTF Exchange,
    /// and Neg Risk Adapter contracts.
    pub async fn approve_all(&self) -> Result<ApprovalResponse, PolymarketError> {
        self.set_approvals(&ApprovalRequest::all()).await
    }
}

fn polymarket_order_type(params: &HashMap<String, String>) -> Result<OrderType, OpenPxError> {
    let order_type = params
        .get("order_type")
        .map(|v| v.as_str())
        .unwrap_or("gtc");

    match order_type {
        "gtc" => Ok(OrderType::GTC),
        "ioc" => Ok(OrderType::FAK),
        "fok" => Ok(OrderType::FOK),
        _ => Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
            format!("invalid order_type '{order_type}' (allowed: gtc, ioc, fok)"),
        ))),
    }
}

/// Normalize a numeric timestamp that may be seconds or milliseconds.
/// Timestamps at or above 1e12 (~2001 in ms, ~33658 in sec) are treated as milliseconds.
fn normalize_timestamp(ts: i64) -> Option<chrono::DateTime<chrono::Utc>> {
    if ts >= 1_000_000_000_000 {
        // Milliseconds → convert to seconds + nanos.
        chrono::DateTime::from_timestamp(ts / 1000, ((ts % 1000) * 1_000_000) as u32)
    } else {
        chrono::DateTime::from_timestamp(ts, 0)
    }
}

impl Exchange for Polymarket {
    fn id(&self) -> &'static str {
        "polymarket"
    }

    fn name(&self) -> &'static str {
        "Polymarket"
    }

    fn manifest(&self) -> &'static ExchangeManifest {
        &POLYMARKET_MANIFEST
    }

    async fn fetch_markets(
        &self,
        params: &FetchMarketsParams,
    ) -> Result<(Vec<Market>, Option<String>), OpenPxError> {
        // ── market_tickers short-circuit: explicit slug lookup, single round-trip ──
        if !params.market_tickers.is_empty() {
            let mut endpoint = String::from("/markets?");
            for (i, slug) in params.market_tickers.iter().enumerate() {
                if i > 0 {
                    endpoint.push('&');
                }
                endpoint.push_str("slug=");
                endpoint.push_str(slug);
            }
            match params.status {
                Some(MarketStatusFilter::Active) | None => endpoint.push_str("&closed=false"),
                Some(MarketStatusFilter::Closed) | Some(MarketStatusFilter::Resolved) => {
                    endpoint.push_str("&closed=true")
                }
                Some(MarketStatusFilter::All) => {}
            }
            self.rate_limit().await;
            let raw_markets: Vec<serde_json::Value> = self
                .client
                .get_gamma(&endpoint)
                .await
                .map_err(|e| OpenPxError::Exchange(e.into()))?;
            let markets: Vec<Market> = raw_markets
                .into_iter()
                .filter_map(|raw| self.parse_market(raw))
                .collect();
            return Ok((markets, None));
        }

        // ── event_ticker short-circuit: fetch a single event's nested markets ──
        // event_ticker is the Polymarket event slug. Numeric event ids are
        // intentionally not accepted here — a future `event_numeric_id` field
        // will carry that case.
        if let Some(ref eid) = params.event_ticker {
            let endpoint = format!("/events/slug/{eid}");
            self.rate_limit().await;

            let event: serde_json::Value = self
                .client
                .get_gamma(&endpoint)
                .await
                .map_err(|e| OpenPxError::Exchange(e.into()))?;

            // Native event identifier — prefer the slug (matches the unified
            // `event_ticker` semantics); fall back to id when slug is missing.
            let native_event_slug = event
                .get("slug")
                .and_then(|v| v.as_str())
                .or_else(|| event.get("id").and_then(|v| v.as_str()))
                .unwrap_or_default()
                .to_string();

            let filter = params.status.unwrap_or(MarketStatusFilter::Active);
            let mut markets = Vec::new();

            if let Some(market_array) = event.get("markets").and_then(|v| v.as_array()) {
                for market_raw in market_array {
                    let raw = market_raw.clone();
                    if let Some(mut parsed) = self.parse_market(raw) {
                        if filter != MarketStatusFilter::All {
                            let status_matches = match filter {
                                MarketStatusFilter::Active => parsed.status == MarketStatus::Active,
                                MarketStatusFilter::Closed | MarketStatusFilter::Resolved => {
                                    matches!(
                                        parsed.status,
                                        MarketStatus::Closed | MarketStatus::Resolved
                                    )
                                }
                                MarketStatusFilter::All => unreachable!(),
                            };
                            if !status_matches {
                                continue;
                            }
                        }
                        if parsed.event_ticker.is_none() {
                            parsed.event_ticker = Some(native_event_slug.clone());
                        }
                        markets.push(parsed);
                    }
                }
            }

            return Ok((markets, None));
        }

        // Polymarket /events/keyset filters via the same `closed` boolean as
        // the offset-paginated endpoint. The 2026-04-09 changelog flipped the
        // default to `closed=false`, so we pass it explicitly to be safe.
        let closed_param = match params.status {
            Some(MarketStatusFilter::Active) | None => "active=true&closed=false",
            Some(MarketStatusFilter::Closed) | Some(MarketStatusFilter::Resolved) => "closed=true",
            Some(MarketStatusFilter::All) => "",
        };

        const PAGE_SIZE: usize = 200;
        let cursor_clause = match params.cursor.as_deref() {
            Some(c) if !c.is_empty() => format!("&after_cursor={c}"),
            _ => String::new(),
        };

        // `series_ticker` is slug-semantic; Polymarket only filters by numeric
        // series id, which will arrive on a future `series_numeric_id` field.
        // Until then, we ignore `series_ticker` on Polymarket.

        let endpoint = if closed_param.is_empty() {
            format!("/events/keyset?limit={PAGE_SIZE}{cursor_clause}")
        } else {
            format!("/events/keyset?limit={PAGE_SIZE}&{closed_param}{cursor_clause}")
        };

        self.rate_limit().await;

        // /events/keyset returns an envelope: { events: [...], next_cursor }
        let envelope: serde_json::Value = self
            .client
            .get_gamma(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let events: Vec<serde_json::Value> = envelope
            .get("events")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let next_cursor = envelope
            .get("next_cursor")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let mut markets = Vec::new();

        for event in events {
            let native_event_slug = event
                .get("slug")
                .and_then(|v| v.as_str())
                .or_else(|| event.get("id").and_then(|v| v.as_str()))
                .unwrap_or_default()
                .to_string();

            let Some(market_array) = event.get("markets").and_then(|v| v.as_array()) else {
                continue;
            };

            let filter = params.status.unwrap_or(MarketStatusFilter::Active);

            for market_raw in market_array {
                let raw = market_raw.clone();
                if let Some(mut parsed) = self.parse_market(raw) {
                    // Events can contain markets with mixed statuses — filter to
                    // only return markets matching the requested status.
                    // Polymarket has no separate "closed" vs "resolved" state, so
                    // accept Resolved for both Closed and Resolved requests.
                    if filter != MarketStatusFilter::All {
                        let status_matches = match filter {
                            MarketStatusFilter::Active => parsed.status == MarketStatus::Active,
                            MarketStatusFilter::Closed | MarketStatusFilter::Resolved => {
                                matches!(
                                    parsed.status,
                                    MarketStatus::Closed | MarketStatus::Resolved
                                )
                            }
                            MarketStatusFilter::All => unreachable!(),
                        };
                        if !status_matches {
                            continue;
                        }
                    }
                    if parsed.event_ticker.is_none() {
                        parsed.event_ticker = Some(native_event_slug.clone());
                    }
                    markets.push(parsed);
                }
            }
        }

        info!(total = markets.len(), "polymarket fetch_markets completed");

        Ok((markets, next_cursor))
    }

    async fn fetch_orderbook(&self, asset_id: &str) -> Result<Orderbook, OpenPxError> {
        self.get_orderbook(asset_id)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))
    }

    async fn fetch_trades(
        &self,
        req: TradesRequest,
    ) -> Result<(Vec<MarketTrade>, Option<String>), OpenPxError> {
        let token_id = req.token_id.clone().ok_or_else(|| {
            OpenPxError::InvalidInput("token_id required for polymarket trades".into())
        })?;

        let desired = req.limit.unwrap_or(200).clamp(1, 500);
        let offset: usize = req
            .cursor
            .as_deref()
            .and_then(|c| c.parse().ok())
            .unwrap_or(0);
        // Trades are returned for the whole condition; over-fetch and filter by outcome token.
        let overfetch = (desired.saturating_mul(5)).clamp(desired, 2000);

        // Need Polymarket conditionId (data-api "market" param), not the Gamma market id.
        let condition_id =
            if let Some(condition_id) = req.market_ref.clone().filter(|s| !s.trim().is_empty()) {
                condition_id
            } else {
                let market = self.fetch_market(&req.market_ticker).await?;
                market.condition_id.clone().unwrap_or_default()
            };

        let raw = self
            .fetch_public_trades(
                Some(&condition_id),
                Some(overfetch),
                Some(offset),
                None,
                None,
                Some(true),
            )
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let start_ts = req
            .start_ts
            .and_then(|s| chrono::DateTime::<chrono::Utc>::from_timestamp(s, 0));
        let end_ts = req
            .end_ts
            .and_then(|s| chrono::DateTime::<chrono::Utc>::from_timestamp(s, 0));

        let raw_count = raw.len();

        let mut trades: Vec<MarketTrade> = raw
            .into_iter()
            .filter(|t| t.asset == token_id)
            .filter(|t| {
                if let Some(start) = start_ts {
                    if t.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = end_ts {
                    if t.timestamp > end {
                        return false;
                    }
                }
                true
            })
            .map(|t| {
                let side_str = t.side.trim().to_string();
                let outcome_str = t.outcome.as_deref().unwrap_or("").trim();
                let is_yes = outcome_str.eq_ignore_ascii_case("Yes");
                let is_no = outcome_str.eq_ignore_ascii_case("No");
                MarketTrade {
                    id: t.transaction_hash.clone(),
                    price: t.price,
                    size: t.size,
                    side: (!side_str.is_empty()).then(|| side_str.clone()),
                    aggressor_side: (!side_str.is_empty()).then_some(side_str),
                    timestamp: t.timestamp,
                    source_channel: Cow::Borrowed("polymarket_rest_trade"),
                    tx_hash: t.transaction_hash.clone(),
                    outcome: t.outcome.clone(),
                    yes_price: if is_yes { Some(t.price) } else { None },
                    no_price: if is_no { Some(t.price) } else { None },
                    taker_address: Some(t.proxy_wallet.clone()),
                }
            })
            .collect();

        // data-api returns newest-first. Keep it that way for tape UIs.
        trades.truncate(desired);

        // Provide next_cursor if we got a full page of raw results (more data likely available).
        let next_cursor = if raw_count >= overfetch {
            Some((offset + overfetch).to_string())
        } else {
            None
        };

        Ok((trades, next_cursor))
    }

    async fn create_order(
        &self,
        market_ticker: &str,
        outcome: &str,
        side: OrderSide,
        price: f64,
        size: f64,
        params: HashMap<String, String>,
    ) -> Result<Order, OpenPxError> {
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        // Resolve signing strategy: local key (self/per-request) or external signer (managed)
        let has_local_signer = self.signer.is_some();
        let has_external_signer = self.external_signer.is_some();
        if !has_local_signer && !has_external_signer {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Authentication(
                "no signing method available — provide a private key or configure an external signer".into(),
            )));
        }

        // Resolve token_id: use params if provided, otherwise fetch from market metadata
        let (token_id, market_neg_risk) = if let Some(tid) = params.get("token_id") {
            (tid.clone(), None)
        } else {
            // Fetch market to get token_ids
            let market = self.fetch_market(market_ticker).await?;
            let mut token_ids = market.token_ids();

            if token_ids.is_empty() {
                let condition_id = market.condition_id.as_deref().unwrap_or("");
                token_ids = self
                    .fetch_token_ids(condition_id)
                    .await
                    .map_err(|e| OpenPxError::Exchange(e.into()))?;
            }

            if token_ids.is_empty() {
                return Err(OpenPxError::InvalidInput(
                    "no token IDs found for market".into(),
                ));
            }

            // Map outcome to token index
            let outcomes = &market.outcomes;
            let outcome_idx = if let Ok(idx) = outcome.parse::<usize>() {
                idx
            } else {
                outcomes
                    .iter()
                    .position(|o| o.label.eq_ignore_ascii_case(outcome))
                    .ok_or_else(|| {
                        OpenPxError::InvalidInput(format!(
                            "outcome '{}' not found in market outcomes {:?}",
                            outcome, outcomes
                        ))
                    })?
            };

            let resolved_token_id = token_ids
                .get(outcome_idx)
                .cloned()
                .ok_or_else(|| OpenPxError::InvalidInput("token not found for outcome".into()))?;

            // Extract neg_risk from market
            let neg_risk_from_market = market.neg_risk;

            (resolved_token_id, neg_risk_from_market)
        };

        // Extract neg_risk from params, falling back to market metadata
        let neg_risk = params
            .get("neg_risk")
            .map(|s| s == "true" || s == "1")
            .or(market_neg_risk)
            .unwrap_or(false);

        // Convert token_id to U256
        let token_id_u256 = U256::from_str(&token_id).map_err(|e| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "invalid token_id: {e}"
            )))
        })?;

        // Convert price and size to Decimal
        let price_decimal = Decimal::try_from(price).map_err(|e| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!("invalid price: {e}")))
        })?;
        let size_decimal = Decimal::try_from(size).map_err(|e| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(format!("invalid size: {e}")))
        })?;

        // Determine SDK side
        let sdk_side = match side {
            OrderSide::Buy => Side::Buy,
            OrderSide::Sell => Side::Sell,
        };

        let guard = sdk_state.client.read().await;

        let order_type = polymarket_order_type(&params)?;

        // Pre-populate neg_risk cache if we know it
        if neg_risk {
            sdk_dispatch!(&*guard, set_neg_risk(token_id_u256, true));
        }

        // Build the order using SDK's order builder
        let signable_order = sdk_dispatch!(
            &*guard,
            limit_order()
                .token_id(token_id_u256)
                .side(sdk_side)
                .price(price_decimal)
                .size(size_decimal)
                .order_type(order_type)
                .build()
                .await
                .map_err(|e| {
                    OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                        "order build failed: {e}"
                    )))
                })?
        );

        // Sign the order: local signer (self/per-request) or external signer (managed/Privy)
        let signed_order = if has_local_signer {
            let signer = self.get_signer()?;
            sdk_dispatch!(
                &*guard,
                sign(signer, signable_order).await.map_err(|e| {
                    OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                        "order signing failed: {e}"
                    )))
                })?
            )
        } else {
            // External signer path: build EIP-712 typed data and sign via Privy
            let ext_signer = self.external_signer.as_ref().ok_or_else(|| {
                OpenPxError::Config("external signer required but not configured".into())
            })?;
            let order = signable_order.order();

            // Determine neg_risk for correct exchange contract
            let neg_risk_result = sdk_dispatch!(
                &*guard,
                neg_risk(order.tokenId).await.map_err(|e| {
                    OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                        "neg_risk lookup failed: {e}"
                    )))
                })?
            );
            let exchange_contract = contract_config(POLYGON, neg_risk_result.neg_risk)
                .ok_or_else(|| {
                    OpenPxError::Exchange(px_core::ExchangeError::Api(
                        "missing contract config for Polygon".into(),
                    ))
                })?
                .exchange;

            // Build EIP-712 typed data for Polymarket Order — V2 schema.
            // Domain version bumped to "2" (CTF Exchange V2 cutover 2026-04-28).
            // V1 fields removed: taker, expiration, nonce, feeRateBps.
            // V2 fields added: timestamp (ms), metadata (bytes32), builder (bytes32).
            // Expiration moved out of the signed struct; it travels on the wire body
            // (signable_order.payload), not the EIP-712 message.
            let typed_data = serde_json::json!({
                "types": {
                    "EIP712Domain": [
                        {"name": "name", "type": "string"},
                        {"name": "version", "type": "string"},
                        {"name": "chainId", "type": "uint256"},
                        {"name": "verifyingContract", "type": "address"}
                    ],
                    "Order": [
                        {"name": "salt", "type": "uint256"},
                        {"name": "maker", "type": "address"},
                        {"name": "signer", "type": "address"},
                        {"name": "tokenId", "type": "uint256"},
                        {"name": "makerAmount", "type": "uint256"},
                        {"name": "takerAmount", "type": "uint256"},
                        {"name": "side", "type": "uint8"},
                        {"name": "signatureType", "type": "uint8"},
                        {"name": "timestamp", "type": "uint256"},
                        {"name": "metadata", "type": "bytes32"},
                        {"name": "builder", "type": "bytes32"}
                    ]
                },
                "primary_type": "Order",
                "domain": {
                    "name": "Polymarket CTF Exchange",
                    "version": "2",
                    "chainId": POLYGON.to_string(),
                    "verifyingContract": format!("{:?}", exchange_contract)
                },
                "message": {
                    "salt": order.salt.to_string(),
                    "maker": format!("{:?}", order.maker),
                    "signer": format!("{:?}", order.signer),
                    "tokenId": order.tokenId.to_string(),
                    "makerAmount": order.makerAmount.to_string(),
                    "takerAmount": order.takerAmount.to_string(),
                    "side": order.side,
                    "signatureType": order.signatureType,
                    "timestamp": order.timestamp.to_string(),
                    "metadata": format!("{:?}", order.metadata),
                    "builder": format!("{:?}", order.builder)
                }
            });

            let sig_hex = ext_signer.sign_typed_data(&typed_data).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "external signing failed: {e}"
                )))
            })?;

            // Parse "0x..."-prefixed hex signature into alloy Signature
            let sig_clean = sig_hex.strip_prefix("0x").unwrap_or(&sig_hex);
            let signature = AlloySig::from_str(sig_clean).map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "invalid signature from external signer: {e}"
                )))
            })?;

            // Get the API key (owner) from the derived credentials
            let api_key_str = &sdk_state.creds.api_key;
            let owner = uuid::Uuid::parse_str(api_key_str).map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "invalid API key UUID: {e}"
                )))
            })?;

            SignedOrder::builder()
                .payload(signable_order.payload)
                .signature(signature)
                .order_type(signable_order.order_type)
                .owner(owner)
                .maybe_post_only(signable_order.post_only)
                .build()
        };

        // Post the order
        let send_start = Instant::now();
        let response = sdk_dispatch!(
            &*guard,
            post_order(signed_order).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::OrderRejected(format!(
                    "post order failed: {e}"
                )))
            })?
        );
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.order_http_send_us",
            "exchange" => "polymarket",
            "operation" => "create_order"
        )
        .record(send_us);

        Ok(Order {
            id: response.order_id,
            market_ticker: market_ticker.to_string(),
            outcome: outcome.to_string(),
            side,
            price,
            size,
            filled: 0.0,
            status: OrderStatus::Open,
            created_at: chrono::Utc::now(),
            updated_at: None,
        })
    }

    async fn cancel_order(
        &self,
        order_id: &str,
        _market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let guard = sdk_state.client.read().await;

        // Fetch order details before cancelling. Polymarket's GET /data/order/{id} endpoint
        // only returns active orders - once cancelled, it 404s. We need the order details
        // for the return value since the cancel response only contains order IDs.
        let pre_cancel = sdk_dispatch!(
            &*guard,
            order(order_id).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "fetch order before cancel failed: {e}"
                )))
            })?
        );

        // Cancel and verify via the response's canceled/not_canceled fields
        let send_start = Instant::now();
        let cancel_resp = sdk_dispatch!(
            &*guard,
            cancel_order(order_id).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "cancel order failed: {e}"
                )))
            })?
        );
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.order_http_send_us",
            "exchange" => "polymarket",
            "operation" => "cancel_order"
        )
        .record(send_us);

        // Verify: order must be in canceled list, not in not_canceled
        if let Some(reason) = cancel_resp.not_canceled.get(order_id) {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "cancel rejected: {reason}"
            ))));
        }

        let mut order = self.parse_sdk_order(&pre_cancel);
        order.status = OrderStatus::Cancelled;
        Ok(order)
    }

    async fn fetch_order(
        &self,
        order_id: &str,
        _market_id: Option<&str>,
    ) -> Result<Order, OpenPxError> {
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let guard = sdk_state.client.read().await;

        let send_start = Instant::now();
        let order_resp = sdk_dispatch!(
            &*guard,
            order(order_id).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "fetch order failed: {e}"
                )))
            })?
        );
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.order_http_send_us",
            "exchange" => "polymarket",
            "operation" => "fetch_order"
        )
        .record(send_us);

        Ok(self.parse_sdk_order(&order_resp))
    }

    async fn fetch_open_orders(
        &self,
        _params: Option<FetchOrdersParams>,
    ) -> Result<Vec<Order>, OpenPxError> {
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let guard = sdk_state.client.read().await;

        let send_start = Instant::now();
        let request = OrdersRequest::default();
        let page = sdk_dispatch!(
            &*guard,
            orders(&request, None).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "fetch open orders failed: {e}"
                )))
            })?
        );
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.order_http_send_us",
            "exchange" => "polymarket",
            "operation" => "fetch_open_orders"
        )
        .record(send_us);

        Ok(page.data.iter().map(|o| self.parse_sdk_order(o)).collect())
    }

    async fn fetch_positions(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Position>, OpenPxError> {
        let owner = self
            .owner_address()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let market_ticker = match market_ticker {
            Some(id) => id,
            None => return self.fetch_all_positions(&owner).await,
        };

        // Use the Data API with condition_id filter for rich position data
        // (avgPrice, curPrice, cashPnl) instead of raw on-chain balances.
        let market = self.fetch_market(market_ticker).await?;
        let condition_id = market.condition_id.as_deref();
        if let Some(cid) = condition_id {
            let data_api_url = &self.config.data_api_url;
            let url =
                format!("{data_api_url}/positions?user={owner}&market={cid}&sizeThreshold=0.01");
            if let Ok(response) = reqwest::get(&url).await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<Vec<serde_json::Value>>().await {
                        let positions: Vec<Position> = data
                            .iter()
                            .filter_map(|item| {
                                let obj = item.as_object()?;
                                let outcome =
                                    obj.get("outcome").and_then(|v| v.as_str())?.to_string();
                                let size = obj
                                    .get("size")
                                    .and_then(|v| {
                                        v.as_str().and_then(|s| s.parse().ok()).or(v.as_f64())
                                    })
                                    .unwrap_or(0.0);
                                if size <= 0.0 {
                                    return None;
                                }
                                let average_price = obj
                                    .get("avgPrice")
                                    .and_then(|v| {
                                        v.as_str().and_then(|s| s.parse().ok()).or(v.as_f64())
                                    })
                                    .unwrap_or(0.0);
                                let current_price = obj
                                    .get("curPrice")
                                    .and_then(|v| {
                                        v.as_str().and_then(|s| s.parse().ok()).or(v.as_f64())
                                    })
                                    .unwrap_or(0.0);
                                Some(Position {
                                    market_ticker: market_ticker.to_string(),
                                    outcome,
                                    size,
                                    average_price,
                                    current_price,
                                })
                            })
                            .collect();
                        tracing::info!(
                            exchange = "polymarket",
                            market_ticker,
                            count = positions.len(),
                            "fetched positions via data-api"
                        );
                        return Ok(positions);
                    }
                }
            }
        }

        // Fallback: on-chain balance queries (no avgPrice available)
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let token_ids: Vec<String> = market.token_ids();

        if token_ids.is_empty() {
            return Ok(vec![]);
        }

        let guard = sdk_state.client.read().await;
        let mut positions = Vec::new();

        for (i, token_id) in token_ids.iter().enumerate() {
            let token_id_u256 = match U256::from_str(token_id) {
                Ok(id) => id,
                Err(_) => continue,
            };

            let request = BalanceAllowanceRequest::builder()
                .asset_type(AssetType::Conditional)
                .token_id(token_id_u256)
                .build();

            let balance_resp = match sdk_dispatch!(&*guard, balance_allowance(request).await) {
                Ok(resp) => resp,
                Err(_) => continue,
            };

            let balance = balance_resp
                .balance
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
                / 1_000_000.0;

            if balance > 0.0 {
                let outcome = market
                    .outcomes
                    .get(i)
                    .map(|o| o.label.clone())
                    .unwrap_or_else(|| if i == 0 { "Yes".into() } else { "No".into() });

                let current_price = market.outcomes.get(i).and_then(|o| o.price).unwrap_or(0.0);

                positions.push(Position {
                    market_ticker: market_ticker.to_string(),
                    outcome,
                    size: balance,
                    average_price: 0.0,
                    current_price,
                });
            }
        }

        tracing::info!(
            exchange = "polymarket",
            market_ticker,
            count = positions.len(),
            "fetched positions via on-chain fallback"
        );

        Ok(positions)
    }

    async fn fetch_fills(
        &self,
        market_ticker: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        let owner = self
            .owner_address()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        // Resolve condition_id when filtering by market
        let condition_id = if let Some(mid) = market_ticker {
            let market = self.fetch_market(mid).await?;
            market.condition_id.clone()
        } else {
            None
        };

        let trades = self
            .fetch_public_trades(
                condition_id.as_deref(),
                limit,
                None,
                Some(&owner),
                None,
                Some(false), // takerOnly=false → get all fills (maker + taker)
            )
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let fills: Vec<Fill> = trades.iter().map(|t| self.parse_poly_fill(t)).collect();

        tracing::info!(
            exchange = "polymarket",
            market = market_ticker.unwrap_or("all"),
            count = fills.len(),
            "fetched fills"
        );

        Ok(fills)
    }

    async fn fetch_balance(&self) -> Result<HashMap<String, f64>, OpenPxError> {
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let guard = sdk_state.client.read().await;

        let request = BalanceAllowanceRequest::builder()
            .asset_type(AssetType::Collateral)
            .build();

        let resp = sdk_dispatch!(
            &*guard,
            balance_allowance(request).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "fetch balance failed: {e}"
                )))
            })?
        );

        let balance = resp.balance.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000.0;

        let mut result = HashMap::new();
        result.insert("USDC".to_string(), balance);
        Ok(result)
    }

    async fn refresh_balance(&self) -> Result<(), OpenPxError> {
        let sdk_state = self
            .ensure_sdk_client()
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let guard = sdk_state.client.read().await;

        let request = BalanceAllowanceRequest::builder()
            .asset_type(AssetType::Collateral)
            .build();

        sdk_dispatch!(
            &*guard,
            update_balance_allowance(request).await.map_err(|e| {
                OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                    "refresh balance failed: {e}"
                )))
            })?
        );

        Ok(())
    }

    async fn fetch_market_lineage(
        &self,
        market_ticker: &str,
    ) -> Result<MarketLineage, OpenPxError> {
        let market = self.fetch_market(market_ticker).await?;
        let (event, series) = match market.event_ticker.as_deref() {
            Some(t) => self
                .fetch_event_with_series(t)
                .await
                .map(|(e, s)| (Some(e), s))
                .unwrap_or((None, None)),
            None => (None, None),
        };
        Ok(MarketLineage {
            market,
            event,
            series,
        })
    }

    async fn fetch_orderbooks_batch(
        &self,
        asset_ids: Vec<String>,
    ) -> Result<Vec<Orderbook>, OpenPxError> {
        if asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let body: Vec<serde_json::Value> = asset_ids
            .iter()
            .map(|t| serde_json::json!({ "token_id": t }))
            .collect();
        let url = format!("{}/books", self.config.clob_url);
        let resp = reqwest::Client::new()
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;
        if !resp.status().is_success() {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "polymarket /books HTTP {}",
                resp.status()
            ))));
        }
        let raw: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;
        Ok(raw.iter().filter_map(parse_polymarket_book).collect())
    }

    async fn cancel_all_orders(
        &self,
        market_ticker: Option<&str>,
    ) -> Result<Vec<Order>, OpenPxError> {
        // Sequential loop over the caller's open orders. The Polymarket
        // CLOB has native `DELETE /cancel-all` and `DELETE /cancel-market-orders`
        // endpoints that are O(1). TODO(batch5-v2): swap to those once the
        // V2 SDK lands and exposes them. For now this works against both V1
        // and V2 SDKs and matches the trait contract.
        let params = FetchOrdersParams {
            market_ticker: market_ticker.map(String::from),
        };
        let open = self.fetch_open_orders(Some(params)).await?;

        let mut cancelled = Vec::with_capacity(open.len());
        for order in open {
            match self.cancel_order(&order.id, market_ticker).await {
                Ok(o) => cancelled.push(o),
                Err(e) => tracing::warn!(
                    order_id = %order.id,
                    error = %e,
                    "polymarket cancel_all_orders: skipping failed cancel"
                ),
            }
        }
        Ok(cancelled)
    }

    async fn create_orders_batch(&self, orders: Vec<NewOrder>) -> Result<Vec<Order>, OpenPxError> {
        // Polymarket caps batches at 15 orders per the V2 migration guide.
        if orders.len() > 15 {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::InvalidOrder(
                "create_orders_batch: Polymarket cap is 15 orders per request".into(),
            )));
        }

        // Sequential per-order submission via the existing create_order path.
        // TODO(batch5-v2): when V2 lands, swap to the native `POST /orders`
        // batch endpoint with array body — single round-trip, deferred
        // execution support. For now this works on V1 and matches the
        // unified contract.
        let mut out = Vec::with_capacity(orders.len());
        for o in orders {
            let mut params: HashMap<String, String> = HashMap::new();
            if let Some(tif) = polymarket_unified_tif(o.order_type) {
                params.insert("order_type".to_string(), tif.to_string());
            }
            if let Some(p) = o.post_only {
                params.insert("post_only".to_string(), p.to_string());
            }
            if let Some(c) = &o.client_order_id {
                params.insert("client_order_id".to_string(), c.clone());
            }
            if let Some(ts) = o.expiration_ts {
                params.insert("expiration".to_string(), ts.to_string());
            }
            let order = self
                .create_order(
                    &o.market_ticker,
                    &o.outcome,
                    o.side,
                    o.price,
                    o.size,
                    params,
                )
                .await?;
            out.push(order);
        }
        Ok(out)
    }

    fn describe(&self) -> ExchangeInfo {
        let authed = self.config.is_authenticated();
        ExchangeInfo {
            id: self.id(),
            name: self.name(),
            has_fetch_markets: true,
            has_create_order: authed,
            has_cancel_order: authed,
            has_fetch_positions: true,
            has_fetch_balance: true,
            has_fetch_orderbook: true,
            has_fetch_trades: true,
            has_fetch_fills: true,
            has_fetch_server_time: false,
            has_approvals: true,
            has_refresh_balance: true,
            has_websocket: true,
            has_fetch_market_lineage: true,
            has_fetch_orderbooks_batch: true,
            has_cancel_all_orders: authed,
            has_create_orders_batch: authed,
        }
    }
}

fn parse_polymarket_event(value: &serde_json::Value) -> Option<Event> {
    let obj = value.as_object()?;

    let numeric_id = obj.get("id").and_then(|v| {
        v.as_str()
            .map(String::from)
            .or_else(|| v.as_i64().map(|i| i.to_string()))
            .or_else(|| v.as_u64().map(|u| u.to_string()))
    });

    let ticker = obj.get("slug").and_then(|v| v.as_str())?.to_string();
    let title = obj
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = obj
        .get("description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);
    let category = obj
        .get("category")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Polymarket events embed `series: Series[]`; the series identifier we
    // surface is its `ticker` field (or `slug` fallback when ticker is null).
    let series_ticker = obj
        .get("series")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|s| {
            s.get("ticker")
                .and_then(|v| v.as_str())
                .filter(|t| !t.is_empty())
                .or_else(|| s.get("slug").and_then(|v| v.as_str()))
                .map(String::from)
        });

    let parse_dt = |key: &str| -> Option<chrono::DateTime<chrono::Utc>> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
    };
    let parse_f64 = |key: &str| -> Option<f64> {
        obj.get(key).and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    };

    let start_ts = parse_dt("startDate");
    let end_ts = parse_dt("endDate");
    let last_updated_ts = parse_dt("updatedAt");
    let volume = parse_f64("volume");
    let open_interest = parse_f64("openInterest");

    let mutually_exclusive = obj
        .get("negRisk")
        .or_else(|| obj.get("mutuallyExclusive"))
        .and_then(|v| v.as_bool());

    let market_tickers = obj
        .get("markets")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("slug").and_then(|v| v.as_str().map(String::from)))
                .collect()
        })
        .unwrap_or_default();

    let status = obj
        .get("closed")
        .and_then(|v| v.as_bool())
        .map(|c| if c { "closed" } else { "open" }.to_string());

    Some(Event {
        ticker,
        numeric_id,
        title,
        description,
        category,
        series_ticker,
        status,
        market_tickers,
        start_ts,
        end_ts,
        volume,
        open_interest,
        mutually_exclusive,
        last_updated_ts,
    })
}

fn parse_polymarket_series(value: &serde_json::Value) -> Option<Series> {
    let obj = value.as_object()?;

    let numeric_id = obj.get("id").and_then(|v| {
        v.as_str()
            .map(String::from)
            .or_else(|| v.as_i64().map(|i| i.to_string()))
            .or_else(|| v.as_u64().map(|u| u.to_string()))
    });

    // Polymarket Series exposes both `ticker` and `slug` (both nullable);
    // prefer the explicit `ticker`, fall back to `slug`. Skip the row entirely
    // when neither is present.
    let ticker = obj
        .get("ticker")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| obj.get("slug").and_then(|v| v.as_str()))
        .map(String::from)?;

    let title = obj
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let category = obj
        .get("category")
        .and_then(|v| v.as_str())
        .map(String::from);
    let frequency = obj
        .get("frequency")
        .or_else(|| obj.get("recurrence"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let last_updated_ts = obj
        .get("updatedAt")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let volume = obj.get("volume").and_then(|v| {
        v.as_f64()
            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
    });

    Some(Series {
        ticker,
        numeric_id,
        title,
        category,
        frequency,
        tags: Vec::new(),
        settlement_sources: Vec::<SettlementSource>::new(),
        fee_type: None,
        volume,
        last_updated_ts,
    })
}

/// Parse a Polymarket CLOB /books or /book response object into an Orderbook.
fn parse_polymarket_book(v: &serde_json::Value) -> Option<Orderbook> {
    let obj = v.as_object()?;
    let asset_id = obj
        .get("asset_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let hash = obj.get("hash").and_then(|v| v.as_str()).map(String::from);
    let timestamp = obj
        .get("timestamp")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
        .and_then(chrono::DateTime::from_timestamp_millis);

    let parse_levels = |key: &str| -> Vec<PriceLevel> {
        obj.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|level| {
                        let lvl = level.as_object()?;
                        let price = lvl.get("price").and_then(|p| {
                            p.as_f64()
                                .or_else(|| p.as_str().and_then(|s| s.parse().ok()))
                        })?;
                        let size = lvl.get("size").and_then(|s| {
                            s.as_f64()
                                .or_else(|| s.as_str().and_then(|s| s.parse().ok()))
                        })?;
                        if price > 0.0 && size > 0.0 {
                            Some(PriceLevel::new(price, size))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut bids = parse_levels("bids");
    let mut asks = parse_levels("asks");
    sort_bids(&mut bids);
    sort_asks(&mut asks);

    Some(Orderbook {
        asset_id,
        bids,
        asks,
        last_update_id: None,
        timestamp,
        hash,
    })
}

/// Map the unified `OrderType` to a Polymarket-recognized time-in-force string
/// (consumed by `create_order` via the params map).
fn polymarket_unified_tif(t: UnifiedOrderType) -> Option<&'static str> {
    match t {
        UnifiedOrderType::Gtc => Some("GTC"),
        UnifiedOrderType::Fok => Some("FOK"),
        // Polymarket calls IOC "FAK" (fill-and-kill); the existing
        // `polymarket_order_type` parser maps both spellings.
        UnifiedOrderType::Ioc => Some("FAK"),
    }
}

#[cfg(test)]
mod tests {
    use super::polymarket_order_type;
    use polymarket_client_sdk_v2::clob::types::OrderType;
    use std::collections::HashMap;

    #[test]
    fn order_type_defaults_to_gtc() {
        let params = HashMap::new();
        assert!(matches!(
            polymarket_order_type(&params).unwrap(),
            OrderType::GTC
        ));
    }

    #[test]
    fn order_type_gtc_maps_to_gtc() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "gtc".to_string());
        assert!(matches!(
            polymarket_order_type(&params).unwrap(),
            OrderType::GTC
        ));
    }

    #[test]
    fn order_type_ioc_maps_to_fak() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "ioc".to_string());
        assert!(matches!(
            polymarket_order_type(&params).unwrap(),
            OrderType::FAK
        ));
    }

    #[test]
    fn order_type_fok_maps_to_fok() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "fok".to_string());
        assert!(matches!(
            polymarket_order_type(&params).unwrap(),
            OrderType::FOK
        ));
    }

    #[test]
    fn invalid_order_type_is_rejected() {
        let mut params = HashMap::new();
        params.insert("order_type".to_string(), "market".to_string());
        assert!(polymarket_order_type(&params).is_err());
    }

    // --- Event / Series parser tests (Batch 2) ---

    use super::{parse_polymarket_event, parse_polymarket_series};

    #[test]
    fn parse_polymarket_event_full_payload() {
        let v = serde_json::json!({
            "id": "abc-123",
            "slug": "us-2028-election",
            "title": "2028 US Election",
            "description": "Who wins?",
            "category": "Politics",
            "startDate": "2028-01-01T00:00:00Z",
            "endDate": "2028-11-07T00:00:00Z",
            "updatedAt": "2026-04-27T12:00:00Z",
            "volume": 1234567.5,
            "openInterest": "50000",
            "negRisk": true,
            "closed": false,
            "markets": [
                { "slug": "trump-2028", "conditionId": "0xCID1" },
                { "slug": "harris-2028", "conditionId": "0xCID2" },
            ],
            "series": [{ "id": "ser-99", "ticker": "us-pres" }],
        });
        let e = parse_polymarket_event(&v).expect("parse");
        assert_eq!(e.ticker, "us-2028-election");
        assert_eq!(e.numeric_id.as_deref(), Some("abc-123"));
        assert_eq!(e.category.as_deref(), Some("Politics"));
        assert_eq!(e.series_ticker.as_deref(), Some("us-pres"));
        assert_eq!(e.status.as_deref(), Some("open"));
        assert_eq!(e.volume, Some(1_234_567.5));
        assert_eq!(e.open_interest, Some(50_000.0));
        assert_eq!(e.mutually_exclusive, Some(true));
        assert_eq!(
            e.market_tickers,
            vec!["trump-2028".to_string(), "harris-2028".to_string()]
        );
        assert!(e.start_ts.is_some());
        assert!(e.end_ts.is_some());
    }

    #[test]
    fn parse_polymarket_event_numeric_id_coerces_to_string() {
        let v = serde_json::json!({ "id": 42, "slug": "n", "title": "n" });
        let e = parse_polymarket_event(&v).expect("parse");
        assert_eq!(e.ticker, "n");
        assert_eq!(e.numeric_id.as_deref(), Some("42"));
    }

    #[test]
    fn parse_polymarket_event_closed_event_yields_closed_status() {
        let v = serde_json::json!({ "id": "x", "slug": "x", "title": "t", "closed": true });
        let e = parse_polymarket_event(&v).expect("parse");
        assert_eq!(e.status.as_deref(), Some("closed"));
    }

    #[test]
    fn parse_polymarket_series_minimal() {
        let v = serde_json::json!({
            "id": 7,
            "ticker": "weekly-nfp",
            "title": "Weekly NFP",
            "category": "Economics",
            "recurrence": "weekly",
            "updatedAt": "2026-04-27T00:00:00Z",
            "volume": "98765.43",
        });
        let s = parse_polymarket_series(&v).expect("parse");
        assert_eq!(s.ticker, "weekly-nfp");
        assert_eq!(s.numeric_id.as_deref(), Some("7"));
        assert_eq!(s.title, "Weekly NFP");
        assert_eq!(s.category.as_deref(), Some("Economics"));
        assert_eq!(s.frequency.as_deref(), Some("weekly"));
        assert_eq!(s.volume, Some(98_765.43));
        assert!(s.last_updated_ts.is_some());
    }

    #[test]
    fn parse_polymarket_series_falls_back_to_slug_when_ticker_missing() {
        let v = serde_json::json!({
            "id": 8,
            "slug": "fed-decisions",
            "title": "Fed Rate Decisions",
        });
        let s = parse_polymarket_series(&v).expect("parse");
        assert_eq!(s.ticker, "fed-decisions");
        assert_eq!(s.numeric_id.as_deref(), Some("8"));
    }

    use super::parse_polymarket_book;

    #[test]
    fn parse_polymarket_book_orders_levels_and_carries_metadata() {
        let v = serde_json::json!({
            "asset_id": "0xTOKEN",
            "market": "0xCONDITION",
            "hash": "abc",
            "timestamp": 1714000000000_i64,
            "bids": [
                { "price": "0.5", "size": "100" },
                { "price": "0.6", "size": "50" },
                { "price": "0.0", "size": "9" }
            ],
            "asks": [
                { "price": "0.7", "size": "80" },
                { "price": "0.65", "size": "40" }
            ]
        });
        let b = parse_polymarket_book(&v).expect("parse");
        assert_eq!(b.asset_id, "0xTOKEN");
        assert_eq!(b.hash.as_deref(), Some("abc"));
        // bids sorted descending — best bid first.
        assert!(b.best_bid().unwrap() >= 0.59);
        // asks sorted ascending — best ask first.
        assert!(b.best_ask().unwrap() <= 0.66);
        // Zero-priced level filtered out.
        assert_eq!(b.bids.len(), 2);
    }

    use super::normalize_timestamp;

    #[test]
    fn normalize_timestamp_seconds_vs_milliseconds() {
        // Seconds: 2024-01-01T00:00:00Z
        let sec = 1_704_067_200i64;
        let dt_sec = normalize_timestamp(sec).unwrap();
        assert_eq!(dt_sec.timestamp(), sec);

        // Milliseconds: same instant
        let ms = sec * 1000 + 123;
        let dt_ms = normalize_timestamp(ms).unwrap();
        assert_eq!(dt_ms.timestamp(), sec);
        assert_eq!(dt_ms.timestamp_subsec_millis(), 123);

        // Edge: very small timestamp (year ~1970) stays as seconds.
        let small = 86400i64;
        let dt_small = normalize_timestamp(small).unwrap();
        assert_eq!(dt_small.timestamp(), small);

        // Threshold edge: exactly 1e12 should be treated as milliseconds.
        let threshold = 1_000_000_000_000i64;
        let dt_threshold = normalize_timestamp(threshold).unwrap();
        assert_eq!(dt_threshold.timestamp(), 1_000_000_000);
    }
}
