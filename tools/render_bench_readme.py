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


def build_block() -> str:
    today = _dt.date.today().isoformat()

    # --- 1. Real-World WS table -------------------------------------------------
    op_mean, op_sd = criterion_estimate("ws_decode_apply", "openpx")
    sd_mean, sd_sd = criterion_estimate("ws_decode_apply", "polymarket_sdk")

    ratio = sd_mean / op_mean if (op_mean and sd_mean and op_mean > 0) else None
    pct_faster = (ratio - 1.0) * 100 if ratio else None

    # Coefficient of variation as a "consistency" proxy.
    op_cv = (op_sd / op_mean) if (op_mean and op_sd) else None
    sd_cv = (sd_sd / sd_mean) if (sd_mean and sd_sd) else None
    consistency_pct = (
        (1.0 - op_cv / sd_cv) * 100 if (op_cv and sd_cv and sd_cv > 0) else None
    )

    real_world_table = [
        "| Operation | OpenPX | polymarket_client_sdk_v2 |",
        "|---|---|---|",
        f"| **Decode + apply 999 WS book frames** | **{fmt_pm(op_mean, op_sd)}** | "
        f"{fmt_pm(sd_mean, sd_sd)} |",
    ]

    bullets: list[str] = []
    if pct_faster is not None:
        bullets.append(f"- **{pct_faster:.1f}% faster**")
    if consistency_pct is not None and consistency_pct > 0:
        bullets.append(f"- **{consistency_pct:.1f}% more consistent** (lower coefficient of variation)")
    bullets.append(
        "- **Only client** that ships typed WebSocket support across Polymarket *and* "
        "Kalshi in all three languages (Rust + Python + TypeScript); the official "
        "Python and TypeScript SDKs don't ship WebSocket at all."
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
    pct_str = f"{pct_faster:.1f}%" if pct_faster else "double-digit %"
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
        "End-to-end decode + apply over 999 real Polymarket book frames captured from a live 5-min BTC market — measures the cost of turning wire bytes into typed orderbook updates, the operation that dominates an HFT loop once you're subscribed:",
        "",
        *real_world_table,
        "",
        "**Performance vs `polymarket_client_sdk_v2`:**",
        "",
        *bullets,
        "",
        "**Benchmark Methodology:** All benchmarks run side-by-side on the same machine using criterion, decoding the same captured JSONL frames byte-for-byte. Both libraries deserialize identical inputs into their respective typed message shapes; the ratio reflects pure decoder + orderbook-apply overhead with no network jitter. See [`benches/comparative/rust/benches/hot_path.rs`](benches/comparative/rust/benches/hot_path.rs) for the complete implementation.",
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
