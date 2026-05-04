//! Helpers for working with rolling-series prediction markets.
//!
//! A "rolling series" is a stream of short-lived markets that open and
//! close on a fixed cadence — Kalshi's 15-minute BTC up/down series
//! (`KXBTC15M-<DATE><HHMM>`), Polymarket's 5-minute BTC up/down event
//! sequence (`btc-updown-5m-<unix-ts>`), etc. HFT consumers want one
//! continuously-updating book and need to roll their WS subscription
//! forward as each market closes.
//!
//! `pick_active_market` is the pure-function piece: given a list of
//! markets in a series and the current time, return the one to listen
//! to right now. Pair with `Exchange::fetch_markets` filtered by
//! `series_ticker` (Kalshi) / `event_ticker` (Polymarket).

use crate::models::{Market, MarketStatus};
use chrono::{DateTime, Utc};

/// Pick the currently-active market in a list — `Active` status with a
/// future close time. When several qualify, returns the one with the
/// soonest `close_time` (in a rolling series this is the next-to-resolve
/// market — i.e. the one a trader should be subscribed to). Markets
/// without a `close_time` are treated as still open (Polymarket events
/// occasionally omit it).
pub fn pick_active_market<'a>(
    markets: &'a [Market],
    now: DateTime<Utc>,
) -> Option<&'a Market> {
    markets
        .iter()
        .filter(|m| {
            matches!(m.status, MarketStatus::Active)
                && m.close_time.is_none_or(|t| t > now)
        })
        .min_by_key(|m| m.close_time.unwrap_or(DateTime::<Utc>::MAX_UTC))
}

/// Active market plus the next one queued behind it, sorted by ascending
/// close time. Useful for zero-downtime rollover: subscribe to both,
/// drop the front one when its `close_time` passes.
pub fn pick_active_and_next<'a>(
    markets: &'a [Market],
    now: DateTime<Utc>,
) -> (Option<&'a Market>, Option<&'a Market>) {
    let mut active: Vec<&Market> = markets
        .iter()
        .filter(|m| {
            matches!(m.status, MarketStatus::Active)
                && m.close_time.is_none_or(|t| t > now)
        })
        .collect();
    active.sort_by_key(|m| m.close_time.unwrap_or(DateTime::<Utc>::MAX_UTC));
    let mut iter = active.into_iter();
    (iter.next(), iter.next())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_market(ticker: &str, status: MarketStatus, close_minutes: Option<i64>) -> Market {
        Market {
            ticker: ticker.into(),
            status,
            close_time: close_minutes.map(|m| Utc::now() + chrono::Duration::minutes(m)),
            ..Default::default()
        }
    }

    #[test]
    fn picks_soonest_active() {
        let now = Utc::now();
        let markets = vec![
            fake_market("a", MarketStatus::Active, Some(30)),
            fake_market("b", MarketStatus::Active, Some(5)),
            fake_market("c", MarketStatus::Active, Some(20)),
        ];
        let pick = pick_active_market(&markets, now).unwrap();
        assert_eq!(pick.ticker, "b");
    }

    #[test]
    fn skips_closed() {
        let now = Utc::now();
        let markets = vec![
            fake_market("a", MarketStatus::Closed, Some(5)),
            fake_market("b", MarketStatus::Active, Some(20)),
        ];
        let pick = pick_active_market(&markets, now).unwrap();
        assert_eq!(pick.ticker, "b");
    }

    #[test]
    fn skips_already_past_close_time() {
        let now = Utc::now();
        let markets = vec![
            fake_market("a", MarketStatus::Active, Some(-1)),
            fake_market("b", MarketStatus::Active, Some(10)),
        ];
        let pick = pick_active_market(&markets, now).unwrap();
        assert_eq!(pick.ticker, "b");
    }

    #[test]
    fn pick_active_and_next_orders() {
        let now = Utc::now();
        let markets = vec![
            fake_market("third", MarketStatus::Active, Some(45)),
            fake_market("first", MarketStatus::Active, Some(5)),
            fake_market("second", MarketStatus::Active, Some(20)),
        ];
        let (a, b) = pick_active_and_next(&markets, now);
        assert_eq!(a.unwrap().ticker, "first");
        assert_eq!(b.unwrap().ticker, "second");
    }

    #[test]
    fn empty_when_none_active() {
        let now = Utc::now();
        let markets = vec![fake_market("a", MarketStatus::Resolved, Some(5))];
        assert!(pick_active_market(&markets, now).is_none());
    }
}
