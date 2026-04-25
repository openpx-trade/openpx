//! Shared WebSocket frame decoding helpers.
//!
//! Every exchange's WS handler needs to (a) handle both single-object and
//! array-of-objects frames, (b) skip the slow `serde_json::Value` +
//! `from_value(value.clone())` double-parse pattern that existed before.
//! `decode_frame` centralises both.
//!
//! When the `simd-json` feature is enabled, large payloads route through
//! simd-json's SIMD tokenizer (~15-20% faster on mid-to-large WS frames);
//! small payloads stay on serde_json, where the SIMD startup cost would
//! otherwise dominate the parse.

use serde::de::DeserializeOwned;

/// A parsed WebSocket frame — either a single object or an array-of-objects.
///
/// Many exchanges send batched updates as a top-level JSON array. Rather
/// than parse twice (once as `Value` to peek, once as the typed struct),
/// `decode_frame` dispatches based on the first non-whitespace byte.
pub enum WsFrame<T> {
    Single(T),
    Array(Vec<T>),
}

impl<T> WsFrame<T> {
    /// Call `f` on each contained `T`, consuming the frame.
    pub fn for_each<F: FnMut(T)>(self, mut f: F) {
        match self {
            Self::Single(item) => f(item),
            Self::Array(items) => items.into_iter().for_each(f),
        }
    }
}

/// Below this payload size, simd-json's startup cost exceeds the tokenizer
/// speedup — `serde_json::from_str` is measurably faster on tiny frames
/// (single-level price-change updates, subscription acks). Above it,
/// simd-json wins steadily.
///
/// Crossover calibrated on the `ws_hot_path` bench: at ~250 bytes (1 book
/// level) serde is ~40% faster; at ~1200 bytes (16 levels) simd-json is
/// ~10% faster; at ~4.5 KB (64 levels) simd-json is ~20% faster. 512 bytes
/// sits comfortably above the worst-case small-frame size.
#[cfg(feature = "simd-json")]
pub const SIMD_CROSSOVER_BYTES: usize = 512;

/// Decode `text` into a `WsFrame<T>` with a single pass of whatever JSON
/// parser is fastest for its size.
///
/// - Small frames: `serde_json::from_str` on the `&str` directly — no alloc.
/// - Large frames (≥ `SIMD_CROSSOVER_BYTES`, `simd-json` feature on): copy
///   to a `Vec<u8>` once, then `simd_json::serde::from_slice` with
///   SIMD-accelerated tokenisation.
///
/// Returns `None` on parse failure; callers typically log and drop such
/// frames. Dispatch rule between single and array: first non-whitespace
/// byte is `[` → array; else single object. Matches polymarket / kalshi
/// WS behaviour (both forms are observed in the wild).
pub fn decode_frame<T: DeserializeOwned>(text: &str) -> Option<WsFrame<T>> {
    #[cfg(feature = "simd-json")]
    if text.len() >= SIMD_CROSSOVER_BYTES {
        let mut bytes = text.as_bytes().to_vec();
        let head = bytes.iter().find(|&&b| !b.is_ascii_whitespace()).copied()?;
        return if head == b'[' {
            simd_json::serde::from_slice::<Vec<T>>(&mut bytes)
                .ok()
                .map(WsFrame::Array)
        } else {
            simd_json::serde::from_slice::<T>(&mut bytes)
                .ok()
                .map(WsFrame::Single)
        };
    }

    let trimmed = text.trim_start();
    if trimmed.starts_with('[') {
        serde_json::from_str::<Vec<T>>(text)
            .ok()
            .map(WsFrame::Array)
    } else {
        serde_json::from_str::<T>(text).ok().map(WsFrame::Single)
    }
}

/// Parse `text` into a `serde_json::Value` using the same size-based simd
/// switching as `decode_frame`. For exchanges (e.g. kalshi) that dispatch
/// on a field inside a loosely typed Value rather than deserialising into
/// a bespoke `RawWsMessage` struct.
pub fn decode_value(text: &str) -> Option<serde_json::Value> {
    #[cfg(feature = "simd-json")]
    if text.len() >= SIMD_CROSSOVER_BYTES {
        let mut bytes = text.as_bytes().to_vec();
        return simd_json::serde::from_slice::<serde_json::Value>(&mut bytes).ok();
    }
    serde_json::from_str::<serde_json::Value>(text).ok()
}

