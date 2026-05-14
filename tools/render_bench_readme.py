#!/usr/bin/env python3
"""Render comparative-benchmark JSON into the README's BENCH block.

Inputs (all optional — missing source = "—" row):

  target/criterion/<group>/<id>/new/estimates.json   (Rust hot-path)
  benches/comparative/results/python_polymarket.json (pytest-benchmark)
  benches/comparative/results/python_kalshi.json     (pytest-benchmark)
  benches/comparative/results/typescript_polymarket.json (tinybench)
  benches/comparative/results/typescript_kalshi.json     (tinybench)

The README block layout mirrors polyfill-rs's two-angle pitch:

  Section 1 — Computational / hot path: where OpenPX architecturally wins.
              Rust criterion timings, pure CPU.
  Section 2 — End-to-end REST API:      OpenPX vs each official SDK,
              all unauthenticated methods, mean ± stddev over 20
              same-machine same-minute iterations.

The block is delimited by `<!-- BENCH:START -->` / `<!-- BENCH:END -->`;
we splice between those markers in place.

Run:
    python3 tools/render_bench_readme.py
"""

from __future__ import annotations

import datetime as _dt
import json
import math
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
RESULTS = ROOT / "benches" / "comparative" / "results"
CRITERION = ROOT / "target" / "criterion"

START = "<!-- BENCH:START -->"
END = "<!-- BENCH:END -->"


# ---------------------------------------------------------------------------
# Formatting
# ---------------------------------------------------------------------------


def fmt_time(ns: Optional[float]) -> str:
    if ns is None or not math.isfinite(ns):
        return "—"
    if ns < 1_000:
        return f"{ns:.0f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    return f"{ns / 1e9:.2f} s"


def fmt_pm(mean_ns: Optional[float], sd_ns: Optional[float]) -> str:
    """mean ± stddev — polyfill methodology."""
    if mean_ns is None or not math.isfinite(mean_ns):
        return "—"
    if sd_ns is None or not math.isfinite(sd_ns) or sd_ns <= 0:
        return fmt_time(mean_ns)
    return f"{fmt_time(mean_ns)} ± {fmt_time(sd_ns)}"


def fmt_speedup(openpx: Optional[float], other: Optional[float]) -> str:
    if not openpx or not other or openpx <= 0:
        return "—"
    ratio = other / openpx
    return f"**{ratio:.2f}×**" if ratio >= 1.0 else f"{ratio:.2f}×"


# ---------------------------------------------------------------------------
# Source readers
# ---------------------------------------------------------------------------


def criterion_mean(group: str, function_id: str) -> Optional[float]:
    path = CRITERION / group / function_id / "new" / "estimates.json"
    if not path.exists():
        return None
    try:
        return float(json.loads(path.read_text())["mean"]["point_estimate"])
    except (KeyError, ValueError, json.JSONDecodeError):
        return None


@dataclass(frozen=True)
class Stat:
    mean_ns: Optional[float]
    sd_ns: Optional[float]


def pytest_stats(filename: str) -> dict[str, Stat]:
    path = RESULTS / filename
    if not path.exists():
        return {}
    out: dict[str, Stat] = {}
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return {}
    for entry in payload.get("benchmarks", []):
        stats = entry.get("stats", {})
        mean = stats.get("mean")
        sd = stats.get("stddev")
        out[entry.get("name", "")] = Stat(
            mean_ns=mean * 1e9 if mean is not None else None,
            sd_ns=sd * 1e9 if sd is not None else None,
        )
    return out


def tinybench_stats(filename: str) -> dict[str, Stat]:
    path = RESULTS / filename
    if not path.exists():
        return {}
    out: dict[str, Stat] = {}
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return {}
    for task in payload.get("tasks", []):
        out[task.get("name", "")] = Stat(
            mean_ns=task.get("mean_ns"), sd_ns=task.get("stddev_ns")
        )
    return out


# ---------------------------------------------------------------------------
# Section rendering
# ---------------------------------------------------------------------------


