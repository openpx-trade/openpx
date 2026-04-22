//! Fast hash-map aliases for hot-path collections.
//!
//! stdlib `HashMap` uses SipHash-1-3. For the short string keys that dominate
//! WS state (asset IDs, market IDs, outcome names) ahash is ~3-5x faster on
//! insert/lookup and DoS-resistance is not a concern for internal state.
//!
//! Usage is opt-in — don't force-migrate every `HashMap`; use this only where
//! profiling shows hashing in the top of the flamegraph.

use std::collections::{HashMap, HashSet};

pub type FastHashMap<K, V> = HashMap<K, V, ahash::RandomState>;
pub type FastHashSet<T> = HashSet<T, ahash::RandomState>;
