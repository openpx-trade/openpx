"""DIY Python WebSocket decode — the cost of rolling your own.

`py-clob-client` and `kalshi-python` don't ship WebSocket support, so a
user wanting live orderbook updates has to write their own pipeline:
parse JSON, dispatch on event_type, maintain a `dict[str, float]`
orderbook, apply deltas. This bench replays the captured WS frames
through that DIY pipeline so the README can show what users pay for
*not* having a typed WS client (vs OpenPX's ~760 µs Rust hot path).

Same captured byte buffers per frame as the Rust bench — no network,
no jitter, just decode + apply cost.

Run:
  pytest benches/comparative/python/bench_ws_diy.py --benchmark-only \\
    --benchmark-json=benches/comparative/results/python_ws_diy.json -q
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

FIXTURES = Path(__file__).resolve().parents[1] / "fixtures"


def _load_jsonl(path: Path) -> list[bytes]:
    if not path.exists():
        return []
    out: list[bytes] = []
    with path.open("rb") as f:
        for line in f:
            line = line.strip()
            # Skip the snapshot array Polymarket sends at subscribe time —
            # the Rust bench does the same, so we measure the same input set.
            if not line or line.startswith(b"["):
                continue
            out.append(line)
    return out


POLY_FRAMES = _load_jsonl(FIXTURES / "polymarket_ws_book.jsonl")
KALSHI_FRAMES = _load_jsonl(FIXTURES / "kalshi_ws_book.jsonl")


# ---------------------------------------------------------------------------
# Polymarket — book / price_change / last_trade_price
# ---------------------------------------------------------------------------


def _diy_polymarket_apply(frames: list[bytes]) -> int:
    """Hand-rolled what users would actually write: json.loads, dispatch on
    event_type, maintain bids/asks as dicts. ~30 lines, the obvious code."""
    bids: dict[str, float] = {}
    asks: dict[str, float] = {}
    applied = 0
    for raw in frames:
        msg = json.loads(raw)
        ev = msg.get("event_type")
        if ev == "book":
            bids.clear()
            asks.clear()
            for lvl in msg.get("bids") or ():
                bids[lvl["price"]] = float(lvl["size"])
            for lvl in msg.get("asks") or ():
                asks[lvl["price"]] = float(lvl["size"])
        elif ev == "price_change":
            for ch in msg.get("price_changes") or ():
                price, size, side = ch.get("price"), ch.get("size"), ch.get("side")
                if price is None:
                    continue
                book = bids if side == "BUY" else asks
                size_f = float(size or 0)
                if size_f == 0.0:
                    book.pop(price, None)
                else:
                    book[price] = size_f
        # `last_trade_price` and others — no orderbook mutation, just observed.
        applied += 1
    return applied


@pytest.mark.skipif(not POLY_FRAMES, reason="no polymarket_ws_book.jsonl fixture")
def test_diy_python_polymarket(benchmark):
    def run() -> int:
        return _diy_polymarket_apply(POLY_FRAMES)

    benchmark.pedantic(run, iterations=1, rounds=5)


# ---------------------------------------------------------------------------
# Kalshi — orderbook_snapshot / orderbook_delta
# ---------------------------------------------------------------------------


def _diy_kalshi_apply(frames: list[bytes]) -> int:
    """Same shape as Polymarket DIY but for Kalshi's `type`-tagged envelope."""
    yes: dict[int, int] = {}
    no: dict[int, int] = {}
    applied = 0
    for raw in frames:
        msg = json.loads(raw)
        msg_type = msg.get("type")
        payload = msg.get("msg") or {}
        if msg_type == "orderbook_snapshot":
            yes.clear()
            no.clear()
            for price, size in payload.get("yes") or ():
                yes[int(price)] = int(size)
            for price, size in payload.get("no") or ():
                no[int(price)] = int(size)
        elif msg_type == "orderbook_delta":
            side = payload.get("side")
            price = payload.get("price")
            delta = payload.get("delta")
            if price is None or delta is None:
                continue
            book = yes if side == "yes" else no
            new_size = book.get(int(price), 0) + int(delta)
            if new_size <= 0:
                book.pop(int(price), None)
            else:
                book[int(price)] = new_size
        applied += 1
    return applied


@pytest.mark.skipif(not KALSHI_FRAMES, reason="no kalshi_ws_book.jsonl fixture")
def test_diy_python_kalshi(benchmark):
    def run() -> int:
        return _diy_kalshi_apply(KALSHI_FRAMES)

    benchmark.pedantic(run, iterations=1, rounds=5)