def render_hot_path() -> str:
    rows: list[tuple[str, Optional[float], Optional[float]]] = [
        (
            "Parse Polymarket book (full orderbook)",
            criterion_mean("parse_polymarket_book", "openpx"),
            criterion_mean("parse_polymarket_book", "polymarket_sdk"),
        ),
    ]
    table = ["| Operation | OpenPX | polymarket_client_sdk_v2 | Speedup |", "|---|---:|---:|---:|"]
    have_data = False
    for op, ox, sdk in rows:
        if ox is not None or sdk is not None:
            have_data = True
        table.append(f"| {op} | {fmt_time(ox)} | {fmt_time(sdk)} | {fmt_speedup(ox, sdk)} |")

    # OpenPX-only "throughput showcase" rows — these are absolute numbers
    # that demonstrate what the unified API gives you, without an SDK
    # counterpart. They live under the same table because they're still
    # Rust hot-path benches.
    throughput: list[tuple[str, Optional[float]]] = [
        ("Apply 1024 book updates (re-sort each)", criterion_mean("apply_book_updates", "openpx_1024_msgs")),
        ("Orderbook `best_bid`", criterion_mean("orderbook_ops", "openpx_best_bid")),
        ("Orderbook `spread`", criterion_mean("orderbook_ops", "openpx_spread")),
        ("Orderbook `mid_price`", criterion_mean("orderbook_ops", "openpx_mid_price")),
    ]
    for op, ox in throughput:
        if ox is not None:
            have_data = True
        table.append(f"| {op} | {fmt_time(ox)} | _n/a_ | _n/a_ |")

    if not have_data:
        return ""
    return (
        "### Rust core — hot path\n\n"
        "_Pure CPU, no network. Same byte buffer in, same op. Kalshi has "
        "no upstream Rust SDK; OpenPX is the only Rust client for it._\n\n"
        + "\n".join(table)
    )


def render_python() -> str:
    poly = pytest_stats("python_polymarket.json")
    kalshi = pytest_stats("python_kalshi.json")
    if not poly and not kalshi:
        return ""

    sections: list[str] = []

    if poly:
        sections.append(_method_table(
            "py-clob-client",
            [
                ("fetch_markets", "test_openpx_fetch_markets", "test_pyclob_fetch_markets"),
                ("fetch_market", "test_openpx_fetch_market", "test_pyclob_fetch_market"),
                ("fetch_orderbook", "test_openpx_fetch_orderbook", "test_pyclob_fetch_orderbook"),
                ("fetch_trades", "test_openpx_fetch_trades", "test_pyclob_fetch_trades"),
            ],
            poly,
        ))

    if kalshi:
        sections.append(_method_table(
            "kalshi-python",
            [
                ("fetch_markets", "test_openpx_fetch_markets", "test_kalshi_python_fetch_markets"),
                ("fetch_market", "test_openpx_fetch_market", "test_kalshi_python_fetch_market"),
                ("fetch_orderbook", "test_openpx_fetch_orderbook", "test_kalshi_python_fetch_orderbook"),
                ("fetch_trades", "test_openpx_fetch_trades", "test_kalshi_python_fetch_trades"),
            ],
            kalshi,
        ))

    return (
        "### Python SDK — end-to-end REST\n\n"
        "_20 iterations × 100 ms gap, same machine, same minute. "
        "Live unauthenticated endpoints — no credentials required to reproduce._\n\n"
        + "\n\n".join(s for s in sections if s)
    )


