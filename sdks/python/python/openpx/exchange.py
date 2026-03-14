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
        markets = exchange.fetch_markets(limit=5)
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
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> list[dict[str, Any]]:
        """Fetch markets. Returns list of Market dicts (Pydantic wrapping in _models.py)."""
        raw = self._native.fetch_markets(limit, cursor)
        try:
            from openpx._models import Market
            return [Market(**m) for m in raw]
        except (ImportError, Exception):
            return raw

    def fetch_market(self, market_id: str) -> Any:
        raw = self._native.fetch_market(market_id)
        try:
            from openpx._models import Market
            return Market(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_all_unified_markets(self) -> list[Any]:
        raw = self._native.fetch_all_unified_markets()
        try:
            from openpx._models import UnifiedMarket
            return [UnifiedMarket(**m) for m in raw]
        except (ImportError, Exception):
            return raw

    def create_order(
        self,
        market_id: str,
        outcome: str,
        side: str,
        price: float,
        size: float,
        **params: str,
    ) -> Any:
        raw = self._native.create_order(market_id, outcome, side, price, size, params or None)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def cancel_order(self, order_id: str, market_id: Optional[str] = None) -> Any:
        raw = self._native.cancel_order(order_id, market_id)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_order(self, order_id: str, market_id: Optional[str] = None) -> Any:
        raw = self._native.fetch_order(order_id, market_id)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_open_orders(self, market_id: Optional[str] = None) -> list[Any]:
        raw = self._native.fetch_open_orders(market_id)
        try:
            from openpx._models import Order
            return [Order(**o) for o in raw]
        except (ImportError, Exception):
            return raw

    def fetch_positions(self, market_id: Optional[str] = None) -> list[Any]:
        raw = self._native.fetch_positions(market_id)
        try:
            from openpx._models import Position
            return [Position(**p) for p in raw]
        except (ImportError, Exception):
            return raw

    def fetch_balance(self) -> dict[str, float]:
        return self._native.fetch_balance()

    def fetch_orderbook(
        self,
        market_id: str,
        outcome: Optional[str] = None,
        token_id: Optional[str] = None,
    ) -> Any:
        raw = self._native.fetch_orderbook(market_id, outcome, token_id)
        try:
            from openpx._models import Orderbook
            return Orderbook(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_fills(
        self,
        market_id: Optional[str] = None,
        limit: Optional[int] = None,
    ) -> list[Any]:
        raw = self._native.fetch_fills(market_id, limit)
        try:
            from openpx._models import Fill
            return [Fill(**f) for f in raw]
        except (ImportError, Exception):
            return raw

    def fetch_price_history(
        self,
        market_id: str,
        interval: str,
        *,
        outcome: Optional[str] = None,
        token_id: Optional[str] = None,
        condition_id: Optional[str] = None,
        start_ts: Optional[int] = None,
        end_ts: Optional[int] = None,
    ) -> list[Any]:
        raw = self._native.fetch_price_history(
            market_id, interval, outcome, token_id, condition_id, start_ts, end_ts
        )
        try:
            from openpx._models import Candlestick
            return [Candlestick(**c) for c in raw]
        except (ImportError, Exception):
            return raw

    def fetch_trades(
        self,
        market_id: str,
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
            market_id, market_ref, outcome, token_id, start_ts, end_ts, limit, cursor
        )
        try:
            from openpx._models import MarketTrade
            return {
                "trades": [MarketTrade(**t) for t in raw["trades"]],
                "cursor": raw.get("cursor"),
            }
        except (ImportError, Exception):
            return raw

    def fetch_orderbook_history(
        self,
        market_id: str,
        *,
        token_id: Optional[str] = None,
        start_ts: Optional[int] = None,
        end_ts: Optional[int] = None,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> dict[str, Any]:
        raw = self._native.fetch_orderbook_history(
            market_id, token_id, start_ts, end_ts, limit, cursor
        )
        try:
            from openpx._models import OrderbookSnapshot
            return {
                "snapshots": [OrderbookSnapshot(**s) for s in raw["snapshots"]],
                "cursor": raw.get("cursor"),
            }
        except (ImportError, Exception):
            return raw
