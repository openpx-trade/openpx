//! Asserts hardcoded contract addresses in src/{swap,approvals}.rs match the
//! vendored snapshot at maintenance/data/polymarket-contracts.snapshot.json.
//!
//! When this test fails, EITHER:
//! - the source was updated and the snapshot needs a paired update, OR
//! - the source drifted incorrectly and should be reverted.
//!
//! NEVER bypass this test. A wrong contract address can move user funds.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct Snapshot {
    constants: HashMap<String, ConstEntry>,
}

#[derive(Deserialize)]
struct ConstEntry {
    address: String,
    file: String,
}

#[test]
fn contracts_match_snapshot() {
    let workspace_root = workspace_root();
    let snapshot_path =
        workspace_root.join("maintenance/data/polymarket-contracts.snapshot.json");
    let snapshot_text = fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|e| panic!("read {}: {}", snapshot_path.display(), e));
    let snapshot: Snapshot =
        serde_json::from_str(&snapshot_text).expect("parse contracts.snapshot.json");

    let mut file_cache: HashMap<String, String> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();

    for (name, entry) in &snapshot.constants {
        let path = workspace_root.join(&entry.file);
        let source = file_cache.entry(entry.file.clone()).or_insert_with(|| {
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
        });

        let needle = format!("pub const {}", name);
        let line = match source.lines().find(|l| l.contains(&needle)) {
            Some(l) => l,
            None => {
                failures.push(format!(
                    "snapshot constant `{}` not found in {} — was it removed or renamed?",
                    name, entry.file
                ));
                continue;
            }
        };

        if !line.contains(&entry.address) {
            failures.push(format!(
                "{}: snapshot says `{}` but source line is:\n    {}",
                name,
                entry.address,
                line.trim()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Polymarket contract address drift detected — {} mismatch(es):\n\n{}\n\n\
         If the source is correct, update maintenance/data/polymarket-contracts.snapshot.json \
         (and verify each new address on https://polygonscan.com/).\n\
         If the snapshot is correct, revert the source change.\n\
         NEVER bypass this test — a wrong address can lose user funds.",
        failures.len(),
        failures.join("\n\n")
    );
}

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("workspace root above engine/exchanges/polymarket")
        .to_path_buf()
}