def render_typescript() -> str:
    poly = tinybench_stats("typescript_polymarket.json")
    kalshi = tinybench_stats("typescript_kalshi.json")
    if not poly and not kalshi:
        return ""

    sections: list[str] = []

    if poly:
        sections.append(_method_table(
            "@polymarket/clob-client",
            [
                ("fetch_markets", "openpx::polymarket::fetch_markets", "polymarket-clob-client::fetch_markets"),
                ("fetch_market", "openpx::polymarket::fetch_market", "polymarket-clob-client::fetch_market"),
                ("fetch_orderbook", "openpx::polymarket::fetch_orderbook", "polymarket-clob-client::fetch_orderbook"),
                ("fetch_trades", "openpx::polymarket::fetch_trades", "polymarket-clob-client::fetch_trades"),
            ],
            poly,
        ))

    if kalshi:
        # Probe for whichever Kalshi SDK name happens to be installed.
        sdk_name = None
        for cand in ("kalshi-typescript", "kalshi-typescript-sdk", "kalshi-ts", "@kalshi/sdk"):
            if any(k.startswith(f"{cand}::") for k in kalshi):
                sdk_name = cand
                break
        if sdk_name:
            sections.append(_method_table(
                sdk_name,
                [
                    ("fetch_markets", "openpx::kalshi::fetch_markets", f"{sdk_name}::kalshi::fetch_markets"),
                    ("fetch_market", "openpx::kalshi::fetch_market", f"{sdk_name}::kalshi::fetch_market"),
                    ("fetch_orderbook", "openpx::kalshi::fetch_orderbook", f"{sdk_name}::kalshi::fetch_orderbook"),
                    ("fetch_trades", "openpx::kalshi::fetch_trades", f"{sdk_name}::kalshi::fetch_trades"),
                ],
                kalshi,
            ))

    return (
        "### TypeScript SDK — end-to-end REST\n\n"
        "_20 iterations × 100 ms gap, same machine, same minute. "
        "Live unauthenticated endpoints._\n\n"
        + "\n\n".join(s for s in sections if s)
    )


def _method_table(sdk_label: str, rows: list[tuple[str, str, str]], data: dict[str, Stat]) -> str:
    header = [
        f"**vs `{sdk_label}`**",
        "",
        f"| Method | OpenPX | {sdk_label} | Speedup |",
        "|---|---:|---:|---:|",
    ]
    have_data = False
    for op, ox_key, sdk_key in rows:
        ox = data.get(ox_key, Stat(None, None))
        sdk = data.get(sdk_key, Stat(None, None))
        if ox.mean_ns is not None or sdk.mean_ns is not None:
            have_data = True
        header.append(
            f"| `{op}` | {fmt_pm(ox.mean_ns, ox.sd_ns)} | "
            f"{fmt_pm(sdk.mean_ns, sdk.sd_ns)} | "
            f"{fmt_speedup(ox.mean_ns, sdk.mean_ns)} |"
        )
    return "\n".join(header) if have_data else ""


# ---------------------------------------------------------------------------
# Splice
# ---------------------------------------------------------------------------


def build_block() -> str:
    today = _dt.date.today().isoformat()
    sections = [render_hot_path(), render_python(), render_typescript()]
    sections = [s for s in sections if s]

    if not sections:
        body = (
            "_Comparative benchmarks pending — run `just bench-compare` "
            "to populate this section. See "
            "[`benches/comparative/README.md`](benches/comparative/README.md) "
            "for methodology._"
        )
    else:
        body = "\n\n".join(sections)

    return "\n".join(
        [
            START,
            "## Performance",
            "",
            "OpenPX vs the official native SDKs, head-to-head against "
            "live 5/15-min BTC markets. Real bytes, no credentials. "
            "Two angles: hot-path CPU benchmarks (where OpenPX wins by "
            "design) and end-to-end REST methods (what users actually feel).",
            "",
            body,
            "",
            (
                f"<sub>Last updated: {today} · Methodology: "
                "[benches/comparative/README.md](benches/comparative/README.md) · "
                "Reproduce: `just bench-compare`</sub>"
            ),
            END,
        ]
    )


def main() -> int:
    text = README.read_text()
    new = build_block()
    pattern = re.compile(re.escape(START) + r".*?" + re.escape(END), re.DOTALL)
    if pattern.search(text):
        updated = pattern.sub(lambda _m: new, text, count=1)
    else:
        # No block yet — insert near the top, right after the first `---`.
        marker = "\n---\n"
        idx = text.find(marker)
        if idx < 0:
            sys.stderr.write(
                "error: README.md has no BENCH block and no `---` separator to anchor it.\n"
            )
            return 1
        insert_at = idx + len(marker)
        updated = text[:insert_at] + "\n" + new + "\n" + text[insert_at:]

    if updated != text:
        README.write_text(updated)
        print(f"updated {README.relative_to(ROOT)}")
    else:
        print(f"{README.relative_to(ROOT)} already up to date")
    return 0


if __name__ == "__main__":
    sys.exit(main())
