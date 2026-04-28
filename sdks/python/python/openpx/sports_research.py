"""Sports research — ESPN data + venue-bridge primitive.

Use this to look up ESPN games, athletes, standings, and other research data,
then call ``markets_for_game`` to find the Kalshi / Polymarket events that
resolve on a specific game.

Example::

    from openpx import Sports

    sports = Sports()

    # Research
    games = sports.list_games(league="nfl")
    game = games[0]

    # Bridge to venues
    venues = sports.markets_for_game(game)
    print(f"{len(venues['kalshi'])} Kalshi events, {len(venues['polymarket'])} Polymarket events")

    # Live state stream
    for state in sports.subscribe_game_state("nfl"):
        print(state["game_id"], state["status"], state.get("score"))
"""

from __future__ import annotations

from typing import Any, Iterator

from openpx._native import NativeGameStateStream, NativeSports


class Sports:
    """Sports research handle. Wraps the ESPN client and the venue-bridge."""

    def __init__(self) -> None:
        self._native = NativeSports()

    def list_sports(self) -> list[dict[str, Any]]:
        """List sports (football, basketball, ...). Returns ``Sport`` dicts."""
        return self._native.list_sports()

    def list_leagues(self, sport_id: str | None = None) -> list[dict[str, Any]]:
        """List leagues, optionally filtered to one sport. Returns ``League`` dicts."""
        return self._native.list_leagues(sport_id)

    def list_games(
        self,
        league: str | None = None,
        date: str | None = None,
        status: str | None = None,
        team: str | None = None,
    ) -> list[dict[str, Any]]:
        """List games matching the filter. Returns ``Game`` dicts.

        :param league: League id (e.g. ``"nfl"``, ``"nba"``). Defaults to NFL.
        :param date: ISO 8601 timestamp; restricts to games on that calendar day.
        :param status: One of ``scheduled``, ``live``, ``final``, ``postponed``,
            ``cancelled``, ``unknown``.
        :param team: Substring match against home/away team names.
        """
        filter_dict: dict[str, Any] = {}
        if league is not None:
            filter_dict["league"] = league
        if date is not None:
            filter_dict["date"] = date
        if status is not None:
            filter_dict["status"] = status
        if team is not None:
            filter_dict["team"] = team
        return self._native.list_games(filter_dict)

    def get_game(self, league: str, game_id: str) -> dict[str, Any]:
        """Fetch a single game by league + ESPN event id."""
        return self._native.get_game(league, game_id)

    def markets_for_game(self, game: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
        """Find the Kalshi and Polymarket events that resolve on this game.

        Returns a dict with ``kalshi`` and ``polymarket`` keys, each a list of
        matching ``Event`` records. Each event carries ``market_ids`` you can
        pass to the venue's ``fetch_market`` call to drill into trade-able markets.
        """
        return self._native.markets_for_game(game)

    def subscribe_game_state(self, league: str) -> Iterator[dict[str, Any]]:
        """Iterator over live ``GameState`` updates. Yields per state change.

        ESPN has no public WebSocket; this is an HTTP-polling adapter
        (default 15s cadence) that suppresses duplicate emissions.
        """
        return self._native.subscribe_game_state(league)


__all__ = ["Sports"]
