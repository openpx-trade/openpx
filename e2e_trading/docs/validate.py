#!/usr/bin/env python3
"""Validate the documentation surface for the authenticated trading endpoints.

Run from repo root:
    sdks/python/.venv/bin/python e2e_trading/docs/validate.py

Checks:
  1. Every authenticated trading endpoint we exercise is referenced by an
     MDX page under docs/api/
  2. Every endpoint has a path entry in docs/openpx.openapi.yaml
  3. Every order/account-related schema mapping exists under schema/mappings/
  4. The rendered MDX page mentions the canonical method name in its
     `openapi:` frontmatter (so the embedded API reference is correct)

Output: writes ../results/docs.json + summary to stdout, exits non-zero on
any miss.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESULTS_DIR = REPO_ROOT / "e2e_trading" / "results"
RESULTS_DIR.mkdir(parents=True, exist_ok=True)
RESULTS_PATH = RESULTS_DIR / "docs.json"

# Canonical method names → expected MDX page (relative to docs/api/) and
# OpenAPI path. The list mirrors what the live e2e suites exercise.
ENDPOINTS = [
    ("create_order",         "orders/create-order.mdx",          "/v1/orders/create_order"),
    ("cancel_order",         "orders/cancel-order.mdx",          "/v1/orders/cancel_order"),
    ("cancel_all_orders",    "orders/cancel-all-orders.mdx",     "/v1/orders/cancel_all_orders"),
    ("create_orders_batch",  "orders/create-orders-batch.mdx",   "/v1/orders/create_orders_batch"),
    ("fetch_order",          "orders/fetch-order.mdx",           "/v1/orders/fetch_order"),
    ("fetch_open_orders",    "orders/fetch-open-orders.mdx",     "/v1/orders/fetch_open_orders"),
    ("fetch_positions",      "positions/fetch-positions.mdx",    "/v1/positions/fetch_positions"),
    ("fetch_balance",        "balance/fetch-balance.mdx",        "/v1/balance/fetch_balance"),
    ("refresh_balance",      "balance/refresh-balance.mdx",      "/v1/balance/refresh_balance"),
    ("fetch_fills",          "fills/fetch-fills.mdx",            "/v1/fills/fetch_fills"),
    ("fetch_trades",         "trades/fetch-trades.mdx",          "/v1/trades/fetch_trades"),
    ("fetch_server_time",    "server/fetch-server-time.mdx",     "/v1/server/fetch_server_time"),
]

# Mapping files we expect under schema/mappings/ for the order/account surface.
EXPECTED_MAPPINGS = ["order.yaml", "position.yaml", "fill.yaml"]


def check_endpoint(name: str, mdx_rel: str, openapi_path: str) -> dict:
    out = {"endpoint": name, "issues": []}

    mdx_path = REPO_ROOT / "docs" / "api" / mdx_rel
    if not mdx_path.exists():
        out["issues"].append(f"MDX page missing: docs/api/{mdx_rel}")
        return out

    text = mdx_path.read_text()
    # Extract the title and the openapi: directive from the frontmatter.
    m = re.search(r"openapi:\s*([^\n]+)", text)
    if not m:
        out["issues"].append("MDX missing `openapi:` frontmatter directive")
    else:
        directive = m.group(1).strip()
        if openapi_path not in directive:
            out["issues"].append(
                f"MDX `openapi:` directive '{directive}' does not reference path {openapi_path}"
            )

    # The page must mention the canonical method name somewhere.
    if name not in text and name.replace("_", "-") not in text:
        out["issues"].append(f"MDX does not mention canonical method '{name}'")

    return out


def check_openapi_paths(openapi_paths: list[str]) -> dict:
    spec_path = REPO_ROOT / "docs" / "openpx.openapi.yaml"
    if not spec_path.exists():
        return {"endpoint": "openapi.yaml", "issues": ["openpx.openapi.yaml missing"]}
    text = spec_path.read_text()
    issues = []
    for p in openapi_paths:
        # Look for a top-level path entry (indented under `paths:`).
        if not re.search(rf"^\s+{re.escape(p)}:\s*$", text, flags=re.MULTILINE):
            issues.append(f"openapi.yaml missing path entry: {p}")
    return {"endpoint": "openapi.yaml", "issues": issues}


def check_mappings() -> dict:
    base = REPO_ROOT / "schema" / "mappings"
    issues = []
    if not base.exists():
        issues.append("schema/mappings/ directory missing")
        return {"endpoint": "mappings", "issues": issues}
    for f in EXPECTED_MAPPINGS:
        if not (base / f).exists():
            issues.append(f"schema/mappings/{f} missing")
    return {"endpoint": "mappings", "issues": issues}


def main() -> int:
    print("=== docs validation ===")
    results = []

    for name, mdx_rel, openapi_path in ENDPOINTS:
        r = check_endpoint(name, mdx_rel, openapi_path)
        results.append(r)
        symbol = "✓" if not r["issues"] else "✗"
        print(f"  {symbol} {name}")
        for issue in r["issues"]:
            print(f"      → {issue}")

    spec_check = check_openapi_paths([p for _, _, p in ENDPOINTS])
    results.append(spec_check)
    print(f"  {'✓' if not spec_check['issues'] else '✗'} openapi.yaml path completeness")
    for issue in spec_check["issues"]:
        print(f"      → {issue}")

    mapping_check = check_mappings()
    results.append(mapping_check)
    print(f"  {'✓' if not mapping_check['issues'] else '✗'} schema/mappings completeness")
    for issue in mapping_check["issues"]:
        print(f"      → {issue}")

    fail = sum(1 for r in results if r["issues"])
    summary = {"checked": len(results), "fail": fail, "results": results}
    RESULTS_PATH.write_text(json.dumps(summary, indent=2))
    print(f"\nPASS {len(results) - fail}\nFAIL {fail}")
    return 0 if fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
