"""WebSocket streaming for real-time orderbook and activity data."""

from __future__ import annotations

from typing import Any, Iterator, Optional

from openpx._native import NativeWebSocket


class WebSocket:
    """Real-time WebSocket connection to a prediction market exchange.

    Usage with structural pattern matching::

        from openpx import Exchange, Snapshot, Delta, Trade, Fill

        exchange = Exchange("kalshi", {"api_key_id": "...", "private_key_pem": "..."})
        ws = exchange.websocket()
        ws.connect()
        ws.subscribe("MARKETID")

        for update in ws.updates():
            match update:
                case Snapshot(market_id, book, exchange_ts, local_ts_ms, seq):
                    ...
                case Delta(market_id, changes, exchange_ts, local_ts_ms, seq):
                    ...
                case Trade(trade, local_ts_ms):
                    ...
                case Fill(fill, local_ts_ms):
                    ...

        ws.disconnect()

    Or classic isinstance dispatch::

        if isinstance(update, Snapshot):
            book = update.book
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

    def updates(self) -> Iterator[Any]:
        """Iterator of `WsUpdate` events for all subscribed markets.

        Each item is an instance of `Snapshot`, `Delta`, `Trade`, or `Fill`
        (importable from `openpx`). Use `match` or `isinstance` to
        dispatch — every variant exposes the standard `__match_args__`
        positional binding plus named attributes.

        Single-consumer: calling twice on the same WebSocket raises. The
        underlying channel is MPMC at the transport layer, but cloning a
        receiver would silently split messages between holders — a second
        "debug sidecar" reader would quietly eat half the stream. Run one
        consumer that re-broadcasts if you need fan-out.
        """
        return self._native.updates()

    def session_events(self) -> Iterator[Any]:
        """Iterator of connection-level events. Single-consumer, take-once.

        Items are instances of `Connected`, `Reconnected`, `Lagged`,
        `BookInvalidated`, or `SessionError`. Distinct from `updates()`
        so one reconnect is observable as one event, not per-market.
        """
        return self._native.session_events()
