#!/usr/bin/env python3
"""Render comparative-benchmark JSON into the README's BENCH block.

The bench suite runs four harnesses, each producing a different JSON
shape that we read here:

  - Rust walltime (criterion / codspeed-criterion-compat):
      target/criterion/<group>/<id>/new/estimates.json
  - Rust CPU + memory (iai-callgrind, valgrind + DHAT):
      target/iai/px-bench-comparative/parse_polymarket_book_iai/
        parse_polymarket_book/<id>/summary.json
  - Python (pytest-benchmark / pytest-codspeed):
      benches/comparative/results/python_polymarket.json
      benches/comparative/results/python_kalshi.json
  - TypeScript (tinybench / @codspeed/tinybench-plugin):
      benches/comparative/results/typescript_polymarket.json
      benches/comparative/results/typescript_kalshi.json

A missing source is rendered as "—" rather than an error so a
half-broken CI run still produces a valid README.

The README block is delimited by `<!-- BENCH:START -->` and
`<!-- BENCH:END -->`; we splice between those markers in place.

Run via:
    python3 tools/render_bench_readme.py
"""

from __future__ import annotations

import datetime as _dt
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
RESULTS_DIR = ROOT / "benches" / "comparative" / "results"
CRITERION_DIR = ROOT / "target" / "criterion"
IAI_DIR = (
    ROOT / "target" / "iai" / "px-bench-comparative" / "parse_polymarket_book_iai"
)

START_MARKER = "<!-- BENCH:START -->"
END_MARKER = "<!-- BENCH:END -->"


# ---------------------------------------------------------------------------
# Formatting helpers
# ---------------------------------------------------------------------------


def fmt_ns(ns: float | None) -> str:
    if ns is None:
        return "—"
    if ns < 10:
        return f"{ns:.2f} ns"
    if ns < 1_000:
        return f"{ns:.0f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    return f"{ns / 1_000_000_000:.2f} s"


def fmt_count(n: float | None) -> str:
    """Integer-ish count: instruction counts, allocation counts."""
    if n is None:
        return "—"
    if n < 1_000:
        return f"{n:,.0f}"
    if n < 1_000_000:
        return f"{n / 1_000:.2f}k"
    if n < 1_000_000_000:
        return f"{n / 1_000_000:.2f}M"
    return f"{n / 1_000_000_000:.2f}B"


def fmt_bytes(b: float | None) -> str:
    if b is None:
        return "—"
    if b < 1024:
        return f"{b:,.0f} B"
    if b < 1024**2:
        return f"{b / 1024:.2f} KiB"
    if b < 1024**3:
        return f"{b / 1024**2:.2f} MiB"
    return f"{b / 1024**3:.2f} GiB"


def fmt_speedup(openpx_v: float | None, other_v: float | None) -> str:
    """Speedup ratio. Works for any metric where lower-is-better
    (walltime, instructions, bytes allocated). Ratio = other / openpx;
    >1 means OpenPX is faster/leaner. Bolded when OpenPX wins."""
    if openpx_v is None or other_v is None or openpx_v <= 0:
        return "—"
    ratio = other_v / openpx_v
    return f"**{ratio:.2f}×**" if ratio >= 1.0 else f"{ratio:.2f}×"


# ---------------------------------------------------------------------------
# Source readers
# ---------------------------------------------------------------------------


def _criterion_estimate(group: str, function_id: str) -> float | None:
    """Mean point estimate (ns) for one criterion bench function."""
    path = CRITERION_DIR / group / function_id / "new" / "estimates.json"
    if not path.exists():
        return None
    try:
        return float(json.loads(path.read_text())["mean"]["point_estimate"])
    except (KeyError, json.JSONDecodeError, ValueError):
        return None


def _walk_for_key(node: Any, key: str) -> Any:
    """Depth-first walk through a JSON tree returning the first value
    found under `key`. iai-callgrind's summary schema has shifted across
    minor versions; this tree walker is intentionally schema-agnostic so
    a version bump won't silently zero out the README rows. Returns the
    raw value (int / float / str / dict) — callers narrow further."""
    if isinstance(node, dict):
        if key in node:
            return node[key]
        for v in node.values():
            found = _walk_for_key(v, key)
            if found is not None:
                return found
    elif isinstance(node, list):
        for v in node:
            found = _walk_for_key(v, key)
            if found is not None:
                return found
    return None


