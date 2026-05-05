//! Geometric detection of common candlestick patterns.
//!
//! Each detector is a pure function over one, two, or three [`Candle`]
//! values. There is no internal state and no trend context — most
//! reversal interpretations (e.g. *hammer* vs *hanging man*) depend on
//! the prior trend, which the caller must track. The function names
//! follow the canonical TA terminology; aliases that share geometry
//! point to the same impl with a doc note.
//!
//! ## Conventions
//!
//! - **Body** = `|close − open|`.
//! - **Upper wick** = `high − max(open, close)`.
//! - **Lower wick** = `min(open, close) − low`.
//! - **Range** = `high − low`. A zero-range candle (high == low) cannot
//!   be classified — every detector returns `false` in that degenerate
//!   case.
//! - "Bullish" candle = `close >= open`; "bearish" = `close < open`.
//!
//! Thresholds are exposed via [`PatternConfig`] for tunability;
//! convenience top-level functions use the canonical defaults
//! (`PatternConfig::default()`).
//!
//! ## High-level scan
//!
//! [`detect_at`] runs every applicable detector on the trailing window
//! of a candle slice and returns a `Vec<Pattern>` describing every
//! pattern whose final bar is the last candle in the window.

use crate::indicators::Candle;

/// Bias direction implied by a pattern in its canonical context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bias {
    /// Pattern hints at upward continuation or reversal up.
    Bullish,
    /// Pattern hints at downward continuation or reversal down.
    Bearish,
    /// Indecision (e.g. doji).
    Neutral,
}

/// Identified pattern, returned by [`detect_at`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    Doji,
    Hammer,
    InvertedHammer,
    ShootingStar,
    HangingMan,
    BullishMarubozu,
    BearishMarubozu,
    BullishEngulfing,
    BearishEngulfing,
    BullishHarami,
    BearishHarami,
    MorningStar,
    EveningStar,
    ThreeWhiteSoldiers,
    ThreeBlackCrows,
}

/// `(kind, bias)` pair returned by [`detect_at`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pattern {
    pub kind: PatternKind,
    pub bias: Bias,
}

/// Tunable thresholds for the geometric tests. Sensible defaults via
/// [`PatternConfig::default`].
#[derive(Debug, Clone, Copy)]
pub struct PatternConfig {
    /// Maximum body-to-range ratio for a Doji. Default 0.1.
    pub doji_body_ratio: f64,
    /// Minimum body-to-range ratio for a Marubozu. Default 0.95.
    pub marubozu_body_ratio: f64,
    /// Minimum lower-wick-to-body ratio for a Hammer / Hanging Man.
    /// Default 2.0.
    pub hammer_wick_ratio: f64,
    /// Maximum upper-wick-to-body ratio for a Hammer (tiny upper wick
    /// expected). Default 0.5.
    pub hammer_upper_wick_ratio: f64,
    /// Star: max body ratio for the middle bar (small/indecision).
    /// Default 0.3.
    pub star_middle_body_ratio: f64,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            doji_body_ratio: 0.1,
            marubozu_body_ratio: 0.95,
            hammer_wick_ratio: 2.0,
            hammer_upper_wick_ratio: 0.5,
            star_middle_body_ratio: 0.3,
        }
    }
}

// ---------------------------------------------------------------------------
// Geometric helpers
// ---------------------------------------------------------------------------

#[inline]
fn body(c: &Candle) -> f64 {
    (c.close - c.open).abs()
}

#[inline]
fn range(c: &Candle) -> f64 {
    c.high - c.low
}

#[inline]
fn upper_wick(c: &Candle) -> f64 {
    c.high - c.open.max(c.close)
}

#[inline]
fn lower_wick(c: &Candle) -> f64 {
    c.open.min(c.close) - c.low
}

#[inline]
fn is_bullish(c: &Candle) -> bool {
    c.close >= c.open
}

