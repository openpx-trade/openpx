"""Crypto Price WebSocket streaming for real-time prices."""

from __future__ import annotations

from typing import Any, Iterator, Sequence

from openpx._native import NativeCryptoPriceWebSocket


class CryptoPriceWebSocket:
    """Stream real-time crypto prices from Binance or Chainlink.

    No authentication required. Requires explicit subscribe/unsubscribe.

    Usage::

        from openpx import CryptoPriceWebSocket

        ws = CryptoPriceWebSocket()
        ws.connect()
        ws.subscribe("binance", ["btcusdt", "ethusdt"])

        for price in ws.stream():
            print(f"{price['symbol']} = {price['value']}")

        ws.disconnect()
    """

    def __init__(self) -> None:
        self._native = NativeCryptoPriceWebSocket()

    def connect(self) -> None:
        """Connect to the crypto price WebSocket server."""
        self._native.connect()

    def disconnect(self) -> None:
        """Disconnect from the crypto price WebSocket server."""
        self._native.disconnect()

    def subscribe(self, source: str, symbols: Sequence[str] = ()) -> None:
        """Subscribe to crypto price updates.

        Args:
            source: Price source — ``"binance"`` or ``"chainlink"``.
            symbols: Symbols to subscribe to. Empty subscribes to all.
        """
        self._native.subscribe(source, list(symbols))

    def unsubscribe(self, source: str, symbols: Sequence[str] = ()) -> None:
        """Unsubscribe from crypto price updates.

        Args:
            source: Price source — ``"binance"`` or ``"chainlink"``.
            symbols: Symbols to unsubscribe from.
        """
        self._native.unsubscribe(source, list(symbols))

    @property
    def state(self) -> str:
        """Current connection state (Disconnected, Connecting, Connected, Reconnecting, Closed)."""
        return self._native.state

    def stream(self) -> Iterator[dict[str, Any]]:
        """Returns a blocking iterator of CryptoPrice dicts.

        Each dict contains:
        - ``symbol``: trading pair or price feed identifier
        - ``timestamp``: unix timestamp
        - ``value``: current price
        - ``source``: ``"binance"`` or ``"chainlink"``
        """
        return self._native.stream()
