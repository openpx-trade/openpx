use serde::Serialize;
use std::io::Write;
use tokio::sync::mpsc;

/// Channel-based async NDJSON writer that decouples the hot path from disk I/O.
///
/// Broadcast sites call `write_record()` which serializes to `Vec<u8>` and does
/// a non-blocking `mpsc::send()` — no lock, no disk I/O on the hot path.
/// A dedicated tokio task owns the `BufWriter` and flushes on a timer.
pub struct NdjsonWriter {
    tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl NdjsonWriter {
    /// Spawn a dedicated writer task that owns `writer` exclusively.
    ///
    /// Accepts `W: Write + Send + 'static` so callers can pass `File`,
    /// `BufWriter<File>`, or a future rotation-aware writer without changing
    /// any broadcast site code.
    pub fn new<W: Write + Send + 'static>(writer: W) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(Self::writer_task(writer, rx));
        Self { tx }
    }

    /// Serialize `msg` to NDJSON and enqueue for writing. Non-blocking.
    /// If the writer task has died (channel closed), the record is silently dropped.
    #[inline]
    pub fn write_record<T: Serialize>(&self, msg: &T) {
        match serde_json::to_vec(msg) {
            Ok(mut buf) => {
                buf.push(b'\n');
                // Non-blocking send — if channel is closed, record is dropped
                if self.tx.send(buf).is_err() {
                    tracing::warn!("ndjson writer channel closed, record dropped");
                }
            }
            Err(e) => {
                tracing::error!("ndjson serialization error: {e}");
                metrics::counter!("openpx.ndjson.write_errors").increment(1);
            }
        }
    }

    /// Background task: drains the channel, writes to BufWriter, flushes every 250ms.
    async fn writer_task<W: Write + Send + 'static>(
        inner: W,
        mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    ) {
        use std::io::BufWriter;

        let mut writer = BufWriter::new(inner);
        let mut flush_interval = tokio::time::interval(std::time::Duration::from_millis(250));

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(buf) => {
                            if let Err(e) = writer.write_all(&buf) {
                                tracing::error!("ndjson write error: {e}");
                                metrics::counter!("openpx.ndjson.write_errors").increment(1);
                            }
                        }
                        None => {
                            // Channel closed — flush and exit
                            let _ = writer.flush();
                            return;
                        }
                    }
                }
                _ = flush_interval.tick() => {
                    if let Err(e) = writer.flush() {
                        tracing::error!("ndjson flush error: {e}");
                        metrics::counter!("openpx.ndjson.write_errors").increment(1);
                    }
                }
            }
        }
    }
}
