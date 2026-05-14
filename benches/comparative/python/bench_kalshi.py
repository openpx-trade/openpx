"""Kalshi REST API head-to-head: OpenPX vs kalshi-python.

20 iterations × 100 ms gap, polyfill methodology. Same series fixture
captured by `tools/capture_bench_fixtures.py`.

Run: `pytest benches/comparative/python/bench_kalshi.py --benchmark-only`
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest

from openpx import Exchange

try:
    from kalshi_python.api_instance_factory import ApiInstanceFactory
    from kalshi_python.configuration import Configuration

    HAS_KALSHI_PY = True
except ImportError:  # pragma: no cover - optional
    HAS_KALSHI_PY = False


FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "kalshi_orderbook.meta.json"
GAP = 0.1


def _ticker() -> str:
    return json.loads(FIXTURE.read_text())["ticker"]


def _event_ticker() -> str:
    return json.loads(FIXTURE.read_text())["event_ticker"]


@pytest.fixture(scope="module")
def openpx_exchange() -> Exchange:
    return Exchange("kalshi")


@pytest.fixture(scope="module")
def kalshi_api():
    if not HAS_KALSHI_PY:
        pytest.skip("kalshi-python not installed")
    cfg = Configuration()
    # ApiInstanceFactory exposes per-resource clients (MarketApi, etc.)
    return ApiInstanceFactory(cfg)


# ---------------------------------------------------------------------------
# fetch_markets
# ---------------------------------------------------------------------------


def test_openpx_fetch_markets(benchmark, openpx_exchange):
    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_markets()

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_kalshi_python_fetch_markets(benchmark, kalshi_api):
    market_api = kalshi_api.get_market_api()

    def run():
        time.sleep(GAP)
        return market_api.get_markets(limit=100)

    benchmark.pedantic(run, iterations=1, rounds=20)


# ---------------------------------------------------------------------------
# fetch_market (single)
# ---------------------------------------------------------------------------


def test_openpx_fetch_market(benchmark, openpx_exchange):
    t = _ticker()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_market(t)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_kalshi_python_fetch_market(benchmark, kalshi_api):
    market_api = kalshi_api.get_market_api()
    t = _ticker()

    def run():
        time.sleep(GAP)
        return market_api.get_market(ticker=t)

    benchmark.pedantic(run, iterations=1, rounds=20)


# ---------------------------------------------------------------------------
# fetch_orderbook
# ---------------------------------------------------------------------------


def test_openpx_fetch_orderbook(benchmark, openpx_exchange):
    t = _ticker()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_orderbook(t)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_kalshi_python_fetch_orderbook(benchmark, kalshi_api):
    market_api = kalshi_api.get_market_api()
    t = _ticker()

    def run():
        time.sleep(GAP)
        return market_api.get_market_orderbook(ticker=t)

    benchmark.pedantic(run, iterations=1, rounds=20)


# ---------------------------------------------------------------------------
# fetch_trades
# ---------------------------------------------------------------------------


def test_openpx_fetch_trades(benchmark, openpx_exchange):
    t = _ticker()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_trades(t)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_kalshi_python_fetch_trades(benchmark, kalshi_api):
    market_api = kalshi_api.get_market_api()
    t = _ticker()

    def run():
        time.sleep(GAP)
        return market_api.get_trades(ticker=t, limit=100)

    benchmark.pedantic(run, iterations=1, rounds=20)
