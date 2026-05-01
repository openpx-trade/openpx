#!/usr/bin/env python3
"""Generate docs/llms.txt from canonical sources.

Inputs (all sources of truth):
  - docs/docs.json                              -> project name + description + nav tabs
  - Cargo.toml                                  -> repository URL
  - engine/exchanges/*/                         -> exchange list (canonical)
  - engine/core/src/exchange/manifests/<id>.rs  -> exchange display name + base URL

Output:
  - docs/llms.txt (overwritten)

Invoked by `just llms-txt`. CI re-runs this and refuses to merge if the
committed file drifts. The artifact is checked in so external consumers can
fetch it from the GitHub raw URL even when the docs site is down.

Spec: https://llmstxt.org/
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS_JSON = ROOT / "docs" / "docs.json"
CARGO_TOML = ROOT / "Cargo.toml"
EXCHANGES_DIR = ROOT / "engine" / "exchanges"
MANIFESTS_DIR = ROOT / "engine" / "core" / "src" / "exchange" / "manifests"
OUTPUT = ROOT / "docs" / "llms.txt"


def read_repo_url() -> str:
    text = CARGO_TOML.read_text()
    m = re.search(r'^repository\s*=\s*"([^"]+)"', text, re.MULTILINE)
    if not m:
        sys.exit("Cargo.toml is missing [workspace.package].repository")
    return m.group(1).rstrip("/")


def list_exchanges() -> list[tuple[str, str, str]]:
    """Return [(id, display_name, base_url)] for each exchange dir."""
    out: list[tuple[str, str, str]] = []
    for entry in sorted(EXCHANGES_DIR.iterdir()):
        if not entry.is_dir() or not (entry / "Cargo.toml").is_file():
            continue
        ex_id = entry.name
        display, base_url = ex_id.title(), ""
        manifest = MANIFESTS_DIR / f"{ex_id}.rs"
        if manifest.is_file():
            text = manifest.read_text()
            m = re.search(r'\bname\s*:\s*"([^"]+)"', text)
            if m:
                display = m.group(1)
            m = re.search(r'\bbase_url\s*:\s*"([^"]+)"', text)
            if m:
                base_url = m.group(1)
        out.append((ex_id, display, base_url))
    return out


def collect_doc_tabs(nav: dict) -> set[str]:
    return {tab["tab"] for tab in nav.get("tabs", []) if "tab" in tab}


def render(
    repo_url: str,
    exchanges: list[tuple[str, str, str]],
    project_name: str,
    project_desc: str,
    tabs: set[str],
) -> str:
    blob = f"{repo_url}/blob/main"
    tree = f"{repo_url}/tree/main"

    lines: list[str] = []
    lines.append(f"# {project_name}")
    lines.append("")
    lines.append(f"> {project_desc}")
    lines.append("")
    lines.append(
        f"{project_name} hides per-exchange quirks behind one Rust `Exchange` "
        "trait. Python (PyO3) and TypeScript (NAPI-RS) SDKs are auto-generated "
        "bindings around the same engine — all domain logic, parsing, and "
        "WebSocket state lives in Rust. Bring your own credentials; trade "
        "through one unified surface."
    )
    lines.append("")

    lines.append("## Documentation")
    lines.append("")
    if "API" in tabs:
        lines.append(
            f"- [API reference]({tree}/docs/api): unified market, order, position, "
            "orderbook, trade, and account endpoints across every supported exchange"
        )
    if "WebSockets" in tabs:
        lines.append(
            f"- [WebSocket streams]({tree}/docs/websockets): real-time orderbook "
            "snapshots, deltas, trades, fills, and session events"
        )
    if "Schemas" in tabs:
        lines.append(
            f"- [Schema mappings]({tree}/docs/schemas/mappings): per-field "
            "crosswalk between unified OpenPX types and Kalshi/Polymarket "
            "upstream sources, auto-generated from `schema/mappings/*.yaml`"
        )
    if "Changelog" in tabs:
        lines.append(
            f"- [Changelog]({blob}/docs/changelog.mdx): release history with "
            "breaking-change notes"
        )
    lines.append("")

    lines.append("## Packages")
    lines.append("")
    lines.append(
        "- [openpx (crates.io)](https://crates.io/crates/openpx): Rust engine — "
        "direct enum dispatch, zero vtable, in-process latency"
    )
    lines.append(
        "- [openpx (PyPI)](https://pypi.org/project/openpx/): Python bindings via "
        "PyO3 with native wheels for linux/macos/windows on x86_64 and arm64"
    )
    lines.append(
        "- [@openpx/sdk (npm)](https://www.npmjs.com/package/@openpx/sdk): "
        "Node.js bindings via NAPI-RS, native addons for the same five platforms"
    )
    lines.append("")

    lines.append("## Exchanges")
    lines.append("")
    for ex_id, display, base_url in exchanges:
        suffix = f" (`{base_url}`)" if base_url else ""
        lines.append(
            f"- [{display}]({tree}/engine/exchanges/{ex_id}): instantiate via "
            f'`Exchange("{ex_id}", config)`{suffix}'
        )
    lines.append("")

    lines.append("## Source")
    lines.append("")
    lines.append(f"- [Repository]({repo_url}): source, issues, pull requests")
    lines.append(
        f"- [JSON Schema]({blob}/schema/openpx.schema.json): canonical types — "
        "both SDKs are generated from this single file"
    )
    lines.append(
        f"- [Exchange trait]({blob}/engine/core/src/exchange/traits.rs): the "
        "contract every exchange implementation conforms to"
    )
    lines.append(
        f"- [Exchange manifests]({tree}/engine/core/src/exchange/manifests): "
        "per-exchange constants — base URLs, rate limits, pagination styles"
    )
    lines.append("")

    lines.append("## Optional")
    lines.append("")
    lines.append(
        f"- [Sports data]({tree}/engine/sports): unified sports schedule + live "
        "state stream for sports prediction markets"
    )
    lines.append(
        f"- [Crypto prices]({tree}/engine/crypto): low-latency crypto price feed "
        "for crypto-settled markets"
    )
    lines.append("")

    return "\n".join(lines)


def main() -> int:
    if not DOCS_JSON.is_file():
        sys.exit(f"missing {DOCS_JSON}")
    docs_cfg = json.loads(DOCS_JSON.read_text())
    project_name = docs_cfg.get("name", "OpenPX")
    project_desc = docs_cfg.get(
        "description", "Unified SDK for prediction markets."
    )
    repo_url = read_repo_url()
    exchanges = list_exchanges()
    if not exchanges:
        sys.exit("no exchanges found under engine/exchanges/")
    tabs = collect_doc_tabs(docs_cfg.get("navigation", {}))

    text = render(repo_url, exchanges, project_name, project_desc, tabs) + "\n"
    OUTPUT.write_text(text)
    rel = OUTPUT.relative_to(ROOT)
    print(f"wrote {rel} ({len(text)} bytes, {len(exchanges)} exchanges)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
