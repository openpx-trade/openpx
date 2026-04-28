//! ESPN public API client.
//!
//! Hits the no-auth `site.api.espn.com` family of endpoints. Coverage spans
//! 17 sports / 100+ leagues; ESPN has no SLA on these endpoints (they're
//! undocumented but stable in practice — used by espn.com itself).

mod scoreboard;
mod state_stream;

pub use state_stream::GameStateStream;

use std::time::Duration;

use reqwest::Client;
use serde::de::DeserializeOwned;

use px_core::error::OpenPxError;

const DEFAULT_BASE: &str = "https://site.api.espn.com/apis/site/v2/sports";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Configuration for the ESPN client. Defaults are sensible — only override
/// `base_url` when pointing at a fixture / mock server.
#[derive(Debug, Clone)]
pub struct EspnConfig {
    pub base_url: String,
    pub timeout: Duration,
    /// Polling interval for `subscribe_game_state`. ESPN scoreboard updates
    /// roughly every ~10s during live games; default 15s gives headroom
    /// without burning quota.
    pub poll_interval: Duration,
}

impl Default for EspnConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE.into(),
            timeout: DEFAULT_TIMEOUT,
            poll_interval: Duration::from_secs(15),
        }
    }
}

/// ESPN client. Cheap to clone (`reqwest::Client` is internally `Arc`-shared).
#[derive(Clone)]
pub struct Espn {
    pub(crate) client: Client,
    pub(crate) config: EspnConfig,
}

impl Espn {
    pub fn new() -> Self {
        Self::with_config(EspnConfig::default())
    }

    pub fn with_config(config: EspnConfig) -> Self {
        let client = px_core::http::tuned_client_builder()
            .timeout(config.timeout)
            .build()
            .expect("reqwest client should build");
        Self { client, config }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, OpenPxError> {
        let url = format!("{}{}", self.config.base_url, path);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| OpenPxError::Other(format!("espn http: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(OpenPxError::Other(format!("espn {status} for {url}")));
        }
        let body = resp
            .text()
            .await
            .map_err(|e| OpenPxError::Other(format!("espn body: {e}")))?;
        serde_json::from_str(&body).map_err(OpenPxError::Serialization)
    }
}

impl Default for Espn {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a callable league id (e.g., "nfl", "nba") to ESPN's `(sport, league)`
/// URL path components. Returns the input split as a fallback for ids the
/// caller passes in pre-formatted (`"football/nfl"`).
pub(crate) fn split_league(league: &str) -> (&str, &str) {
    if let Some((sport, lg)) = league.split_once('/') {
        return (sport, lg);
    }
    match league {
        "nfl" => ("football", "nfl"),
        "ncaaf" | "college-football" => ("football", "college-football"),
        "nba" => ("basketball", "nba"),
        "wnba" => ("basketball", "wnba"),
        "ncaam" | "mens-college-basketball" => ("basketball", "mens-college-basketball"),
        "ncaaw" | "womens-college-basketball" => ("basketball", "womens-college-basketball"),
        "mlb" => ("baseball", "mlb"),
        "nhl" => ("hockey", "nhl"),
        "mls" => ("soccer", "usa.1"),
        "epl" => ("soccer", "eng.1"),
        "laliga" => ("soccer", "esp.1"),
        "bundesliga" => ("soccer", "ger.1"),
        "seriea" => ("soccer", "ita.1"),
        "ucl" => ("soccer", "uefa.champions"),
        "pga" | "pga-tour" => ("golf", "pga"),
        "f1" => ("racing", "f1"),
        "ufc" => ("mma", "ufc"),
        // Default: treat the bare id as a basketball-style league (best-effort).
        other => ("basketball", other),
    }
}
