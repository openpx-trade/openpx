//! Asserts that every JSON key read by `engine/exchanges/<id>/src/sports.rs`
//! (or, for ESPN, `engine/sports/src/providers/espn/...`) is either declared
//! in the corresponding `SportsManifest` or listed in the per-provider
//! allowlist file.
//!
//! Mirrors `maintenance/tests/manifest_coverage.rs` for the trading surface.
//! Sports manifests are smaller — only `Game` and `GameState` mappings — but
//! the same drift-prevention contract applies.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::visit::Visit;

use px_sports::manifest::SportsManifest;
use px_sports::manifests::POLYMARKET_SPORTS_MANIFEST;

#[test]
fn polymarket_sports_manifest_covers_sports_reads() {
    check_provider(
        "polymarket",
        "engine/exchanges/polymarket/src/sports.rs",
        &POLYMARKET_SPORTS_MANIFEST,
    );
}

fn check_provider(id: &str, source_rel_path: &str, manifest: &SportsManifest) {
    let workspace = workspace_root();
    let source_path = workspace.join(source_rel_path);
    let allowlist_path = workspace.join(format!("maintenance/manifest-allowlists/sports-{id}.txt"));

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
        "{} unmanaged JSON key read(s) in {}:\n\n  - {}\n\n\
         For each key, either:\n  \
         (a) add a FieldMapping entry in engine/sports/src/manifests/{}.rs, OR\n  \
         (b) add the key to {} with a one-line justification.\n\n\
         The sports manifest is the contract for the unified Game/GameState \
         mapping. This test prevents silent drift between the manifest spec \
         and what the provider actually reads.",
        undeclared.len(),
        source_path.display(),
        undeclared.join("\n  - "),
        id,
        allowlist_path.display(),
    );
}

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

fn manifest_declared_keys(manifest: &SportsManifest) -> HashSet<&'static str> {
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
        .expect("workspace root above engine/sports")
        .to_path_buf()
}