def _iai_metric(function_id: str, metric: str) -> float | None:
    """Read one metric from iai-callgrind's per-bench summary.json.

    Supported metrics:
      - "Ir"          → cachegrind instruction count (CPU simulation)
      - "total_bytes" → DHAT cumulative bytes allocated (memory)

    iai-callgrind nests the actual count under various keys depending on
    the version (`new`, `count`, raw int). We accept any of those so the
    parser survives runner upgrades."""
    path = IAI_DIR / "parse_polymarket_book" / function_id / "summary.json"
    if not path.exists():
        return None
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return None
    raw = _walk_for_key(payload, metric)
    if raw is None:
        return None
    if isinstance(raw, (int, float)):
        return float(raw)
    if isinstance(raw, dict):
        for k in ("new", "count", "value", "total"):
            v = raw.get(k)
            if isinstance(v, (int, float)):
                return float(v)
    return None


def _pytest_benchmarks(filename: str) -> dict[str, float]:
    """Map pytest-benchmark name → mean ns from a JSON dump."""
    path = RESULTS_DIR / filename
    if not path.exists():
        return {}
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return {}
    out: dict[str, float] = {}
    for entry in payload.get("benchmarks", []):
        name = entry.get("name") or ""
        mean_s = entry.get("stats", {}).get("mean")
        if mean_s is not None:
            out[name] = float(mean_s) * 1e9
    return out


def _tinybench_results(filename: str) -> dict[str, float]:
    """Map tinybench task name → mean ns from a JSON dump."""
    path = RESULTS_DIR / filename
    if not path.exists():
        return {}
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return {}
    out: dict[str, float] = {}
    for task in payload.get("tasks", []):
        name = task.get("name")
        mean = task.get("mean_ns")
        if name and mean is not None:
            out[name] = float(mean)
    return out


# ---------------------------------------------------------------------------
# Per-section renderers
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Row:
    operation: str
    metric: str
    openpx: float | None
    other: float | None
    formatter: Any  # callable: float | None -> str


def render_table(other_label: str, rows: list[Row]) -> str:
    """Five-column table: Operation | Metric | OpenPX | <other> | Speedup.

    Each row picks its own formatter (ns / instruction count / bytes)
    so the Rust table can mix walltime, CPU instructions, and heap
    allocations under one header. Drops rows where both sides are
    missing so an empty section collapses cleanly.
    """
    rows = [r for r in rows if r.openpx is not None or r.other is not None]
    if not rows:
        return ""
    out = [
        f"| Operation | Metric | OpenPX | {other_label} | Speedup |",
        "|---|---|---:|---:|---:|",
    ]
    for r in rows:
        out.append(
            f"| {r.operation} | {r.metric} | {r.formatter(r.openpx)} | "
            f"{r.formatter(r.other)} | {fmt_speedup(r.openpx, r.other)} |"
        )
    return "\n".join(out)


def render_rust_section() -> str:
    op = "Parse Polymarket book (5-min BTC fixture)"
    rows = [
        Row(
            op,
            "Walltime",
            _criterion_estimate("parse_polymarket_book", "openpx"),
            _criterion_estimate("parse_polymarket_book", "polymarket_sdk"),
            fmt_ns,
        ),
        Row(
            op,
            "CPU instructions (cachegrind)",
            _iai_metric("openpx", "Ir"),
            _iai_metric("polymarket_sdk", "Ir"),
            fmt_count,
        ),
        Row(
            op,
            "Heap allocations (DHAT)",
            _iai_metric("openpx", "total_bytes"),
            _iai_metric("polymarket_sdk", "total_bytes"),
            fmt_bytes,
        ),
    ]
    table = render_table("polymarket_client_sdk_v2", rows)
    if not table:
        return ""
    return (
        "### Rust core\n\n"
        "_Kalshi has no upstream Rust SDK — OpenPX is the only Rust client "
        "that supports it. Walltime from criterion; CPU instructions and "
        "heap allocations from valgrind via iai-callgrind (deterministic, "
        "hardware-agnostic — same instruments Codspeed uses)._\n\n" + table
    )


def render_python_section() -> str:
    poly = _pytest_benchmarks("python_polymarket.json")
    kalshi = _pytest_benchmarks("python_kalshi.json")

    sections: list[str] = []

    poly_table = render_table(
        "py-clob-client",
        [
            Row(
                "Polymarket fetch_orderbook (5-min BTC, live)",
                "Walltime",
                poly.get("test_openpx_fetch_polymarket_book"),
                poly.get("test_pyclob_fetch_polymarket_book"),
                fmt_ns,
            ),
        ],
    )
    if poly_table:
        sections.append(poly_table)

    kalshi_table = render_table(
        "kalshi-python",
        [
            Row(
                "Kalshi fetch_orderbook (15-min BTC, live)",
                "Walltime",
                kalshi.get("test_openpx_fetch_kalshi_book"),
                kalshi.get("test_kalshi_python_fetch_kalshi_book"),
                fmt_ns,
            ),
        ],
    )
    if kalshi_table:
        sections.append(kalshi_table)

    if not sections:
        return ""
    return (
        "### Python SDK\n\n"
        "_Walltime over real, unauthenticated HTTP round-trips. CPU "
        "instructions and heap allocations aren't reported per-language "
        "in the README — Codspeed only exposes those instruments for "
        "compiled-language harnesses (see Rust above and the dashboard "
        "for trends)._\n\n" + "\n\n".join(sections)
    )


