#!/usr/bin/env python3
"""Detect drift in the Kalshi and Polymarket changelogs against a per-entry lock.

The orchestrator (`.claude/agents/orchestrator.md`) is the only consumer.
Each `<Update label="MMM DD, YYYY">...</Update>` block in the upstream
markdown is parsed structurally and hashed individually so drift is
reported per entry, not as a unified diff.

Usage:
    python3 maintenance/scripts/check_docs_drift.py            # check, exit 0 (clean) or 1 (drift)
    python3 maintenance/scripts/check_docs_drift.py --json     # machine-readable output
    python3 maintenance/scripts/check_docs_drift.py --update   # refresh lock with current state
    python3 maintenance/scripts/check_docs_drift.py --bootstrap  # one-time migration from old lock shape
    python3 maintenance/scripts/check_docs_drift.py --exchange kalshi

Lock file shape (`maintenance/scripts/exchange-docs.lock.json`):

    {
      "<exchange>": {
        "url": "...",
        "entries": {
          "<id>": {
            "label": "Apr 22, 2026",
            "title": "<rss.title or empty>",
            "hash": "<sha256 of the full <Update>...</Update> block>",
            "body": "<full <Update>...</Update> block markdown>"
          },
          ...
        }
      }
    }

`<id>` is `YYYY-MM-DD` for the first entry on a given date, and
`YYYY-MM-DD-2`, `-3`, ... for subsequent same-date entries in source order.
The id is used as the orchestrator's dedup key (PR label `cl/<exchange>/<id>`).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import urllib.request
from datetime import datetime
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent.parent
LOCK_PATH = ROOT / "maintenance" / "scripts" / "exchange-docs.lock.json"

CHANGELOGS: dict[str, str] = {
    "kalshi": "https://docs.kalshi.com/changelog.md",
    "polymarket": "https://docs.polymarket.com/changelog.md",
}

UPDATE_RE = re.compile(
    r"<Update\b(?P<attrs>[^>]*?)>(?P<body>.*?)</Update>",
    re.DOTALL,
)
LABEL_RE = re.compile(r'label="([^"]+)"')
TITLE_RE = re.compile(r'title:\s*"([^"]+)"')


def fetch(url: str, timeout: int = 30) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "openpx-changelog-drift/2"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read().decode("utf-8", errors="replace")


def sha256(s: str) -> str:
    return hashlib.sha256(s.encode("utf-8")).hexdigest()


def label_to_iso(label: str) -> str | None:
    """Convert 'Apr 22, 2026' -> '2026-04-22'. Returns None if unparseable."""
    try:
        return datetime.strptime(label.strip(), "%b %d, %Y").strftime("%Y-%m-%d")
    except ValueError:
        return None


def parse_entries(markdown: str) -> dict[str, dict[str, str]]:
    """Parse <Update> blocks and assign stable per-entry ids.

    Returns an ordered dict of `{id: {label, title, hash, body}}` keyed by
    `YYYY-MM-DD` (or `YYYY-MM-DD-N` for the Nth same-date entry in source
    order). Skips blocks whose label cannot be parsed as a date.
    """
    by_date_count: dict[str, int] = {}
    out: dict[str, dict[str, str]] = {}
    for m in UPDATE_RE.finditer(markdown):
        attrs = m.group("attrs")
        label_m = LABEL_RE.search(attrs)
        if not label_m:
            continue
        label = label_m.group(1)
        iso = label_to_iso(label)
        if not iso:
            continue
        title_m = TITLE_RE.search(attrs)
        title = title_m.group(1) if title_m else ""
        block = markdown[m.start():m.end()]
        n = by_date_count.get(iso, 0) + 1
        by_date_count[iso] = n
        eid = iso if n == 1 else f"{iso}-{n}"
        out[eid] = {
            "label": label,
            "title": title,
            "hash": sha256(block),
            "body": block,
        }
    return out


def load_lock() -> dict[str, Any]:
    if LOCK_PATH.exists():
        return json.loads(LOCK_PATH.read_text())
    return {}


def save_lock(data: dict[str, Any]) -> None:
    LOCK_PATH.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def diff_entries(
    prev: dict[str, dict[str, str]],
    curr: dict[str, dict[str, str]],
) -> dict[str, list[dict[str, str]]]:
    """Compute per-entry drift between prev and curr.

    Returns `{new, amended, removed}` lists. Each item carries the entry
    dict (label/title/hash/body) plus its `id`. `amended` items also carry
    `prev_hash` for the audit trail.
    """
    new: list[dict[str, str]] = []
    amended: list[dict[str, str]] = []
    removed: list[dict[str, str]] = []

    for eid, entry in curr.items():
        prior = prev.get(eid)
        if prior is None:
            new.append({"id": eid, **entry})
        elif prior.get("hash") != entry["hash"]:
            amended.append({"id": eid, "prev_hash": prior.get("hash", ""), **entry})

    for eid, entry in prev.items():
        if eid not in curr:
            removed.append({"id": eid, **entry})

    return {"new": new, "amended": amended, "removed": removed}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    ap.add_argument("--update", action="store_true", help="Refresh lock with current state")
    ap.add_argument("--bootstrap", action="store_true",
                    help="One-time: fetch and write the new per-entry lock format, ignoring prior shape")
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
            curr_markdown = fetch(url)
        except Exception as e:
            err = f"{type(e).__name__}: {e}"
            if args.json:
                json_out[ex] = {"error": err}
            else:
                print(f"=== {ex} ===\n  ERROR fetching {url}: {err}", file=sys.stderr)
            return 3

        curr_entries = parse_entries(curr_markdown)

        if args.update or args.bootstrap:
            lock[ex] = {"url": url, "entries": curr_entries}
            if not args.json:
                print(f"=== {ex} ===  {len(curr_entries)} entries")
            continue

        prev_entry_block = lock.get(ex, {})
        prev_entries = prev_entry_block.get("entries")
        if prev_entries is None:
            msg = f"no prior lock for {ex}; run with --bootstrap to initialize"
            if args.json:
                json_out[ex] = {"status": "uninitialized", "message": msg}
            else:
                print(f"=== {ex} ===\n  {msg}")
            drift_found = True
            continue

        diff = diff_entries(prev_entries, curr_entries)
        if not (diff["new"] or diff["amended"] or diff["removed"]):
            if args.json:
                json_out[ex] = {"status": "clean"}
            else:
                print(f"=== {ex} ===  clean ({len(curr_entries)} entries)")
            continue

        drift_found = True
        if args.json:
            json_out[ex] = {"status": "drift", "url": url, **diff}
        else:
            print(f"=== {ex} ===  DRIFT ({url})")
            for kind, items in diff.items():
                if items:
                    print(f"  {kind}: {len(items)}")
                    for item in items:
                        print(f"    - {item['id']}: {item.get('title') or item['label']}")

    if args.update or args.bootstrap:
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
