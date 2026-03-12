use napi_derive::napi;

/// Orderbook stream wrapper for Node.js.
/// Structural placeholder for the async generator / callback-based API.
#[napi]
pub struct OrderbookStream {
    _closed: bool,
}

impl Default for OrderbookStream {
    fn default() -> Self {
        Self { _closed: true }
    }
}

#[napi]
impl OrderbookStream {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}
