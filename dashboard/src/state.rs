use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use px_sdk::ExchangeInner;

/// Shared application state for the dashboard server.
#[derive(Clone)]
pub struct AppState {
    pub exchanges: Arc<RwLock<HashMap<String, ExchangeInner>>>,
    /// Cached balances from startup verification. Updated on subsequent balance fetches.
    pub balances: Arc<RwLock<HashMap<String, HashMap<String, f64>>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            exchanges: Arc::new(RwLock::new(HashMap::new())),
            balances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
