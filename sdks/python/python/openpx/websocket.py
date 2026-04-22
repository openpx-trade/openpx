"""WebSocket streaming for real-time orderbook and activity data."""

from __future__ import annotations

from typing import Any, Iterator, Optional

from openpx._native import NativeWebSocket


class WebSocket:
    """Real-time WebSocket connection to a prediction market exchange.

    Usage::

        from openpx import Exchange

        exchange = Exchange("kalshi", {"api_key_id": "...", "private_key_pem": "..."})
        ws = exchange.websocket()
        ws.connect()
        ws.subscribe("MARKETID")

        for update in ws.updates():
            kind = update["kind"]
            if kind == "Snapshot":
                book = update["book"]
                ...
            elif kind == "Delta":
                changes = update["changes"]
                ...
            elif kind in ("Trade", "Fill"):
                ...

        ws.disconnect()
    """

    def __init__(self, exchange_id: str, config: Optional[dict[str, Any]] = None) -> None:
        self._native = NativeWebSocket(exchange_id, config or {})

    def connect(self) -> None:
        """Connect to the exchange WebSocket server."""
        self._native.connect()

    def disconnect(self) -> None:
        """Disconnect from the WebSocket server."""
        self._native.disconnect()

    def subscribe(self, market_id: str) -> None:
        """Subscribe to orderbook updates for a market."""
        self._native.subscribe(market_id)

    def unsubscribe(self, market_id: str) -> None:
        """Unsubscribe from a market."""
        self._native.unsubscribe(market_id)

    @property
    def state(self) -> str:
        """Current connection state (Disconnected, Connecting, Connected, Reconnecting, Closed)."""
        return self._native.state()

    def updates(self) -> Iterator[dict[str, Any]]:
        """Multiplexed iterator of `WsUpdate` events for all subscribed markets.

        Each item is a tagged dict with a ``kind`` discriminator::

            {"kind": "Snapshot", "market_id": "...", "book": {...}, "exchange_ts": 173..., "local_ts_ms": 173..., "seq": 0}
            {"kind": "Delta",    "market_id": "...", "changes": [...],     "exchange_ts": ..., "local_ts_ms": ..., "seq": 1}
            {"kind": "Trade",    "trade": {...}, "local_ts_ms": ...}
            {"kind": "Fill",     "fill":  {...}, "local_ts_ms": ...}
            {"kind": "Raw",      "exchange": "...", "value": {...}, "local_ts_ms": ...}

        Multiple calls to `updates()` return co-consumers of the same queue —
        each emitted update goes to one receiver, first-grabbed. For fan-out,
        run a single consumer that re-broadcasts.
        """
        return self._native.updates()

    def session_events(self) -> Iterator[dict[str, Any]]:
        """Iterator of connection-level events.

        Distinct from `updates()` so a reconnect is observable as one event,
        not per-market. Items::

            {"kind": "Connected"}
            {"kind": "Reconnected", "gap_ms": 12345}
            {"kind": "Lagged", "dropped": 1, "first_seq": 0, "last_seq": 0}
            {"kind": "BookInvalidated", "market_id": "...", "reason": "Reconnect"}
            {"kind": "Error", "message": "..."}
        """
        return self._native.session_events()
