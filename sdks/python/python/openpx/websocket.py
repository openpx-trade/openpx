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

        for update in ws.orderbook_stream("MARKETID"):
            print(update)

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

    def orderbook_stream(self, market_id: str) -> Iterator[dict[str, Any]]:
        """Returns a blocking iterator of WsMessage-wrapped orderbook updates.

        Each item is a ``WsMessage`` envelope dict::

            {
                "seq": 1,              # per-market monotonic sequence number
                "exchange_time": ...,  # server timestamp (use for ordering)
                "received_at": ...,    # local capture time (use for latency)
                "data": {              # OrderbookUpdate payload
                    "type": "Snapshot",
                    "Snapshot": {...}
                }
            }

        The ``data`` field contains one of:
        - ``{"type": "Snapshot", "Snapshot": {...}}`` — full orderbook
        - ``{"type": "Delta", "Delta": {"changes": [...], "timestamp": "..."}}`` — incremental
        - ``{"type": "Reconnected"}`` — connection was re-established, state is stale
        """
        return self._native.orderbook_stream(market_id)

    def activity_stream(self, market_id: str) -> Iterator[dict[str, Any]]:
        """Returns a blocking iterator of WsMessage-wrapped activity events.

        Each item is a ``WsMessage`` envelope dict::

            {
                "seq": 1,              # per-market monotonic sequence number
                "exchange_time": ...,  # server timestamp (use for ordering)
                "received_at": ...,    # local capture time (use for latency)
                "data": {              # ActivityEvent payload
                    "Trade": {...}
                }
            }

        The ``data`` field contains one of:
        - ``{"Trade": {...}}`` — market trade
        - ``{"Fill": {...}}`` — user fill (requires authenticated config)
        """
        return self._native.activity_stream(market_id)
