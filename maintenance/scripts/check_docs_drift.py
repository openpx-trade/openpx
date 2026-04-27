#!/usr/bin/env python3
"""Detect drift in the Kalshi and Polymarket changelogs against a checked-in lock file.

The orchestrator (`.claude/agents/orchestrator.md`) is the only consumer.
Drift is changelog-only by design — the bot's single job is to mirror what
the upstreams announce. Other documentation pages (specs, llms.txt, prose)
are no longer tracked here.

Usage:
    python3 maintenance/scripts/check_docs_drift.py            # check, exit 0 (clean) or 1 (drift)
    python3 maintenance/scripts/check_docs_drift.py --json     # machine-readable output
    python3 maintenance/scripts/check_docs_drift.py --update   # refresh lock with current state
    python3 maintenance/scripts/check_docs_drift.py --exchange kalshi

Lock file shape (`maintenance/scripts/exchange-docs.lock.json`):
    {
      "<exchange>": {
        "url": "...",
        "last_checked": "ISO-8601",
        "sha256": "...",
        "body": "<full markdown of the changelog>"
      },
      ...
    }

The `body` field is committed so the orchestrator can compute a unified
diff between last-known and live without re-fetching the previous state.
"""

from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent.parent
LOCK_PATH = ROOT / "maintenance" / "scripts" / "exchange-docs.lock.json"

CHANGELOGS: dict[str, str] = {
    "kalshi": "https://docs.kalshi.com/changelog.md",
    "polymarket": "https://docs.polymarket.com/changelog.md",
}


def fetch(url: str, timeout: int = 30) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "openpx-changelog-drift/1"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read().decode("utf-8", errors="replace")


def sha256(s: str) -> str:
    return hashlib.sha256(s.encode("utf-8")).hexdigest()


def load_lock() -> dict[str, Any]:
    if LOCK_PATH.exists():
        return json.loads(LOCK_PATH.read_text())
    return {}


def save_lock(data: dict[str, Any]) -> None:
    LOCK_PATH.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def unified_diff(prev: str, curr: str, label: str) -> str:
    return "".join(difflib.unified_diff(
        prev.splitlines(keepends=True),
        curr.splitlines(keepends=True),
        fromfile=f"{label}.prev",
        tofile=f"{label}.curr",
        n=3,
    ))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    ap.add_argument("--update", action="store_true", help="Refresh lock with current state")
    ap.add_argument("--exchange", choices=list(CHANGELOGS), help="Limit to one exchange")
    ap.add_argument("--json", action="store_true", help="Emit JSON instead of human report")
    args = ap.parse_args()

    targets = [args.exchange] if args.exchange else list(CHANGELOGS)
    lock = load_lock()
    drift_found = False
    json_out: dict[str, Any] = {}

    for ex in targets:
        url = CHANGELOGS[ex]
        try:
            curr = fetch(url)
        except Exception as e:
            err = f"{type(e).__name__}: {e}"
            if args.json:
                json_out[ex] = {"error": err}
            else:
                print(f"=== {ex} ===\n  ERROR fetching {url}: {err}", file=sys.stderr)
            return 3

        curr_hash = sha256(curr)
        prev_entry = lock.get(ex, {})
        prev_hash = prev_entry.get("sha256")
        prev_body = prev_entry.get("body", "")

        if args.update:
            lock[ex] = {
                "url": url,
                "last_checked": datetime.now(timezone.utc).isoformat(timespec="seconds"),
                "sha256": curr_hash,
                "body": curr,
            }
            if not args.json:
                print(f"=== {ex} ===  updated ({len(curr)} bytes)")
            continue

        if not prev_hash:
            msg = f"no prior lock for {ex}; run with --update to bootstrap"
            if args.json:
                json_out[ex] = {"status": "uninitialized", "message": msg}
            else:
                print(f"=== {ex} ===\n  {msg}")
            drift_found = True
            continue

        if prev_hash == curr_hash:
            if args.json:
                json_out[ex] = {"status": "clean"}
            else:
                print(f"=== {ex} ===\n  clean")
            continue

        diff = unified_diff(prev_body, curr, ex)
        drift_found = True
        if args.json:
            json_out[ex] = {
                "status": "drift",
                "url": url,
                "prev_sha256": prev_hash,
                "curr_sha256": curr_hash,
                "diff": diff,
            }
        else:
            print(f"=== {ex} ===\n  DRIFT ({url})")
            print(f"  prev: {prev_hash[:12]}  curr: {curr_hash[:12]}")
            print("  --- diff ---")
            print(diff)
            print("  ------------")

    if args.update:
        save_lock(lock)
        if not args.json:
            print(f"\nlock written: {LOCK_PATH.relative_to(ROOT)}")
        return 0

    if args.json:
        json.dump(json_out, sys.stdout, indent=2)
        sys.stdout.write("\n")

    return 1 if drift_found else 0


if __name__ == "__main__":
    sys.exit(main())
