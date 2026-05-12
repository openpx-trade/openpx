#!/usr/bin/env python3
"""Render comparative-benchmark JSON into the README's BENCH block.

The bench suite runs three harnesses, each producing a different
JSON shape that we read here:

  - Rust (criterion / codspeed-criterion-compat):
      target/criterion/<group>/<id>/new/estimates.json
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

ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
RESULTS_DIR = ROOT / "benches" / "comparative" / "results"
CRITERION_DIR = ROOT / "target" / "criterion"

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


def fmt_speedup(openpx_ns: float | None, other_ns: float | None) -> str:
    if openpx_ns is None or other_ns is None or openpx_ns <= 0:
        return "—"
    ratio = other_ns / openpx_ns
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
    openpx_ns: float | None
    other_ns: float | None


def render_table(other_label: str, rows: list[Row]) -> str:
    """Three-column table: Operation | OpenPX | <other> | Speedup.

    Drops rows where both sides are missing so an empty section
    collapses cleanly (the section header is added by the caller).
    """
    rows = [r for r in rows if r.openpx_ns is not None or r.other_ns is not None]
    if not rows:
        return ""
    out = [
        f"| Operation | OpenPX | {other_label} | Speedup |",
        "|---|---:|---:|---:|",
    ]
    for r in rows:
        out.append(
            f"| {r.operation} | {fmt_ns(r.openpx_ns)} | {fmt_ns(r.other_ns)} | "
            f"{fmt_speedup(r.openpx_ns, r.other_ns)} |"
        )
    return "\n".join(out)


def render_rust_section() -> str:
    rows = [
        Row(
            "Parse Polymarket book (5-min BTC fixture)",
            _criterion_estimate("parse_polymarket_book", "openpx"),
            _criterion_estimate("parse_polymarket_book", "polymarket_sdk"),
        ),
    ]
    table = render_table("polymarket_client_sdk_v2", rows)
    if not table:
        return ""
    return (
        "### Rust core\n\n"
        "_Kalshi has no upstream Rust SDK — OpenPX is the only Rust client "
        "that supports it._\n\n" + table
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
                poly.get("test_openpx_fetch_polymarket_book"),
                poly.get("test_pyclob_fetch_polymarket_book"),
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
                kalshi.get("test_openpx_fetch_kalshi_book"),
                kalshi.get("test_kalshi_python_fetch_kalshi_book"),
            ),
        ],
    )
    if kalshi_table:
        sections.append(kalshi_table)

    if not sections:
        return ""
    return "### Python SDK\n\n" + "\n\n".join(sections)


def render_typescript_section() -> str:
    poly = _tinybench_results("typescript_polymarket.json")
    kalshi = _tinybench_results("typescript_kalshi.json")

    sections: list[str] = []

    poly_table = render_table(
        "@polymarket/clob-client",
        [
            Row(
                "Polymarket fetch_orderbook (5-min BTC, live)",
                poly.get("openpx::polymarket::fetch_orderbook"),
                poly.get("polymarket-clob-client::fetch_orderbook"),
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
                kalshi.get("openpx::kalshi::fetch_orderbook"),
                kalshi_other,
            ),
        ],
    )
    if kalshi_table:
        sections.append(kalshi_table)

    if not sections:
        return ""
    return "### TypeScript SDK\n\n" + "\n\n".join(sections)


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
