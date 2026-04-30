#!/usr/bin/env python3
"""Fetch every upstream OpenAPI / AsyncAPI spec into schema/upstream/.

Single source of truth for which specs OpenPX tracks. Run by the daily GitHub
Actions cron (.github/workflows/upstream-specs.yml) and on demand via
`just fetch-upstream-specs`. Idempotent — files are only rewritten when bytes
differ, so the workflow's create-pull-request step opens a PR only on real
upstream changes.

URL list mirrors `.claude/references.md`. Update there too if you edit this.
"""
from __future__ import annotations

import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEST = ROOT / "schema" / "upstream"

SPECS: list[tuple[str, str]] = [
    # Kalshi
    ("https://docs.kalshi.com/openapi.yaml", "kalshi.openapi.yaml"),
    ("https://docs.kalshi.com/asyncapi.yaml", "kalshi.asyncapi.yaml"),
    # Polymarket REST
    ("https://docs.polymarket.com/api-spec/gamma-openapi.yaml", "polymarket-gamma.openapi.yaml"),
    ("https://docs.polymarket.com/api-spec/clob-openapi.yaml", "polymarket-clob.openapi.yaml"),
    ("https://docs.polymarket.com/api-spec/data-openapi.yaml", "polymarket-data.openapi.yaml"),
    ("https://docs.polymarket.com/api-spec/bridge-openapi.yaml", "polymarket-bridge.openapi.yaml"),
    ("https://docs.polymarket.com/api-spec/relayer-openapi.yaml", "polymarket-relayer.openapi.yaml"),
    # Polymarket WS
    ("https://docs.polymarket.com/asyncapi.json", "polymarket.asyncapi.json"),
    ("https://docs.polymarket.com/asyncapi-user.json", "polymarket-user.asyncapi.json"),
]

USER_AGENT = "openpx-upstream-spec-fetcher/1.0 (+https://github.com/openpx-trade/openpx)"
TIMEOUT_SECS = 30


def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=TIMEOUT_SECS) as resp:
        return resp.read()


def main() -> int:
    DEST.mkdir(parents=True, exist_ok=True)
    changed: list[str] = []
    failed: list[tuple[str, str]] = []

    for url, name in SPECS:
        out = DEST / name
        try:
            body = fetch(url)
        except Exception as e:
            failed.append((name, f"{type(e).__name__}: {e}"))
            print(f"FAIL  {name:40s} {url}\n      {type(e).__name__}: {e}", file=sys.stderr)
            continue

        prev = out.read_bytes() if out.is_file() else None
        if prev == body:
            print(f"same  {name:40s} ({len(body):,} bytes)")
            continue

        out.write_bytes(body)
        changed.append(name)
        delta = len(body) - (len(prev) if prev else 0)
        print(f"WROTE {name:40s} ({len(body):,} bytes, Δ {delta:+,})")

    print()
    print(f"{len(SPECS)} specs total, {len(changed)} changed, {len(failed)} failed")
    if failed:
        print("\nFailures (non-fatal — fetcher exits non-zero on at least one failure):")
        for name, err in failed:
            print(f"  {name}: {err}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
