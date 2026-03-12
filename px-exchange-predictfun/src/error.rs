use px_core::define_exchange_error;

define_exchange_error!(PredictFunError {
    #[error("network error: {0}")]
    Network(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    #[error("signing error: {0}")]
    Signing(String),
});

impl From<PredictFunError> for px_core::ExchangeError {
    fn from(err: PredictFunError) -> Self {
        match err {
            PredictFunError::MarketNotFound(id) => px_core::ExchangeError::MarketNotFound(id),
            PredictFunError::AuthRequired | PredictFunError::Auth(_) => {
                px_core::ExchangeError::Authentication(err.to_string())
            }
            PredictFunError::InvalidOrder(msg) => px_core::ExchangeError::InvalidOrder(msg),
            PredictFunError::Api(msg) => px_core::ExchangeError::Api(msg),
            other => px_core::ExchangeError::Api(other.to_string()),
        }
    }
}
