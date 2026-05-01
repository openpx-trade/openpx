"""Pure Python wrapper around the native Rust extension.

Calls _native.NativeExchange methods (which return plain dicts via pythonize),
then wraps them into auto-generated Pydantic models for type safety and autocomplete.
"""

from __future__ import annotations

from typing import Any, Optional

from openpx._native import NativeExchange


class Exchange:
    """Unified prediction market exchange client.

    Usage::

        from openpx import Exchange

        exchange = Exchange("kalshi", {"api_key_id": "...", "private_key_pem": "..."})
        markets = exchange.fetch_markets()
        for m in markets:
            print(m)  # Pydantic Market model with autocomplete
    """

    def __init__(self, exchange_id: str, config: Optional[dict[str, Any]] = None) -> None:
        self._exchange_id = exchange_id
        self._config = config or {}
        self._native = NativeExchange(exchange_id, self._config)

    def websocket(self) -> "WebSocket":
        """Create a WebSocket connection using this exchange's credentials."""
        from openpx.websocket import WebSocket

        return WebSocket(self._exchange_id, self._config)

    @property
    def id(self) -> str:
        return self._native.id

    @property
    def name(self) -> str:
        return self._native.name

    def describe(self) -> dict[str, Any]:
        return self._native.describe()

    def fetch_markets(
        self,
        *,
        status: Optional[str] = None,
        cursor: Optional[str] = None,
        market_tickers: Optional[list[str]] = None,
        series_ticker: Optional[str] = None,
        event_ticker: Optional[str] = None,
    ) -> dict[str, Any]:
        """Fetch markets from the exchange.

        Pass `market_tickers=[...]` for an explicit lookup (single round-trip, no
        pagination), or omit it to page through the catalog with `cursor`.
        Returns ``{"markets": [...], "cursor": "..." | None}``.
        """
        raw = self._native.fetch_markets(status, cursor, market_tickers, series_ticker, event_ticker)
        try:
            from openpx._models import Market
            return {
                "markets": [Market(**m) for m in raw["markets"]],
                "cursor": raw.get("cursor"),
            }
        except (ImportError, Exception):
            return raw

    def fetch_market_lineage(self, market_ticker: str) -> Any:
        """Fetch a market plus its parent event and series in one call.

        Returns ``{"market": Market, "event": Event | None, "series": Series | None}``.
        Event and series are optional — a dangling parent reference yields
        ``None`` rather than failing the whole call.
        """
        raw = self._native.fetch_market_lineage(market_ticker)
        try:
            from openpx._models import Event, Market, Series
            event = Event(**raw["event"]) if raw.get("event") else None
            series = Series(**raw["series"]) if raw.get("series") else None
            return {
                "market": Market(**raw["market"]),
                "event": event,
                "series": series,
            }
        except (ImportError, Exception):
            return raw

    def create_order(
        self,
        market_ticker: str,
        outcome: str,
        side: str,
        price: float,
        size: float,
        **params: str,
    ) -> Any:
        raw = self._native.create_order(market_ticker, outcome, side, price, size, params or None)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def cancel_order(self, order_id: str, market_ticker: Optional[str] = None) -> Any:
        raw = self._native.cancel_order(order_id, market_ticker)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_order(self, order_id: str, market_ticker: Optional[str] = None) -> Any:
        raw = self._native.fetch_order(order_id, market_ticker)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_open_orders(self, market_ticker: Optional[str] = None) -> list[Any]:
        raw = self._native.fetch_open_orders(market_ticker)
        try:
            from openpx._models import Order
            return [Order(**o) for o in raw]
        except (ImportError, Exception):
            return raw

    def fetch_positions(self, market_ticker: Optional[str] = None) -> list[Any]:
        raw = self._native.fetch_positions(market_ticker)
        try:
            from openpx._models import Position
            return [Position(**p) for p in raw]
        except (ImportError, Exception):
            return raw

    def fetch_balance(self) -> dict[str, float]:
        return self._native.fetch_balance()

    def fetch_orderbook(
        self,
        market_ticker: str,
        outcome: Optional[str] = None,
        token_id: Optional[str] = None,
    ) -> Any:
        raw = self._native.fetch_orderbook(market_ticker, outcome, token_id)
        try:
            from openpx._models import Orderbook
            return Orderbook(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_fills(
        self,
        market_ticker: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> list[Any]:
        raw = self._native.fetch_fills(market_ticker, limit)
        try:
            from openpx._models import Fill
            return [Fill(**f) for f in raw]
        except (ImportError, Exception):
            return raw

    def fetch_trades(
        self,
        market_ticker: str,
        *,
        market_ref: Optional[str] = None,
        outcome: Optional[str] = None,
        token_id: Optional[str] = None,
        start_ts: Optional[int] = None,
        end_ts: Optional[int] = None,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> dict[str, Any]:
        raw = self._native.fetch_trades(
            market_ticker, market_ref, outcome, token_id, start_ts, end_ts, limit, cursor
        )
        try:
            from openpx._models import MarketTrade
            return {
                "trades": [MarketTrade(**t) for t in raw["trades"]],
                "cursor": raw.get("cursor"),
            }
        except (ImportError, Exception):
            return raw

