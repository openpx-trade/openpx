//! Asserts that every JSON key read by `engine/exchanges/<id>/src/exchange.rs`
//! is either declared in `manifests/<id>.rs::field_mappings.source_paths`,
//! present in `manifests/<id>.rs::status_map`, or explicitly listed in the
//! per-exchange allowlist file.
//!
//! The manifest is the contract for the unified schema mapping. This test
//! prevents the manifest from silently drifting away from the actual code.
//!
//! When this test fails, EITHER:
//! - the source code reads a new JSON field that should be mapped through the
//!   manifest (add a `FieldMapping` entry), OR
//! - the source code reads a JSON field that isn't part of the unified Market
//!   schema (e.g. order/fill/position parsing). Add it to the per-exchange
//!   allowlist with a one-line justification.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::visit::Visit;

use px_core::exchange::manifest::ExchangeManifest;
use px_core::exchange::manifests::{KALSHI_MANIFEST, POLYMARKET_MANIFEST};

#[test]
fn kalshi_manifest_covers_exchange_reads() {
    check_exchange("kalshi", &KALSHI_MANIFEST);
}

#[test]
fn polymarket_manifest_covers_exchange_reads() {
    check_exchange("polymarket", &POLYMARKET_MANIFEST);
}

fn check_exchange(id: &str, manifest: &ExchangeManifest) {
    let workspace = workspace_root();
    let source_path = workspace.join(format!("engine/exchanges/{id}/src/exchange.rs"));
    let allowlist_path = workspace.join(format!("maintenance/manifest-allowlists/{id}.txt"));

    let read_keys = collect_json_keys(&source_path);
    let manifest_keys = manifest_declared_keys(manifest);
    let allowlist = parse_allowlist(&allowlist_path);

    let mut undeclared: Vec<&str> = read_keys
        .iter()
        .filter(|k| !manifest_keys.contains(k.as_str()) && !allowlist.contains(k.as_str()))
        .map(String::as_str)
        .collect();
    undeclared.sort();

    assert!(
        undeclared.is_empty(),
        "{} unmanaged JSON key read(s) in {}:\n\n{}\n\n\
         For each key, either:\n  \
         (a) add a FieldMapping entry in engine/core/src/exchange/manifests/{}.rs, OR\n  \
         (b) add the key to {} with a one-line justification.\n\n\
         The manifest is the contract for the unified schema mapping. This test \
         prevents silent drift between the manifest spec and what exchange.rs actually reads.",
        undeclared.len(),
        source_path.display(),
        undeclared.join("\n  - "),
        id,
        allowlist_path.display(),
    );
}

/// JSON keys read by the source file, collected from `.get("...")` method
/// calls, `[...]` index expressions, and `#[serde(rename = "...")]` attributes.
fn collect_json_keys(source_path: &Path) -> HashSet<String> {
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|e| panic!("read {}: {}", source_path.display(), e));
    let file = syn::parse_file(&source)
        .unwrap_or_else(|e| panic!("parse {}: {}", source_path.display(), e));

    let mut visitor = KeyCollector::default();
    visitor.visit_file(&file);
    visitor.keys
}

#[derive(Default)]
struct KeyCollector {
    keys: HashSet<String>,
}

impl<'ast> Visit<'ast> for KeyCollector {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "get" {
            if let Some(syn::Expr::Lit(lit)) = node.args.first() {
                if let syn::Lit::Str(s) = &lit.lit {
                    self.keys.insert(s.value());
                }
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_index(&mut self, node: &'ast syn::ExprIndex) {
        if let syn::Expr::Lit(lit) = node.index.as_ref() {
            if let syn::Lit::Str(s) = &lit.lit {
                self.keys.insert(s.value());
            }
        }
        syn::visit::visit_expr_index(self, node);
    }

    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if attr.path().is_ident("serde") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    if let Ok(value) = meta.value() {
                        if let Ok(lit) = value.parse::<syn::LitStr>() {
                            self.keys.insert(lit.value());
                        }
                    }
                }
                Ok(())
            });
        }
        syn::visit::visit_attribute(self, attr);
    }
}

/// Keys the manifest knows about: every entry in `field_mappings.source_paths`
/// (and the first segment of any dotted path), plus every `status_map` key.
fn manifest_declared_keys(manifest: &ExchangeManifest) -> HashSet<&'static str> {
    let mut out: HashSet<&'static str> = HashSet::new();
    for fm in manifest.field_mappings {
        for path in fm.source_paths {
            out.insert(*path);
            if let Some(first) = path.split('.').next() {
                out.insert(first);
            }
        }
    }
    for (status, _) in manifest.status_map {
        out.insert(*status);
    }
    out
}

/// Parses the per-exchange allowlist. One key per line; `#` starts a comment;
/// trailing inline `# justification` is allowed and stripped.
fn parse_allowlist(path: &Path) -> HashSet<String> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return HashSet::new(),
    };
    text.lines()
        .filter_map(|line| {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect()
}

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root above engine/core")
        .to_path_buf()
}
