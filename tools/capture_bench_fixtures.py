#!/usr/bin/env python3
"""Capture real, unauthenticated orderbook fixtures for the comparative bench.

Uses OpenPX's `next_active_market_in_series` primitive to resolve the
currently-active market in a revolving series (5/15-min BTC up-or-down)
on each exchange, then fetches the raw wire bytes of that market's
orderbook from the upstream public endpoint and writes them to
`benches/comparative/fixtures/`.

Why revolving 5/15-min BTC: those series always have a fresh, deep,
fast-rolling book — perfect for benchmarks that need realistic data on
every refresh. SeriesRoller (the watcher layer that pre-fetches the
*next* market in the series for zero-downtime WS handoff) builds on
top of this same primitive.

The fixtures are committed so contributors and CI can replay real
exchange responses without ever hitting the network or carrying any
credentials. Re-run this script periodically — it's idempotent and
only rewrites files when bytes differ.

Usage:
    python3 tools/capture_bench_fixtures.py
"""

from __future__ import annotations

import datetime as _dt
import json
import sys
import urllib.request
from pathlib import Path

# OpenPX primitive — same one users will call to subscribe to revolving
# markets. We dogfood it here so the capture path stays in lockstep with
# the runtime path.
from openpx import Exchange

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "benches" / "comparative" / "fixtures"
TIMEOUT = 15

USER_AGENT = "openpx-bench-capture/0.1 (+https://github.com/openpx-trade/openpx)"

# Hardcoded series tickers — always the most liquid revolving BTC series
# on each exchange. Update these only if the upstream series identifier
# itself changes; the captured fixture refreshes automatically.
POLYMARKET_SERIES = "btc-up-or-down-5m"  # Polymarket /series slug
KALSHI_SERIES = "KXBTC15M"  # Kalshi native series_ticker


def _http_get(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=TIMEOUT) as resp:
        return resp.read()


def _write_if_changed(path: Path, content: bytes) -> bool:
    if path.exists() and path.read_bytes() == content:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(content)
    return True


def _save_pair(stem: str, body: bytes, meta: dict) -> bool:
    body_changed = _write_if_changed(FIXTURES / f"{stem}.json", body)
    _write_if_changed(
        FIXTURES / f"{stem}.meta.json",
        (json.dumps(meta, indent=2) + "\n").encode("utf-8"),
    )
    return body_changed


# ---------------------------------------------------------------------------
# Polymarket
# ---------------------------------------------------------------------------


def capture_polymarket() -> None:
    market = Exchange("polymarket", {}).next_active_market_in_series(POLYMARKET_SERIES)
    if market is None or not market.outcomes:
        sys.exit(f"polymarket: no active market in series '{POLYMARKET_SERIES}'")

    # Polymarket binary outcomes are paired tokens (Up/Down). Either side
    # works as the bench fixture — we pick the first one with a token_id.
    token_id = next((o.token_id for o in market.outcomes if o.token_id), None)
    if token_id is None:
        sys.exit(f"polymarket: market has no outcome token_ids: {market.openpx_id}")

    source_url = f"https://clob.polymarket.com/book?token_id={token_id}"
    body = _http_get(source_url)

    parsed = json.loads(body)
    meta = {
        "captured_at": _dt.datetime.now(_dt.timezone.utc).isoformat(),
        "source_url": source_url,
        "series_ticker": POLYMARKET_SERIES,
        "market_event_ticker": market.event_ticker,
        "market_close_time": market.close_time.isoformat() if market.close_time else None,
        "openpx_id": market.openpx_id,
        "token_id": token_id,
        "bids_count": len(parsed.get("bids", [])),
        "asks_count": len(parsed.get("asks", [])),
    }

    # Re-serialize to canonical pretty form so diffs are reviewable. The
    # bench loads bytes from disk and parses — same path either way.
    pretty = (json.dumps(parsed, indent=2) + "\n").encode("utf-8")
    changed = _save_pair("polymarket_book", pretty, meta)
    print(
        f"polymarket: {'updated' if changed else 'unchanged'} "
        f"({meta['bids_count']} bids / {meta['asks_count']} asks, "
        f"closes {meta['market_close_time']})"
    )


# ---------------------------------------------------------------------------
# Kalshi
# ---------------------------------------------------------------------------


def capture_kalshi() -> None:
    market = Exchange("kalshi", {}).next_active_market_in_series(KALSHI_SERIES)
    if market is None:
        sys.exit(f"kalshi: no active market in series '{KALSHI_SERIES}'")

    # Kalshi's orderbook endpoint takes the *full* market ticker
    # (e.g. "KXBTC15M-26MAY051515-15"). The unified `event_ticker` field
    # surfaces only the event-level prefix, so we pull the full ticker
    # from `openpx_id`, which is "kalshi:<full-ticker>".
    if not market.openpx_id or not market.openpx_id.startswith("kalshi:"):
        sys.exit(f"kalshi: unexpected openpx_id: {market.openpx_id}")
    ticker = market.openpx_id.removeprefix("kalshi:")

    source_url = (
        f"https://api.elections.kalshi.com/trade-api/v2/markets/{ticker}/orderbook"
    )
    body = _http_get(source_url)

    parsed = json.loads(body)
    # Kalshi returns either {"orderbook": {"yes": [...], "no": [...]}}
    # (legacy) or {"orderbook_fp": {"yes_dollars": [...], "no_dollars": [...]}}
    # (current). Probe both for the level-count metadata.
    ob = parsed.get("orderbook") or parsed.get("orderbook_fp") or {}
    yes_levels = ob.get("yes") or ob.get("yes_dollars") or []
    no_levels = ob.get("no") or ob.get("no_dollars") or []
    meta = {
        "captured_at": _dt.datetime.now(_dt.timezone.utc).isoformat(),
        "source_url": source_url,
        "series_ticker": KALSHI_SERIES,
        "market_ticker": ticker,
        "market_close_time": market.close_time.isoformat() if market.close_time else None,
        "openpx_id": market.openpx_id,
        "yes_levels": len(yes_levels),
        "no_levels": len(no_levels),
    }

    pretty = (json.dumps(parsed, indent=2) + "\n").encode("utf-8")
    changed = _save_pair("kalshi_orderbook", pretty, meta)
    print(
        f"kalshi: {'updated' if changed else 'unchanged'} "
        f"({meta['yes_levels']} yes / {meta['no_levels']} no levels, "
        f"closes {meta['market_close_time']})"
    )


def main() -> int:
    capture_polymarket()
    capture_kalshi()
    return 0


if __name__ == "__main__":
    sys.exit(main())
