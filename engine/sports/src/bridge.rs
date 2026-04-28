//! Bridge primitive: given an ESPN `Game`, find the Kalshi and Polymarket
//! events that resolve on it. The output is keyed by venue so callers can
//! fan out into `fetch_market` calls for whichever venue they trade on.
//!
//! Matching is intentionally simple: extract the last word of each team name
//! (typically the mascot — "Chiefs", "Bills", "Lakers") and look for both
//! tokens in the event title (case-insensitive). Time window is ±36 hours
//! around the game's scheduled start. Misses are expected on edge cases
//! (international tournaments, exotic naming); callers can refine manually.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use px_core::error::OpenPxError;
use px_core::models::Event;
use px_core::{EventsRequest, Exchange};

use crate::types::Game;

const TIME_WINDOW_HOURS: i64 = 36;
const PAGE_LIMIT: usize = 200;

/// Venue events matching an ESPN game. Each `Event` carries `market_ids`;
/// callers fetch full market details via `Exchange::fetch_market(id)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MarketsByVenue {
    pub kalshi: Vec<Event>,
    pub polymarket: Vec<Event>,
}

/// Run the bridge against any pair of `Exchange` implementations. Generic so
/// callers can mock either side in tests.
pub async fn markets_for_game<K, P>(
    game: &Game,
    kalshi: &K,
    polymarket: &P,
) -> Result<MarketsByVenue, OpenPxError>
where
    K: Exchange + Sync,
    P: Exchange + Sync,
{
    // Both venues fetched in parallel — the calls are independent.
    let (k_res, p_res) = tokio::join!(
        kalshi.fetch_events(EventsRequest {
            limit: Some(PAGE_LIMIT),
            status: Some("open".into()),
            ..Default::default()
        }),
        polymarket.fetch_events(EventsRequest {
            limit: Some(PAGE_LIMIT),
            status: Some("open".into()),
            ..Default::default()
        })
    );
    let (k_events, _) = k_res?;
    let (p_events, _) = p_res?;

    Ok(MarketsByVenue {
        kalshi: k_events.into_iter().filter(|e| matches(game, e)).collect(),
        polymarket: p_events.into_iter().filter(|e| matches(game, e)).collect(),
    })
}

fn matches(game: &Game, event: &Event) -> bool {
    let title = event.title.to_lowercase();

    let home_token = mascot_token(game.home_team.as_deref());
    let away_token = mascot_token(game.away_team.as_deref());
    let teams_match = match (home_token, away_token) {
        (Some(h), Some(a)) => title.contains(&h) && title.contains(&a),
        (Some(t), None) | (None, Some(t)) => title.contains(&t),
        (None, None) => false,
    };
    if !teams_match {
        return false;
    }

    match (game.start_time, event_anchor(event)) {
        (Some(g), Some(e)) => (e - g).abs() <= Duration::hours(TIME_WINDOW_HOURS),
        // No start time on either side — fall back to team match alone.
        _ => true,
    }
}

fn mascot_token(name: Option<&str>) -> Option<String> {
    name.and_then(|n| n.split_whitespace().last())
        .map(str::to_lowercase)
        .filter(|t| t.len() >= 3) // skip noise like "FC", "AC"
}

/// Pick the most relevant timestamp from an `Event` for proximity matching.
fn event_anchor(event: &Event) -> Option<chrono::DateTime<Utc>> {
    event.start_ts.or(event.end_ts).or(event.last_updated_ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GameId, GameStatus};
    use chrono::TimeZone;

    fn nfl_game() -> Game {
        Game {
            id: GameId::new("401671793"),
            league: "nfl".into(),
            home_team: Some("Kansas City Chiefs".into()),
            away_team: Some("Buffalo Bills".into()),
            start_time: Some(Utc.with_ymd_and_hms(2025, 1, 26, 23, 30, 0).unwrap()),
            status: GameStatus::Scheduled,
            raw_status: None,
            venue: None,
        }
    }

    fn ev(title: &str, start: chrono::DateTime<Utc>) -> Event {
        Event {
            id: "e".into(),
            slug: None,
            title: title.into(),
            description: None,
            category: Some("Sports".into()),
            series_id: None,
            status: Some("open".into()),
            market_ids: vec!["m1".into()],
            start_ts: Some(start),
            end_ts: None,
            volume: None,
            open_interest: None,
            mutually_exclusive: None,
            last_updated_ts: None,
        }
    }

    #[test]
    fn matches_when_both_mascots_in_title() {
        let g = nfl_game();
        let e = ev("Chiefs vs Bills — AFC Championship", g.start_time.unwrap());
        assert!(matches(&g, &e));
    }

    #[test]
    fn matches_long_form_team_names() {
        let g = nfl_game();
        let e = ev("Kansas City Chiefs at Buffalo Bills", g.start_time.unwrap());
        assert!(matches(&g, &e));
    }

    #[test]
    fn rejects_when_one_team_missing() {
        let g = nfl_game();
        let e = ev("Chiefs vs Ravens — AFC Championship", g.start_time.unwrap());
        assert!(!matches(&g, &e));
    }

    #[test]
    fn rejects_outside_time_window() {
        let g = nfl_game();
        let too_late = g.start_time.unwrap() + Duration::days(7);
        let e = ev("Chiefs vs Bills", too_late);
        assert!(!matches(&g, &e));
    }

    #[test]
    fn allows_match_when_event_has_no_anchor_time() {
        let g = nfl_game();
        let mut e = ev("Chiefs vs Bills", g.start_time.unwrap());
        e.start_ts = None;
        e.end_ts = None;
        e.last_updated_ts = None;
        assert!(matches(&g, &e));
    }
}
