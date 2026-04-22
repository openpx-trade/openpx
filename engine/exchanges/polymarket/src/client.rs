use metrics::histogram;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Instant;

use crate::config::PolymarketConfig;
use crate::error::PolymarketError;

pub struct HttpClient {
    client: Client,
    gamma_url: String,
    clob_url: String,
    verbose: bool,
}

impl HttpClient {
    pub fn new(config: &PolymarketConfig) -> Result<Self, PolymarketError> {
        // px_core::http::tuned_client_builder() pre-applies the openpx-wide
        // HTTP tunings (HTTP/2 stream window, TCP_NODELAY, pool sizing,
        // keep-alive). Per-exchange overrides layer on top.
        let client = px_core::http::tuned_client_builder()
            .timeout(config.base.timeout)
            .build()?;

        Ok(Self {
            client,
            gamma_url: config.gamma_url.clone(),
            clob_url: config.clob_url.clone(),
            verbose: config.base.verbose,
        })
    }

    pub async fn get_gamma<T: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T, PolymarketError> {
        let url = format!("{}{}", self.gamma_url, endpoint);
        self.get(&url).await
    }

    pub async fn get_clob<T: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T, PolymarketError> {
        let url = format!("{}{}", self.clob_url, endpoint);
        self.get(&url).await
    }

    /// Like `get_clob` but with a per-request timeout override.
    /// Use for slow CLOB endpoints (e.g. `/orderbook-history` which can take 10-60s).
    pub async fn get_clob_slow<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        timeout: std::time::Duration,
    ) -> Result<T, PolymarketError> {
        let url = format!("{}{}", self.clob_url, endpoint);

        if self.verbose {
            tracing::debug!("GET {} (timeout={}s)", url, timeout.as_secs());
        }

        let send_start = Instant::now();
        let response = self.client.get(&url).timeout(timeout).send().await?;
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.http_send_us", "exchange" => "polymarket").record(send_us);
        let status = response.status();
        let headers = response.headers().clone();

        if status == 429 {
            let retry_after = headers
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            return Err(PolymarketError::RateLimited { retry_after });
        }

        let body_start = Instant::now();
        let body = response.text().await?;
        let body_us = body_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.http_body_us", "exchange" => "polymarket").record(body_us);

        if !status.is_success() {
            return Err(PolymarketError::Api(format!("{status}: {body}")));
        }

        let parse_start = Instant::now();
        let parsed = serde_json::from_str(&body)
            .map_err(|e| PolymarketError::InvalidResponse(format!("parse error: {e}")))?;
        let parse_us = parse_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.json_parse_us", "exchange" => "polymarket").record(parse_us);

        Ok(parsed)
    }

    pub async fn get_response(&self, url: &str) -> Result<reqwest::Response, PolymarketError> {
        if self.verbose {
            tracing::debug!("GET {}", url);
        }

        let send_start = Instant::now();
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| PolymarketError::Network(e.to_string()))?;
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.http_send_us", "exchange" => "polymarket").record(send_us);

        Ok(response)
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, PolymarketError> {
        if self.verbose {
            tracing::debug!("GET {}", url);
        }

        let send_start = Instant::now();
        let response = self.client.get(url).send().await?;
        let send_us = send_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.http_send_us", "exchange" => "polymarket").record(send_us);
        let status = response.status();
        let headers = response.headers().clone();

        if status == 429 {
            let retry_after = headers
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            return Err(PolymarketError::RateLimited { retry_after });
        }

        let body_start = Instant::now();
        let body = response.text().await?;
        let body_us = body_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.http_body_us", "exchange" => "polymarket").record(body_us);

        if !status.is_success() {
            return Err(PolymarketError::Api(format!("{status}: {body}")));
        }

        let parse_start = Instant::now();
        let parsed = serde_json::from_str(&body)
            .map_err(|e| PolymarketError::InvalidResponse(format!("parse error: {e}")))?;
        let parse_us = parse_start.elapsed().as_secs_f64() * 1_000_000.0;
        histogram!("openpx.exchange.json_parse_us", "exchange" => "polymarket").record(parse_us);

        Ok(parsed)
    }
}
