#!/usr/bin/env python3
"""Capture live Polymarket BTC fixtures for the comparative WebSocket bench.

The fixtures aren't synthetic. We dogfood OpenPX's own
`next_active_market_in_series` SeriesRoller primitive to resolve the
*currently-live* 5-min BTC up/down market, then capture:

  - `polymarket_book.json`     — the REST orderbook snapshot used to
    seed the orderbook for `apply_book_updates` and `orderbook_ops`.
  - `polymarket_ws_book.jsonl` — 1000 real WS book + price_change +
    last_trade_price frames the `ws_decode_apply` head-to-head replays.

Run from the repo root:

    python3 tools/capture_bench_fixtures.py

Requires the OpenPX Python SDK to be built locally (`just python-build`).
"""

from __future__ import annotations

import json
import os
import sys
import urllib.request
from pathlib import Path

from openpx import Exchange

# Polymarket's edge rejects urllib's default `Python-urllib/...` UA with 403.
_UA = "openpx-bench/0.1 (https://github.com/openpx-trade/openpx)"


def _fetch_bytes(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": _UA})
    with urllib.request.urlopen(req) as resp:
        return resp.read()

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "benches" / "comparative" / "fixtures"
FIXTURES.mkdir(parents=True, exist_ok=True)


def _write(name: str, body: bytes) -> None:
    path = FIXTURES / name
    path.write_bytes(body)
    print(f"  wrote {path.relative_to(ROOT)} ({len(body):,} bytes)")


def capture_polymarket_ws(asset_id: str, target_msgs: int = 1000, timeout_s: int = 90) -> None:
    """Stream real book + price_change frames from Polymarket's market WS and
    serialize them as JSONL. The bench replays these bytes to time the
    decode + apply hot path of each client without network jitter."""
    import asyncio

    import websockets  # type: ignore

    url = "wss://ws-subscriptions-clob.polymarket.com/ws/market"
    out_path = FIXTURES / "polymarket_ws_book.jsonl"

    async def collect() -> int:
        sub = json.dumps({"assets_ids": [asset_id], "type": "market"})
        captured = 0
        with out_path.open("wb") as out:
            async with websockets.connect(url, max_size=None) as ws:
                await ws.send(sub)
                while captured < target_msgs:
                    try:
                        msg = await asyncio.wait_for(ws.recv(), timeout=timeout_s)
                    except asyncio.TimeoutError:
                        break
                    if isinstance(msg, str):
                        out.write(msg.encode("utf-8"))
                    else:
                        out.write(msg)
                    out.write(b"\n")
                    captured += 1
        return captured

    print(f"  capturing live WS frames into {out_path.relative_to(ROOT)}...")
    n = asyncio.run(collect())
    size = out_path.stat().st_size
    print(f"  wrote {n} frames, {size:,} bytes")


def capture_polymarket() -> None:
    print("polymarket: resolving live 5-min BTC market via SeriesRoller...")
    ex = Exchange("polymarket")
    market = ex.next_active_market_in_series("btc-up-or-down-5m")
    # Polymarket markets are binary — pick the YES outcome's token_id.
    yes = next((o for o in market.outcomes if o.label.lower() in ("up", "yes")), market.outcomes[0])
    asset_id = yes.token_id
    print(f"  market: {market.title!r}")
    print(f"  asset_id: {asset_id}")

    # Fetch the full orderbook bytes directly from the public REST endpoint.
    url = f"https://clob.polymarket.com/book?token_id={asset_id}"
    body = _fetch_bytes(url)
    _write("polymarket_book.json", body)
    # WS capture is opt-in via env (slow + needs network); skip when
    # OPENPX_BENCH_WS=0 to keep local iteration fast.
    if os.environ.get("OPENPX_BENCH_WS", "1") != "0":
        capture_polymarket_ws(asset_id)


def main() -> int:
    print(f"capturing live fixtures into {FIXTURES.relative_to(ROOT)}/...")
    try:
        capture_polymarket()
    except Exception as e:  # pragma: no cover
        print(f"  polymarket capture failed: {e}", file=sys.stderr)
        return 1
    print("done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
