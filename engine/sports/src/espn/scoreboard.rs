//! Parity-core methods + the ESPN-specific catalog.
//!
//! All methods read from `site.api.espn.com`. Response shapes are best-effort
//! parsed via `serde_json::Value`; ESPN occasionally tweaks field names but
//! keeps the public surface stable enough that this approach holds up.

use chrono::{DateTime, Utc};
use serde_json::Value;

use px_core::error::OpenPxError;

use super::{split_league, Espn};
use crate::types::{Game, GameFilter, GameId, GameStatus, League, Score, Sport};

impl Espn {
    /// List the canonical sports ESPN exposes. Static list — ESPN doesn't
    /// publish a discoverable sports index, so this is hand-maintained.
    pub async fn list_sports(&self) -> Result<Vec<Sport>, OpenPxError> {
        Ok(SUPPORTED_SPORTS
            .iter()
            .map(|(id, name)| Sport {
                id: (*id).into(),
                name: (*name).into(),
            })
            .collect())
    }

    /// List the leagues ESPN exposes within a given sport (or all when `None`).
    /// Same caveat as `list_sports` — hand-maintained taxonomy.
    pub async fn list_leagues(&self, sport_id: Option<&str>) -> Result<Vec<League>, OpenPxError> {
        Ok(SUPPORTED_LEAGUES
            .iter()
            .filter(|(_, sport, _, _)| sport_id.is_none_or(|s| *sport == s))
            .map(|(id, sport, name, abbr)| League {
                id: (*id).into(),
                name: (*name).into(),
                sport_id: (*sport).into(),
                abbreviation: abbr.map(String::from),
            })
            .collect())
    }

    /// List games matching the filter. ESPN scoreboard endpoints are league-
    /// scoped; if `filter.league` is `None`, defaults to NFL.
    pub async fn list_games(&self, filter: GameFilter) -> Result<Vec<Game>, OpenPxError> {
        let league = filter.league.as_deref().unwrap_or("nfl");
        let (sport, lg) = split_league(league);
        let mut path = format!("/{sport}/{lg}/scoreboard");
        if let Some(date) = filter.date {
            path.push_str(&format!("?dates={}", date.format("%Y%m%d")));
        }

        let raw: Value = self.get(&path).await?;
        let events = raw
            .get("events")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let team_filter = filter.team.as_deref().map(str::to_lowercase);

        Ok(events
            .into_iter()
            .filter_map(|e| parse_game(&e, league))
            .filter(|g| match (&filter.status, g.status) {
                (Some(want), got) => *want == got,
                _ => true,
            })
            .filter(|g| match &team_filter {
                Some(t) => team_matches(g, t),
                None => true,
            })
            .collect())
    }

    /// Fetch a single game by id. ESPN's scoreboard endpoint accepts an
    /// `event=` query param for direct lookup.
    pub async fn get_game(&self, league: &str, id: &GameId) -> Result<Game, OpenPxError> {
        let (sport, lg) = split_league(league);
        let path = format!("/{sport}/{lg}/scoreboard?event={}", id.as_str());
        let raw: Value = self.get(&path).await?;
        raw.get("events")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|e| parse_game(e, league))
            .ok_or_else(|| OpenPxError::Other(format!("espn: game {id} not found")))
    }

    // ---------- ESPN-only research extensions ----------

    /// Teams in a league (rosters, conference/division grouping).
    pub async fn teams(&self, league: &str) -> Result<Value, OpenPxError> {
        let (sport, lg) = split_league(league);
        self.get(&format!("/{sport}/{lg}/teams")).await
    }

    /// Single team by id (depth chart, injuries, schedule).
    pub async fn team(&self, league: &str, team_id: &str) -> Result<Value, OpenPxError> {
        let (sport, lg) = split_league(league);
        self.get(&format!("/{sport}/{lg}/teams/{team_id}")).await
    }

    /// Athletes in a league.
    pub async fn athletes(&self, league: &str) -> Result<Value, OpenPxError> {
        let (sport, lg) = split_league(league);
        self.get(&format!("/{sport}/{lg}/athletes")).await
    }

    /// Standings (conference/division). Returns ESPN's raw structure since
    /// the schema varies meaningfully across sports.
    pub async fn standings(&self, league: &str) -> Result<Value, OpenPxError> {
        let (sport, lg) = split_league(league);
        self.get(&format!("/{sport}/{lg}/standings")).await
    }

    /// Schedule for a league on a specific date (UTC). Wraps the scoreboard
    /// endpoint with `dates=YYYYMMDD`.
    pub async fn schedule(
        &self,
        league: &str,
        date: DateTime<Utc>,
    ) -> Result<Vec<Game>, OpenPxError> {
        self.list_games(GameFilter {
            league: Some(league.into()),
            date: Some(date),
            ..Default::default()
        })
        .await
    }

    /// News headlines for a league.
    pub async fn news(&self, league: &str) -> Result<Value, OpenPxError> {
        let (sport, lg) = split_league(league);
        self.get(&format!("/{sport}/{lg}/news")).await
    }

    /// Play-by-play for a single game. ESPN exposes this via the `summary`
    /// endpoint — returns drives/plays plus box-score data.
    pub async fn play_by_play(&self, league: &str, id: &GameId) -> Result<Value, OpenPxError> {
        let (sport, lg) = split_league(league);
        self.get(&format!("/{sport}/{lg}/summary?event={}", id.as_str()))
            .await
    }
}