/// Reusable simd-json scratch space for high-throughput WS decoders.
///
/// Hold one per WS connection (WS frames are processed serially on a single
/// task, so no sync is needed). Each call to `parse_value` reuses the
/// internal `simd_json::Buffers` allocation, so steady-state is zero
/// allocation in the parser itself.
///
/// ```ignore
/// let mut scratch = TapeScratch::new();
/// while let Some(frame) = ws.next().await {
///     let mut bytes = frame.into_bytes();
///     let v = scratch.parse_value(&mut bytes)?;
///     // walk v to extract fields — BorrowedValue points into `bytes`
/// }
/// ```
#[cfg(feature = "simd-json")]
pub struct TapeScratch {
    buffers: simd_json::Buffers,
}

#[cfg(feature = "simd-json")]
impl TapeScratch {
    /// A scratch sized for typical WS frames (a few KB). Grows automatically.
    pub fn new() -> Self {
        Self::with_capacity(16 * 1024)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buffers: simd_json::Buffers::new(cap),
        }
    }

    /// Parse `bytes` in place, returning a `BorrowedValue` that references
    /// the input bytes (no string allocation per field). `bytes` is mutated
    /// — callers must own them.
    pub fn parse_value<'a>(
        &mut self,
        bytes: &'a mut [u8],
    ) -> Result<simd_json::BorrowedValue<'a>, simd_json::Error> {
        simd_json::to_borrowed_value_with_buffers(bytes, &mut self.buffers)
    }
}

#[cfg(feature = "simd-json")]
impl Default for TapeScratch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Msg {
        event: String,
        seq: u64,
    }

    #[test]
    fn single_object() {
        let text = r#"{"event":"book","seq":42}"#;
        match decode_frame::<Msg>(text).unwrap() {
            WsFrame::Single(m) => assert_eq!(
                m,
                Msg {
                    event: "book".into(),
                    seq: 42
                }
            ),
            WsFrame::Array(_) => panic!("expected single"),
        }
    }

    #[test]
    fn array_of_objects() {
        let text = r#"[{"event":"book","seq":1},{"event":"trade","seq":2}]"#;
        match decode_frame::<Msg>(text).unwrap() {
            WsFrame::Array(items) => assert_eq!(items.len(), 2),
            WsFrame::Single(_) => panic!("expected array"),
        }
    }

    #[test]
    fn whitespace_prefix() {
        let text = "   \n  [{\"event\":\"book\",\"seq\":1}]";
        assert!(matches!(decode_frame::<Msg>(text), Some(WsFrame::Array(_))));
    }

    #[test]
    fn malformed_returns_none() {
        assert!(decode_frame::<Msg>("{not json").is_none());
        assert!(decode_frame::<Msg>("").is_none());
    }

    #[test]
    fn large_frame_uses_simd() {
        // Build a frame safely above the crossover so the SIMD branch runs.
        let mut inner = String::new();
        for i in 0..100 {
            if i > 0 {
                inner.push(',');
            }
            inner.push_str(&format!(r#"{{"event":"tick","seq":{i}}}"#));
        }
        let text = format!("[{inner}]");
        match decode_frame::<Msg>(&text).unwrap() {
            WsFrame::Array(items) => assert_eq!(items.len(), 100),
            WsFrame::Single(_) => panic!("expected array"),
        }
    }

    #[test]
    fn decode_value_handles_both_sizes() {
        // small path
        let small = r#"{"msgType":"ping","seq":1}"#;
        let v = decode_value(small).unwrap();
        assert_eq!(v.get("msgType").and_then(|v| v.as_str()), Some("ping"));

        // large path (SIMD branch when feature is enabled)
        let mut fields = String::new();
        for i in 0..200 {
            if i > 0 {
                fields.push(',');
            }
            fields.push_str(&format!(r#""k{i}":"value_{i}""#));
        }
        let large = format!("{{{fields}}}");
        let v = decode_value(&large).unwrap();
        assert_eq!(v.get("k0").and_then(|v| v.as_str()), Some("value_0"));
    }
}
