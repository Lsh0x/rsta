//! Price/oscillator divergence detection.
//!
//! A **bearish divergence** occurs when price makes a higher high but an
//! oscillator (RSI, MACD, …) makes a lower high — momentum is fading even
//! as price climbs, often anticipating a top. A **bullish divergence** is
//! the mirror image: price makes a lower low while the oscillator makes
//! a higher low.
//!
//! The detector tracks pivots (swing highs and swing lows) with a
//! configurable confirmation window (`lookback`) and compares each new
//! pivot to the previous one of the same kind. A pivot at index `i` is
//! confirmed `lookback` bars *after* it occurs, so the signal is
//! intrinsically delayed by `lookback` bars — that delay is the price of
//! noise rejection.

use crate::indicators::IndicatorError;
use crate::signals::{Signal, SignalEvent};
use std::collections::VecDeque;

/// One confirmed pivot — either a swing high or a swing low.
#[derive(Debug, Clone, Copy)]
struct Pivot {
    price: f64,
    osc: f64,
    /// Zero-based bar index at which the pivot is centered.
    bar: usize,
}

/// Streaming divergence detector. See module docs.
///
/// # Example
/// ```
/// use rsta::signals::{Divergence, Signal, SignalEvent};
///
/// // Lookback of 2 → pivot is confirmed 2 bars after it occurs.
/// let mut div = Divergence::new(2).unwrap();
///
/// // Feed (price, oscillator) pairs. Sequence below stages a bullish
/// // divergence: price prints a lower low while the oscillator prints a
/// // higher low.
/// let series = [
///     // first low at bar 2: price 8, osc 25
///     (10.0, 50.0), (9.0, 35.0), (8.0, 25.0), (9.0, 35.0), (10.0, 45.0),
///     // intermediate rise (won't trigger anything)
///     (12.0, 60.0),
///     // second low at bar 8: price 6 (lower), osc 30 (higher)
///     (10.0, 50.0), (8.0, 40.0), (6.0, 30.0), (7.0, 38.0), (9.0, 50.0),
/// ];
/// let mut events = vec![];
/// for &x in &series {
///     if let Some(e) = div.next(x) {
///         events.push(e);
///     }
/// }
/// assert!(events.iter().any(|e| matches!(e, SignalEvent::Long)));
/// ```
#[derive(Debug)]
pub struct Divergence {
    lookback: usize,
    min_distance: usize,
    /// Sliding window of `(price, oscillator)` pairs over `2*lookback + 1` bars.
    window: VecDeque<(f64, f64)>,
    last_high: Option<Pivot>,
    last_low: Option<Pivot>,
    /// Number of bars ingested since construction (or last `reset()`).
    seen: usize,
}

impl Divergence {
    /// Create a new detector with the given pivot confirmation lookback.
    ///
    /// `lookback` is the half-window: a center bar is a swing high (low) if
    /// it is strictly greater (less) than the `lookback` bars on each side.
    /// Pivots are therefore confirmed `lookback` bars after they occur.
    ///
    /// `min_distance` defaults to `lookback + 1` to avoid comparing two
    /// adjacent micro-pivots; tune via [`Self::with_min_distance`].
    ///
    /// # Errors
    /// Returns `IndicatorError::InvalidParameter` if `lookback` is `0`.
    pub fn new(lookback: usize) -> Result<Self, IndicatorError> {
        if lookback == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Divergence lookback must be at least 1".to_string(),
            ));
        }
        Ok(Self {
            lookback,
            min_distance: lookback + 1,
            window: VecDeque::with_capacity(2 * lookback + 1),
            last_high: None,
            last_low: None,
            seen: 0,
        })
    }

    /// Override the minimum number of bars between two compared pivots.
    /// Defaults to `lookback + 1`.
    pub fn with_min_distance(mut self, min_distance: usize) -> Self {
        self.min_distance = min_distance;
        self
    }

    /// Returns the bar index of the most recent confirmed swing high, or
    /// `None` if no high has been confirmed yet.
    pub fn last_swing_high_bar(&self) -> Option<usize> {
        self.last_high.map(|p| p.bar)
    }

    /// Returns the bar index of the most recent confirmed swing low.
    pub fn last_swing_low_bar(&self) -> Option<usize> {
        self.last_low.map(|p| p.bar)
    }
}

impl Signal for Divergence {
    type Input = (f64, f64);

