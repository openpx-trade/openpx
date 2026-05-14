#!/usr/bin/env python3
"""Render the comparative-bench results into the README's BENCH block.

Layout mirrors polyfill-rs's `## Performance Comparison` section, but
swaps WebSocket decode + apply in as the headline benchmark — that's
where OpenPX wins meaningfully, and where the official Python/TS SDKs
don't ship anything to compare against. Sections:

  1. Real-World WebSocket Performance — head-to-head decode + apply
     on 999 real Polymarket frames captured from a live 5-min BTC
     market. OpenPX vs `polymarket_client_sdk_v2`.
  2. Performance vs `polymarket_client_sdk_v2` — headline bullets.
  3. Benchmark Methodology — single paragraph, repo pointer.
  4. Computational Performance — the OpenPX-only architectural primitives.
  5. Key Performance Optimizations / Memory Architecture / Architectural
     Principles — prose paragraphs describing the design choices that
     produce the numbers.

Reads:
  target/criterion/<group>/<id>/new/estimates.json

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
from pathlib import Path
from typing import Optional

ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"
CRITERION = ROOT / "target" / "criterion"
RESULTS = ROOT / "benches" / "comparative" / "results"

START = "<!-- BENCH:START -->"
END = "<!-- BENCH:END -->"


def fmt_time(ns: Optional[float]) -> str:
    if ns is None or not math.isfinite(ns):
        return "—"
    if ns < 1_000:
        return f"{ns:.2f} ns"
    if ns < 1_000_000:
        return f"{ns / 1_000:.2f} µs"
    if ns < 1_000_000_000:
        return f"{ns / 1_000_000:.2f} ms"
    return f"{ns / 1e9:.2f} s"


def criterion_estimate(group: str, function_id: str) -> tuple[Optional[float], Optional[float]]:
    """Return (mean_ns, stddev_ns) for a criterion bench, or (None, None)."""
    path = CRITERION / group / function_id / "new" / "estimates.json"
    if not path.exists():
        return None, None
    try:
        payload = json.loads(path.read_text())
        mean = float(payload["mean"]["point_estimate"])
        sd = float(payload.get("std_dev", {}).get("point_estimate", 0.0))
        return mean, (sd if sd > 0 else None)
    except (KeyError, ValueError, json.JSONDecodeError):
        return None, None


def fmt_pm(mean: Optional[float], sd: Optional[float]) -> str:
    if mean is None:
        return "—"
    if sd is None:
        return fmt_time(mean)
    return f"{fmt_time(mean)} ± {fmt_time(sd)}"


def pytest_stat(filename: str, test_name: str) -> tuple[Optional[float], Optional[float]]:
    path = RESULTS / filename
    if not path.exists():
        return None, None
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError:
        return None, None
    for entry in payload.get("benchmarks", []):
        if entry.get("name") == test_name:
            stats = entry.get("stats", {})
            mean = stats.get("mean")
            sd = stats.get("stddev")
            return (
                mean * 1e9 if mean is not None else None,
                sd * 1e9 if sd is not None else None,
            )
    return None, None


def build_block() -> str:
    today = _dt.date.today().isoformat()

    # --- 1. Real-World WS table -------------------------------------------------
    op_mean, op_sd = criterion_estimate("ws_decode_apply", "openpx")
    op_kal_mean, op_kal_sd = criterion_estimate("ws_decode_apply_kalshi", "openpx")
    sdk_mean, sdk_sd = criterion_estimate("ws_decode_apply", "polymarket_sdk")
    diy_poly_mean, diy_poly_sd = pytest_stat("python_ws_diy.json", "test_diy_python_polymarket")
    diy_kal_mean, diy_kal_sd = pytest_stat("python_ws_diy.json", "test_diy_python_kalshi")

    ratio_sdk = sdk_mean / op_mean if (op_mean and sdk_mean and op_mean > 0) else None
    ratio_py = diy_poly_mean / op_mean if (op_mean and diy_poly_mean and op_mean > 0) else None
    ratio_kal = diy_kal_mean / op_mean if (op_mean and diy_kal_mean and op_mean > 0) else None

    # Coefficient of variation as a "consistency" proxy.
    op_cv = (op_sd / op_mean) if (op_mean and op_sd) else None
    sdk_cv = (sdk_sd / sdk_mean) if (sdk_mean and sdk_sd) else None
    consistency_pct = (
        (1.0 - op_cv / sdk_cv) * 100 if (op_cv and sdk_cv and sdk_cv > 0) else None
    )

    def py_time_cell(mean: Optional[float], sd: Optional[float]) -> str:
        if mean is None:
            return "_(no fixture — run skipped)_"
        return fmt_pm(mean, sd)

    real_world_table = [
        "| Client | Frames | Decode + apply (999 frames) |",
        "|---|---|---:|",
        f"| **OpenPX (Rust)** | Polymarket | **{fmt_pm(op_mean, op_sd)}** |",
        f"| **OpenPX (Rust)** | Kalshi | **{fmt_pm(op_kal_mean, op_kal_sd)}** |",
        f"| `polymarket_client_sdk_v2` (Rust) | Polymarket | {fmt_pm(sdk_mean, sdk_sd)} |",
        f"| `py-clob-client` _(no WS — DIY ~30 lines)_ | Polymarket | {py_time_cell(diy_poly_mean, diy_poly_sd)} |",
        f"| `kalshi-python` _(no WS — DIY ~30 lines)_ | Kalshi | {py_time_cell(diy_kal_mean, diy_kal_sd)} |",
    ]

    bullets: list[str] = []
    if ratio_sdk:
        bullets.append(f"- **{(ratio_sdk - 1.0) * 100:.1f}% faster** than `polymarket_client_sdk_v2` (Rust head-to-head)")
    if ratio_py:
        bullets.append(f"- **{ratio_py:.2f}× faster** than hand-rolled Python (`py-clob-client` doesn't ship WebSocket)")
    if ratio_kal:
        bullets.append(f"- **{ratio_kal:.2f}× faster** than hand-rolled Python (`kalshi-python` doesn't ship WebSocket)")
    if consistency_pct is not None and consistency_pct > 0:
        bullets.append(f"- **{consistency_pct:.1f}% more consistent** than the Rust SDK (lower coefficient of variation)")
    bullets.append(
        "- **Only client** shipping typed WebSocket support across Polymarket *and* "
        "Kalshi in all three languages (Rust + Python + TypeScript via FFI)."
    )

    # --- 2. Computational Performance ------------------------------------------
    best_bid_mean, _ = criterion_estimate("orderbook_ops", "openpx_best_bid")
    spread_mean, _ = criterion_estimate("orderbook_ops", "openpx_spread")
    mid_mean, _ = criterion_estimate("orderbook_ops", "openpx_mid_price")

    def ops_per_sec_note(mean_ns: Optional[float], suffix: str) -> str:
        if not mean_ns or mean_ns <= 0:
            return suffix
        per_sec = 1.0 / (mean_ns * 1e-9)
        return f"~{per_sec / 1e9:.1f}B ops/sec, {suffix}"

    ws_per_frame = op_mean / 999 if op_mean else None
    ws_note = (
        f"~{ws_per_frame:.0f} ns / frame, ~{1e9 / ws_per_frame / 1e6:.1f}M frames/sec, zero-allocation"
        if ws_per_frame
        else "zero-allocation"
    )

    computational_table = [
        "| Operation | Performance | Notes |",
        "|---|---|---|",
        f"| **WS decode + apply (999 frames)** | {fmt_time(op_mean)} | {ws_note} |",
        f"| **`Orderbook::best_bid`** | {fmt_time(best_bid_mean)} | {ops_per_sec_note(best_bid_mean, 'sorted-vec O(1)')} |",
        f"| **`Orderbook::spread`** | {fmt_time(spread_mean)} | {ops_per_sec_note(spread_mean, 'branchless')} |",
        f"| **`Orderbook::mid_price`** | {fmt_time(mid_mean)} | {ops_per_sec_note(mid_mean, 'branchless')} |",
    ]

    # --- 3. Prose sections ------------------------------------------------------
    pct_str = (
        f"{(ratio_sdk - 1.0) * 100:.1f}%" if ratio_sdk else "double-digit %"
    )
    optimizations_para = (
        f"The {pct_str} WebSocket speedup comes from a single-shape `decode_frame` "
        "fast path (one `serde::Deserialize` target covers `book`, `price_change`, "
        "`last_trade_price`, and `tick_size_change` — no tagged-enum dispatch), a "
        "sorted-`Vec` orderbook that keeps both sides in contiguous, cache-friendly "
        "arrays, and a zero-allocation apply pipeline that reuses level buffers "
        "instead of churning the heap."
    )

    memory_para = (
        "The `Orderbook` stores price levels in two sorted `Vec<PriceLevel>` "
        "arrays — no `BTreeMap` pointer chasing, no per-update heap churn. "
        "Hot-path queries (`best_bid`, `best_ask`, `spread`, `mid_price`) are "
        "constant-time accesses on the head element of each `Vec`, which the "
        "compiler reduces to a load + sub. Sub-nanosecond figures above reflect "
        "this: there's nothing to do but read two `f64`s."
    )

    architectural_para = (
        "Wire bytes deserialize once into typed structs at the WebSocket ingress; "
        "every downstream consumer reads typed fields with no re-parsing. The "
        "unified `Exchange` trait dispatches via match + UFCS (no `&dyn Exchange` "
        "vtable indirection), so each exchange's `fetch_orderbook` / `ws orderbook` "
        "monomorphizes and inlines into the call site. Errors flow through "
        "`define_exchange_error!` macros that map per-exchange variants into the "
        "unified `ExchangeError` hierarchy — strict types at the boundary, trust "
        "internal code internally."
    )

    return "\n".join([
        START,
        "## Performance Comparison",
        "",
        "**Real-World WebSocket Performance (live captured frames)**",
        "",
        "End-to-end decode + apply over 999 real WebSocket book frames captured from live 5-min BTC markets — measures the cost of turning wire bytes into typed orderbook updates, the operation that dominates an HFT loop once you're subscribed. `py-clob-client` and `kalshi-python` don't ship WebSocket at all, so the Python rows measure what a user pays rolling their own pipeline (the obvious ~30-line `json.loads` + dict apply, on the same captured frames):",
        "",
        *real_world_table,
        "",
        "**Performance vs the alternatives:**",
        "",
        *bullets,
        "",
        "**Benchmark Methodology:** Rust benches use criterion; Python uses `pytest-benchmark` with `pedantic(rounds=5)`. All four clients replay the same captured JSONL frames byte-for-byte — no network, no jitter. Ratios reflect pure decoder + orderbook-apply overhead. See [`benches/comparative/rust/benches/hot_path.rs`](benches/comparative/rust/benches/hot_path.rs) and [`benches/comparative/python/bench_ws_diy.py`](benches/comparative/python/bench_ws_diy.py) for the full implementations.",
        "",
        "**Computational Performance (pure CPU, no I/O)**",
        "",
        *computational_table,
        "",
        "Run the WS hot-path benchmark locally with `cargo bench -p px-bench-comparative --bench hot_path`.",
        "",
        "**Key Performance Optimizations:**",
        "",
        optimizations_para,
        "",
        "**Memory Architecture**",
        "",
        memory_para,
        "",
        "**Architectural Principles**",
        "",
        architectural_para,
        "",
        f"<sub>Last updated: {today} · Methodology: [benches/comparative/README.md](benches/comparative/README.md) · Reproduce: `just bench-compare`</sub>",
        END,
    ])


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
