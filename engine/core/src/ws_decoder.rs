//! Shared WebSocket frame decoding helpers.
//!
//! Every exchange's WS handler needs to (a) handle both single-object and
//! array-of-objects frames, (b) skip the slow `serde_json::Value` +
//! `from_value(value.clone())` double-parse pattern that existed before.
//! `decode_frame` centralises both.
//!
//! For zero-allocation parsing (simd-json with reusable Tape buffers), see
//! `TapeScratch` behind the `simd-json` feature.

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

/// Decode `text` into a `WsFrame<T>` with a single pass of serde_json.
///
/// Returns `None` on parse failure — callers typically log and drop such
/// frames since a half-parsed WS message is not recoverable.
///
/// Dispatch rule: if the first non-whitespace byte is `[`, parse as `Vec<T>`;
/// otherwise parse as `T`. This matches polymarket / kalshi / opinion WS
/// behavior (both forms are observed in the wild).
pub fn decode_frame<T: DeserializeOwned>(text: &str) -> Option<WsFrame<T>> {
    let trimmed = text.trim_start();
    if trimmed.starts_with('[') {
        serde_json::from_str::<Vec<T>>(text)
            .ok()
            .map(WsFrame::Array)
    } else {
        serde_json::from_str::<T>(text).ok().map(WsFrame::Single)
    }
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
}
