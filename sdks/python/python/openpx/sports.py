"""Sports WebSocket streaming for real-time scores and game state."""

from __future__ import annotations

from typing import Any, Iterator

from openpx._native import NativeSportsWebSocket


class SportsWebSocket:
    """Stream real-time sports scores from Polymarket's Sports API.

    No authentication required. Connect and receive all active events.

    Usage::

        from openpx import SportsWebSocket

        ws = SportsWebSocket()
        ws.connect()

        for score in ws.stream():
            print(f"{score['away_team']} @ {score['home_team']}: {score.get('score')}")

        ws.disconnect()
    """

    def __init__(self) -> None:
        self._native = NativeSportsWebSocket()

    def connect(self) -> None:
        """Connect to the sports WebSocket server."""
        self._native.connect()

    def disconnect(self) -> None:
        """Disconnect from the sports WebSocket server."""
        self._native.disconnect()

    @property
    def state(self) -> str:
        """Current connection state (Disconnected, Connecting, Connected, Reconnecting, Closed)."""
        return self._native.state

    def stream(self) -> Iterator[dict[str, Any]]:
        """Returns a blocking iterator of SportResult dicts.

        Each dict contains:
        - ``game_id``: unique game identifier
        - ``league_abbreviation``: league code (e.g. ``nfl``, ``nba``)
        - ``home_team`` / ``away_team``: team names
        - ``status``: game status
        - ``score``: current score (optional)
        - ``live``: whether the game is in progress
        - ``ended``: whether the game has finished
        """
        return self._native.stream()
