use alloy::primitives::{Address, ChainId, Signature as AlloySig, B256};
use k256::ecdsa::SigningKey;
use metrics::{counter, histogram};
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
    canonical_event_id, manifests::POLYMARKET_MANIFEST, sort_asks, sort_bids, Candlestick,
    Exchange, ExchangeInfo, ExchangeManifest, FetchMarketsParams, FetchOrdersParams,
    FetchUserActivityParams, Fill, Market, MarketStatus, MarketStatusFilter, MarketTrade,
    MarketType, OpenPxError, Order, OrderSide, OrderStatus, Orderbook, OrderbookHistoryRequest,
    OrderbookSnapshot, OutcomeToken, Position, PriceHistoryInterval, PriceHistoryRequest,
    PriceLevel, PublicTrade, RateLimiter, TradesRequest,
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

/// Parse CLOB `/orderbook-history` response data into `OrderbookSnapshot`s.
fn parse_orderbook_snapshots(data: &serde_json::Value) -> Vec<OrderbookSnapshot> {
    let history = data
        .get("data")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    history
        .into_iter()
        .filter_map(|item| {
            let ts_ms = item.get("timestamp").and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })?;
            let dt = chrono::DateTime::from_timestamp_millis(ts_ms)?;
            let hash = item.get("hash").and_then(|v| v.as_str()).map(String::from);

            let parse_levels = |key: &str| -> Vec<PriceLevel> {
                item.get(key)
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|level| {
                                let price = level.get("price").and_then(|v| {
                                    v.as_f64()
                                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                                })?;
                                let size = level.get("size").and_then(|v| {
                                    v.as_f64()
                                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                                })?;
                                Some(PriceLevel::new(price, size))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };

            Some(OrderbookSnapshot {
                timestamp: dt,
                recorded_at: None,
                hash,
                bids: parse_levels("bids"),
                asks: parse_levels("asks"),
            })
        })
        .collect()
}

