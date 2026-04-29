//! Wiring contract: every directory under `engine/exchanges/` must be plumbed
//! through every layer of the unified surface. If you add or rename an
//! exchange, this test fails until it's threaded through `Cargo.toml`, the
//! `ExchangeInner` enum + dispatch, the config parser, and the manifests
//! module.
//!
//! Pmxt-style "implicit API" repos discover this drift at runtime via missing
//! method errors. We catch it in `cargo test` before the PR can merge, with a
//! single failure message naming exactly which layer is missing the new
//! exchange. Python and TypeScript SDKs are dispatch-by-id-string and so are
//! agnostic to the exchange list — only the Rust core needs threading.
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p
}

fn read(rel: &str) -> String {
    let path = workspace_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

fn list_exchanges() -> Vec<String> {
    let dir = workspace_root().join("engine/exchanges");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("missing engine/exchanges/: {e}"))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && e.path().join("Cargo.toml").is_file()
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| !n.starts_with('.'))
        .collect();
    names.sort();
    names
}

fn title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[test]
fn exchange_dir_names_are_lowercase_alphanumeric() {
    for ex in list_exchanges() {
        assert!(
            ex.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "exchange dir name '{ex}' must be lowercase alphanumeric — the wiring \
             test derives Rust identifiers from the dir name and does not handle \
             hyphens or mixed case. Pick a flat name."
        );
    }
}

#[test]
fn every_exchange_is_wired_into_unified_surface() {
    let exchanges = list_exchanges();
    assert!(
        !exchanges.is_empty(),
        "no exchanges found under engine/exchanges/ — refusing to pass a vacuous wiring test"
    );

    let cargo_toml = read("Cargo.toml");
    let sdk_lib = read("engine/sdk/src/lib.rs");
    let sdk_config = read("engine/sdk/src/config.rs");
    let manifests_mod = read("engine/core/src/exchange/manifests/mod.rs");

    let mut failures: Vec<String> = Vec::new();
    let mut check = |present: bool, layer: &str, ex: &str, hint: &str| {
        if !present {
            failures.push(format!("[{ex}] {layer} — expected: {hint}"));
        }
    };

    for ex in &exchanges {
        let title = title_case(ex);
        let upper = ex.to_uppercase();
        let crate_name = format!("px-exchange-{ex}");
        let crate_path = format!("engine/exchanges/{ex}");

        check(
            cargo_toml.contains(&format!("\"{crate_path}\"")),
            "Cargo.toml [workspace.members]",
            ex,
            &format!("\"{crate_path}\""),
        );

        check(
            cargo_toml.contains(&crate_name),
            "Cargo.toml [workspace.dependencies]",
            ex,
            &format!("{crate_name} = {{ version = \"...\", path = \"{crate_path}\" }}"),
        );

        check(
            sdk_lib.contains(&format!("px_exchange_{ex}")),
            "engine/sdk/src/lib.rs (re-export)",
            ex,
            &format!("pub use px_exchange_{ex}::..."),
        );

        check(
            sdk_lib.contains(&format!("ExchangeInner::{title}")),
            "engine/sdk/src/lib.rs (dispatch)",
            ex,
            &format!("ExchangeInner::{title} arms in dispatch! / dispatch_sync!"),
        );

        check(
            sdk_lib.contains(&format!("\"{ex}\" =>")),
            "engine/sdk/src/lib.rs (ExchangeInner::new match)",
            ex,
            &format!("\"{ex}\" => ... in ExchangeInner::new"),
        );

        check(
            sdk_config.contains(&format!("parse_{ex}")),
            "engine/sdk/src/config.rs",
            ex,
            &format!("pub fn parse_{ex}(...)"),
        );

        let manifest_file =
            workspace_root().join(format!("engine/core/src/exchange/manifests/{ex}.rs"));
        check(
            manifest_file.is_file(),
            "engine/core/src/exchange/manifests/",
            ex,
            &format!("{ex}.rs file with pub const {upper}_MANIFEST"),
        );

        check(
            manifests_mod.contains(&format!("mod {ex}")),
            "engine/core/src/exchange/manifests/mod.rs",
            ex,
            &format!("mod {ex};"),
        );

        check(
            manifests_mod.contains(&format!("{upper}_MANIFEST")),
            "engine/core/src/exchange/manifests/mod.rs",
            ex,
            &format!("pub use {ex}::{upper}_MANIFEST"),
        );
    }

    assert!(
        failures.is_empty(),
        "exchange wiring drift detected — {} layer(s) missing for {} exchange(s):\n  {}\n\n\
         Fix: thread the named exchange through each named layer above. The dispatch is by \
         id-string at runtime, so the Python and TypeScript SDKs are agnostic — only the \
         Rust core needs updating.",
        failures.len(),
        exchanges.len(),
        failures.join("\n  ")
    );
}
