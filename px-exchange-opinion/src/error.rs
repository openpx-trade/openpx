use px_core::define_exchange_error;

define_exchange_error!(OpinionError {
    #[error("not supported: {0}")]
    NotSupported(String),
});

impl From<OpinionError> for px_core::ExchangeError {
    fn from(err: OpinionError) -> Self {
        match err {
            OpinionError::MarketNotFound(id) => px_core::ExchangeError::MarketNotFound(id),
            OpinionError::AuthRequired => {
                px_core::ExchangeError::Authentication("authentication required".into())
            }
            other => px_core::ExchangeError::Api(other.to_string()),
        }
    }
}
