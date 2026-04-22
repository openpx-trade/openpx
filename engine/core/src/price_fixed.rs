//! Integer-only string → fixed-point price/qty parsing.
//!
//! Polymarket and Kalshi both wire prices as decimal strings ("0.5432",
//! "100.00", "1.23"). The WS hot path does `s.parse::<f64>()` → build a
//! `FixedPrice::from_f64` which round-trips through f64 and is ~10-100x slower
//! than an integer scan.
//!
//! `parse_price_str` / `parse_qty_str` do the scan directly — no f64, no
//! allocation. Output is a u32/i64 scaled by `SCALE_FACTOR` (10,000). The
//! existing `FixedPrice(u64)` in `models/orderbook.rs` uses the same scale,
//! so the raw output can be handed straight to `FixedPrice::from_raw`.

/// Ticks per unit. 1 tick = 0.0001. Matches the existing `FixedPrice::SCALE`.
pub const SCALE_FACTOR: i64 = 10_000;

/// Integer price type — ticks, 0..=10_000 for prediction market probabilities.
pub type Price = u32;

/// Integer quantity type — ticks of size (0.0001 unit).
pub type Qty = i64;

/// Parse a decimal string "0.5432" to 5432 ticks. Returns `None` on any
/// malformed input. No allocation; no f64 round-trip. Accepts:
///   - integer form:    "50"     → 50 * SCALE_FACTOR (for kalshi cents)
///   - fractional form: "0.5432" → 5432
///   - trailing zeros:  "0.5"    → 5000
///   - negative inputs: rejected (prices are non-negative).
#[inline]
pub fn parse_price_str(s: &str) -> Option<Price> {
    parse_scaled_unsigned(s, SCALE_FACTOR).and_then(|v| u32::try_from(v).ok())
}

/// Parse a decimal string to a `Qty` scaled by `SCALE_FACTOR`. Accepts
/// negative values.
#[inline]
pub fn parse_qty_str(s: &str) -> Option<Qty> {
    parse_scaled_signed(s, SCALE_FACTOR)
}

/// Convenience wrapper every exchange uses: parse a `(price_str, size_str)`
/// pair into a `PriceLevel`. Returns `None` on any malformed input or when
/// either value is non-positive.
///
/// Size is exposed as `f64` for backward compatibility with the public
/// `PriceLevel` type; the parse itself stays in integer ticks.
#[inline]
pub fn parse_level(price: &str, size: &str) -> Option<crate::PriceLevel> {
    let price_raw = parse_price_str(price)?;
    let size_raw = parse_qty_str(size)?;
    if price_raw == 0 || size_raw <= 0 {
        return None;
    }
    let size_f64 = size_raw as f64 / SCALE_FACTOR as f64;
    Some(crate::PriceLevel::with_fixed(
        crate::FixedPrice::from_raw(price_raw as u64),
        size_f64,
    ))
}

fn parse_scaled_unsigned(s: &str, scale: i64) -> Option<u64> {
    let b = s.as_bytes();
    if b.is_empty() {
        return None;
    }
    let (neg, digits) = match b[0] {
        b'-' => (true, &b[1..]),
        b'+' => (false, &b[1..]),
        _ => (false, b),
    };
    if neg || digits.is_empty() {
        return None;
    }
    let signed = parse_scaled_signed_bytes(digits, scale)?;
    if signed < 0 {
        return None;
    }
    Some(signed as u64)
}

fn parse_scaled_signed(s: &str, scale: i64) -> Option<i64> {
    let b = s.as_bytes();
    if b.is_empty() {
        return None;
    }
    let (neg, digits) = match b[0] {
        b'-' => (true, &b[1..]),
        b'+' => (false, &b[1..]),
        _ => (false, b),
    };
    let magnitude = parse_scaled_signed_bytes(digits, scale)?;
    Some(if neg { -magnitude } else { magnitude })
}

fn parse_scaled_signed_bytes(b: &[u8], scale: i64) -> Option<i64> {
    if b.is_empty() {
        return None;
    }
    let mut dot = None;
    for (i, c) in b.iter().enumerate() {
        match c {
            b'0'..=b'9' => {}
            b'.' if dot.is_none() => dot = Some(i),
            _ => return None,
        }
    }
    let (int_part, frac_part) = match dot {
        Some(i) => (&b[..i], &b[i + 1..]),
        None => (b, &b[..0]),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }

    let mut acc: i64 = 0;
    for &c in int_part {
        acc = acc.checked_mul(10)?.checked_add((c - b'0') as i64)?;
    }
    acc = acc.checked_mul(scale)?;

    let mut frac_scale = scale / 10;
    for &c in frac_part {
        if frac_scale == 0 {
            break;
        }
        acc = acc.checked_add((c - b'0') as i64 * frac_scale)?;
        frac_scale /= 10;
    }
    Some(acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fractional() {
        assert_eq!(parse_price_str("0.5432"), Some(5432));
        assert_eq!(parse_price_str("0.5"), Some(5000));
        assert_eq!(parse_price_str("0.0001"), Some(1));
        assert_eq!(parse_price_str("1.0"), Some(10000));
        assert_eq!(parse_price_str("0"), Some(0));
    }

    #[test]
    fn parse_integer() {
        assert_eq!(parse_price_str("50"), Some(500_000));
        assert_eq!(parse_price_str("99"), Some(990_000));
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(parse_price_str(""), None);
        assert_eq!(parse_price_str("abc"), None);
        assert_eq!(parse_price_str("1.2.3"), None);
        assert_eq!(parse_price_str("-1"), None);
    }

    #[test]
    fn qty_handles_signed() {
        assert_eq!(parse_qty_str("100.0"), Some(1_000_000));
        assert_eq!(parse_qty_str("-5.25"), Some(-52_500));
    }

    #[test]
    fn excess_fraction_truncates_to_tick() {
        assert_eq!(parse_price_str("0.54329"), Some(5432));
    }

    #[test]
    fn upper_bound() {
        assert_eq!(parse_price_str("1.0"), Some(10000));
        assert!(parse_price_str("99999999999999999999").is_none());
    }
}