/// Parse composite cursor `"chunk_idx:offset"`. Legacy bare offset `"500"` → `(0, 500)`.
fn parse_composite_cursor(cursor: Option<&str>) -> (usize, usize) {
    match cursor {
        None => (0, 0),
        Some(s) => match s.split_once(':') {
            Some((chunk, off)) => (chunk.parse().unwrap_or(0), off.parse().unwrap_or(0)),
            None => (0, s.parse().unwrap_or(0)),
        },
    }
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
            market_id: String::new(),
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
            market_id: format!("{:#x}", resp.market),
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
            let market_id = market
                .get("condition_id")
                .or_else(|| market.get("id"))
                .and_then(|v| v.as_str());

            if market_id == Some(condition_id) {
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
            market_id: trade.condition_id.clone(),
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

                let market_id = obj.get("conditionId").and_then(|v| v.as_str())?.to_string();

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
                    market_id,
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
        self.fetch_positions(Some(&market.id))
            .await
            .map_err(|e| PolymarketError::Api(format!("{e}")))
    }

    /// Fetch current open interest from Polymarket data-api.
    /// Returns the OI value (USDC-denominated outstanding token pairs).
    pub async fn fetch_open_interest(
        &self,
        condition_id: &str,
    ) -> Result<Option<f64>, PolymarketError> {
        let data_api_url = &self.config.data_api_url;
        let url = format!("{data_api_url}/oi?market={condition_id}");
        let response = reqwest::get(&url)
            .await
            .map_err(|e| PolymarketError::Network(e.to_string()))?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| PolymarketError::Api(e.to_string()))?;
        let oi = data
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|obj| obj.get("value"))
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            });
        Ok(oi)
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

        // condition_id is Polymarket's authoritative identifier: on-chain,
        // in every WS payload, and accepted by REST alongside the numeric
        // id. Use it as Market.id so `Market.id == WsUpdate.market_id`
        // without any translation layer. Polymarket's REST numeric `id` is
        // stored separately in `native_numeric_id` for deep-linking callers.
        let id = obj
            .get("conditionId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())?
            .to_string();
        let native_numeric_id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let title = obj
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let outcomes: Vec<String> = obj
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

        // Parse outcome prices
        let mut outcome_prices = HashMap::new();
        if let Some(prices_val) = obj.get("outcomePrices") {
            let price_list: Vec<f64> = if let Some(arr) = prices_val.as_array() {
                arr.iter()
                    .filter_map(|v| {
                        v.as_str()
                            .and_then(|s| s.parse().ok())
                            .or_else(|| v.as_f64())
                    })
                    .collect()
            } else if let Some(s) = prices_val.as_str() {
                // API returns JSON-encoded string array: "[\"0.0045\", \"0.9955\"]"
                serde_json::from_str::<Vec<String>>(s)
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|p| p.parse::<f64>().ok())
                    .collect()
            } else {
                vec![]
            };

            for (outcome, price) in outcomes.iter().zip(price_list.iter()) {
                if *price > 0.0 {
                    outcome_prices.insert(outcome.clone(), *price);
                }
            }
        }

        let volume = parse_f64(obj, "volumeNum")
            .or_else(|| parse_f64(obj, "volume"))
            .unwrap_or(0.0);

        let liquidity = parse_f64(obj, "liquidityNum").or_else(|| parse_f64(obj, "liquidity"));

        let tick_size = parse_f64(obj, "minimum_tick_size").or(Some(0.01));

        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract token IDs from clobTokenIds
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

        let token_id_yes = clob_token_ids.first().cloned();
        let token_id_no = clob_token_ids.get(1).cloned();

        // Build outcome_tokens from outcomes + token IDs
        let outcome_tokens: Vec<OutcomeToken> = outcomes
            .iter()
            .enumerate()
            .filter_map(|(i, outcome)| {
                clob_token_ids.get(i).map(|tid| OutcomeToken {
                    outcome: outcome.clone(),
                    token_id: tid.clone(),
                })
            })
            .collect();

        // Group / event IDs
        let group_id = obj
            .get("events")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|e| e.get("id"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let event_id = group_id
            .as_deref()
            .and_then(|gid| canonical_event_id("polymarket", gid));

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

        // Derive market type
        let market_type = if outcomes.len() == 2 {
            MarketType::Binary
        } else {
            MarketType::Categorical
        };

        let accepting_orders = obj
            .get("acceptingOrders")
            .and_then(|v| v.as_bool())
            .unwrap_or(is_active && !is_closed);

        // Compute spread from best_bid / best_ask
        let best_bid = parse_f64(obj, "bestBid");
        let best_ask = parse_f64(obj, "bestAsk");
        let spread = match (best_bid, best_ask) {
            (Some(b), Some(a)) if a > b => Some(a - b),
            _ => None,
        };

        Some(Market {
            openpx_id: Market::make_openpx_id("polymarket", &id),
            exchange: "polymarket".into(),
            id,
            group_id,
            event_id,
            title: title.clone(),
            question: Some(title),
            description,
            slug: parse_str(obj, "slug"),
            status,
            market_type,
            accepting_orders,
            outcomes,
            outcome_tokens,
            outcome_prices,
            token_id_yes,
            token_id_no,
            condition_id: parse_str(obj, "conditionId"),
            question_id: parse_str(obj, "questionID"),
            native_numeric_id,
            volume,
            volume_24h: parse_f64(obj, "volume24hr"),
            volume_1wk: parse_f64(obj, "volume1wk"),
            volume_1mo: parse_f64(obj, "volume1mo"),
            liquidity,
            last_trade_price: parse_f64(obj, "lastTradePrice"),
            best_bid,
            best_ask,
            spread,
            price_change_1d: parse_f64(obj, "oneDayPriceChange"),
            price_change_1h: parse_f64(obj, "oneHourPriceChange"),
            price_change_1wk: parse_f64(obj, "oneWeekPriceChange"),
            price_change_1mo: parse_f64(obj, "oneMonthPriceChange"),
            tick_size,
            min_order_size: parse_f64(obj, "orderMinSize"),
            close_time: parse_datetime(obj, "endDate")
                .or_else(|| parse_datetime(obj, "endDateIso")),
            open_time: parse_datetime(obj, "startDate")
                .or_else(|| parse_datetime(obj, "startDateIso")),
            created_at: parse_datetime(obj, "createdAt"),
            image_url: parse_str(obj, "image"),
            icon_url: parse_str(obj, "icon"),
            neg_risk: obj.get("negRisk").and_then(|v| v.as_bool()),
            neg_risk_market_id: parse_str(obj, "negRiskMarketID"),
            maker_fee_bps: parse_f64(obj, "makerBaseFee"),
            taker_fee_bps: parse_f64(obj, "takerBaseFee"),
            denomination_token: parse_str(obj, "denominationToken"),
            ..Default::default()
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

    /// Fetch raw `(DateTime, f64)` price points from `/prices-history`.
    ///
    /// Handles the 15-day API range limit by chunking long ranges into sequential
    /// requests and concatenating the results.
    async fn fetch_sub_interval_prices(
        &self,
        token_id: &str,
        start_ts: i64,
        end_ts: i64,
        fidelity_minutes: i64,
    ) -> Result<Vec<(chrono::DateTime<chrono::Utc>, f64)>, OpenPxError> {
        const MAX_RANGE_SECS: i64 = 1_296_000; // 15 days

        let mut all_points = Vec::new();
        let chunks = split_time_range(start_ts, end_ts, MAX_RANGE_SECS);

        for (chunk_start, chunk_end) in chunks {
            self.rate_limit().await;
            let endpoint = format!(
                "/prices-history?market={}&startTs={}&endTs={}&fidelity={}",
                token_id, chunk_start, chunk_end, fidelity_minutes
            );

            let data: serde_json::Value = self
                .client
                .get_clob(&endpoint)
                .await
                .map_err(|e| OpenPxError::Exchange(e.into()))?;

            let history = data
                .get("history")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            for item in history {
                let t = item.get("t").and_then(|v| v.as_i64());
                let p = item.get("p").and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                });
                if let (Some(timestamp), Some(price)) = (t, p) {
                    if let Some(dt) = chrono::DateTime::from_timestamp(timestamp, 0) {
                        all_points.push((dt, price));
                    }
                }
            }
        }

        all_points.sort_by_key(|p| p.0);
        all_points.dedup_by(|a, b| a.0 == b.0);
        Ok(all_points)
    }

    /// Best-effort volume enrichment from data-api `/trades`.
    ///
    /// Paginates through trades, sums `size` per bucket, and tracks the earliest
    /// trade timestamp so callers know which buckets have coverage.
    /// Returns `(volume_map, earliest_trade_ts)`.
    async fn fetch_trade_volume_by_bucket(
        &self,
        condition_id: &str,
        token_id: &str,
        start_ts: i64,
        end_ts: i64,
        bucket_secs: i64,
    ) -> (HashMap<i64, f64>, Option<i64>) {
        const PAGE_SIZE: usize = 500;
        const MAX_OFFSET: usize = 3000;

        let mut volume_map: HashMap<i64, f64> = HashMap::new();
        let mut earliest_trade_ts: Option<i64> = None;
        let mut offset = 0usize;

        loop {
            if offset > MAX_OFFSET {
                break;
            }

            let trades = match self
                .fetch_public_trades(
                    Some(condition_id),
                    Some(PAGE_SIZE),
                    Some(offset),
                    None,
                    None,
                    None,
                )
                .await
            {
                Ok(t) => t,
                Err(_) => break, // network error → return what we have
            };

            if trades.is_empty() {
                break;
            }

            for trade in &trades {
                // Filter by the specific outcome token
                if trade.asset != token_id {
                    continue;
                }
                let ts = trade.timestamp.timestamp();
                // Skip trades outside the requested time window.
                if ts < start_ts || ts > end_ts {
                    continue;
                }
                let bucket_ts = align_to_bucket(ts, bucket_secs);
                *volume_map.entry(bucket_ts).or_default() += trade.size;

                match earliest_trade_ts {
                    Some(e) if ts < e => earliest_trade_ts = Some(ts),
                    None => earliest_trade_ts = Some(ts),
                    _ => {}
                }
            }

            // Stop conditions: oldest trade in page is before our window.
            let oldest_in_page = trades
                .iter()
                .map(|t| t.timestamp.timestamp())
                .min()
                .unwrap_or(i64::MAX);

            if oldest_in_page < start_ts {
                break;
            }

            offset += trades.len();
        }

        (volume_map, earliest_trade_ts)
    }

    /// Fetch a single page of orderbook history from the CLOB with retry on transient errors.
    ///
    /// Returns `(snapshots, total_count)` where `total_count` is the CLOB's reported total
    /// for this time window (used to know if more pages exist).
    async fn fetch_orderbook_history_page(
        &self,
        token_id: &str,
        start_ms: i64,
        end_ms: i64,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<OrderbookSnapshot>, usize), OpenPxError> {
        let endpoint = format!(
            "/orderbook-history?asset_id={}&startTs={}&endTs={}&limit={}&offset={}",
            token_id, start_ms, end_ms, limit, offset
        );

        let max_retries = 2;
        for attempt in 0..=max_retries {
            self.rate_limit().await;

            let result: Result<serde_json::Value, PolymarketError> = self
                .client
                .get_clob_slow(&endpoint, std::time::Duration::from_secs(90))
                .await;

            match result {
                Ok(data) => {
                    let snapshots = parse_orderbook_snapshots(&data);
                    let count = data
                        .get("count")
                        .and_then(|v| {
                            v.as_u64()
                                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                        })
                        .unwrap_or(snapshots.len() as u64) as usize;
                    return Ok((snapshots, count));
                }
                Err(PolymarketError::Api(ref msg)) if msg.starts_with("50") => {
                    if attempt < max_retries {
                        let delay_secs = 2 * (attempt as u64 + 1);
                        tracing::warn!(
                            attempt,
                            delay_secs,
                            "orderbook-history 5xx error, retrying"
                        );
                        counter!("openpx.orderbook_history.retry", "reason" => "5xx").increment(1);
                        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                        continue;
                    }
                }
                Err(PolymarketError::RateLimited { retry_after }) => {
                    if attempt < max_retries {
                        tracing::warn!(
                            attempt,
                            retry_after,
                            "orderbook-history rate limited, retrying"
                        );
                        counter!("openpx.orderbook_history.retry", "reason" => "429").increment(1);
                        tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                        continue;
                    }
                }
                Err(PolymarketError::Http(ref e)) if e.is_timeout() => {
                    if attempt < max_retries {
                        tracing::warn!(attempt, "orderbook-history timeout, retrying");
                        counter!("openpx.orderbook_history.retry", "reason" => "timeout")
                            .increment(1);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                }
                Err(e) => {
                    return Err(OpenPxError::Exchange(e.into()));
                }
            }
        }

        Err(OpenPxError::Exchange(px_core::ExchangeError::Api(
            "orderbook-history failed after retries".into(),
        )))
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

fn align_to_bucket(ts: i64, bucket_secs: i64) -> i64 {
    ts.div_euclid(bucket_secs) * bucket_secs
}

fn first_fully_covered_bucket(earliest_trade_ts: i64, bucket_secs: i64) -> i64 {
    let bucket_start = align_to_bucket(earliest_trade_ts, bucket_secs);
    if earliest_trade_ts.rem_euclid(bucket_secs) == 0 {
        bucket_start
    } else {
        bucket_start + bucket_secs
    }
}

fn split_time_range(start_ts: i64, end_ts: i64, max_range_secs: i64) -> Vec<(i64, i64)> {
    if max_range_secs <= 0 || start_ts >= end_ts {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut chunk_start = start_ts;

    while chunk_start < end_ts {
        let chunk_end = (chunk_start + max_range_secs).min(end_ts);
        chunks.push((chunk_start, chunk_end));
        chunk_start = chunk_end;
    }

    chunks
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

/// Returns `(sub_fidelity_minutes, bucket_seconds)` for reconstructed OHLC.
///
/// We fetch a higher-resolution fidelity from `/prices-history` and aggregate
/// multiple sub-interval samples into each output candle.  For 1-minute candles
/// no sub-interval exists, so we return the native 1-min fidelity and the caller
/// falls back to synthetic prev-close candles.
fn sub_fidelity_for_interval(interval: PriceHistoryInterval) -> (i64, i64) {
    match interval {
        PriceHistoryInterval::OneMinute => (1, 60), // no sub-interval
        PriceHistoryInterval::OneHour => (5, 3_600), // 12 points/candle
        PriceHistoryInterval::SixHours => (30, 21_600), // 12 points/candle
        PriceHistoryInterval::OneDay => (60, 86_400), // 24 points/candle
        PriceHistoryInterval::OneWeek => (360, 604_800), // ~28 points/candle
        PriceHistoryInterval::Max => (60, 86_400),  // daily buckets, 24 pts
    }
}

/// Aggregate sub-interval `(DateTime, f64)` price samples into OHLCV candles.
///
/// Rules:
/// - Bucket alignment: `floor(unix_ts / bucket_secs) * bucket_secs`
/// - Open rule: For non-first buckets, `open = previous_bucket_close` (price continuity).
/// - H/L: `max/min(open, samples...)`, ensuring the open is always within the wick.
/// - Forward-fill: Missing buckets get a flat candle at prev close (O=H=L=C).
fn aggregate_sub_prices_to_candles(
    raw: &[(chrono::DateTime<chrono::Utc>, f64)],
    bucket_secs: i64,
) -> Vec<Candlestick> {
    if raw.is_empty() {
        return Vec::new();
    }

    // Group raw samples by bucket timestamp.
    let mut buckets: std::collections::BTreeMap<i64, Vec<f64>> = std::collections::BTreeMap::new();
    for &(dt, price) in raw {
        let ts = dt.timestamp();
        let bucket_ts = align_to_bucket(ts, bucket_secs);
        buckets.entry(bucket_ts).or_default().push(price);
    }

    let bucket_keys: Vec<i64> = buckets.keys().copied().collect();
    if bucket_keys.is_empty() {
        return Vec::new();
    }
    let first_bucket = bucket_keys[0];
    let last_bucket = bucket_keys[bucket_keys.len() - 1];

    // Build contiguous range of buckets (forward-fill gaps).
    let mut candles = Vec::new();
    let mut prev_close: Option<f64> = None;
    let mut ts = first_bucket;

    while ts <= last_bucket {
        if let Some(samples) = buckets.get(&ts) {
            let open = prev_close.unwrap_or(samples[0]);
            let sample_max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let sample_min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
            let high = open.max(sample_max);
            let low = open.min(sample_min);
            let close = samples[samples.len() - 1];

            candles.push(Candlestick {
                timestamp: chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default(),
                open,
                high,
                low,
                close,
                volume: 0.0,
                open_interest: None,
            });
            prev_close = Some(close);
        } else if let Some(pc) = prev_close {
            // Forward-fill gap: flat candle at prev close.
            candles.push(Candlestick {
                timestamp: chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default(),
                open: pc,
                high: pc,
                low: pc,
                close: pc,
                volume: 0.0,
                open_interest: None,
            });
            // prev_close stays the same
        }
        ts += bucket_secs;
    }

    candles
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
        // ── event_id short-circuit: fetch a single event's nested markets ──
        if let Some(ref eid) = params.event_id {
            // Numeric → /events/{id}, otherwise → /events/slug/{slug}
            let endpoint = if eid.chars().all(|c| c.is_ascii_digit()) {
                format!("/events/{eid}")
            } else {
                format!("/events/slug/{eid}")
            };
            self.rate_limit().await;

            let event: serde_json::Value = self
                .client
                .get_gamma(&endpoint)
                .await
                .map_err(|e| OpenPxError::Exchange(e.into()))?;

            let native_event_id = event
                .get("id")
                .and_then(|v| v.as_str())
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
                        if parsed.group_id.is_none() {
                            parsed.group_id = Some(native_event_id.clone());
                        }
                        if parsed.event_id.is_none() {
                            parsed.event_id = canonical_event_id("polymarket", &native_event_id);
                        }
                        markets.push(parsed);
                    }
                }
            }

            return Ok((markets, None));
        }

        // Polymarket events use boolean query params. The events endpoint
        // filters at the event level, so we need closed=false to avoid
        // getting events whose markets are all closed.
        // Polymarket has no separate "resolved" state — closed=true means settled.
        let status_param = match params.status {
            Some(MarketStatusFilter::Active) | None => Some("active=true&closed=false"),
            Some(MarketStatusFilter::Closed) | Some(MarketStatusFilter::Resolved) => {
                Some("closed=true")
            }
            Some(MarketStatusFilter::All) => None,
        };

        let offset = params
            .cursor
            .as_ref()
            .and_then(|c| c.parse::<usize>().ok())
            .unwrap_or(0);

        let mut endpoint = match status_param {
            Some(sp) => format!("/events?limit=200&{sp}&offset={offset}"),
            None => format!("/events?limit=200&offset={offset}"),
        };
        if let Some(ref sid) = params.series_id {
            endpoint.push_str(&format!("&series_id={sid}"));
        }

        self.rate_limit().await;

        let events: Vec<serde_json::Value> = self
            .client
            .get_gamma(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let events_len = events.len();
        let mut markets = Vec::new();

        for event in events {
            let native_event_id = event
                .get("id")
                .and_then(|v| v.as_str())
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
                    if parsed.group_id.is_none() {
                        parsed.group_id = Some(native_event_id.clone());
                    }
                    if parsed.event_id.is_none() {
                        parsed.event_id = canonical_event_id("polymarket", &native_event_id);
                    }
                    markets.push(parsed);
                }
            }
        }

        let next_cursor = if events_len == 200 {
            Some((offset + events_len).to_string())
        } else {
            None
        };

        info!(total = markets.len(), "polymarket fetch_markets completed");

        Ok((markets, next_cursor))
    }

    async fn fetch_market(&self, market_id: &str) -> Result<Market, OpenPxError> {
        self.rate_limit().await;

        // Use query endpoint instead of /markets/{id} because the query form includes
        // richer event context (`events[0].id`) needed for normalized group_id.
        let endpoint = format!("/markets?id={market_id}");
        let mut data: Vec<serde_json::Value> = self
            .client
            .get_gamma(&endpoint)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;
        let data = data.pop().ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_id.into()))
        })?;

        let map_start = Instant::now();
        let mut parsed = self.parse_market(data).ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::MarketNotFound(market_id.into()))
        })?;
        let map_us = map_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!(
            "openpx.exchange.mapping_us",
            "exchange" => "polymarket",
            "operation" => "fetch_market"
        )
        .record(map_us);

        if parsed.get_token_ids().is_empty() {
            let condition_id = parsed.condition_id.as_deref().unwrap_or(&parsed.id);

            if let Ok(token_ids) = self.fetch_token_ids(condition_id).await {
                if !token_ids.is_empty() {
                    // Update outcome_tokens, token_id_yes, token_id_no
                    parsed.outcome_tokens = parsed
                        .outcomes
                        .iter()
                        .enumerate()
                        .filter_map(|(i, outcome)| {
                            token_ids.get(i).map(|tid| OutcomeToken {
                                outcome: outcome.clone(),
                                token_id: tid.clone(),
                            })
                        })
                        .collect();
                    parsed.token_id_yes = token_ids.first().cloned();
                    parsed.token_id_no = token_ids.get(1).cloned();
                }
            }
        }

        // Enrich with open interest from data-api
        if let Some(condition_id) = parsed.condition_id.clone() {
            if let Ok(Some(oi)) = self.fetch_open_interest(&condition_id).await {
                parsed.open_interest = Some(oi);
            }
        }

        Ok(parsed)
    }

    async fn fetch_orderbook(
        &self,
        req: px_core::OrderbookRequest,
    ) -> Result<Orderbook, OpenPxError> {
        let token_id = if let Some(token_id) = req.token_id.clone() {
            token_id
        } else {
            let market = self.fetch_market(&req.market_id).await?;
            let mut token_ids = market.get_token_ids();

            if token_ids.is_empty() {
                let condition_id = market.condition_id.as_deref().unwrap_or(&market.id);
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

            let outcomes = &market.outcomes;
            let yes_no = outcomes.len() == 2
                && outcomes.iter().any(|o| o.eq_ignore_ascii_case("yes"))
                && outcomes.iter().any(|o| o.eq_ignore_ascii_case("no"));

            let outcome_idx = match req.outcome.as_deref() {
                Some(raw) => {
                    if raw.trim().is_empty() {
                        return Err(OpenPxError::InvalidInput("invalid outcome".into()));
                    }
                    if let Ok(idx) = raw.parse::<usize>() {
                        idx
                    } else {
                        outcomes
                            .iter()
                            .position(|o| o.eq_ignore_ascii_case(raw))
                            .ok_or_else(|| OpenPxError::InvalidInput("invalid outcome".into()))?
                    }
                }
                None => {
                    if yes_no {
                        0
                    } else {
                        return Err(OpenPxError::InvalidInput(
                            "outcome required for non-binary markets".into(),
                        ));
                    }
                }
            };

            token_ids
                .get(outcome_idx)
                .cloned()
                .ok_or_else(|| OpenPxError::InvalidInput("token not found for outcome".into()))?
        };

        let mut orderbook = self
            .get_orderbook(&token_id)
            .await
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        orderbook.market_id = req.market_id.clone();
        orderbook.asset_id = token_id;
        Ok(orderbook)
    }

    async fn fetch_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candlestick>, OpenPxError> {
        let token_id = req.token_id.as_deref().ok_or_else(|| {
            OpenPxError::Exchange(px_core::ExchangeError::Api(
                "token_id required for Polymarket price history".into(),
            ))
        })?;

        let (sub_fidelity, bucket_secs) = sub_fidelity_for_interval(req.interval);

        let now = chrono::Utc::now().timestamp();
        let start_ts = req.start_ts.unwrap_or(now - 86400);
        let end_ts = req.end_ts.unwrap_or(now);

        // Fetch sub-interval prices (always) and trade volume (only when condition_id present).
        let volume_future = async {
            if let Some(cid) = req.condition_id.as_deref() {
                self.fetch_trade_volume_by_bucket(cid, token_id, start_ts, end_ts, bucket_secs)
                    .await
            } else {
                (HashMap::new(), None)
            }
        };
        let (raw_result, (volume_map, earliest_trade_ts)) = tokio::join!(
            self.fetch_sub_interval_prices(token_id, start_ts, end_ts, sub_fidelity),
            volume_future
        );
        let raw = raw_result?;

        // Aggregate sub-interval samples into OHLC candles.
        let mut candles = aggregate_sub_prices_to_candles(&raw, bucket_secs);

        // Attach volume where we have trade coverage.
        let min_fully_covered_bucket =
            earliest_trade_ts.map(|ts| first_fully_covered_bucket(ts, bucket_secs));
        for candle in &mut candles {
            let bucket_ts = candle.timestamp.timestamp();
            if let Some(&vol) = volume_map.get(&bucket_ts) {
                // Only trust volume for buckets within our trade data window.
                if min_fully_covered_bucket.is_some_and(|min_bucket| bucket_ts >= min_bucket) {
                    candle.volume = vol;
                }
            }
        }

        // Attach current OI to the latest candle (historical per-candle OI not available).
        if let Some(last) = candles.last_mut() {
            if let Some(cid) = req.condition_id.as_deref() {
                if let Ok(Some(oi)) = self.fetch_open_interest(cid).await {
                    last.open_interest = Some(oi);
                }
            }
        }

        Ok(candles)
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
                let market = self.fetch_market(&req.market_id).await?;
                market
                    .condition_id
                    .as_deref()
                    .unwrap_or(&market.id)
                    .to_string()
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

    // -----------------------------------------------------------------------
    // CLOB /orderbook-history: Data availability notes (Feb 22, 2026)
    // -----------------------------------------------------------------------
    // This method fetches historical L2 snapshots from Polymarket's CLOB
    // /orderbook-history endpoint. This is an UNDOCUMENTED endpoint — it is
    // not in official Polymarket docs or any client library. It could break
    // or be deprecated without notice.
    //
    // KEY FINDING (verified Feb 22, 2026):
    //   The CLOB orderbook-history indexer is currently STUCK. Every market
    //   globally shows its newest snapshot at exactly 2026-02-20 20:04 UTC.
    //   No new snapshots have been indexed for 50+ hours.
    //
    //   Evidence:
    //   - Tested 20+ markets: ALL markets with data hit the same ceiling
    //   - 14-day continuity scan: data was healthy (6k-17k snaps/day) before
    //     the Feb 20 20:04 UTC cutoff, then zero snapshots after
    //   - The lag grows over time (49.5h → 50h+), confirming the indexer is
    //     frozen rather than running with a fixed delay
    //   - This is a Polymarket infrastructure issue, not ours
    //
    //   NOTE: CLOB /prices-history (used by our /price handler) is a
    //   SEPARATE pipeline and is confirmed LIVE with 0-minute lag.
    //
    // When the indexer IS healthy:
    //   - Records every orderbook state change (event-driven, not sampled)
    //   - Active markets: ~3,000 snapshots/hour (sub-second between changes)
    //   - Quiet markets: ~120 snapshots/hour (~30s between changes)
    //   - 35× more granular than Dome's ~86 snapshots/hour sampling
    //   - Our data is hash-for-hash identical to raw CLOB (verified 3,006/3,006)
    //
    // TODO(2026-02-23): Re-test daily through Feb 25 to check if indexer
    //   recovers. If it stays down, evaluate:
    //   (a) Dome API as a fallback source (lower granularity but always fresh)
    //   (b) Recording our own snapshots from WS feed into the data lake (Phase 2)
    // -----------------------------------------------------------------------
    async fn fetch_orderbook_history(
        &self,
        req: OrderbookHistoryRequest,
    ) -> Result<(Vec<OrderbookSnapshot>, Option<String>), OpenPxError> {
        let token_id = req.token_id.clone().ok_or_else(|| {
            OpenPxError::InvalidInput("token_id required for polymarket orderbook history".into())
        })?;

        let user_limit = req.limit.unwrap_or(500).clamp(1, 1000);

        // CLOB expects milliseconds; default to last 24h if no time range.
        let now_ms = chrono::Utc::now().timestamp_millis();
        let start_ms = req
            .start_ts
            .map(|s| s * 1000)
            .unwrap_or(now_ms - 86_400_000);
        let end_ms = req.end_ts.map(|s| s * 1000).unwrap_or(now_ms);

        // Split [start_ms, end_ms) into 1-day chunks (ascending, oldest first).
        // 1-day windows always succeed; multi-day windows are non-deterministic.
        const DAY_MS: i64 = 86_400_000;
        let mut chunks: Vec<(i64, i64)> = Vec::new();
        let mut cursor = start_ms;
        while cursor < end_ms {
            let chunk_end = (cursor + DAY_MS).min(end_ms);
            chunks.push((cursor, chunk_end));
            cursor = chunk_end;
        }

        // Parse composite cursor → (chunk_idx, offset_within_chunk).
        let (start_chunk_idx, start_offset) = parse_composite_cursor(req.cursor.as_deref());

        let mut collected: Vec<OrderbookSnapshot> = Vec::new();
        let mut last_error: Option<OpenPxError> = None;
        let mut last_visited_chunk = start_chunk_idx;

        for (chunk_idx, &(chunk_start, chunk_end)) in chunks.iter().enumerate() {
            // Skip chunks before cursor position.
            if chunk_idx < start_chunk_idx {
                continue;
            }

            last_visited_chunk = chunk_idx;

            // Stop early if we already have enough snapshots.
            if collected.len() >= user_limit {
                break;
            }

            let remaining = user_limit - collected.len();
            let fetch_limit = remaining.min(1000);
            let page_offset = if chunk_idx == start_chunk_idx {
                start_offset
            } else {
                0
            };

            match self
                .fetch_orderbook_history_page(
                    &token_id,
                    chunk_start,
                    chunk_end,
                    fetch_limit,
                    page_offset,
                )
                .await
            {
                Ok((snapshots, total_count)) => {
                    let page_len = snapshots.len();
                    collected.extend(snapshots);

                    // If this page is full AND there are more records in this chunk,
                    // stay in the current chunk with advanced offset for the next cursor.
                    if page_len >= fetch_limit
                        && (page_offset + page_len) < total_count
                        && collected.len() >= user_limit
                    {
                        // We'll encode the cursor pointing into this chunk.
                        let next_offset = page_offset + page_len;
                        collected.truncate(user_limit);
                        return Ok((collected, Some(format!("{}:{}", chunk_idx, next_offset))));
                    }
                }
                Err(e) => {
                    // Partial failure: if we collected some data, return it with a
                    // cursor pointing to the failed chunk so the user can retry.
                    if !collected.is_empty() {
                        tracing::warn!(
                            chunk_idx,
                            chunks_total = chunks.len(),
                            collected = collected.len(),
                            "orderbook-history chunk failed, returning partial results"
                        );
                        collected.truncate(user_limit);
                        return Ok((collected, Some(format!("{}:0", chunk_idx))));
                    }
                    last_error = Some(e);
                    break;
                }
            }
        }

        // If we collected nothing and there was an error, propagate it.
        if collected.is_empty() {
            if let Some(e) = last_error {
                return Err(e);
            }
        }

        collected.truncate(user_limit);

        // Build next cursor: if we stopped because we hit user_limit and more
        // chunks remain, encode the position. Otherwise all data is exhausted.
        let next_cursor = if collected.len() >= user_limit && last_visited_chunk + 1 < chunks.len()
        {
            Some(format!("{}:0", last_visited_chunk + 1))
        } else {
            None
        };

        Ok((collected, next_cursor))
    }

    async fn create_order(
        &self,
        market_id: &str,
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
            let market = self.fetch_market(market_id).await?;
            let mut token_ids = market.get_token_ids();

            if token_ids.is_empty() {
                let condition_id = market.condition_id.as_deref().unwrap_or(&market.id);
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
                    .position(|o| o.eq_ignore_ascii_case(outcome))
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
            market_id: market_id.to_string(),
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

    async fn fetch_positions(&self, market_id: Option<&str>) -> Result<Vec<Position>, OpenPxError> {
        let owner = self
            .owner_address()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        let market_id = match market_id {
            Some(id) => id,
            None => return self.fetch_all_positions(&owner).await,
        };

        // Use the Data API with condition_id filter for rich position data
        // (avgPrice, curPrice, cashPnl) instead of raw on-chain balances.
        let market = self.fetch_market(market_id).await?;
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
                                    market_id: market_id.to_string(),
                                    outcome,
                                    size,
                                    average_price,
                                    current_price,
                                })
                            })
                            .collect();
                        tracing::info!(
                            exchange = "polymarket",
                            market_id,
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

        let token_ids: Vec<String> = market.get_token_ids();

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
                let outcome = market.outcomes.get(i).cloned().unwrap_or_else(|| {
                    if i == 0 {
                        "Yes".into()
                    } else {
                        "No".into()
                    }
                });

                let current_price = market.outcome_prices.get(&outcome).copied().unwrap_or(0.0);

                positions.push(Position {
                    market_id: market_id.to_string(),
                    outcome,
                    size: balance,
                    average_price: 0.0,
                    current_price,
                });
            }
        }

        tracing::info!(
            exchange = "polymarket",
            market_id,
            count = positions.len(),
            "fetched positions via on-chain fallback"
        );

        Ok(positions)
    }

    async fn fetch_fills(
        &self,
        market_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<Fill>, OpenPxError> {
        let owner = self
            .owner_address()
            .map_err(|e| OpenPxError::Exchange(e.into()))?;

        // Resolve condition_id when filtering by market
        let condition_id = if let Some(mid) = market_id {
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
            market = market_id.unwrap_or("all"),
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

    async fn fetch_balance_raw(&self) -> Result<serde_json::Value, OpenPxError> {
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

        // Convert allowances HashMap to JSON
        let allowances_json: serde_json::Map<String, serde_json::Value> = resp
            .allowances
            .iter()
            .map(|(k, v)| (format!("{:#x}", k), serde_json::Value::String(v.clone())))
            .collect();

        Ok(serde_json::json!({
            "balance": resp.balance.to_string(),
            "allowances": allowances_json
        }))
    }

    async fn fetch_user_activity(
        &self,
        params: FetchUserActivityParams,
    ) -> Result<serde_json::Value, OpenPxError> {
        let data_api_url = &self.config.data_api_url;

        let address = &params.address;
        if address.len() != 42
            || !address.starts_with("0x")
            || !address[2..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(OpenPxError::InvalidInput(format!(
                "invalid Ethereum address: {address}"
            )));
        }

        let limit = params.limit.unwrap_or(100);

        let profile_url = format!("{}/public-profile?address={address}", self.config.gamma_url);
        let positions_url =
            format!("{data_api_url}/positions?user={address}&limit={limit}&sizeThreshold=0");
        let trades_url = format!("{data_api_url}/trades?user={address}&limit={limit}");

        let (profile_result, positions_result, trades_result) = tokio::join!(
            reqwest::get(&profile_url),
            reqwest::get(&positions_url),
            reqwest::get(&trades_url),
        );

        // Profile is soft — 404 or any failure → null
        let profile: serde_json::Value = match profile_result {
            Ok(resp) if resp.status().is_success() => {
                resp.json().await.unwrap_or(serde_json::Value::Null)
            }
            _ => serde_json::Value::Null,
        };

        // Positions — hard failure
        let positions_resp = positions_result
            .map_err(|e| OpenPxError::Network(px_core::NetworkError::Http(e.to_string())))?;
        if !positions_resp.status().is_success() {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "positions HTTP {}",
                positions_resp.status()
            ))));
        }
        let positions: serde_json::Value = positions_resp
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;

        // Trades — hard failure
        let trades_resp = trades_result
            .map_err(|e| OpenPxError::Network(px_core::NetworkError::Http(e.to_string())))?;
        if !trades_resp.status().is_success() {
            return Err(OpenPxError::Exchange(px_core::ExchangeError::Api(format!(
                "trades HTTP {}",
                trades_resp.status()
            ))));
        }
        let trades: serde_json::Value = trades_resp
            .json()
            .await
            .map_err(|e| OpenPxError::Exchange(px_core::ExchangeError::Api(e.to_string())))?;

        Ok(serde_json::json!({
            "profile": profile,
            "positions": positions,
            "trades": trades,
        }))
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
            has_fetch_price_history: true,
            has_fetch_trades: true,
            has_fetch_user_activity: true,
            has_fetch_fills: true,
            has_fetch_server_time: false,
            has_approvals: true,
            has_refresh_balance: true,
            has_websocket: true,
            // CLOB /orderbook-history indexer was temporarily offline (Feb 20-24 2026),
            // now restored. Data range: Nov 12 2025 → Feb 20 2026 (indexer ceiling).
            // Flag is false because historical data is now served from S3 Parquet
            // (backfilled via historical data pipeline), not proxied through CLOB.
            has_fetch_orderbook_history: false,
            has_fetch_events: false,
            has_fetch_event: false,
            has_fetch_orderbooks_batch: false,
            has_fetch_series: false,
            has_fetch_series_one: false,
            has_fetch_midpoint: false,
            has_fetch_midpoints_batch: false,
            has_fetch_spread: false,
            has_fetch_last_trade_price: false,
            has_fetch_open_interest: false,
            has_fetch_user_trades: false,
            has_fetch_market_tags: false,
            has_cancel_all_orders: false,
            has_create_orders_batch: false,
        }
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

    // --- Reconstructed OHLCV tests ---

    use super::{
        aggregate_sub_prices_to_candles, first_fully_covered_bucket, normalize_timestamp,
        split_time_range, sub_fidelity_for_interval,
    };
    use chrono::DateTime;
    use px_core::PriceHistoryInterval;

    fn dt(ts: i64) -> DateTime<chrono::Utc> {
        DateTime::from_timestamp(ts, 0).unwrap()
    }

    #[test]
    fn sub_fidelity_mapping() {
        assert_eq!(
            sub_fidelity_for_interval(PriceHistoryInterval::OneMinute),
            (1, 60)
        );
        assert_eq!(
            sub_fidelity_for_interval(PriceHistoryInterval::OneHour),
            (5, 3_600)
        );
        assert_eq!(
            sub_fidelity_for_interval(PriceHistoryInterval::SixHours),
            (30, 21_600)
        );
        assert_eq!(
            sub_fidelity_for_interval(PriceHistoryInterval::OneDay),
            (60, 86_400)
        );
        assert_eq!(
            sub_fidelity_for_interval(PriceHistoryInterval::OneWeek),
            (360, 604_800)
        );
        assert_eq!(
            sub_fidelity_for_interval(PriceHistoryInterval::Max),
            (60, 86_400)
        );
    }

    #[test]
    fn split_time_range_chunks_exactly_as_expected() {
        let chunks = split_time_range(10, 40, 15);
        assert_eq!(chunks, vec![(10, 25), (25, 40)]);

        let exact_boundary = split_time_range(0, 1_296_000, 1_296_000);
        assert_eq!(exact_boundary, vec![(0, 1_296_000)]);

        let plus_one = split_time_range(0, 1_296_001, 1_296_000);
        assert_eq!(plus_one, vec![(0, 1_296_000), (1_296_000, 1_296_001)]);
    }

    #[test]
    fn split_time_range_handles_invalid_inputs() {
        assert!(split_time_range(100, 100, 15).is_empty());
        assert!(split_time_range(101, 100, 15).is_empty());
        assert!(split_time_range(0, 100, 0).is_empty());
    }

    #[test]
    fn aggregate_multi_point_bucket() {
        // 12 sub-points in a single 1h bucket (base_ts = 3600)
        let base = 3600i64;
        let raw: Vec<(DateTime<chrono::Utc>, f64)> = (0..12)
            .map(|i| (dt(base + i * 300), 0.50 + (i as f64) * 0.01))
            .collect();

        let candles = aggregate_sub_prices_to_candles(&raw, 3600);
        assert_eq!(candles.len(), 1);

        let c = &candles[0];
        assert_eq!(c.timestamp, dt(base));
        assert_eq!(c.open, 0.50); // first bucket uses first sample
        assert!((c.close - 0.61).abs() < 1e-10); // 0.50 + 11*0.01
        assert!((c.high - 0.61).abs() < 1e-10);
        assert!((c.low - 0.50).abs() < 1e-10);
    }

    #[test]
    fn aggregate_single_point_uses_prev_close() {
        // Two buckets: first has multiple points, second has one.
        let raw = vec![
            (dt(3600), 0.40),
            (dt(3900), 0.45),
            (dt(4200), 0.50), // close of bucket 3600
            (dt(7200), 0.55), // single point in bucket 7200
        ];
        let candles = aggregate_sub_prices_to_candles(&raw, 3600);
        assert_eq!(candles.len(), 2);

        let c1 = &candles[1];
        assert_eq!(c1.timestamp, dt(7200));
        assert!((c1.open - 0.50).abs() < 1e-10); // prev close, not 0.55
        assert!((c1.close - 0.55).abs() < 1e-10);
        assert!((c1.high - 0.55).abs() < 1e-10);
        assert!((c1.low - 0.50).abs() < 1e-10);
    }

    #[test]
    fn aggregate_forward_fills_gaps() {
        // Bucket at 3600, gap at 7200, bucket at 10800.
        let raw = vec![(dt(3600), 0.40), (dt(3900), 0.50), (dt(10800), 0.60)];
        let candles = aggregate_sub_prices_to_candles(&raw, 3600);
        assert_eq!(candles.len(), 3); // 3600, 7200 (gap), 10800

        // Gap candle at 7200 should be flat at prev close (0.50).
        let gap = &candles[1];
        assert_eq!(gap.timestamp, dt(7200));
        assert!((gap.open - 0.50).abs() < 1e-10);
        assert!((gap.high - 0.50).abs() < 1e-10);
        assert!((gap.low - 0.50).abs() < 1e-10);
        assert!((gap.close - 0.50).abs() < 1e-10);
    }

    #[test]
    fn aggregate_partial_first_bucket() {
        // Only 3 of the expected 12 sub-points in the first bucket.
        let raw = vec![(dt(3600), 0.45), (dt(3900), 0.48), (dt(4200), 0.42)];
        let candles = aggregate_sub_prices_to_candles(&raw, 3600);
        assert_eq!(candles.len(), 1);

        let c = &candles[0];
        assert!((c.open - 0.45).abs() < 1e-10);
        assert!((c.close - 0.42).abs() < 1e-10);
        assert!((c.high - 0.48).abs() < 1e-10);
        assert!((c.low - 0.42).abs() < 1e-10);
    }

    #[test]
    fn aggregate_open_continuity() {
        // 5 consecutive 1h buckets: each candle's open == previous candle's close.
        let mut raw = Vec::new();
        let prices = [0.50, 0.52, 0.48, 0.55, 0.53];
        for (i, &p) in prices.iter().enumerate() {
            raw.push((dt(3600 + (i as i64) * 3600), p));
        }
        let candles = aggregate_sub_prices_to_candles(&raw, 3600);
        assert_eq!(candles.len(), 5);

        for i in 1..candles.len() {
            assert!(
                (candles[i].open - candles[i - 1].close).abs() < 1e-10,
                "candle {}: open {} != prev close {}",
                i,
                candles[i].open,
                candles[i - 1].close
            );
        }
    }

    #[test]
    fn aggregate_empty_input() {
        let candles = aggregate_sub_prices_to_candles(&[], 3600);
        assert!(candles.is_empty());
    }

    #[test]
    fn volume_coverage_boundary() {
        // Simulate: earliest_trade_ts covers bucket 7200 but not 3600.
        let candles = aggregate_sub_prices_to_candles(&[(dt(3600), 0.50), (dt(7200), 0.55)], 3600);
        assert_eq!(candles.len(), 2);

        let mut volume_map = HashMap::new();
        volume_map.insert(3600i64, 100.0);
        volume_map.insert(7200i64, 200.0);
        let earliest_trade_ts: Option<i64> = Some(7200);
        let bucket_secs = 3600i64;
        let min_fully_covered_bucket =
            earliest_trade_ts.map(|ts| first_fully_covered_bucket(ts, bucket_secs));

        // Replicate the volume-attach logic from fetch_price_history.
        let mut candles = candles;
        for candle in &mut candles {
            let bucket_ts = candle.timestamp.timestamp();
            if let Some(&vol) = volume_map.get(&bucket_ts) {
                if min_fully_covered_bucket.is_some_and(|min_bucket| bucket_ts >= min_bucket) {
                    candle.volume = vol;
                }
            }
        }

        assert!((candles[0].volume - 0.0).abs() < 1e-10); // before coverage
        assert!((candles[1].volume - 200.0).abs() < 1e-10); // within coverage
    }

    #[test]
    fn volume_partial_first_bucket_is_excluded() {
        // earliest_trade_ts is mid-bucket (7300), so bucket 7200 is partial and excluded.
        let candles = aggregate_sub_prices_to_candles(&[(dt(7200), 0.50), (dt(10800), 0.55)], 3600);
        assert_eq!(candles.len(), 2);

        let mut volume_map = HashMap::new();
        volume_map.insert(7200i64, 100.0);
        volume_map.insert(10800i64, 200.0);
        let bucket_secs = 3600i64;
        let min_fully_covered_bucket = Some(first_fully_covered_bucket(7300, bucket_secs));
        assert_eq!(min_fully_covered_bucket, Some(10800));

        let mut candles = candles;
        for candle in &mut candles {
            let bucket_ts = candle.timestamp.timestamp();
            if let Some(&vol) = volume_map.get(&bucket_ts) {
                if min_fully_covered_bucket.is_some_and(|min_bucket| bucket_ts >= min_bucket) {
                    candle.volume = vol;
                }
            }
        }

        assert!((candles[0].volume - 0.0).abs() < 1e-10); // partial bucket excluded
        assert!((candles[1].volume - 200.0).abs() < 1e-10); // full bucket included
    }

    #[test]
    fn aggregate_chunked_range_concatenation() {
        // Simulate two 15-day chunks concatenated: data from day 0-14 and day 15-29.
        // Each "day" gets one sub-interval point. bucket_secs = 86400 (1d).
        let bucket_secs = 86_400i64;
        let base = 1_700_000_000i64; // arbitrary epoch-ish start
        let base_aligned = (base / bucket_secs) * bucket_secs;

        // 30 daily points across two hypothetical 15-day API chunks.
        let raw: Vec<(DateTime<chrono::Utc>, f64)> = (0..30)
            .map(|day| {
                let ts = base_aligned + day * bucket_secs + 3600; // 1h into each day
                (dt(ts), 0.50 + (day as f64) * 0.005)
            })
            .collect();

        let candles = aggregate_sub_prices_to_candles(&raw, bucket_secs);
        assert_eq!(candles.len(), 30);

        // Verify continuity across the chunk boundary (day 14 → day 15).
        assert!(
            (candles[15].open - candles[14].close).abs() < 1e-10,
            "chunk boundary: candle 15 open {} != candle 14 close {}",
            candles[15].open,
            candles[14].close
        );

        // First and last candle prices are correct.
        assert!((candles[0].close - 0.50).abs() < 1e-10);
        assert!((candles[29].close - (0.50 + 29.0 * 0.005)).abs() < 1e-10);
    }

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
