"""Kalshi REST head-to-head: OpenPX vs kalshi-python.

Single fair comparison: `fetch_orderbook`. Both libraries hit the
same upstream endpoint and return the same shape.

20 iterations × 100 ms gap, polyfill methodology.

Run: `pytest benches/comparative/python/bench_kalshi.py --benchmark-only`
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest

from openpx import Exchange

try:
    from kalshi_python import KalshiClient, MarketsApi

    HAS_KALSHI_PY = True
except ImportError:  # pragma: no cover - optional
    HAS_KALSHI_PY = False


FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "kalshi_orderbook.meta.json"
GAP = 0.1


def _ticker() -> str:
    return json.loads(FIXTURE.read_text())["ticker"]


@pytest.fixture(scope="module")
def openpx_exchange() -> Exchange:
    return Exchange("kalshi")


@pytest.fixture(scope="module")
def kalshi_markets():
    if not HAS_KALSHI_PY:
        pytest.skip("kalshi-python not installed")
    return MarketsApi(KalshiClient())


def test_openpx_fetch_orderbook(benchmark, openpx_exchange):
    t = _ticker()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_orderbook(t)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_kalshi_python_fetch_orderbook(benchmark, kalshi_markets):
    t = _ticker()

    def run():
        time.sleep(GAP)
        return kalshi_markets.get_market_orderbook(ticker=t)

    benchmark.pedantic(run, iterations=1, rounds=20)
