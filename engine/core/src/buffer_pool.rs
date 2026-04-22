//! Reusable byte-buffer pool.
//!
//! Used by the WS hot path and HTTP body reads to avoid realloc churn. Each
//! buffer defaults to 512 KB; pool size 10. Buffers that have grown past 2x
//! the configured size on return are shrunk back so the pool does not
//! retain pathologically large allocations.

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct BufferPool {
    buffers: Arc<Mutex<Vec<Vec<u8>>>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl BufferPool {
    pub const DEFAULT_BUFFER_SIZE: usize = 512 * 1024;
    pub const DEFAULT_POOL_SIZE: usize = 10;

    pub fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::with_capacity(max_pool_size))),
            buffer_size,
            max_pool_size,
        }
    }

    /// A `Vec<u8>` with capacity `buffer_size` (or more). If the pool is empty,
    /// allocates a fresh one.
    pub async fn get(&self) -> Vec<u8> {
        let mut pool = self.buffers.lock().await;
        pool.pop()
            .map(|mut b| {
                b.clear();
                b
            })
            .unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }

    /// Returns `buf` to the pool, shrinking it if it has grown past 2x the
    /// configured size. If the pool is full, the buffer is dropped.
    pub async fn return_buffer(&self, mut buf: Vec<u8>) {
        buf.clear();
        if buf.capacity() > self.buffer_size * 2 {
            buf.shrink_to(self.buffer_size);
        }
        let mut pool = self.buffers.lock().await;
        if pool.len() < self.max_pool_size {
            pool.push(buf);
        }
    }

    /// Pre-allocates `count` buffers at startup so the first request does not
    /// pay the allocation cost.
    pub async fn prewarm(&self, count: usize) {
        let mut pool = self.buffers.lock().await;
        let target = count.min(self.max_pool_size);
        while pool.len() < target {
            pool.push(Vec::with_capacity(self.buffer_size));
        }
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(Self::DEFAULT_BUFFER_SIZE, Self::DEFAULT_POOL_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_returns_allocated_vec() {
        let pool = BufferPool::new(1024, 4);
        let buf = pool.get().await;
        assert!(buf.capacity() >= 1024);
        assert!(buf.is_empty());
    }

    #[tokio::test]
    async fn return_reuses_under_limit() {
        let pool = BufferPool::new(1024, 2);
        let buf = pool.get().await;
        pool.return_buffer(buf).await;
        pool.return_buffer(Vec::with_capacity(1024)).await;
        pool.return_buffer(Vec::with_capacity(1024)).await;
        assert_eq!(pool.buffers.lock().await.len(), 2);
    }

    #[tokio::test]
    async fn overgrown_buffer_is_shrunk() {
        let pool = BufferPool::new(1024, 4);
        let buf = vec![0u8; 1024 * 8];
        pool.return_buffer(buf).await;
        let reclaimed = pool.get().await;
        assert!(reclaimed.capacity() <= 1024 * 2);
    }

    #[tokio::test]
    async fn prewarm_fills_pool() {
        let pool = BufferPool::new(1024, 8);
        pool.prewarm(5).await;
        assert_eq!(pool.buffers.lock().await.len(), 5);
    }
}
