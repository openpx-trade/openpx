use px_core::define_exchange_error;

define_exchange_error!(LimitlessError {
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("invalid order: {0}")]
    InvalidOrder(String),
});

impl From<LimitlessError> for px_core::ExchangeError {
    fn from(err: LimitlessError) -> Self {
        match err {
            LimitlessError::MarketNotFound(id) => px_core::ExchangeError::MarketNotFound(id),
            LimitlessError::AuthRequired => {
                px_core::ExchangeError::Authentication("authentication required".into())
            }
            LimitlessError::Auth(msg) => px_core::ExchangeError::Authentication(msg),
            LimitlessError::InvalidOrder(msg) => px_core::ExchangeError::InvalidOrder(msg),
            other => px_core::ExchangeError::Api(other.to_string()),
        }
    }
}
