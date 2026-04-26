#!/usr/bin/env python3
"""Detect drift in Kalshi and Polymarket documentation against a checked-in lock file.

Usage:
    python3 maintenance/scripts/check_docs_drift.py                  # check (fast: specs + tier1)
    python3 maintenance/scripts/check_docs_drift.py --full           # also walks every llms.txt URL
    python3 maintenance/scripts/check_docs_drift.py --update         # refresh lock file
    python3 maintenance/scripts/check_docs_drift.py --exchange kalshi
    python3 maintenance/scripts/check_docs_drift.py --json           # machine-readable output

Exit codes:
    0  clean
    1  Tier 1 drift (specs or tier1 pages — block-merging severity)
    2  Watch drift only (new/removed/changed pages from llms.txt)
    3  Network or parse error
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent.parent
LOCK_PATH = ROOT / "maintenance" / "scripts" / "exchange-docs.lock.json"

CONFIGS: dict[str, dict[str, Any]] = {
    "kalshi": {
        "specs": {
            "openapi.yaml": "https://docs.kalshi.com/openapi.yaml",
            "asyncapi.yaml": "https://docs.kalshi.com/asyncapi.yaml",
        },
        "tier1": [
            "https://docs.kalshi.com/changelog.md",
            "https://docs.kalshi.com/llms.txt",
        ],
        "llms_txt": "https://docs.kalshi.com/llms.txt",
    },
    "polymarket": {
        "specs": {},
        "tier1": [
            "https://docs.polymarket.com/changelog.md",
            "https://docs.polymarket.com/api-reference/introduction.md",
            "https://docs.polymarket.com/api-reference/authentication.md",
            "https://docs.polymarket.com/resources/contracts.md",
            "https://docs.polymarket.com/llms.txt",
        ],
        "llms_txt": "https://docs.polymarket.com/llms.txt",
    },
}

LLMS_URL_RE = re.compile(r"\((https?://[^)]+\.md)\)")


def fetch(url: str, timeout: int = 30) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "openpx-docs-drift/1"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def spec_version(text: str) -> str | None:
    in_info = False
    for line in text.splitlines():
        if line.startswith("info:"):
            in_info = True
            continue
        if in_info:
            if line and not line.startswith((" ", "\t")):
                break
            m = re.match(r"\s+version:\s*(.+)$", line)
            if m:
                return m.group(1).strip().strip('"').strip("'")
    return None


def parse_llms_urls(text: str) -> list[str]:
    return sorted(set(LLMS_URL_RE.findall(text)))


def load_lock() -> dict[str, Any]:
    if LOCK_PATH.exists():
        return json.loads(LOCK_PATH.read_text())
    return {}


def save_lock(data: dict[str, Any]) -> None:
    LOCK_PATH.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def snapshot(exchange: str, full_watch: bool, polite_delay: float = 0.1) -> dict[str, Any]:
    cfg = CONFIGS[exchange]
    state: dict[str, Any] = {
        "last_checked": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "specs": {},
        "tier1": {},
        "watch": {},
    }

    for name, url in cfg["specs"].items():
        body = fetch(url).decode("utf-8", errors="replace")
        state["specs"][name] = {
            "url": url,
            "version": spec_version(body),
            "sha256": sha256(body.encode()),
        }

    for url in cfg["tier1"]:
        state["tier1"][url] = sha256(fetch(url))
        time.sleep(polite_delay)

    if full_watch:
        llms = fetch(cfg["llms_txt"]).decode("utf-8", errors="replace")
        for url in parse_llms_urls(llms):
            try:
                state["watch"][url] = sha256(fetch(url))
            except urllib.error.HTTPError as e:
                state["watch"][url] = f"HTTP_{e.code}"
            except Exception as e:
                state["watch"][url] = f"ERROR_{type(e).__name__}"
            time.sleep(polite_delay)

    return state


def diff_state(prev: dict[str, Any], curr: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {
        "specs_changed": [],
        "tier1_changed": [],
        "watch_changed": [],
        "watch_added": [],
        "watch_removed": [],
    }

    for name, info in curr.get("specs", {}).items():
        prev_info = prev.get("specs", {}).get(name, {})
        if prev_info.get("sha256") != info["sha256"]:
            out["specs_changed"].append({
                "name": name,
                "prev_version": prev_info.get("version"),
                "curr_version": info["version"],
            })

    for url, h in curr.get("tier1", {}).items():
        if prev.get("tier1", {}).get(url) != h:
            out["tier1_changed"].append(url)

    prev_watch = prev.get("watch", {})
    curr_watch = curr.get("watch", {})
    for url, h in curr_watch.items():
        if url not in prev_watch:
            out["watch_added"].append(url)
        elif prev_watch[url] != h:
            out["watch_changed"].append(url)
    for url in prev_watch:
        if url not in curr_watch:
            out["watch_removed"].append(url)

    return out


def severity(drift: dict[str, Any]) -> int:
    if drift["specs_changed"] or drift["tier1_changed"]:
        return 1
    if drift["watch_changed"] or drift["watch_added"] or drift["watch_removed"]:
        return 2
    return 0


def report_human(exchange: str, drift: dict[str, Any]) -> None:
    print(f"\n=== {exchange} ===")
    if drift["specs_changed"]:
        print("  [TIER 1] specs changed:")
        for s in drift["specs_changed"]:
            print(f"    {s['name']}: {s['prev_version']} -> {s['curr_version']}")
    if drift["tier1_changed"]:
        print("  [TIER 1] pages changed:")
        for url in drift["tier1_changed"]:
            print(f"    {url}")
    if drift["watch_added"]:
        print(f"  [INFO] {len(drift['watch_added'])} new pages in llms.txt")
        for url in drift["watch_added"][:5]:
            print(f"    + {url}")
        if len(drift["watch_added"]) > 5:
            print(f"    ... and {len(drift['watch_added']) - 5} more")
    if drift["watch_removed"]:
        print(f"  [INFO] {len(drift['watch_removed'])} pages removed from llms.txt")
        for url in drift["watch_removed"][:5]:
            print(f"    - {url}")
    if drift["watch_changed"]:
        print(f"  [INFO] {len(drift['watch_changed'])} watched pages changed body")

    if severity(drift) == 0:
        print("  clean")


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Detect Kalshi/Polymarket documentation drift against maintenance/scripts/exchange-docs.lock.json"
    )
    ap.add_argument("--update", action="store_true", help="Refresh lock with current state")
    ap.add_argument("--full", action="store_true", help="Also fetch every URL from llms.txt")
    ap.add_argument("--exchange", choices=list(CONFIGS), help="Limit to one exchange")
    ap.add_argument("--json", action="store_true", help="Emit JSON instead of human-readable report")
    args = ap.parse_args()

    exchanges = [args.exchange] if args.exchange else list(CONFIGS)
    lock = load_lock()
    max_sev = 0
    json_out: dict[str, Any] = {}

    for ex in exchanges:
        try:
            curr = snapshot(ex, full_watch=args.full)
        except Exception as e:
            print(f"\n=== {ex} ===\n  ERROR: {type(e).__name__}: {e}", file=sys.stderr)
            max_sev = max(max_sev, 3)
            continue

        if args.update:
            lock[ex] = curr
            if not args.json:
                print(f"=== {ex} ===  updated: {len(curr['tier1'])} tier1, "
                      f"{len(curr['specs'])} specs, {len(curr['watch'])} watch")
            continue

        prev = lock.get(ex, {})
        if not prev:
            msg = f"no prior lock for {ex}; run with --update to bootstrap"
            if args.json:
                json_out[ex] = {"error": msg}
            else:
                print(f"\n=== {ex} ===\n  {msg}")
            max_sev = max(max_sev, 2)
            continue

        drift = diff_state(prev, curr)
        if args.json:
            json_out[ex] = {"severity": severity(drift), "drift": drift}
        else:
            report_human(ex, drift)
        max_sev = max(max_sev, severity(drift))

    if args.update:
        save_lock(lock)
        if not args.json:
            print(f"\nlock written: {LOCK_PATH.relative_to(ROOT)}")
        return 0

    if args.json:
        json.dump(json_out, sys.stdout, indent=2)
        sys.stdout.write("\n")

    return max_sev


if __name__ == "__main__":
    sys.exit(main())
