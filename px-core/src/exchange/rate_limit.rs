use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;

pub struct RateLimiter {
    last_request: Instant,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let min_interval = if requests_per_second > 0 {
            Duration::from_secs_f64(1.0 / requests_per_second as f64)
        } else {
            Duration::ZERO
        };

        Self {
            last_request: Instant::now() - min_interval,
            min_interval,
        }
    }

    pub async fn wait(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            sleep(wait_time).await;
        }
        self.last_request = Instant::now();
    }
}

/// A concurrent rate limiter that enforces a global rate limit across multiple
/// concurrent streams. Uses a semaphore for concurrency control and a shared
/// timestamp to ensure min_interval between ANY two requests globally.
pub struct ConcurrentRateLimiter {
    semaphore: Arc<Semaphore>,
    next_allowed_at: Arc<tokio::sync::Mutex<Instant>>,
    min_interval: Duration,
}

impl ConcurrentRateLimiter {
    /// Create a new concurrent rate limiter.
    ///
    /// # Arguments
    /// * `requests_per_second` - Target requests per second rate limit
    /// * `max_concurrent` - Maximum concurrent requests allowed
    pub fn new(requests_per_second: u32, max_concurrent: usize) -> Self {
        let min_interval = if requests_per_second > 0 {
            Duration::from_secs_f64(1.0 / requests_per_second as f64)
        } else {
            Duration::ZERO
        };

        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            next_allowed_at: Arc::new(tokio::sync::Mutex::new(Instant::now() - min_interval)),
            min_interval,
        }
    }

    /// Acquire a rate limit permit. Waits for both:
    /// 1. A semaphore permit (concurrency limit)
    /// 2. The global rate limit interval since last request
    pub async fn acquire(&self) -> OwnedSemaphorePermit {
        // First acquire semaphore permit for concurrency control
        // Safety: semaphore is never closed (we hold an Arc to it).
        // If it were closed (e.g., memory corruption), panic is appropriate.
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("ConcurrentRateLimiter semaphore unexpectedly closed");

        // Reserve the next globally-allowed send slot while holding the mutex,
        // then sleep outside the lock to avoid serializing concurrent waiters.
        let wait_time = {
            let mut next_allowed = self.next_allowed_at.lock().await;
            let now = Instant::now();
            let scheduled = if now >= *next_allowed {
                now
            } else {
                *next_allowed
            };
            let wait = scheduled.saturating_duration_since(now);
            *next_allowed = scheduled + self.min_interval;
            wait
        };
        if !wait_time.is_zero() {
            sleep(wait_time).await;
        }

        permit
    }
}

pub async fn retry_with_backoff<T, E, F, Fut>(
    max_attempts: u32,
    initial_delay: Duration,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;

    for attempt in 0..max_attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(_) if attempt + 1 < max_attempts => {
                sleep(delay).await;
                delay *= 2;
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_respects_interval() {
        let mut limiter = RateLimiter::new(10);
        let start = Instant::now();

        limiter.wait().await;
        limiter.wait().await;

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(90));
    }
}
