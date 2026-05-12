"""OpenPX vs `kalshi-python` — end-to-end orderbook fetch on Kalshi.

The market ticker comes from `next_active_market_in_series('KXBTC15M')`,
so the bench always hits whatever 15-min BTC market is open right now.
Real network, real bytes, no credentials.

Run via:
    pytest --codspeed benches/comparative/python/bench_kalshi.py
"""

from __future__ import annotations

import pytest

from openpx import Exchange

KALSHI_SERIES = "KXBTC15M"

try:
    import kalshi_python  # type: ignore

    HAS_KALSHI = hasattr(kalshi_python, "MarketsApi")
except ImportError:
    HAS_KALSHI = False


@pytest.fixture(scope="module")
def openpx_exchange() -> Exchange:
    return Exchange("kalshi", {})


@pytest.fixture(scope="module")
def market_ticker(openpx_exchange: Exchange) -> str:
    market = openpx_exchange.next_active_market_in_series(KALSHI_SERIES)
    if market is None:
        pytest.skip(f"no active kalshi market in series '{KALSHI_SERIES}'")
    if not market.openpx_id or not market.openpx_id.startswith("kalshi:"):
        pytest.skip(f"unexpected openpx_id: {market.openpx_id}")
    return market.openpx_id.removeprefix("kalshi:")


@pytest.fixture(scope="module")
def kalshi_markets_api():
    if not HAS_KALSHI:
        pytest.skip("kalshi-python not installed")
    # `kalshi-python` is an OpenAPI-generated client — orderbook lives on
    # `MarketsApi`, not the top-level `KalshiClient`. Configure against
    # the public election-markets host (no credentials needed for read).
    config = kalshi_python.Configuration(  # type: ignore[attr-defined]
        host="https://api.elections.kalshi.com/trade-api/v2"
    )
    return kalshi_python.MarketsApi(  # type: ignore[attr-defined]
        kalshi_python.ApiClient(config)  # type: ignore[attr-defined]
    )


@pytest.mark.benchmark(group="kalshi_fetch_book")
def test_openpx_fetch_kalshi_book(
    benchmark, openpx_exchange: Exchange, market_ticker: str
) -> None:
    benchmark(lambda: openpx_exchange.fetch_orderbook(market_ticker))


@pytest.mark.benchmark(group="kalshi_fetch_book")
@pytest.mark.skipif(not HAS_KALSHI, reason="kalshi-python not installed")
def test_kalshi_python_fetch_kalshi_book(
    benchmark, kalshi_markets_api, market_ticker: str
) -> None:
    benchmark(lambda: kalshi_markets_api.get_market_orderbook(market_ticker))
