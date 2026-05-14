#!/usr/bin/env python3
"""Render comparative-benchmark JSON into the README's BENCH block.

Three sections:

  1. Rust hot path:
     - `ws_decode_apply::openpx` vs `polymarket_sdk` (head-to-head)
     - `apply_book_updates::openpx_*` (OpenPX-only throughput)
     - `orderbook_ops::openpx_*` (OpenPX-only architectural primitives)

  2. REST fetch_orderbook (Python + TypeScript × Polymarket + Kalshi):
     OpenPX vs the official SDK on the one operation where both
     libraries hit the same upstream endpoint and return the same shape.

  3. WebSocket: feature matrix + DIY-cost numbers from the Rust
     head-to-head and the Python/TS DIY benches. None of the official
     SDKs ship WebSocket — that's the headline.

Reads:
  target/criterion/<group>/<id>/new/estimates.json   (Rust)
  benches/comparative/results/python_polymarket.json (pytest-benchmark)
  benches/comparative/results/python_kalshi.json     (pytest-benchmark)
  benches/comparative/results/python_ws_diy.json     (pytest-benchmark)
  benches/comparative/results/typescript_polymarket.json (tinybench)
  benches/comparative/results/typescript_kalshi.json     (tinybench)
  benches/comparative/results/typescript_ws_diy.json     (tinybench)

Splices between `<!-- BENCH:START -->` / `<!-- BENCH:END -->`. Missing
sources render as `—` rather than failing.

Run: `python3 tools/render_bench_readme.py`
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
    if ns < 1:
        # sub-nanosecond, rare but happens for trivially-inlined ops
        return f"{ns:.2f} ns"
    if ns < 1_000:
        return f"{ns:.1f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    return f"{ns / 1e9:.2f} s"


def fmt_pm(mean_ns: Optional[float], sd_ns: Optional[float]) -> str:
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
    """Rust hot-path section: WS head-to-head + OpenPX-only architectural ops."""
    ws_openpx = criterion_mean("ws_decode_apply", "openpx")
    ws_sdk = criterion_mean("ws_decode_apply", "polymarket_sdk")
    apply_1024 = criterion_mean("apply_book_updates", "openpx_1024_msgs")
    best_bid = criterion_mean("orderbook_ops", "openpx_best_bid")
    spread = criterion_mean("orderbook_ops", "openpx_spread")
    mid_price = criterion_mean("orderbook_ops", "openpx_mid_price")

    have_any = any(
        v is not None for v in (ws_openpx, apply_1024, best_bid, spread, mid_price)
    )
    if not have_any:
        return ""

    lines = [
        "### Rust hot path",
        "",
        "_Pure CPU, no network. Same byte buffer in, same op._",
        "",
        "**Head-to-head:** decode + apply 999 real Polymarket WebSocket frames "
        "(book + price_change + last_trade_price) captured from a live 5-min "
        "BTC market.",
        "",
        "| Decode + apply 999 WS frames | OpenPX | polymarket_client_sdk_v2 | Speedup |",
        "|---|---:|---:|---:|",
        f"| Polymarket book channel | {fmt_time(ws_openpx)} | {fmt_time(ws_sdk)} | {fmt_speedup(ws_openpx, ws_sdk)} |",
        "",
        "**OpenPX-only — architectural primitives the SDKs don't expose:**",
        "",
        "| Operation | OpenPX | Note |",
        "|---|---:|---|",
        f"| Apply 1024 book updates (sustained) | {fmt_time(apply_1024)} | "
        + (f"≈ {1024 * 1e9 / apply_1024 / 1_000_000:.1f} M ops/sec" if apply_1024 else "—")
        + " |",
        f"| `Orderbook::best_bid` (sorted-vec) | {fmt_time(best_bid)} | constant-time |",
        f"| `Orderbook::spread` | {fmt_time(spread)} | constant-time |",
        f"| `Orderbook::mid_price` | {fmt_time(mid_price)} | constant-time |",
    ]
    return "\n".join(lines)


def render_rest() -> str:
    """fetch_orderbook head-to-head: the one fair REST comparison."""
    py_poly = pytest_stats("python_polymarket.json")
    py_kalshi = pytest_stats("python_kalshi.json")
    ts_poly = tinybench_stats("typescript_polymarket.json")
    ts_kalshi = tinybench_stats("typescript_kalshi.json")

    rows: list[tuple[str, str, str, Stat, Stat]] = []

    if py_poly:
        rows.append((
            "Python", "Polymarket", "py-clob-client",
            py_poly.get("test_openpx_fetch_orderbook", Stat(None, None)),
            py_poly.get("test_pyclob_fetch_orderbook", Stat(None, None)),
        ))
    if py_kalshi:
        rows.append((
            "Python", "Kalshi", "kalshi-python",
            py_kalshi.get("test_openpx_fetch_orderbook", Stat(None, None)),
            py_kalshi.get("test_kalshi_python_fetch_orderbook", Stat(None, None)),
        ))
    if ts_poly:
        rows.append((
            "TypeScript", "Polymarket", "@polymarket/clob-client",
            ts_poly.get("openpx::polymarket::fetch_orderbook", Stat(None, None)),
            ts_poly.get("polymarket-clob-client::fetch_orderbook", Stat(None, None)),
        ))
    if ts_kalshi:
        # Kalshi TS SDK package name varies — find whichever ran.
        sdk_label = None
        sdk_stat = Stat(None, None)
        for cand in ("kalshi-typescript", "kalshi-typescript-sdk", "kalshi-ts", "@kalshi/sdk"):
            k = f"{cand}::kalshi::fetch_orderbook"
            if k in ts_kalshi:
                sdk_label = cand
                sdk_stat = ts_kalshi[k]
                break
        if sdk_label:
            rows.append((
                "TypeScript", "Kalshi", sdk_label,
                ts_kalshi.get("openpx::kalshi::fetch_orderbook", Stat(None, None)),
                sdk_stat,
            ))

    if not rows:
        return ""

    lines = [
        "### REST `fetch_orderbook` — head-to-head",
        "",
        "_20 iterations × 100 ms gap, same machine, same minute. Live "
        "unauthenticated endpoints — both libraries hit the same upstream URL "
        "and return the same shape, so the ratio reflects real client-side "
        "overhead._",
        "",
        "| Lang | Exchange | OpenPX | Official SDK | Speedup |",
        "|---|---|---:|---:|---:|",
    ]
    for lang, ex, sdk, ox, other in rows:
        lines.append(
            f"| {lang} | {ex} | {fmt_pm(ox.mean_ns, ox.sd_ns)} | "
            f"{sdk} {fmt_pm(other.mean_ns, other.sd_ns)} | "
            f"{fmt_speedup(ox.mean_ns, other.mean_ns)} |"
        )
    return "\n".join(lines)


def render_websocket() -> str:
    """WebSocket section: feature matrix + DIY cost from Python/TS benches."""
    py_diy = pytest_stats("python_ws_diy.json")
    ts_diy = tinybench_stats("typescript_ws_diy.json")
    py_diy_stat = py_diy.get("test_diy_ws_decode_apply", Stat(None, None))
    ts_diy_stat = ts_diy.get("diy::polymarket::ws_decode_apply", Stat(None, None))
    rust_openpx = criterion_mean("ws_decode_apply", "openpx")

    lines = [
        "### WebSocket — typed, unified, OpenPX-exclusive",
        "",
        "_None of the official Python or TypeScript SDKs ship WebSocket "
        "support. Users replicate it themselves: connect, subscribe, "
        "parse JSON, maintain orderbook state, handle reconnects/auth. "
        "OpenPX gives you `exchange.websocket().orderbook(asset_id)` "
        "returning typed orderbook deltas, same shape across both exchanges._",
        "",
        "| Feature | OpenPX | py-clob-client | @polymarket/clob-client | kalshi-python | kalshi-typescript-sdk |",
        "|---|:---:|:---:|:---:|:---:|:---:|",
        "| WebSocket orderbook | ✅ Typed, unified | ❌ Not supported | ❌ Not supported | ❌ Not supported | ❌ Not supported |",
        "| WebSocket trades/fills | ✅ Typed, unified | ❌ | ❌ | ❌ | ❌ |",
        "| Reconnect + resync | ✅ | DIY | DIY | DIY | DIY |",
        "| Same API across exchanges | ✅ | n/a | n/a | n/a | n/a |",
    ]

    if any(s.mean_ns is not None for s in (py_diy_stat, ts_diy_stat)) or rust_openpx:
        lines.extend([
            "",
            "**DIY decode + apply cost** — what users pay rolling their own. "
            "Same 999 captured Polymarket WS frames, replayed deterministically.",
            "",
            "| Path | Time for 999 frames | per-message | Note |",
            "|---|---:|---:|---|",
        ])
        if rust_openpx is not None:
            lines.append(
                f"| **OpenPX (Rust hot path)** | {fmt_time(rust_openpx)} | "
                f"{fmt_time(rust_openpx / 999)} | what runs under the FFI for Python/TS users |"
            )
        if py_diy_stat.mean_ns is not None:
            lines.append(
                f"| DIY Python (`json.loads` + `dict`) | {fmt_pm(py_diy_stat.mean_ns, py_diy_stat.sd_ns)} | "
                f"{fmt_time(py_diy_stat.mean_ns / 999) if py_diy_stat.mean_ns else '—'} | hand-rolled, ~30 lines |"
            )
        if ts_diy_stat.mean_ns is not None:
            lines.append(
                f"| DIY TypeScript (`JSON.parse` + `Map`) | {fmt_pm(ts_diy_stat.mean_ns, ts_diy_stat.sd_ns)} | "
                f"{fmt_time(ts_diy_stat.mean_ns / 999) if ts_diy_stat.mean_ns else '—'} | hand-rolled, ~30 lines |"
            )

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Splice
# ---------------------------------------------------------------------------


def build_block() -> str:
    today = _dt.date.today().isoformat()
    sections = [render_hot_path(), render_rest(), render_websocket()]
    sections = [s for s in sections if s]

    if not sections:
        body = (
            "_Comparative benchmarks pending — run `just bench-compare` to "
            "populate this section. See "
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
            "OpenPX vs the official native SDKs on real, unauthenticated 5/15-min "
            "BTC markets. Three angles: Rust hot path (where OpenPX wins by "
            "design), REST `fetch_orderbook` (the operation both clients run "
            "identically), and WebSocket (which the SDKs don't ship at all).",
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
