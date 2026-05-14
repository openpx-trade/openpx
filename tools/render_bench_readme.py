#!/usr/bin/env python3
"""Render comparative-benchmark JSON into the README's BENCH block.

Polyfill-rs–shaped layout. Two operations, each with a 6-row table
(Rust + Python + TypeScript × Polymarket + Kalshi), followed by
headline bullets and a methodology paragraph.

Reads:
  benches/comparative/results/rust_polymarket.json       (custom bin)
  benches/comparative/results/rust_kalshi.json           (custom bin)
  benches/comparative/results/python_polymarket.json     (pytest-benchmark)
  benches/comparative/results/python_kalshi.json         (pytest-benchmark)
  benches/comparative/results/typescript_polymarket.json (tinybench)
  benches/comparative/results/typescript_kalshi.json     (tinybench)
  target/criterion/ws_decode_apply/{openpx,polymarket_sdk}/new/estimates.json

Splices between `<!-- BENCH:START -->` / `<!-- BENCH:END -->` in
README.md. Missing sources render as `—`.

Run: `python3 tools/render_bench_readme.py` (or `just bench-compare`).
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


def fmt_time(ns: Optional[float]) -> str:
    if ns is None or not math.isfinite(ns):
        return "—"
    if ns < 1_000:
        return f"{ns:.1f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    return f"{ns / 1e9:.2f} s"


def fmt_pm(mean_ns: Optional[float], sd_ns: Optional[float], bold: bool = False) -> str:
    if mean_ns is None or not math.isfinite(mean_ns):
        return "—"
    if sd_ns is None or not math.isfinite(sd_ns) or sd_ns <= 0:
        s = fmt_time(mean_ns)
    else:
        s = f"{fmt_time(mean_ns)} ± {fmt_time(sd_ns)}"
    return f"**{s}**" if bold else s


def fmt_speedup(openpx: Optional[float], other: Optional[float]) -> str:
    if not openpx or not other or openpx <= 0:
        return "—"
    ratio = other / openpx
    return f"**{ratio:.2f}× faster**" if ratio >= 1.0 else f"{ratio:.2f}×"


@dataclass(frozen=True)
class Stat:
    mean_ns: Optional[float]
    sd_ns: Optional[float]


def _pytest_stats(filename: str) -> dict[str, Stat]:
    path = RESULTS / filename
    if not path.exists():
        return {}
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return {}
    out: dict[str, Stat] = {}
    for entry in payload.get("benchmarks", []):
        stats = entry.get("stats", {})
        mean = stats.get("mean")
        sd = stats.get("stddev")
        out[entry.get("name", "")] = Stat(
            mean_ns=mean * 1e9 if mean is not None else None,
            sd_ns=sd * 1e9 if sd is not None else None,
        )
    return out


def _tinybench_stats(filename: str) -> dict[str, Stat]:
    """Used for tinybench (TS) and our custom Rust bin — same JSON shape."""
    path = RESULTS / filename
    if not path.exists():
        return {}
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return {}
    out: dict[str, Stat] = {}
    for task in payload.get("tasks", []):
        out[task.get("name", "")] = Stat(
            mean_ns=task.get("mean_ns"), sd_ns=task.get("stddev_ns")
        )
    return out


def _criterion_mean(group: str, function_id: str) -> Optional[float]:
    path = CRITERION / group / function_id / "new" / "estimates.json"
    if not path.exists():
        return None
    try:
        return float(json.loads(path.read_text())["mean"]["point_estimate"])
    except (KeyError, ValueError, json.JSONDecodeError):
        return None


@dataclass(frozen=True)
class Row:
    lang: str
    exchange: str
    openpx: Stat
    sdk_name: str  # display name; "—" if no official SDK
    sdk: Stat


def _rest_rows() -> list[Row]:
    rust_poly = _tinybench_stats("rust_polymarket.json")
    rust_kal = _tinybench_stats("rust_kalshi.json")
    py_poly = _pytest_stats("python_polymarket.json")
    py_kal = _pytest_stats("python_kalshi.json")
    ts_poly = _tinybench_stats("typescript_polymarket.json")
    ts_kal = _tinybench_stats("typescript_kalshi.json")

    def pick(d: dict[str, Stat], key: str) -> Stat:
        return d.get(key, Stat(None, None))

    # TypeScript Kalshi SDK package name varies — match whichever ran.
    ts_kal_sdk = "@kalshi/typescript-sdk"
    ts_kal_stat = Stat(None, None)
    for cand in (
        "@kalshi/typescript-sdk",
        "kalshi-typescript",
        "kalshi-typescript-sdk",
        "kalshi-ts",
        "@kalshi/sdk",
    ):
        k = f"{cand}::kalshi::fetch_orderbook"
        if k in ts_kal:
            ts_kal_sdk = cand
            ts_kal_stat = ts_kal[k]
            break

    return [
        Row(
            "Rust", "Polymarket",
            pick(rust_poly, "openpx::polymarket::fetch_orderbook"),
            "polymarket_client_sdk_v2",
            pick(rust_poly, "polymarket_client_sdk_v2::polymarket::fetch_orderbook"),
        ),
        Row(
            "Rust", "Kalshi",
            pick(rust_kal, "openpx::kalshi::fetch_orderbook"),
            "—",
            Stat(None, None),
        ),
        Row(
            "Python", "Polymarket",
            pick(py_poly, "test_openpx_fetch_orderbook"),
            "py-clob-client",
            pick(py_poly, "test_pyclob_fetch_orderbook"),
        ),
        Row(
            "Python", "Kalshi",
            pick(py_kal, "test_openpx_fetch_orderbook"),
            "kalshi-python",
            pick(py_kal, "test_kalshi_python_fetch_orderbook"),
        ),
        Row(
            "TypeScript", "Polymarket",
            pick(ts_poly, "openpx::polymarket::fetch_orderbook"),
            "@polymarket/clob-client",
            pick(ts_poly, "polymarket-clob-client::fetch_orderbook"),
        ),
        Row(
            "TypeScript", "Kalshi",
            pick(ts_kal, "openpx::kalshi::fetch_orderbook"),
            ts_kal_sdk,
            ts_kal_stat,
        ),
    ]


def _render_rest_table(rows: list[Row]) -> str:
    lines = [
        "| Lang | Exchange | OpenPX | Official SDK | Speedup |",
        "|---|---|---:|---|---:|",
    ]
    for r in rows:
        if r.sdk_name == "—":
            sdk_cell = "_no official Rust SDK_"
        elif r.sdk.mean_ns is None:
            sdk_cell = f"{r.sdk_name} _(not installed)_"
        else:
            sdk_cell = f"{r.sdk_name} {fmt_pm(r.sdk.mean_ns, r.sdk.sd_ns)}"
        lines.append(
            f"| {r.lang} | {r.exchange} | "
            f"{fmt_pm(r.openpx.mean_ns, r.openpx.sd_ns, bold=True)} | "
            f"{sdk_cell} | "
            f"{fmt_speedup(r.openpx.mean_ns, r.sdk.mean_ns)} |"
        )
    return "\n".join(lines)


def _rest_bullets(rows: list[Row]) -> list[str]:
    bullets: list[str] = []
    for r in rows:
        if r.openpx.mean_ns is None or r.sdk.mean_ns is None or r.openpx.mean_ns <= 0:
            continue
        ratio = r.sdk.mean_ns / r.openpx.mean_ns
        if ratio >= 1.05:
            bullets.append(
                f"- **{ratio:.2f}× faster** than `{r.sdk_name}` ({r.lang} · {r.exchange})"
            )
        elif ratio >= 0.95:
            bullets.append(
                f"- **on par** with `{r.sdk_name}` ({r.lang} · {r.exchange}, ratio {ratio:.2f}×)"
            )
    return bullets


def _render_ws_table() -> tuple[str, Optional[float], Optional[float]]:
    ws_openpx = _criterion_mean("ws_decode_apply", "openpx")
    ws_sdk = _criterion_mean("ws_decode_apply", "polymarket_sdk")

    rust_poly_openpx = fmt_time(ws_openpx)
    rust_poly_sdk = (
        f"polymarket_client_sdk_v2 {fmt_time(ws_sdk)}" if ws_sdk else "polymarket_client_sdk_v2 —"
    )
    rust_poly_speedup = fmt_speedup(ws_openpx, ws_sdk)

    rows = [
        ("Rust", "Polymarket", f"**{rust_poly_openpx}**", rust_poly_sdk, rust_poly_speedup),
        ("Rust", "Kalshi", "✅ Typed deltas", "_no official Rust SDK_", "—"),
        ("Python", "Polymarket", "✅ Typed deltas (via FFI)", "`py-clob-client` _doesn't ship WS_", "—"),
        ("Python", "Kalshi", "✅ Typed deltas (via FFI)", "`kalshi-python` _doesn't ship WS_", "—"),
        ("TypeScript", "Polymarket", "✅ Typed deltas (via FFI)", "`@polymarket/clob-client` _doesn't ship WS_", "—"),
        ("TypeScript", "Kalshi", "✅ Typed deltas (via FFI)", "`@kalshi/typescript-sdk` _doesn't ship WS_", "—"),
    ]
    lines = [
        "| Lang | Exchange | OpenPX | Official SDK | Speedup |",
        "|---|---|---:|---|---:|",
    ]
    for lang, ex, op, sdk, sp in rows:
        lines.append(f"| {lang} | {ex} | {op} | {sdk} | {sp} |")
    return "\n".join(lines), ws_openpx, ws_sdk


def build_block() -> str:
    today = _dt.date.today().isoformat()
    rest_rows = _rest_rows()
    rest_table = _render_rest_table(rest_rows)
    rest_bullets = _rest_bullets(rest_rows)
    ws_table, ws_openpx, ws_sdk = _render_ws_table()

    ws_bullets = ["- **Only client** that ships typed WebSocket support across Polymarket *and* Kalshi in all three languages."]
    if ws_openpx and ws_sdk:
        ratio = ws_sdk / ws_openpx
        ws_bullets.append(
            f"- Rust hot path decodes + applies 999 captured Polymarket book frames "
            f"**{ratio:.2f}× faster** than `polymarket_client_sdk_v2`'s WS decoder."
        )
    ws_bullets.append(
        "- Reconnect + resync, auth, and orderbook state are first-class — the official SDKs leave all of that to you."
    )

    parts: list[str] = [
        START,
        "## Performance Comparison",
        "",
        "**Real-World API Performance (with network I/O)** — `fetch_orderbook`",
        "",
        "End-to-end performance against live Polymarket and Kalshi orderbook endpoints, including network latency, JSON parsing, and decompression:",
        "",
        rest_table,
    ]
    if rest_bullets:
        parts.extend(["", "**Performance vs official SDKs:**", "", *rest_bullets])

    parts.extend([
        "",
        "**Benchmark Methodology:** All benchmarks run side-by-side on the same machine, same network, same time using 20 iterations, 100 ms delay between requests against the public `/book` (Polymarket) and `/markets/{ticker}/orderbook` (Kalshi) endpoints. Best performance achieved with HTTP keep-alive enabled. See [`benches/comparative/`](benches/comparative/README.md) for the full implementation.",
        "",
        "**WebSocket Support (real-time orderbook streams)**",
        "",
        "OpenPX gives you `exchange.websocket().orderbook(asset_id)` returning typed orderbook deltas — same shape across both exchanges. The official SDKs leave WebSocket handling to the user:",
        "",
        ws_table,
        "",
        "**WebSocket vs official SDKs:**",
        "",
        *ws_bullets,
        "",
        (
            f"<sub>Last updated: {today} · Reproduce: `just bench-compare`</sub>"
        ),
        END,
    ])
    return "\n".join(parts)


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