#[inline]
fn is_bearish(c: &Candle) -> bool {
    c.close < c.open
}

// ---------------------------------------------------------------------------
// 1-candle patterns
// ---------------------------------------------------------------------------

/// **Doji** — open and close are nearly equal. Indecision pattern.
pub fn is_doji(c: &Candle) -> bool {
    is_doji_with(c, &PatternConfig::default())
}

/// [`is_doji`] with a custom threshold.
pub fn is_doji_with(c: &Candle, cfg: &PatternConfig) -> bool {
    let r = range(c);
    if r <= 0.0 {
        return false;
    }
    body(c) / r <= cfg.doji_body_ratio
}

/// **Hammer** — small body in the upper half of the range, long lower
/// wick (≥ `hammer_wick_ratio × body`), tiny upper wick. In a *downtrend*
/// signals a bullish reversal; in an *uptrend* the same shape is a
/// **Hanging Man** (bearish — see [`is_hanging_man`]). The geometric
/// test is identical; only prior-trend context differs.
pub fn is_hammer(c: &Candle) -> bool {
    is_hammer_with(c, &PatternConfig::default())
}

/// [`is_hammer`] with custom thresholds.
pub fn is_hammer_with(c: &Candle, cfg: &PatternConfig) -> bool {
    let b = body(c);
    if b <= 0.0 {
        return false;
    }
    let lw = lower_wick(c);
    let uw = upper_wick(c);
    lw >= cfg.hammer_wick_ratio * b && uw <= cfg.hammer_upper_wick_ratio * b
}

/// **Hanging Man** — same shape as [`is_hammer`]; the bearish
/// interpretation only applies after an uptrend, which the caller is
/// responsible for confirming.
pub fn is_hanging_man(c: &Candle) -> bool {
    is_hammer(c)
}

/// **Inverted Hammer** — small body in the lower half, long upper wick,
/// tiny lower wick. After a downtrend, signals potential bullish
/// reversal; after an uptrend, the same shape is a **Shooting Star**
/// (bearish — see [`is_shooting_star`]).
pub fn is_inverted_hammer(c: &Candle) -> bool {
    is_inverted_hammer_with(c, &PatternConfig::default())
}

/// [`is_inverted_hammer`] with custom thresholds.
pub fn is_inverted_hammer_with(c: &Candle, cfg: &PatternConfig) -> bool {
    let b = body(c);
    if b <= 0.0 {
        return false;
    }
    let lw = lower_wick(c);
    let uw = upper_wick(c);
    uw >= cfg.hammer_wick_ratio * b && lw <= cfg.hammer_upper_wick_ratio * b
}

/// **Shooting Star** — same shape as [`is_inverted_hammer`]; bearish
/// only after an uptrend.
pub fn is_shooting_star(c: &Candle) -> bool {
    is_inverted_hammer(c)
}

/// **Bullish Marubozu** — bullish (close > open) candle with a body that
/// fills almost the entire range (no/tiny wicks). Strong continuation.
pub fn is_bullish_marubozu(c: &Candle) -> bool {
    is_bullish_marubozu_with(c, &PatternConfig::default())
}

/// [`is_bullish_marubozu`] with a custom threshold.
pub fn is_bullish_marubozu_with(c: &Candle, cfg: &PatternConfig) -> bool {
    if !is_bullish(c) {
        return false;
    }
    let r = range(c);
    if r <= 0.0 {
        return false;
    }
    body(c) / r >= cfg.marubozu_body_ratio
}

/// **Bearish Marubozu** — bearish candle filling almost the whole range.
pub fn is_bearish_marubozu(c: &Candle) -> bool {
    is_bearish_marubozu_with(c, &PatternConfig::default())
}

/// [`is_bearish_marubozu`] with a custom threshold.
pub fn is_bearish_marubozu_with(c: &Candle, cfg: &PatternConfig) -> bool {
    if !is_bearish(c) {
        return false;
    }
    let r = range(c);
    if r <= 0.0 {
        return false;
    }
    body(c) / r >= cfg.marubozu_body_ratio
}

