//! Shared helpers for the comparative bench crate.
//!
//! Fixture loading + tight-loop scaffolding live here so each bench file
//! stays focused on the measurement itself.

use std::path::{Path, PathBuf};

/// Returns the path to `benches/comparative/fixtures/` relative to the
/// crate manifest.
pub fn fixtures_dir() -> PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest).join("..").join("fixtures")
}

/// Reads a fixture file as raw bytes. Panics if the fixture is missing —
/// run `python3 tools/capture_bench_fixtures.py` to populate.
pub fn load_fixture(name: &str) -> Vec<u8> {
    let path = fixtures_dir().join(name);
    std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "missing fixture {}: {} — run `python3 tools/capture_bench_fixtures.py`",
            path.display(),
            e
        )
    })
}
