"""OpenPX — Unified Python SDK for prediction markets."""

from openpx.exchange import Exchange
from openpx.websocket import WebSocket
from openpx.sports import SportsWebSocket
from openpx.crypto import CryptoPriceWebSocket
from openpx._native import (
    OpenPxError,
    NetworkError,
    ExchangeError,
    AuthenticationError,
    # WsUpdate variants — isinstance-matchable
    Snapshot,
    Delta,
    Trade,
    Fill,
    # SessionEvent variants
    Connected,
    Reconnected,
    Lagged,
    BookInvalidated,
    SessionError,
)

__all__ = [
    "Exchange",
    "WebSocket",
    "SportsWebSocket",
    "CryptoPriceWebSocket",
    "OpenPxError",
    "NetworkError",
    "ExchangeError",
    "AuthenticationError",
    "Snapshot",
    "Delta",
    "Trade",
    "Fill",
    "Connected",
    "Reconnected",
    "Lagged",
    "BookInvalidated",
    "SessionError",
]