// ---------------------------------------------------------------------------
// 2-candle patterns
// ---------------------------------------------------------------------------

/// **Bullish Engulfing** — bearish bar followed by a bullish bar whose
/// body completely engulfs the previous body.
pub fn is_bullish_engulfing(prev: &Candle, curr: &Candle) -> bool {
    if !is_bearish(prev) || !is_bullish(curr) {
        return false;
    }
    curr.open <= prev.close && curr.close >= prev.open
}

/// **Bearish Engulfing** — bullish bar followed by a bearish bar whose
/// body engulfs the previous body.
pub fn is_bearish_engulfing(prev: &Candle, curr: &Candle) -> bool {
    if !is_bullish(prev) || !is_bearish(curr) {
        return false;
    }
    curr.open >= prev.close && curr.close <= prev.open
}

/// **Bullish Harami** — large bearish bar followed by a small bullish
/// bar whose body sits inside the previous body.
pub fn is_bullish_harami(prev: &Candle, curr: &Candle) -> bool {
    if !is_bearish(prev) || !is_bullish(curr) {
        return false;
    }
    let prev_high_body = prev.open;
    let prev_low_body = prev.close;
    curr.open >= prev_low_body && curr.close <= prev_high_body && body(curr) < body(prev)
}

/// **Bearish Harami** — mirror of [`is_bullish_harami`].
pub fn is_bearish_harami(prev: &Candle, curr: &Candle) -> bool {
    if !is_bullish(prev) || !is_bearish(curr) {
        return false;
    }
    let prev_high_body = prev.close;
    let prev_low_body = prev.open;
    curr.close >= prev_low_body && curr.open <= prev_high_body && body(curr) < body(prev)
}

// ---------------------------------------------------------------------------
// 3-candle patterns
// ---------------------------------------------------------------------------

/// **Morning Star** — three-bar bullish reversal: bearish, then a small
/// indecision body, then a bullish bar that closes above the midpoint
/// of the first bar's body.
pub fn is_morning_star(c1: &Candle, c2: &Candle, c3: &Candle) -> bool {
    is_morning_star_with(c1, c2, c3, &PatternConfig::default())
}

/// [`is_morning_star`] with custom middle-body threshold.
pub fn is_morning_star_with(c1: &Candle, c2: &Candle, c3: &Candle, cfg: &PatternConfig) -> bool {
    if !is_bearish(c1) || !is_bullish(c3) {
        return false;
    }
    let r2 = range(c2);
    if r2 <= 0.0 {
        return false;
    }
    if body(c2) / r2 > cfg.star_middle_body_ratio {
        return false;
    }
    let midpoint_c1 = (c1.open + c1.close) / 2.0;
    c3.close > midpoint_c1
}

/// **Evening Star** — mirror of [`is_morning_star`] (bearish reversal).
pub fn is_evening_star(c1: &Candle, c2: &Candle, c3: &Candle) -> bool {
    is_evening_star_with(c1, c2, c3, &PatternConfig::default())
}

/// [`is_evening_star`] with custom middle-body threshold.
pub fn is_evening_star_with(c1: &Candle, c2: &Candle, c3: &Candle, cfg: &PatternConfig) -> bool {
    if !is_bullish(c1) || !is_bearish(c3) {
        return false;
    }
    let r2 = range(c2);
    if r2 <= 0.0 {
        return false;
    }
    if body(c2) / r2 > cfg.star_middle_body_ratio {
        return false;
    }
    let midpoint_c1 = (c1.open + c1.close) / 2.0;
    c3.close < midpoint_c1
}

