"""WebSocket hot-path bench (Python): OpenPX-equivalent vs DIY.

The official Polymarket and Kalshi Python SDKs don't ship WebSocket
support. A user who wants real-time data has to roll their own client:
connect, subscribe, parse JSON, maintain orderbook state. This bench
measures that DIY cost on real captured frames.

It's not strictly head-to-head against an "SDK" — there is no SDK.
The comparison is "OpenPX gives you this for free; here's what
you'd pay to write the same thing yourself in pure Python."

The OpenPX-side number is sourced from the Rust hot-path bench
(`cargo bench -p px-bench-comparative --bench hot_path` group
`ws_decode_apply::openpx`) — OpenPX's WS hot path runs in Rust under
the FFI, so the Python user pays roughly that cost plus a few µs of
PyO3 marshalling per message.

Run: `pytest benches/comparative/python/bench_ws_diy.py --benchmark-only`
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

FIXTURE = (
    Path(__file__).resolve().parents[1] / "fixtures" / "polymarket_ws_book.jsonl"
)


def _load_frames() -> list[bytes]:
    """Read the JSONL fixture once at module-load. Skip array-wrapped
    snapshot frames so DIY and OpenPX bench the same frame set."""
    if not FIXTURE.exists():
        pytest.skip(f"missing fixture {FIXTURE}; run capture_bench_fixtures.py")
    out: list[bytes] = []
    with FIXTURE.open("rb") as fh:
        for line in fh:
            line = line.rstrip(b"\n")
            if not line or line[:1] == b"[":
                continue
            out.append(line)
    return out


FRAMES = _load_frames()


def _diy_decode_apply(frames: list[bytes]) -> tuple[int, int]:
    """A representative hand-rolled Python WS handler — what a user would
    write to replace OpenPX. `json.loads` + dict-per-side orderbook
    state, with delta application by event_type. Roughly 30 lines, no
    framework. Returns `(messages_processed, levels_after)`."""
    bids: dict[str, str] = {}
    asks: dict[str, str] = {}
    processed = 0
    for frame in frames:
        msg = json.loads(frame)
        etype = msg.get("event_type")
        if etype == "book":
            bids = {lvl["price"]: lvl["size"] for lvl in msg.get("bids", [])}
            asks = {lvl["price"]: lvl["size"] for lvl in msg.get("asks", [])}
        elif etype == "price_change":
            for ch in msg.get("price_changes", []):
                side = (ch.get("side") or "").upper()
                book = bids if side == "BUY" else asks
                size = ch.get("size") or "0"
                price = ch.get("price") or ""
                if not price:
                    continue
                if float(size) == 0.0:
                    book.pop(price, None)
                else:
                    book[price] = size
        # last_trade_price has no book impact in this minimal handler
        processed += 1
    return processed, len(bids) + len(asks)


def test_diy_ws_decode_apply(benchmark):
    """DIY Python WS handler: json.loads + dict state machine.
    20 rounds, deterministic on captured bytes — no network involved.
    The README cites this directly."""
    if not FRAMES:
        pytest.skip("no frames")
    result = benchmark.pedantic(
        _diy_decode_apply,
        args=(FRAMES,),
        iterations=1,
        rounds=20,
    )
    assert result[0] > 0
