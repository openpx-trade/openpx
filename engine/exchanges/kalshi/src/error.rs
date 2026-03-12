use px_core::define_exchange_error;

define_exchange_error!(KalshiError {
    #[error("not supported: {0}")]
    NotSupported(String),
    #[error("insufficient balance: {0}")]
    InsufficientBalance(String),
    #[error("order rejected: {0}")]
    OrderRejected(String),
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("rsa error: {0}")]
    Rsa(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
});

impl From<KalshiError> for px_core::ExchangeError {
    fn from(err: KalshiError) -> Self {
        match err {
            KalshiError::MarketNotFound(id) => px_core::ExchangeError::MarketNotFound(id),
            KalshiError::AuthRequired => {
                px_core::ExchangeError::Authentication("authentication required".into())
            }
            KalshiError::AuthFailed(msg) => px_core::ExchangeError::Authentication(msg),
            KalshiError::InsufficientBalance(msg) => px_core::ExchangeError::InsufficientFunds(msg),
            KalshiError::OrderRejected(msg) => px_core::ExchangeError::OrderRejected(msg),
            other => px_core::ExchangeError::Api(other.to_string()),
        }
    }
}