/// **Three White Soldiers** — three consecutive bullish bars, each
/// closing higher than the previous, opening within the previous body.
pub fn is_three_white_soldiers(c1: &Candle, c2: &Candle, c3: &Candle) -> bool {
    if !(is_bullish(c1) && is_bullish(c2) && is_bullish(c3)) {
        return false;
    }
    if !(c2.close > c1.close && c3.close > c2.close) {
        return false;
    }
    // Each opens inside the previous body (basic version).
    c2.open >= c1.open && c2.open <= c1.close && c3.open >= c2.open && c3.open <= c2.close
}

/// **Three Black Crows** — mirror of [`is_three_white_soldiers`].
pub fn is_three_black_crows(c1: &Candle, c2: &Candle, c3: &Candle) -> bool {
    if !(is_bearish(c1) && is_bearish(c2) && is_bearish(c3)) {
        return false;
    }
    if !(c2.close < c1.close && c3.close < c2.close) {
        return false;
    }
    c2.open <= c1.open && c2.open >= c1.close && c3.open <= c2.open && c3.open >= c2.close
}

// ---------------------------------------------------------------------------
// High-level scan
// ---------------------------------------------------------------------------

/// Run every detector against the trailing window of `candles` and
/// return the patterns whose final bar is the last candle in the slice.
///
/// The window is read from the *end* of the slice — `candles.last()` is
/// the "current" bar. Patterns of length 2 or 3 silently skip if the
/// slice is shorter than required.
///
/// Order of the returned vec is the order of detection (1-bar, 2-bar,
/// 3-bar). Multiple non-exclusive patterns can fire on the same bar
/// (e.g. *Hammer* and *Doji* if the body is tiny enough).
pub fn detect_at(candles: &[Candle]) -> Vec<Pattern> {
    detect_at_with(candles, &PatternConfig::default())
}

