use std::time::Duration;
use thiserror::Error;

/// Generates a per-exchange error enum with the five variants every exchange
/// shares (`Http`, `Api`, `RateLimited`, `AuthRequired`, `MarketNotFound`),
/// plus any exchange-specific variants passed in the body.
#[macro_export]
macro_rules! define_exchange_error {
    ($ErrorType:ident { $($unique_variant:tt)* }) => {
        #[derive(Debug, thiserror::Error)]
        pub enum $ErrorType {
            #[error("http error: {0}")]
            Http(#[from] reqwest::Error),
            #[error("api error: {0}")]
            Api(String),
            #[error("rate limited")]
            RateLimited,
            #[error("authentication required")]
            AuthRequired,
            #[error("market not found: {0}")]
            MarketNotFound(String),
            $($unique_variant)*
        }
    };
}

#[derive(Debug, Error)]
pub enum OpenPxError {
    #[error("network error: {0}")]
    Network(#[from] NetworkError),

    #[error("exchange error: {0}")]
    Exchange(#[from] ExchangeError),

    #[error("websocket error: {0}")]
    WebSocket(#[from] WebSocketError),

    #[error("signing error: {0}")]
    Signing(#[from] SigningError),

    #[error("rate limit exceeded")]
    RateLimitExceeded,

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("{0}")]
    Other(String),
}

impl OpenPxError {
    /// Whether this error is transient and the operation can be retried.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::RateLimitExceeded => true,
            Self::Exchange(e) => e.is_retryable(),
            Self::WebSocket(e) => e.is_retryable(),
            Self::Signing(_) | Self::Config(_) | Self::InvalidInput(_) => false,
            Self::Serialization(_) => false,
            Self::Other(_) => false,
        }
    }

    /// Suggested delay before retrying, if applicable.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimitExceeded => Some(Duration::from_secs(1)),
            Self::Network(NetworkError::Timeout(_)) => Some(Duration::from_millis(500)),
            Self::Network(_) => Some(Duration::from_millis(100)),
            Self::WebSocket(e) => e.retry_after(),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("http request failed: {0}")]
    Http(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("connection failed: {0}")]
    Connection(String),
}

#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("market not found: {0}")]
    MarketNotFound(String),

    #[error("invalid order: {0}")]
    InvalidOrder(String),

    #[error("order rejected: {0}")]
    OrderRejected(String),

    #[error("insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("authentication failed: {0}")]
    Authentication(String),

    #[error("not supported: {0}")]
    NotSupported(String),

    #[error("api error: {0}")]
    Api(String),
}

#[derive(Debug, Clone, Error)]
pub enum WebSocketError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("connection closed")]
    Closed,

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("subscription failed: {0}")]
    Subscription(String),
}

impl WebSocketError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Connection(_) | Self::Closed)
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Connection(_) | Self::Closed => Some(Duration::from_millis(500)),
            _ => None,
        }
    }
}

impl ExchangeError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Api(_))
    }
}

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("invalid private key")]
    InvalidKey,

    #[error("signing failed: {0}")]
    SigningFailed(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),
}
