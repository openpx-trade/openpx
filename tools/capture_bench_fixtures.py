#!/usr/bin/env python3
"""Capture live Polymarket + Kalshi BTC fixtures for the comparative WS bench.

Fixtures are real, not synthetic. For each exchange we dogfood OpenPX's
own `next_active_market_in_series` SeriesRoller primitive to resolve
the currently-live BTC market, then capture:

  - `polymarket_book.json`     — REST orderbook snapshot used to seed
    the orderbook for `apply_book_updates` and `orderbook_ops`.
  - `polymarket_ws_book.jsonl` — 1000 real WS book + price_change +
    last_trade_price frames the Polymarket head-to-head replays.
  - `kalshi_ws_book.jsonl`     — 1000 real orderbook_snapshot +
    orderbook_delta frames for the Kalshi DIY-Python row.

Kalshi WS requires RSA-PSS auth — set `KALSHI_API_KEY_ID` and
`KALSHI_PRIVATE_KEY_PATH` in `.env`. If either is missing the Kalshi
capture is skipped with a warning (the rest of the suite still runs).

Run from the repo root:

    python3 tools/capture_bench_fixtures.py

Requires the OpenPX Python SDK to be built locally (`just python-build`)
plus `cryptography` for the Kalshi RSA-PSS signature.
"""

from __future__ import annotations

import json
import os
import sys
import time
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
    decode hot path of each client without network jitter."""
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


# ---------------------------------------------------------------------------
# Kalshi
# ---------------------------------------------------------------------------


def _load_dotenv() -> None:
    """Tiny .env loader — we only need KALSHI_* vars; no full dotenv dep."""
    env = ROOT / ".env"
    if not env.exists():
        return
    for raw in env.read_text().splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        os.environ.setdefault(k.strip(), v.strip().strip('"').strip("'"))


def _kalshi_sign(private_key_pem: bytes, timestamp_ms: int, method: str, path: str) -> str:
    """RSA-PSS-SHA256 sign of `{timestamp_ms}{METHOD}{path}`, base64-encoded.
    Mirrors `engine/exchanges/kalshi/src/auth.rs::Auth::sign`."""
    import base64

    from cryptography.hazmat.primitives import hashes, serialization
    from cryptography.hazmat.primitives.asymmetric import padding

    key = serialization.load_pem_private_key(private_key_pem, password=None)
    path_no_query = path.split("?", 1)[0]
    message = f"{timestamp_ms}{method.upper()}{path_no_query}".encode()
    sig = key.sign(
        message,
        padding.PSS(mgf=padding.MGF1(hashes.SHA256()), salt_length=padding.PSS.DIGEST_LENGTH),
        hashes.SHA256(),
    )
    return base64.b64encode(sig).decode()


def capture_kalshi_ws(ticker: str, target_msgs: int = 1000, timeout_s: int = 90) -> None:
    """Subscribe to Kalshi's authenticated /ws/v2 endpoint and capture
    orderbook_snapshot + orderbook_delta frames for the given market
    ticker. Output mirrors the Polymarket JSONL shape so the DIY bench
    can read either with the same loader."""
    import asyncio

    import websockets  # type: ignore

    _load_dotenv()
    key_id = os.environ.get("KALSHI_API_KEY_ID")
    pem_path = os.environ.get("KALSHI_PRIVATE_KEY_PATH")
    if not (key_id and pem_path):
        print(
            "  skipping kalshi WS capture: KALSHI_API_KEY_ID / KALSHI_PRIVATE_KEY_PATH not set",
            file=sys.stderr,
        )
        return
    pem_bytes = Path(pem_path).expanduser().read_bytes()

    ws_host = "api.elections.kalshi.com"
    ws_path = "/trade-api/ws/v2"
    url = f"wss://{ws_host}{ws_path}"
    ts_ms = int(time.time() * 1000)
    signature = _kalshi_sign(pem_bytes, ts_ms, "GET", ws_path)

    headers = [
        ("KALSHI-ACCESS-KEY", key_id),
        ("KALSHI-ACCESS-SIGNATURE", signature),
        ("KALSHI-ACCESS-TIMESTAMP", str(ts_ms)),
    ]

    out_path = FIXTURES / "kalshi_ws_book.jsonl"

    async def collect() -> int:
        sub = json.dumps({
            "id": 1,
            "cmd": "subscribe",
            "params": {"channels": ["orderbook_delta"], "market_tickers": [ticker]},
        })
        captured = 0
        with out_path.open("wb") as out:
            async with websockets.connect(url, additional_headers=headers, max_size=None) as ws:
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

    print(f"  capturing live kalshi WS frames into {out_path.relative_to(ROOT)}...")
    n = asyncio.run(collect())
    size = out_path.stat().st_size
    print(f"  wrote {n} frames, {size:,} bytes")


def capture_kalshi() -> None:
    print("kalshi: resolving live 15-min BTC market via SeriesRoller...")
    ex = Exchange("kalshi")
    market = ex.next_active_market_in_series("KXBTC15M")
    ticker = market.ticker
    print(f"  market: {market.title!r}")
    print(f"  ticker: {ticker}")
    if os.environ.get("OPENPX_BENCH_WS", "1") != "0":
        capture_kalshi_ws(ticker)


def main() -> int:
    print(f"capturing live fixtures into {FIXTURES.relative_to(ROOT)}/...")
    try:
        capture_polymarket()
    except Exception as e:  # pragma: no cover
        print(f"  polymarket capture failed: {e}", file=sys.stderr)
        return 1
    try:
        capture_kalshi()
    except Exception as e:  # pragma: no cover
        print(f"  kalshi capture failed: {e}", file=sys.stderr)
        # Non-fatal — Kalshi WS row will simply be absent from the README.
    print("done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