/// [`detect_at`] with custom thresholds.
pub fn detect_at_with(candles: &[Candle], cfg: &PatternConfig) -> Vec<Pattern> {
    let mut out = Vec::new();
    let Some(curr) = candles.last() else {
        return out;
    };

    // 1-bar patterns
    if is_doji_with(curr, cfg) {
        out.push(Pattern {
            kind: PatternKind::Doji,
            bias: Bias::Neutral,
        });
    }
    if is_hammer_with(curr, cfg) {
        // Same geometry as Hanging Man — caller decides which from trend
        // context. Report Hammer as the canonical bullish form.
        out.push(Pattern {
            kind: PatternKind::Hammer,
            bias: Bias::Bullish,
        });
    }
    if is_inverted_hammer_with(curr, cfg) {
        out.push(Pattern {
            kind: PatternKind::InvertedHammer,
            bias: Bias::Bullish,
        });
    }
    if is_bullish_marubozu_with(curr, cfg) {
        out.push(Pattern {
            kind: PatternKind::BullishMarubozu,
            bias: Bias::Bullish,
        });
    }
    if is_bearish_marubozu_with(curr, cfg) {
        out.push(Pattern {
            kind: PatternKind::BearishMarubozu,
            bias: Bias::Bearish,
        });
    }

    // 2-bar patterns
    if candles.len() >= 2 {
        let prev = &candles[candles.len() - 2];
        if is_bullish_engulfing(prev, curr) {
            out.push(Pattern {
                kind: PatternKind::BullishEngulfing,
                bias: Bias::Bullish,
            });
        }
        if is_bearish_engulfing(prev, curr) {
            out.push(Pattern {
                kind: PatternKind::BearishEngulfing,
                bias: Bias::Bearish,
            });
        }
        if is_bullish_harami(prev, curr) {
            out.push(Pattern {
                kind: PatternKind::BullishHarami,
                bias: Bias::Bullish,
            });
        }
        if is_bearish_harami(prev, curr) {
            out.push(Pattern {
                kind: PatternKind::BearishHarami,
                bias: Bias::Bearish,
            });
        }
    }

    // 3-bar patterns
    if candles.len() >= 3 {
        let c1 = &candles[candles.len() - 3];
        let c2 = &candles[candles.len() - 2];
        let c3 = curr;
        if is_morning_star_with(c1, c2, c3, cfg) {
            out.push(Pattern {
                kind: PatternKind::MorningStar,
                bias: Bias::Bullish,
            });
        }
        if is_evening_star_with(c1, c2, c3, cfg) {
            out.push(Pattern {
                kind: PatternKind::EveningStar,
                bias: Bias::Bearish,
            });
        }
        if is_three_white_soldiers(c1, c2, c3) {
            out.push(Pattern {
                kind: PatternKind::ThreeWhiteSoldiers,
                bias: Bias::Bullish,
            });
        }
        if is_three_black_crows(c1, c2, c3) {
            out.push(Pattern {
                kind: PatternKind::ThreeBlackCrows,
                bias: Bias::Bearish,
            });
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            timestamp: 0,
            open,
            high,
            low,
            close,
            volume: 1.0,
        }
    }

    // ----- 1-bar patterns -----

    #[test]
    fn doji_open_close_equal() {
        // body = 0, range = 4 → doji.
        assert!(is_doji(&c(100.0, 102.0, 98.0, 100.0)));
    }

    #[test]
    fn doji_rejects_wide_body() {
        assert!(!is_doji(&c(100.0, 105.0, 95.0, 104.0)));
    }

    #[test]
    fn doji_rejects_zero_range() {
        assert!(!is_doji(&c(100.0, 100.0, 100.0, 100.0)));
    }

    #[test]
    fn hammer_long_lower_wick_small_body() {
        // body 0.5, lower wick 5, upper wick 0.5 → hammer.
        assert!(is_hammer(&c(100.0, 100.5, 95.0, 100.5)));
    }

    #[test]
    fn hammer_rejects_long_upper_wick() {
        assert!(!is_hammer(&c(100.0, 105.0, 99.5, 100.5)));
    }

    #[test]
    fn inverted_hammer_long_upper_wick() {
        // body 0.5, upper wick 5, lower wick 0 → inverted hammer / shooting star.
        // Lower wick must be <= hammer_upper_wick_ratio × body = 0.25 by default.
        assert!(is_inverted_hammer(&c(100.0, 105.5, 100.0, 100.5)));
        assert!(is_shooting_star(&c(100.0, 105.5, 100.0, 100.5)));
    }

    #[test]
    fn marubozu_full_body() {
        // body fills 99% of range → bullish marubozu.
        assert!(is_bullish_marubozu(&c(100.0, 110.0, 99.95, 109.95)));
        // Mirror: bearish marubozu.
        assert!(is_bearish_marubozu(&c(110.0, 110.05, 100.0, 100.0)));
    }

    #[test]
    fn marubozu_rejects_wicky_candle() {
        assert!(!is_bullish_marubozu(&c(100.0, 115.0, 95.0, 108.0)));
    }

    // ----- 2-bar patterns -----

    #[test]
    fn bullish_engulfing_classic() {
        let prev = c(105.0, 106.0, 100.0, 101.0); // bearish: open 105, close 101
        let curr = c(100.0, 108.0, 99.0, 107.0); // bullish: open 100, close 107
        assert!(is_bullish_engulfing(&prev, &curr));
    }

    #[test]
    fn bullish_engulfing_rejects_inside_body() {
        let prev = c(105.0, 106.0, 100.0, 101.0);
        let curr = c(102.0, 105.0, 101.5, 104.0); // bullish but doesn't engulf
        assert!(!is_bullish_engulfing(&prev, &curr));
    }

    #[test]
    fn bearish_engulfing_classic() {
        let prev = c(101.0, 105.0, 100.0, 104.0); // bullish
        let curr = c(105.0, 106.0, 99.0, 100.0); // bearish: opens above prev close,
                                                 // closes below prev open
        assert!(is_bearish_engulfing(&prev, &curr));
    }

    #[test]
    fn bullish_harami_inside_body() {
        let prev = c(110.0, 111.0, 100.0, 101.0); // big bearish, body 110→101
        let curr = c(103.0, 106.0, 102.0, 105.0); // small bullish inside
        assert!(is_bullish_harami(&prev, &curr));
    }

    #[test]
    fn bearish_harami_mirror() {
        let prev = c(101.0, 111.0, 100.0, 110.0); // big bullish 101→110
        let curr = c(108.0, 109.0, 104.0, 105.0); // small bearish inside
        assert!(is_bearish_harami(&prev, &curr));
    }

    // ----- 3-bar patterns -----

    #[test]
    fn morning_star_textbook() {
        let c1 = c(110.0, 111.0, 100.0, 101.0); // big bear
        let c2 = c(99.0, 100.0, 98.0, 99.5); // small body (indecision)
        let c3 = c(100.0, 110.0, 99.0, 108.0); // big bull, closes above midpoint(c1)=105.5
        assert!(is_morning_star(&c1, &c2, &c3));
    }

    #[test]
    fn morning_star_rejects_high_body_middle() {
        let c1 = c(110.0, 111.0, 100.0, 101.0);
        let c2 = c(99.0, 105.0, 98.0, 104.0); // body too big
        let c3 = c(100.0, 110.0, 99.0, 108.0);
        assert!(!is_morning_star(&c1, &c2, &c3));
    }

    #[test]
    fn evening_star_mirror() {
        let c1 = c(101.0, 111.0, 100.0, 110.0);
        let c2 = c(110.5, 112.0, 110.0, 110.5);
        let c3 = c(110.0, 111.0, 100.0, 102.0); // closes below midpoint(c1)=105.5
        assert!(is_evening_star(&c1, &c2, &c3));
    }

    #[test]
    fn three_white_soldiers_consecutive_bull() {
        let c1 = c(100.0, 103.0, 99.5, 102.0);
        let c2 = c(101.0, 104.0, 100.5, 103.5);
        let c3 = c(102.0, 105.0, 101.5, 104.5);
        assert!(is_three_white_soldiers(&c1, &c2, &c3));
    }

    #[test]
    fn three_black_crows_mirror() {
        let c1 = c(110.0, 110.5, 105.0, 106.0);
        let c2 = c(108.0, 108.5, 103.0, 104.5);
        let c3 = c(106.0, 106.5, 101.0, 102.5);
        assert!(is_three_black_crows(&c1, &c2, &c3));
    }

    // ----- detect_at -----

    #[test]
    fn detect_at_finds_doji_alone() {
        let window = [c(100.0, 102.0, 98.0, 100.0)];
        let p = detect_at(&window);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].kind, PatternKind::Doji);
        assert_eq!(p[0].bias, Bias::Neutral);
    }

    #[test]
    fn detect_at_finds_three_white_soldiers() {
        let window = [
            c(100.0, 103.0, 99.5, 102.0),
            c(101.0, 104.0, 100.5, 103.5),
            c(102.0, 105.0, 101.5, 104.5),
        ];
        let p = detect_at(&window);
        assert!(
            p.iter()
                .any(|x| x.kind == PatternKind::ThreeWhiteSoldiers && x.bias == Bias::Bullish),
            "expected ThreeWhiteSoldiers in {p:?}",
        );
    }

    #[test]
    fn detect_at_empty_returns_empty() {
        assert!(detect_at(&[]).is_empty());
    }

    #[test]
    fn detect_at_short_window_skips_3bar() {
        // 2 candles → no 3-bar pattern, but 2-bar can fire.
        let window = [c(105.0, 106.0, 100.0, 101.0), c(100.0, 108.0, 99.0, 107.0)];
        let p = detect_at(&window);
        assert!(p.iter().any(|x| x.kind == PatternKind::BullishEngulfing));
    }
}