    fn next(&mut self, (price, osc): (f64, f64)) -> Option<SignalEvent> {
        self.seen += 1;
        self.window.push_back((price, osc));
        let cap = 2 * self.lookback + 1;
        if self.window.len() < cap {
            return None; // still warming up
        }
        if self.window.len() > cap {
            self.window.pop_front();
        }

        // The center of the window represents the bar `lookback` ahead of
        // the most recently ingested bar (`seen - 1`).
        let center_bar = self.seen - 1 - self.lookback;
        let (cp, co) = self.window[self.lookback];

        // Strict swing detection: center is the unique max (or min) over the window.
        let is_high = self
            .window
            .iter()
            .enumerate()
            .all(|(i, &(p, _))| i == self.lookback || p < cp);
        let is_low = self
            .window
            .iter()
            .enumerate()
            .all(|(i, &(p, _))| i == self.lookback || p > cp);

        if is_high {
            let new = Pivot {
                price: cp,
                osc: co,
                bar: center_bar,
            };
            let event = self.last_high.and_then(|prev| {
                if center_bar.saturating_sub(prev.bar) < self.min_distance {
                    return None;
                }
                // Bearish divergence: price prints a higher high, oscillator
                // prints a lower high.
                if new.price > prev.price && new.osc < prev.osc {
                    Some(SignalEvent::Short)
                } else {
                    None
                }
            });
            self.last_high = Some(new);
            return Some(event.unwrap_or(SignalEvent::Hold));
        }
        if is_low {
            let new = Pivot {
                price: cp,
                osc: co,
                bar: center_bar,
            };
            let event = self.last_low.and_then(|prev| {
                if center_bar.saturating_sub(prev.bar) < self.min_distance {
                    return None;
                }
                // Bullish divergence: price prints a lower low, oscillator
                // prints a higher low.
                if new.price < prev.price && new.osc > prev.osc {
                    Some(SignalEvent::Long)
                } else {
                    None
                }
            });
            self.last_low = Some(new);
            return Some(event.unwrap_or(SignalEvent::Hold));
        }
        Some(SignalEvent::Hold)
    }

    fn reset(&mut self) {
        self.window.clear();
        self.last_high = None;
        self.last_low = None;
        self.seen = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(div: &mut Divergence, series: &[(f64, f64)]) -> Vec<SignalEvent> {
        let mut out = Vec::new();
        for &x in series {
            if let Some(e) = div.next(x) {
                out.push(e);
            }
        }
        out
    }

    #[test]
    fn validates_lookback() {
        assert!(Divergence::new(0).is_err());
        assert!(Divergence::new(1).is_ok());
    }

    #[test]
    fn warmup_emits_none() {
        let mut d = Divergence::new(2).unwrap();
        // 2*lookback + 1 = 5 bars needed before the first emission.
        for i in 0..4 {
            let v = i as f64;
            assert!(d.next((v, v)).is_none(), "premature emission at bar {i}");
        }
        assert!(d.next((4.0, 4.0)).is_some());
    }

    #[test]
    fn bullish_divergence_emits_long() {
        let mut d = Divergence::new(2).unwrap();
        // Two distinct swing lows: first at bar 2 (price 8, osc 25),
        // second at bar 8 (price 6, osc 30 — lower price, higher osc).
        let series = [
            (10.0, 50.0),
            (9.0, 35.0),
            (8.0, 25.0), // first low
            (9.0, 35.0),
            (10.0, 45.0),
            (12.0, 60.0),
            (10.0, 50.0),
            (8.0, 40.0),
            (6.0, 30.0), // second low — bullish divergence
            (7.0, 38.0),
            (9.0, 50.0),
        ];
        let events = run(&mut d, &series);
        assert!(
            events.iter().any(|e| matches!(e, SignalEvent::Long)),
            "no Long event in {events:?}"
        );
        assert!(events.iter().all(|e| !matches!(e, SignalEvent::Short)));
    }

    #[test]
    fn bearish_divergence_emits_short() {
        let mut d = Divergence::new(2).unwrap();
        // Two distinct swing highs: first at bar 2 (price 12, osc 60),
        // second at bar 8 (price 14, osc 50 — higher price, lower osc).
        let series = [
            (10.0, 50.0),
            (11.0, 55.0),
            (12.0, 60.0), // first high
            (11.0, 55.0),
            (10.0, 50.0),
            (8.0, 40.0),
            (10.0, 45.0),
            (12.0, 48.0),
            (14.0, 50.0), // second high — bearish divergence
            (13.0, 47.0),
            (11.0, 40.0),
        ];
        let events = run(&mut d, &series);
        assert!(
            events.iter().any(|e| matches!(e, SignalEvent::Short)),
            "no Short event in {events:?}"
        );
        assert!(events.iter().all(|e| !matches!(e, SignalEvent::Long)));
    }

    #[test]
    fn matching_trends_dont_fire() {
        // Two highs with the oscillator confirming each direction: no divergence.
        let mut d = Divergence::new(2).unwrap();
        let series = [
            (10.0, 50.0),
            (11.0, 55.0),
            (12.0, 60.0), // first high
            (11.0, 55.0),
            (10.0, 50.0),
            (8.0, 40.0),
            (10.0, 50.0),
            (12.0, 60.0),
            (14.0, 70.0), // second high — both make HH
            (13.0, 65.0),
            (11.0, 55.0),
        ];
        let events = run(&mut d, &series);
        assert!(
            events.iter().all(|e| matches!(e, SignalEvent::Hold)),
            "expected only Hold, got {events:?}",
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut d = Divergence::new(1).unwrap();
        // Feed enough bars to populate state.
        for i in 0..5 {
            d.next((i as f64, i as f64));
        }
        d.reset();
        // After reset, must emit None until the warmup window is full again.
        assert!(d.next((10.0, 10.0)).is_none());
        assert!(d.next((11.0, 11.0)).is_none());
        assert!(d.next((12.0, 12.0)).is_some());
    }
}