def render_typescript_section() -> str:
    poly = _tinybench_results("typescript_polymarket.json")
    kalshi = _tinybench_results("typescript_kalshi.json")

    sections: list[str] = []

    poly_table = render_table(
        "@polymarket/clob-client",
        [
            Row(
                "Polymarket fetch_orderbook (5-min BTC, live)",
                "Walltime",
                poly.get("openpx::polymarket::fetch_orderbook"),
                poly.get("polymarket-clob-client::fetch_orderbook"),
                fmt_ns,
            ),
        ],
    )
    if poly_table:
        sections.append(poly_table)

    # Kalshi TS upstream package name has churned — `kalshi-typescript`
    # is the official OpenAPI-generated client (npm: `kalshi`); the others
    # are community / older variants that may show up in some envs.
    kalshi_candidates = ("kalshi-typescript", "kalshi-typescript-sdk", "kalshi-ts", "@kalshi/sdk")
    kalshi_other_name = "kalshi-typescript"
    kalshi_other = None
    for name in kalshi_candidates:
        v = kalshi.get(f"{name}::kalshi::fetch_orderbook")
        if v is not None:
            kalshi_other = v
            kalshi_other_name = name
            break
    kalshi_table = render_table(
        kalshi_other_name,
        [
            Row(
                "Kalshi fetch_orderbook (15-min BTC, live)",
                "Walltime",
                kalshi.get("openpx::kalshi::fetch_orderbook"),
                kalshi_other,
                fmt_ns,
            ),
        ],
    )
    if kalshi_table:
        sections.append(kalshi_table)

    if not sections:
        return ""
    return (
        "### TypeScript SDK\n\n"
        "_Walltime over real, unauthenticated HTTP round-trips._\n\n"
        + "\n\n".join(sections)
    )


# ---------------------------------------------------------------------------
# Splice
# ---------------------------------------------------------------------------


def build_block() -> str:
    today = _dt.date.today().isoformat()

    sections = [
        render_rust_section(),
        render_python_section(),
        render_typescript_section(),
    ]
    sections = [s for s in sections if s]

    if not sections:
        body = (
            "_Comparative benchmarks pending — first run will populate this section. "
            "See [`benches/comparative/README.md`](benches/comparative/README.md)._"
        )
    else:
        body = "\n\n".join(sections)

    return "\n".join(
        [
            START_MARKER,
            "## Performance",
            "",
            "OpenPX vs the official native SDKs, head-to-head against live "
            "5/15-min BTC markets. Real bytes, no credentials, refreshes on "
            "every push to `main`.",
            "",
            "[![CodSpeed](https://img.shields.io/endpoint?url=https%3A%2F%2Fcodspeed.io%2Fbadge.json)](https://codspeed.io/openpx-trade/openpx)"
            " — Rust benches also tracked under Codspeed CPU-simulation and "
            "memory-allocation instruments. Click the badge for the full "
            "per-metric history and PR-level regression alerts.",
            "",
            body,
            "",
            (
                f"<sub>Last updated: {today} · Methodology: "
                "[benches/comparative/README.md](benches/comparative/README.md) · "
                "Raw data: [benches/comparative/results/](benches/comparative/results/)</sub>"
            ),
            END_MARKER,
        ]
    )


def main() -> int:
    text = README.read_text()
    pattern = re.compile(
        re.escape(START_MARKER) + r".*?" + re.escape(END_MARKER),
        flags=re.DOTALL,
    )
    if not pattern.search(text):
        sys.stderr.write(
            f"error: README.md missing {START_MARKER}/{END_MARKER} markers — "
            "insert them where the perf table should land.\n"
        )
        return 1
    new_block = build_block()
    updated = pattern.sub(lambda _m: new_block, text, count=1)
    if updated != text:
        README.write_text(updated)
        print(f"updated {README.relative_to(ROOT)}")
    else:
        print(f"{README.relative_to(ROOT)} already up to date")
    return 0


if __name__ == "__main__":
    sys.exit(main())
