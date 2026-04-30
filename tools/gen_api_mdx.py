#!/usr/bin/env python3
"""Generate `docs/api/<tag>/<method>.mdx` for every Exchange-trait method.

The MDX is a thin shell — only frontmatter + an optional intro paragraph.
Mintlify auto-renders Query Parameters and Response from the OpenAPI spec
referenced by the `openapi:` frontmatter. There are no hand-written
`<ParamField>` / `<ResponseField>` sections; the OpenAPI spec is the single
source of truth.

Sources:
  - engine/core/src/exchange/traits.rs   -> method names + first-line doc
                                            comments (used for description +
                                            optional intro paragraph).

Per-method output (one MDX file each):
  ---
  title: <method>
  sidebarTitle: <Humanized method>
  openapi: <VERB> <path>
  description: <first sentence of doc comment>
  ---

  <optional intro paragraph from doc comment continuation>
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
TRAITS_RS = ROOT / "engine" / "core" / "src" / "exchange" / "traits.rs"
OUTPUT_DIR = ROOT / "docs" / "api"

# Per-method tag + HTTP verb. Verb is cosmetic — Mintlify renders it as a
# badge — but we pick semantically: reads are GET, writes are POST.
METHOD_META: dict[str, tuple[str, str]] = {
    "fetch_markets": ("Markets", "GET"),
    "fetch_market": ("Markets", "GET"),
    "fetch_events": ("Events", "GET"),
    "fetch_event": ("Events", "GET"),
    "fetch_series": ("Series", "GET"),
    "fetch_series_one": ("Series", "GET"),
    "fetch_market_tags": ("Tags", "GET"),
    "fetch_orderbook": ("Orderbook", "GET"),
    "fetch_orderbooks_batch": ("Orderbook", "GET"),
    "fetch_orderbook_history": ("Orderbook History", "GET"),
    "fetch_midpoint": ("Pricing", "GET"),
    "fetch_midpoints_batch": ("Pricing", "GET"),
    "fetch_spread": ("Pricing", "GET"),
    "fetch_last_trade_price": ("Pricing", "GET"),
    "fetch_open_interest": ("Pricing", "GET"),
    "fetch_trades": ("Trades", "GET"),
    "fetch_user_trades": ("Trades", "GET"),
    "fetch_price_history": ("Price History", "GET"),
    "create_order": ("Orders", "POST"),
    "create_orders_batch": ("Orders", "POST"),
    "cancel_order": ("Orders", "POST"),
    "cancel_all_orders": ("Orders", "POST"),
    "fetch_order": ("Orders", "GET"),
    "fetch_open_orders": ("Orders", "GET"),
    "fetch_positions": ("Positions", "GET"),
    "fetch_balance": ("Balance", "GET"),
    "fetch_balance_raw": ("Balance", "GET"),
    "refresh_balance": ("Balance", "POST"),
    "fetch_fills": ("Fills", "GET"),
    "fetch_user_activity": ("Fills", "GET"),
    "fetch_server_time": ("Server", "GET"),
}


METHOD_RE = re.compile(
    r"((?:^[ \t]*///[^\n]*\n)*)"
    r"^[ \t]*(async\s+)?fn\s+"
    r"([a-zA-Z_][a-zA-Z0-9_]*)"
    r"\s*\(",
    re.MULTILINE | re.DOTALL,
)


def extract_doc(block: str) -> str:
    lines: list[str] = []
    for line in block.splitlines():
        line = line.strip()
        if line.startswith("///"):
            lines.append(line[3:].strip())
    return "\n".join(lines).strip()


def parse_traits() -> list[dict[str, Any]]:
    text = TRAITS_RS.read_text()
    methods: list[dict[str, Any]] = []
    for m in METHOD_RE.finditer(text):
        name = m.group(3)
        if name not in METHOD_META:
            continue
        doc = extract_doc(m.group(1) or "")
        methods.append({"name": name, "doc": doc})
    return methods


def humanize(name: str) -> str:
    return name.replace("_", " ").capitalize()


def slug(name: str) -> str:
    return name.replace("_", "-")


def tag_dir(tag: str) -> str:
    return tag.lower().replace(" ", "-")


def render_method_mdx(method: dict[str, Any]) -> str:
    """Emit MDX with `openapi:` frontmatter + a prose body.

    Mintlify auto-renders Query Parameters / Body / Response panels from the
    referenced operation, AND auto-injects the OpenAPI spec under a `##
    OpenAPI` section when the page is copied — but only when the MDX has
    actual body content. Frontmatter-only MDX makes Mintlify's "Copy page"
    fall back to raw HTML; emitting the full doc comment as prose gives copy
    a real markdown surface."""
    name = method["name"]
    tag, verb = METHOD_META[name]
    doc = method["doc"]

    api_path = f"/v1/{tag_dir(tag)}/{name}"

    out = "---\n"
    out += f"title: {json.dumps(name)}\n"
    out += f"sidebarTitle: {json.dumps(humanize(name))}\n"
    out += f"openapi: /openpx.openapi.yaml {verb} {api_path}\n"
    out += "---\n\n"

    if doc:
        out += doc + "\n"
    else:
        out += humanize(name) + ".\n"

    return out


def main() -> int:
    methods = parse_traits()
    if not methods:
        sys.exit("no methods parsed from traits.rs")

    written = 0
    for method in methods:
        name = method["name"]
        tag, _ = METHOD_META[name]
        out_path = OUTPUT_DIR / tag_dir(tag) / f"{slug(name)}.mdx"
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(render_method_mdx(method))
        written += 1

    print(f"wrote {written} MDX files under {OUTPUT_DIR.relative_to(ROOT)}/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
