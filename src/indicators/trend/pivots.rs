//! Pivot Points — daily/session support and resistance levels derived from
//! the prior session's high, low, and close.
//!
//! Three flavours are supplied:
//!
//! - [`pivot_classic`] (Floor) — the textbook formula
//! - [`pivot_fibonacci`] — uses Fibonacci ratios (0.382, 0.618, 1.000)
//! - [`pivot_camarilla`] — tighter intraday levels (1.1/12, 1.1/6, 1.1/4, 1.1/2)
//!
//! These are pure functions of the prior period's `(high, low, close)`. They
//! are not [`Indicator`](crate::indicators::Indicator)s in the streaming
//! sense — pivot levels are computed once per session boundary and used
//! as fixed reference levels for the next session.
//!
//! # Example
//! ```
//! use rsta::indicators::trend::pivots::pivot_classic;
//!
//! let p = pivot_classic(105.0, 95.0, 102.0);
//! // PP = (105 + 95 + 102) / 3 = 100.6666…
//! assert!((p.pp - (302.0 / 3.0)).abs() < 1e-9);
//! // R1 = 2 * PP - L = 2*100.6666 - 95 = 106.3333…
//! assert!((p.r1 - (2.0 * (302.0 / 3.0) - 95.0)).abs() < 1e-9);
//! // S1 = 2 * PP - H
//! assert!((p.s1 - (2.0 * (302.0 / 3.0) - 105.0)).abs() < 1e-9);
//! ```

/// Pivot levels for a single session.
///
/// Carries the central pivot (`pp`) plus three resistance and three support
/// levels. The Camarilla variant also populates `r4` / `s4`; the classic and
/// fibonacci variants leave them at `f64::NAN`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PivotResult {
    /// Central pivot — same formula in all three variants.
    pub pp: f64,
    /// First resistance level.
    pub r1: f64,
    /// Second resistance level.
    pub r2: f64,
    /// Third resistance level.
    pub r3: f64,
    /// Fourth resistance level (Camarilla only; NaN otherwise).
    pub r4: f64,
    /// First support level.
    pub s1: f64,
    /// Second support level.
    pub s2: f64,
    /// Third support level.
    pub s3: f64,
    /// Fourth support level (Camarilla only; NaN otherwise).
    pub s4: f64,
}

/// Classic (Floor) pivot points.
///
/// `PP = (H + L + C) / 3`
/// `R1 = 2*PP - L`,   `S1 = 2*PP - H`
/// `R2 = PP + (H-L)`, `S2 = PP - (H-L)`
/// `R3 = H + 2*(PP-L)`, `S3 = L - 2*(H-PP)`
pub fn pivot_classic(prev_high: f64, prev_low: f64, prev_close: f64) -> PivotResult {
    let pp = (prev_high + prev_low + prev_close) / 3.0;
    let range = prev_high - prev_low;
    PivotResult {
        pp,
        r1: 2.0 * pp - prev_low,
        s1: 2.0 * pp - prev_high,
        r2: pp + range,
        s2: pp - range,
        r3: prev_high + 2.0 * (pp - prev_low),
        s3: prev_low - 2.0 * (prev_high - pp),
        r4: f64::NAN,
        s4: f64::NAN,
    }
}

/// Fibonacci pivot points: `PP ± k * (H - L)` with `k ∈ {0.382, 0.618, 1.0}`.
pub fn pivot_fibonacci(prev_high: f64, prev_low: f64, prev_close: f64) -> PivotResult {
    let pp = (prev_high + prev_low + prev_close) / 3.0;
    let range = prev_high - prev_low;
    PivotResult {
        pp,
        r1: pp + 0.382 * range,
        s1: pp - 0.382 * range,
        r2: pp + 0.618 * range,
        s2: pp - 0.618 * range,
        r3: pp + range,
        s3: pp - range,
        r4: f64::NAN,
        s4: f64::NAN,
    }
}

/// Camarilla pivot points — tight intraday levels around the prior close.
///
/// Uses the multipliers `1.1/12`, `1.1/6`, `1.1/4`, `1.1/2` applied to the
/// prior day's range. `pp` is still `(H + L + C) / 3` for consistency with
/// the other variants.
pub fn pivot_camarilla(prev_high: f64, prev_low: f64, prev_close: f64) -> PivotResult {
    let pp = (prev_high + prev_low + prev_close) / 3.0;
    let range = prev_high - prev_low;
    let m1 = 1.1 / 12.0;
    let m2 = 1.1 / 6.0;
    let m3 = 1.1 / 4.0;
    let m4 = 1.1 / 2.0;
    PivotResult {
        pp,
        r1: prev_close + range * m1,
        s1: prev_close - range * m1,
        r2: prev_close + range * m2,
        s2: prev_close - range * m2,
        r3: prev_close + range * m3,
        s3: prev_close - range * m3,
        r4: prev_close + range * m4,
        s4: prev_close - range * m4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn classic_matches_textbook_formula() {
        let p = pivot_classic(105.0, 95.0, 102.0);
        let pp = 302.0 / 3.0;
        assert!(approx(p.pp, pp));
        assert!(approx(p.r1, 2.0 * pp - 95.0));
        assert!(approx(p.s1, 2.0 * pp - 105.0));
        assert!(approx(p.r2, pp + 10.0));
        assert!(approx(p.s2, pp - 10.0));
        assert!(approx(p.r3, 105.0 + 2.0 * (pp - 95.0)));
        assert!(approx(p.s3, 95.0 - 2.0 * (105.0 - pp)));
        assert!(p.r4.is_nan() && p.s4.is_nan());
    }

    #[test]
    fn fibonacci_uses_canonical_ratios() {
        let p = pivot_fibonacci(110.0, 100.0, 105.0);
        let pp = 315.0 / 3.0;
        let range = 10.0;
        assert!(approx(p.pp, pp));
        assert!(approx(p.r1, pp + 0.382 * range));
        assert!(approx(p.s1, pp - 0.382 * range));
        assert!(approx(p.r2, pp + 0.618 * range));
        assert!(approx(p.s2, pp - 0.618 * range));
        assert!(approx(p.r3, pp + range));
        assert!(approx(p.s3, pp - range));
    }

    #[test]
    fn camarilla_levels_are_anchored_on_prev_close() {
        let p = pivot_camarilla(110.0, 100.0, 105.0);
        let range = 10.0;
        assert!(approx(p.r1, 105.0 + range * 1.1 / 12.0));
        assert!(approx(p.s1, 105.0 - range * 1.1 / 12.0));
        assert!(approx(p.r4, 105.0 + range * 1.1 / 2.0));
        assert!(approx(p.s4, 105.0 - range * 1.1 / 2.0));
    }

    #[test]
    fn ordering_invariants_hold_for_all_variants() {
        for builder in [pivot_classic, pivot_fibonacci, pivot_camarilla] {
            let p = builder(110.0, 100.0, 105.0);
            assert!(p.r1 > p.pp, "R1 should be above PP");
            assert!(p.s1 < p.pp, "S1 should be below PP");
            assert!(p.r2 >= p.r1, "R2 should be >= R1");
            assert!(p.s2 <= p.s1, "S2 should be <= S1");
            assert!(p.r3 >= p.r2, "R3 should be >= R2");
            assert!(p.s3 <= p.s2, "S3 should be <= S2");
        }
    }
}
