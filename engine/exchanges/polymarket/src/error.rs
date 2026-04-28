use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolymarketError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api error: {0}")]
    Api(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },
    #[error("authentication required")]
    AuthRequired,
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("market not found: {0}")]
    MarketNotFound(String),
    #[error("signing error: {0}")]
    Signing(String),
}

impl From<PolymarketError> for px_core::ExchangeError {
    fn from(err: PolymarketError) -> Self {
        match err {
            PolymarketError::MarketNotFound(id) => px_core::ExchangeError::MarketNotFound(id),
            PolymarketError::AuthRequired | PolymarketError::Auth(_) => {
                px_core::ExchangeError::Authentication(err.to_string())
            }
            PolymarketError::Api(msg) => px_core::ExchangeError::Api(msg),
            other => px_core::ExchangeError::Api(other.to_string()),
        }
    }
}

impl From<polymarket_client_sdk_v2::error::Error> for PolymarketError {
    fn from(err: polymarket_client_sdk_v2::error::Error) -> Self {
        use polymarket_client_sdk_v2::error::Kind;
        match err.kind() {
            Kind::Status => PolymarketError::Api(err.to_string()),
            Kind::Validation => PolymarketError::Config(err.to_string()),
            Kind::Synchronization => PolymarketError::Auth(err.to_string()),
            Kind::Geoblock => PolymarketError::Api(format!("geoblocked: {err}")),
            Kind::Internal | Kind::WebSocket => PolymarketError::Api(err.to_string()),
            _ => PolymarketError::Api(err.to_string()),
        }
    }
}