fn team_matches(g: &Game, t_lower: &str) -> bool {
    let h = g.home_team.as_deref().unwrap_or("").to_lowercase();
    let a = g.away_team.as_deref().unwrap_or("").to_lowercase();
    h.contains(t_lower) || a.contains(t_lower)
}

pub(crate) fn parse_game(event: &Value, league: &str) -> Option<Game> {
    let id = event.get("id")?.as_str()?.to_string();
    let start_time = event
        .get("date")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let competition = event
        .get("competitions")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());

    let (home_team, away_team) = competition
        .map(|c| {
            let competitors = c
                .get("competitors")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let mut home = None;
            let mut away = None;
            for c in &competitors {
                let name = c
                    .get("team")
                    .and_then(|t| t.get("displayName"))
                    .and_then(|v| v.as_str())
                    .map(String::from);
                match c.get("homeAway").and_then(|v| v.as_str()) {
                    Some("home") => home = name,
                    Some("away") => away = name,
                    _ => {}
                }
            }
            (home, away)
        })
        .unwrap_or((None, None));

    let venue = competition
        .and_then(|c| c.get("venue"))
        .and_then(|v| v.get("fullName"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let raw_status = event
        .get("status")
        .and_then(|s| s.get("type"))
        .and_then(|t| t.get("name"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let status = raw_status
        .as_deref()
        .map(map_status)
        .unwrap_or(GameStatus::Unknown);

    Some(Game {
        id: GameId(id),
        league: league.into(),
        home_team,
        away_team,
        start_time,
        status,
        raw_status,
        venue,
    })
}

pub(crate) fn map_status(raw: &str) -> GameStatus {
    match raw {
        "STATUS_SCHEDULED" => GameStatus::Scheduled,
        "STATUS_IN_PROGRESS" | "STATUS_HALFTIME" | "STATUS_END_PERIOD" => GameStatus::Live,
        "STATUS_FINAL" | "STATUS_FULL_TIME" => GameStatus::Final,
        "STATUS_POSTPONED" | "STATUS_DELAYED" | "STATUS_RAIN_DELAY" | "STATUS_SUSPENDED" => {
            GameStatus::Postponed
        }
        "STATUS_CANCELED" | "STATUS_FORFEIT" => GameStatus::Cancelled,
        _ => GameStatus::Unknown,
    }
}

/// Best-effort score parse from competitors[*].score. ESPN ships the score as
/// a numeric string per competitor; we try to combine into Score{home, away}.
pub(crate) fn parse_score(event: &Value) -> Option<Score> {
    let competitors = event
        .get("competitions")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("competitors"))
        .and_then(|v| v.as_array())?;

    let mut home: Option<u32> = None;
    let mut away: Option<u32> = None;
    let mut raw_pair: Option<(String, String)> = None;
    let mut h_raw: Option<String> = None;
    let mut a_raw: Option<String> = None;

    for c in competitors {
        let score = c
            .get("score")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| {
                c.get("score")
                    .and_then(|v| v.as_u64())
                    .map(|n| n.to_string())
            });
        let parsed = score.as_deref().and_then(|s| s.parse::<u32>().ok());
        match c.get("homeAway").and_then(|v| v.as_str()) {
            Some("home") => {
                home = parsed;
                h_raw = score;
            }
            Some("away") => {
                away = parsed;
                a_raw = score;
            }
            _ => {}
        }
    }

    if let (Some(h), Some(a)) = (h_raw, a_raw) {
        raw_pair = Some((h, a));
    }

    let raw = raw_pair.map(|(h, a)| format!("{h}-{a}"));
    Some(Score { home, away, raw })
}

const SUPPORTED_SPORTS: &[(&str, &str)] = &[
    ("football", "Football"),
    ("basketball", "Basketball"),
    ("baseball", "Baseball"),
    ("hockey", "Hockey"),
    ("soccer", "Soccer"),
    ("golf", "Golf"),
    ("racing", "Racing"),
    ("mma", "MMA"),
    ("tennis", "Tennis"),
    ("cricket", "Cricket"),
];

const SUPPORTED_LEAGUES: &[(&str, &str, &str, Option<&str>)] = &[
    ("nfl", "football", "National Football League", Some("NFL")),
    (
        "college-football",
        "football",
        "NCAA Football",
        Some("NCAAF"),
    ),
    (
        "nba",
        "basketball",
        "National Basketball Association",
        Some("NBA"),
    ),
    ("wnba", "basketball", "WNBA", Some("WNBA")),
    (
        "mens-college-basketball",
        "basketball",
        "NCAA Men's Basketball",
        Some("NCAAM"),
    ),
    (
        "womens-college-basketball",
        "basketball",
        "NCAA Women's Basketball",
        Some("NCAAW"),
    ),
    ("mlb", "baseball", "Major League Baseball", Some("MLB")),
    ("nhl", "hockey", "National Hockey League", Some("NHL")),
    ("usa.1", "soccer", "Major League Soccer", Some("MLS")),
    ("eng.1", "soccer", "English Premier League", Some("EPL")),
    ("esp.1", "soccer", "La Liga", None),
    ("ger.1", "soccer", "Bundesliga", None),
    ("ita.1", "soccer", "Serie A", None),
    (
        "uefa.champions",
        "soccer",
        "UEFA Champions League",
        Some("UCL"),
    ),
    ("pga", "golf", "PGA Tour", None),
    ("f1", "racing", "Formula 1", Some("F1")),
    ("ufc", "mma", "Ultimate Fighting Championship", Some("UFC")),
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_nfl_event() {
        let event = json!({
            "id": "401671793",
            "date": "2025-01-26T18:30:00Z",
            "status": { "type": { "name": "STATUS_IN_PROGRESS" } },
            "competitions": [{
                "competitors": [
                    {"homeAway": "home", "team": {"displayName": "Kansas City Chiefs"}, "score": "16"},
                    {"homeAway": "away", "team": {"displayName": "Buffalo Bills"}, "score": "3"}
                ],
                "venue": {"fullName": "Arrowhead Stadium"}
            }]
        });
        let g = parse_game(&event, "nfl").expect("should parse");
        assert_eq!(g.id.as_str(), "401671793");
        assert_eq!(g.home_team.as_deref(), Some("Kansas City Chiefs"));
        assert_eq!(g.away_team.as_deref(), Some("Buffalo Bills"));
        assert_eq!(g.status, GameStatus::Live);
        assert_eq!(g.venue.as_deref(), Some("Arrowhead Stadium"));
        assert!(g.start_time.is_some());

        let s = parse_score(&event).expect("should parse score");
        assert_eq!(s.home, Some(16));
        assert_eq!(s.away, Some(3));
        assert_eq!(s.raw.as_deref(), Some("16-3"));
    }

    #[test]
    fn maps_status_strings() {
        assert_eq!(map_status("STATUS_SCHEDULED"), GameStatus::Scheduled);
        assert_eq!(map_status("STATUS_IN_PROGRESS"), GameStatus::Live);
        assert_eq!(map_status("STATUS_HALFTIME"), GameStatus::Live);
        assert_eq!(map_status("STATUS_FINAL"), GameStatus::Final);
        assert_eq!(map_status("STATUS_POSTPONED"), GameStatus::Postponed);
        assert_eq!(map_status("STATUS_CANCELED"), GameStatus::Cancelled);
        assert_eq!(map_status("UNKNOWN_STRING"), GameStatus::Unknown);
    }

    #[test]
    fn split_league_known_and_unknown() {
        assert_eq!(super::super::split_league("nfl"), ("football", "nfl"));
        assert_eq!(super::super::split_league("nba"), ("basketball", "nba"));
        assert_eq!(
            super::super::split_league("football/nfl"),
            ("football", "nfl")
        );
        assert_eq!(super::super::split_league("epl"), ("soccer", "eng.1"));
    }
}
