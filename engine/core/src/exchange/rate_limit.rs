use std::sync::atomic::{AtomicU64, Ordering};
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
/// concurrent streams. Uses a semaphore for concurrency control and an atomic
/// timestamp to ensure min_interval between ANY two requests globally.
/// Lock-free: uses AtomicU64 CAS loop instead of a mutex for the timestamp.
pub struct ConcurrentRateLimiter {
    semaphore: Arc<Semaphore>,
    /// Nanoseconds since `epoch` when the next request is allowed.
    next_allowed_nanos: AtomicU64,
    /// Reference instant for converting between Instant and u64 nanos.
    epoch: Instant,
    min_interval_nanos: u64,
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

        let epoch = Instant::now();

        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            next_allowed_nanos: AtomicU64::new(0),
            epoch,
            min_interval_nanos: min_interval.as_nanos() as u64,
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

        // Reserve the next globally-allowed send slot via atomic CAS loop,
        // then sleep outside the atomic to avoid serializing concurrent waiters.
        let wait_nanos = loop {
            let now_nanos = self.epoch.elapsed().as_nanos() as u64;
            let current = self.next_allowed_nanos.load(Ordering::Acquire);
            let scheduled = if now_nanos >= current {
                now_nanos
            } else {
                current
            };
            let next = scheduled + self.min_interval_nanos;
            match self.next_allowed_nanos.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break scheduled.saturating_sub(now_nanos),
                Err(_) => continue, // Another thread won the CAS, retry
            }
        };

        if wait_nanos > 0 {
            sleep(Duration::from_nanos(wait_nanos)).await;
        }

        permit
    }
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
