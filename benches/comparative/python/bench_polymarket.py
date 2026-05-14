"""Polymarket REST API head-to-head: OpenPX vs py-clob-client.

Each method runs as a pytest-benchmark function pair: `openpx_<op>` vs
`pyclob_<op>`. Same asset_id, same network, same minute. 20 iterations
per benchmark with a 100 ms gap (polyfill methodology) so absolute
numbers are a fair reflection of what users feel.

Run: `pytest benches/comparative/python/bench_polymarket.py --benchmark-only`
"""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest

# `openpx` is the unified SDK under test.
from openpx import Exchange

# Optional comparison target. Skip the SDK column gracefully if the
# upstream package isn't installed on this runner.
try:
    from py_clob_client.client import ClobClient
    from py_clob_client.constants import POLYGON

    HAS_PYCLOB = True
except ImportError:  # pragma: no cover - optional
    HAS_PYCLOB = False


FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "polymarket_book.meta.json"
HOST = "https://clob.polymarket.com"
GAP = 0.1  # seconds between iterations — matches polyfill-rs methodology


def _asset_id() -> str:
    meta = json.loads(FIXTURE.read_text())
    return meta["asset_id"]


def _condition_id() -> str:
    meta = json.loads(FIXTURE.read_text())
    return meta["condition_id"]


@pytest.fixture(scope="module")
def openpx_exchange() -> Exchange:
    return Exchange("polymarket")


@pytest.fixture(scope="module")
def pyclob_client() -> "ClobClient":
    if not HAS_PYCLOB:
        pytest.skip("py-clob-client not installed")
    return ClobClient(HOST, chain_id=POLYGON)


# ---------------------------------------------------------------------------
# fetch_markets
# ---------------------------------------------------------------------------


def test_openpx_fetch_markets(benchmark, openpx_exchange):
    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_markets()

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_pyclob_fetch_markets(benchmark, pyclob_client):
    def run():
        time.sleep(GAP)
        return pyclob_client.get_sampling_markets()

    benchmark.pedantic(run, iterations=1, rounds=20)


# ---------------------------------------------------------------------------
# fetch_market (single)
# ---------------------------------------------------------------------------


def test_openpx_fetch_market(benchmark, openpx_exchange):
    cid = _condition_id()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_market(cid)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_pyclob_fetch_market(benchmark, pyclob_client):
    cid = _condition_id()

    def run():
        time.sleep(GAP)
        return pyclob_client.get_market(cid)

    benchmark.pedantic(run, iterations=1, rounds=20)


# ---------------------------------------------------------------------------
# fetch_orderbook
# ---------------------------------------------------------------------------


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


# ---------------------------------------------------------------------------
# fetch_trades
# ---------------------------------------------------------------------------


def test_openpx_fetch_trades(benchmark, openpx_exchange):
    aid = _asset_id()

    def run():
        time.sleep(GAP)
        return openpx_exchange.fetch_trades(aid)

    benchmark.pedantic(run, iterations=1, rounds=20)


def test_pyclob_fetch_trades(benchmark, pyclob_client):
    aid = _asset_id()

    def run():
        time.sleep(GAP)
        # py-clob-client exposes trades via the same get_market_trades path
        return pyclob_client.get_market_trades_events(condition_id=_condition_id())

    benchmark.pedantic(run, iterations=1, rounds=20)
