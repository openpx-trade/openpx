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

    Authentication
    --------------
    **Kalshi** — pass ``api_key_id`` plus either ``private_key_path`` (file
    path to the PKCS#8 PEM) or ``private_key_pem`` (inline PEM contents).
    Public market-data calls work with no config.

    **Polymarket** — pick the credential path matching your wallet:

    * MetaMask EOA + Polymarket Safe → ``private_key`` (EOA) + ``funder`` (Safe).
      ``signature_type`` is auto-detected as ``gnosis_safe``.
    * Plain EOA (no Safe) → ``private_key`` only. ``signature_type`` defaults to ``eoa``.
    * Pre-derived API keys (most reliable behind VPNs / Cloudflare) →
      ``api_key`` + ``api_secret`` + ``api_passphrase`` (and ``private_key`` for
      order signing). Skips the ``derive-api-key`` flow.

    Setting ``signature_type="eoa"`` while ``funder`` is also set is invalid
    per Polymarket's SDK; OpenPX overrides it to ``gnosis_safe`` with a
    warning. Full credential matrix:
    https://docs.openpx.io/setup/polymarket-credentials
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
        limit: Optional[int] = None,
    ) -> dict[str, Any]:
        """Fetch markets from the exchange.

        Pass `market_tickers=[...]` for an explicit lookup (single round-trip, no
        pagination), or omit it to page through the catalog with `cursor`.
        Returns ``{"markets": [...], "cursor": "..." | None}``.
        """
        raw = self._native.fetch_markets(
            status, cursor, market_tickers, series_ticker, event_ticker, limit
        )
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
        asset_id: str,
        outcome: str,
        side: str,
        price: float,
        size: float,
        order_type: str = "gtc",
    ) -> Any:
        """Submit a new order on the exchange.

        ``asset_id`` is the per-outcome identifier — Kalshi market ticker or
        Polymarket CTF token id (same convention as ``fetch_orderbook``).
        Polymarket callers who only have a market slug + outcome label must
        resolve the token id first via ``fetch_market``.

        ``outcome`` is ``"yes"`` / ``"no"`` on Kalshi (drives YES-frame
        bid/ask side selection at the wire). On Polymarket the outcome is
        already encoded in ``asset_id``; this argument is just a label hint
        used for the response ``Order.outcome`` field.

        ``order_type`` is ``"gtc"`` (default), ``"ioc"``, or ``"fok"``.
        """
        raw = self._native.create_order(asset_id, outcome, side, price, size, order_type)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def cancel_order(self, order_id: str) -> Any:
        raw = self._native.cancel_order(order_id)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def cancel_all_orders(self, asset_id: Optional[str] = None) -> list[Any]:
        """Cancel all open orders, optionally scoped to one ``asset_id``."""
        raw = self._native.cancel_all_orders(asset_id)
        try:
            from openpx._models import Order
            return [Order(**o) for o in raw]
        except (ImportError, Exception):
            return raw

    def create_orders_batch(self, orders: list[dict]) -> list[Any]:
        """Submit multiple orders in one round-trip.

        Each entry is a dict with the same fields as ``create_order`` —
        ``asset_id``, ``outcome`` (``yes``/``no``/``{label: ...}``), ``side``
        (``buy``/``sell``), ``price``, ``size``, optional ``order_type`` (default ``gtc``).
        Polymarket caps batches at 15 orders; Kalshi enforces a token-budget cap.
        """
        raw = self._native.create_orders_batch(orders)
        try:
            from openpx._models import Order
            return [Order(**o) for o in raw]
        except (ImportError, Exception):
            return raw

    def fetch_order(self, order_id: str) -> Any:
        raw = self._native.fetch_order(order_id)
        try:
            from openpx._models import Order
            return Order(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_open_orders(self, asset_id: Optional[str] = None) -> list[Any]:
        """Fetch open orders, optionally filtered by ``asset_id`` (Kalshi
        market ticker | Polymarket CTF token id — same convention as
        ``fetch_orderbook`` and ``create_order``)."""
        raw = self._native.fetch_open_orders(asset_id)
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

    def refresh_balance(self) -> None:
        """Refresh cached balance/allowance state from the exchange.

        Polymarket: pulls latest collateral allowance via the CLOB
        ``GET /balance-allowance/update``. Kalshi: no-op (no allowance model).
        """
        self._native.refresh_balance()

    def fetch_server_time(self) -> str:
        """Return the exchange's current wall-clock time as RFC3339 UTC.

        Polymarket: dedicated ``GET /time`` (Unix seconds). Kalshi: HTTP
        ``Date`` header from a public ``GET /exchange/status`` response.
        """
        return self._native.fetch_server_time()

    def fetch_orderbook(self, asset_id: str) -> Any:
        """Fetch the full-depth L2 orderbook for an ``asset_id`` — Kalshi market
        ticker or Polymarket CTF token id (same convention as ``create_order``)."""
        raw = self._native.fetch_orderbook(asset_id)
        try:
            from openpx._models import Orderbook
            return Orderbook(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_orderbooks_batch(self, asset_ids: list[str]) -> list[Any]:
        """Fetch full-depth L2 orderbooks for multiple asset_ids in one
        round-trip. Cap: 100 on Kalshi; no documented cap on Polymarket."""
        raw = self._native.fetch_orderbooks_batch(asset_ids)
        try:
            from openpx._models import Orderbook
            return [Orderbook(**b) for b in raw]
        except (ImportError, Exception):
            return raw

    def fetch_orderbook_stats(self, asset_id: str) -> Any:
        """Top-of-book stats: best bid/ask, mid, spread (bps), weighted-mid,
        top-10 imbalance, and total bid/ask depth."""
        raw = self._native.fetch_orderbook_stats(asset_id)
        try:
            from openpx._models import OrderbookStats
            return OrderbookStats(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_orderbook_impact(self, asset_id: str, size: float) -> Any:
        """Slippage curve at a single requested ``size``. Walks the book and
        returns partial fills with ``fill_pct < 100.0`` if the side
        exhausts. ``size`` must be > 0."""
        raw = self._native.fetch_orderbook_impact(asset_id, size)
        try:
            from openpx._models import OrderbookImpact
            return OrderbookImpact(**raw)
        except (ImportError, Exception):
            return raw

    def fetch_orderbook_microstructure(self, asset_id: str) -> Any:
        """Microstructure signals: cumulative depth at 10/50/100 bps tiers,
        OLS slope of cumulative size vs distance-from-mid, largest gap, and
        per-side level counts."""
        raw = self._native.fetch_orderbook_microstructure(asset_id)
        try:
            from openpx._models import OrderbookMicrostructure
            return OrderbookMicrostructure(**raw)
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
        asset_id: str,
        *,
        start_ts: Optional[int] = None,
        end_ts: Optional[int] = None,
        limit: Optional[int] = None,
        cursor: Optional[str] = None,
    ) -> dict[str, Any]:
        raw = self._native.fetch_trades(asset_id, start_ts, end_ts, limit, cursor)
        try:
            from openpx._models import MarketTrade
            return {
                "trades": [MarketTrade(**t) for t in raw["trades"]],
                "cursor": raw.get("cursor"),
            }
        except (ImportError, Exception):
            return raw

