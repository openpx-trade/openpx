"""OpenPX vs `py-clob-client` — end-to-end orderbook fetch on Polymarket.

The token id comes from `next_active_market_in_series('btc-up-or-down-5m')`,
so the bench always hits whatever 5-min BTC market is open right now.
Real network, real bytes, no credentials.

Walltime is the only Codspeed instrument that applies to Python.

Run via:
    pytest --codspeed benches/comparative/python/bench_polymarket.py
"""

from __future__ import annotations

import pytest

from openpx import Exchange

POLYMARKET_SERIES = "btc-up-or-down-5m"

try:
    from py_clob_client.client import ClobClient  # type: ignore

    HAS_PYCLOB = True
except ImportError:
    HAS_PYCLOB = False


@pytest.fixture(scope="module")
def openpx_exchange() -> Exchange:
    return Exchange("polymarket", {})


@pytest.fixture(scope="module")
def token_id(openpx_exchange: Exchange) -> str:
    market = openpx_exchange.next_active_market_in_series(POLYMARKET_SERIES)
    if market is None or not market.outcomes:
        pytest.skip(f"no active polymarket market in series '{POLYMARKET_SERIES}'")
    tid = next((o.token_id for o in market.outcomes if o.token_id), None)
    if tid is None:
        pytest.skip("active market has no outcome token_ids")
    return tid


@pytest.fixture(scope="module")
def pyclob_client() -> "ClobClient":
    if not HAS_PYCLOB:
        pytest.skip("py-clob-client not installed")
    return ClobClient(host="https://clob.polymarket.com", chain_id=137)


@pytest.mark.benchmark(group="polymarket_fetch_book")
def test_openpx_fetch_polymarket_book(
    benchmark, openpx_exchange: Exchange, token_id: str
) -> None:
    benchmark(lambda: openpx_exchange.fetch_orderbook(token_id))


@pytest.mark.benchmark(group="polymarket_fetch_book")
@pytest.mark.skipif(not HAS_PYCLOB, reason="py-clob-client not installed")
def test_pyclob_fetch_polymarket_book(
    benchmark, pyclob_client: "ClobClient", token_id: str
) -> None:
    benchmark(lambda: pyclob_client.get_order_book(token_id))
