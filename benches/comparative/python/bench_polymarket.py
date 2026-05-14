"""Polymarket REST head-to-head: OpenPX vs py-clob-client.

Single fair comparison: `fetch_orderbook`. Both libraries hit the same
upstream endpoint (`/book?token_id=...`) and return the same shape, so
the ratio reflects real client-side overhead — not endpoint choice.

`fetch_markets` was deliberately dropped: OpenPX hits Gamma
`/events/keyset` (rich event tree → typed unified Market) while
py-clob hits CLOB `/sampling-markets` (raw dicts, 60% smaller payload).
Different operations, can't fairly compare.

20 iterations × 100 ms gap, polyfill methodology.

Run: `pytest benches/comparative/python/bench_polymarket.py --benchmark-only`
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest

from openpx import Exchange

try:
    from py_clob_client.client import ClobClient
    from py_clob_client.constants import POLYGON

    HAS_PYCLOB = True
except ImportError:  # pragma: no cover - optional
    HAS_PYCLOB = False


FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "polymarket_book.meta.json"
HOST = "https://clob.polymarket.com"
GAP = 0.1


def _asset_id() -> str:
    return json.loads(FIXTURE.read_text())["asset_id"]


@pytest.fixture(scope="module")
def openpx_exchange() -> Exchange:
    return Exchange("polymarket")


@pytest.fixture(scope="module")
def pyclob_client() -> "ClobClient":
    if not HAS_PYCLOB:
        pytest.skip("py-clob-client not installed")
    return ClobClient(HOST, chain_id=POLYGON)


def test_openpx_fetch_orderbook(benchmark, openpx_exchange):
    aid = _asset_id()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_orderbook(aid)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_pyclob_fetch_orderbook(benchmark, pyclob_client):
    aid = _asset_id()

    def run():
        time.sleep(GAP)
        return pyclob_client.get_order_book(aid)

    benchmark.pedantic(run, iterations=1, rounds=20)
