use serde::{Deserialize, Serialize};

/// An open position held by the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Position {
    /// Unified market ticker the position is held on (e.g. `"KXBTCD-25APR1517"`).
    pub market_ticker: String,
    /// Outcome label as published by the exchange (e.g. `"Yes"`, `"No"`).
    pub outcome: String,
    /// Number of contracts held (e.g. `100.0`).
    pub size: f64,
    /// Volume-weighted average entry price as YES probability in `[0, 1]` (e.g. `0.55`).
    pub average_price: f64,
    /// Current mark price as YES probability in `[0, 1]` (e.g. `0.62`).
    pub current_price: f64,
}

impl Position {
    #[inline]
    pub fn cost_basis(&self) -> f64 {
        self.size * self.average_price
    }

    #[inline]
    pub fn current_value(&self) -> f64 {
        self.size * self.current_price
    }

    #[inline]
    pub fn unrealized_pnl(&self) -> f64 {
        self.current_value() - self.cost_basis()
    }

    #[inline]
    pub fn unrealized_pnl_percent(&self) -> f64 {
        let cost = self.cost_basis();
        if cost == 0.0 {
            return 0.0;
        }
        (self.unrealized_pnl() / cost) * 100.0
    }
}
