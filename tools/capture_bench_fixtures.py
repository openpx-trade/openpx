#!/usr/bin/env python3
"""Capture live 5/15-min BTC orderbook fixtures for the comparative bench.

The bench fixtures aren't synthetic. We dogfood OpenPX's own
`next_active_market_in_series` SeriesRoller primitive to resolve the
*currently-live* market in each exchange's revolving series, then write
the raw orderbook bytes alongside a small `.meta.json` that the
benches read for the asset_id / ticker / condition_id.

- Polymarket: `btc-up-or-down-5m` → live 5-min BTC up/down market
- Kalshi:     `KXBTC15M`          → live 15-min BTC up/down market

Run from the repo root:

    python3 tools/capture_bench_fixtures.py

Requires the OpenPX Python SDK to be built locally
(`just python-build`).
"""

from __future__ import annotations

import json
import sys
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

from openpx import Exchange

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "benches" / "comparative" / "fixtures"
FIXTURES.mkdir(parents=True, exist_ok=True)


def _now() -> str:
    return datetime.now(timezone.utc).isoformat()


def _write(name: str, body: bytes) -> None:
    path = FIXTURES / name
    path.write_bytes(body)
    print(f"  wrote {path.relative_to(ROOT)} ({len(body):,} bytes)")


def _write_meta(name: str, meta: dict) -> None:
    path = FIXTURES / name
    meta["captured_at"] = _now()
    path.write_text(json.dumps(meta, indent=2) + "\n")
    print(f"  wrote {path.relative_to(ROOT)}")


def capture_polymarket() -> None:
    print("polymarket: resolving live 5-min BTC market via SeriesRoller...")
    ex = Exchange("polymarket")
    market = ex.next_active_market_in_series("btc-up-or-down-5m")
    asset_id = market["yes_token_id"]
    condition_id = market["condition_id"]
    print(f"  market: {market.get('question', condition_id)!r}")
    print(f"  asset_id: {asset_id}")

    # Fetch the full orderbook bytes directly from the public REST endpoint.
    url = f"https://clob.polymarket.com/book?token_id={asset_id}"
    with urllib.request.urlopen(url) as resp:
        body = resp.read()
    _write("polymarket_book.json", body)
    _write_meta(
        "polymarket_book.meta.json",
        {
            "asset_id": asset_id,
            "condition_id": condition_id,
            "series": "btc-up-or-down-5m",
            "endpoint": url,
        },
    )


def capture_kalshi() -> None:
    print("kalshi: resolving live 15-min BTC market via SeriesRoller...")
    ex = Exchange("kalshi")
    market = ex.next_active_market_in_series("KXBTC15M")
    ticker = market["ticker"]
    event_ticker = market.get("event_ticker", "")
    print(f"  market: {market.get('title', ticker)!r}")
    print(f"  ticker: {ticker}")

    # Public orderbook endpoint — no auth needed.
    url = f"https://api.elections.kalshi.com/trade-api/v2/markets/{ticker}/orderbook"
    with urllib.request.urlopen(url) as resp:
        body = resp.read()
    _write("kalshi_orderbook.json", body)
    _write_meta(
        "kalshi_orderbook.meta.json",
        {
            "ticker": ticker,
            "event_ticker": event_ticker,
            "series": "KXBTC15M",
            "endpoint": url,
        },
    )


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
        return 1
    print("done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
