use crate::indicators::utils::validate_data_length;
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Parabolic SAR (Stop and Reverse) — Welles Wilder.
///
/// A trailing stop that converges on price during a trend and flips to the
/// other side when the trend reverses. Output is a single price level per
/// bar; combine with the bar's price to derive a long/short bias (price
/// above SAR → long, price below SAR → short).
///
/// Algorithm:
/// 1. **Trend direction** is initialised on the second bar by comparing
///    closes (close[1] >= close[0] → long).
/// 2. **Acceleration Factor (AF)** starts at `af_start` (typically 0.02).
///    Each time the trend extends to a new extreme, AF is incremented by
///    `af_step` (0.02), capped at `af_max` (0.20).
/// 3. **Extreme Point (EP)**: the highest high in a long trend or the
///    lowest low in a short trend, updated whenever a new extreme is set.
/// 4. **SAR update** for the *next* bar:
///    `SAR' = SAR + AF * (EP - SAR)`. In a long trend, SAR is also clamped
///    to be no higher than the lowest low of the prior two bars (and
///    symmetrically for shorts) to avoid the SAR cutting into the body of
///    the recent range.
/// 5. **Reversal**: if the current bar's price crosses SAR, the trend
///    flips, SAR is reset to the prior EP, AF resets to `af_start`, and EP
///    becomes the current bar's high (new long trend) or low (new short).
///
/// First emission appears on the **second** bar (the first bar only seeds
/// state).
///
/// # Example
/// ```no_run
/// use rsta::indicators::trend::Sar;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut sar = Sar::new(0.02, 0.02, 0.20).unwrap();
/// let candles: Vec<Candle> = (0..30).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 1.0,
///     low: i as f64 - 1.0, close: i as f64, volume: 1.0,
/// }).collect();
/// let values = sar.calculate(&candles).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct Sar {
    af_start: f64,
    af_step: f64,
    af_max: f64,
    /// `true` for an active long trend, `false` for a short trend.
    long: bool,
    sar: f64,
    /// Extreme point (highest high in a long trend, lowest low in a short).
    ep: f64,
    af: f64,
    /// Previous candle's high — used to clamp SAR in a long trend.
    prev_high: f64,
    /// Previous candle's low — used to clamp SAR in a short trend.
    prev_low: f64,
    /// `0` = uninitialised, `1` = seed candle ingested, `>= 2` = active.
    seen: usize,
}

impl Sar {
    /// Create a new Parabolic SAR with the given AF schedule.
    ///
    /// # Errors
    /// Returns `IndicatorError::InvalidParameter` if any value is `<= 0`,
    /// or if `af_start > af_max` or `af_step > af_max`.
    pub fn new(af_start: f64, af_step: f64, af_max: f64) -> Result<Self, IndicatorError> {
        if af_start <= 0.0 || af_step <= 0.0 || af_max <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "Parabolic SAR factors must be positive".to_string(),
            ));
        }
        if af_start > af_max || af_step > af_max {
            return Err(IndicatorError::InvalidParameter(
                "af_start and af_step must be <= af_max".to_string(),
            ));
        }
        Ok(Self {
            af_start,
            af_step,
            af_max,
            long: true,
            sar: 0.0,
            ep: 0.0,
            af: af_start,
            prev_high: 0.0,
            prev_low: 0.0,
            seen: 0,
        })
    }

    /// SAR with the canonical default parameters (0.02 / 0.02 / 0.20).
    pub fn default_params() -> Self {
        Self::new(0.02, 0.02, 0.20).expect("canonical params are valid")
    }

    /// Reset internal state — the next bar will re-seed direction.
    pub fn reset_state(&mut self) {
        self.long = true;
        self.sar = 0.0;
        self.ep = 0.0;
        self.af = self.af_start;
        self.prev_high = 0.0;
        self.prev_low = 0.0;
        self.seen = 0;
    }

    fn step(&mut self, candle: Candle) -> Option<f64> {
        self.seen += 1;
        if self.seen == 1 {
            // Seed bar: remember its high/low; no output yet.
            self.prev_high = candle.high;
            self.prev_low = candle.low;
            return None;
        }
        if self.seen == 2 {
            // Initialise direction by comparing this bar's close to the seed.
            // If we trend up (close >= prev close-ish, use prev_high/low to
            // disambiguate), start long; otherwise short.
            self.long = candle.close >= (self.prev_high + self.prev_low) / 2.0;
            if self.long {
                self.sar = self.prev_low;
                self.ep = candle.high.max(self.prev_high);
            } else {
                self.sar = self.prev_high;
                self.ep = candle.low.min(self.prev_low);
            }
            self.af = self.af_start;
            // Emit this bar's SAR (which was seeded for entry into the bar).
            let out = self.sar;
            // Update for next bar.
            self.advance(candle);
            return Some(out);
        }

        // Steady-state: emit the SAR computed for the current bar (already in
        // self.sar) then advance for the next.
        let out = self.sar;
        // Detect reversal: price has crossed the SAR.
        let reversed = if self.long {
            candle.low < self.sar
        } else {
            candle.high > self.sar
        };
        if reversed {
            // Flip direction. New SAR is the old EP; AF resets; EP becomes
            // this bar's extreme on the new side.
            self.long = !self.long;
            self.sar = self.ep;
            self.af = self.af_start;
            self.ep = if self.long { candle.high } else { candle.low };
            // After a reversal we still emit the old SAR for this bar —
            // it's the actual stop level that was hit. Update prev_*
            // and return.
            self.prev_high = candle.high;
            self.prev_low = candle.low;
            return Some(out);
        }
        self.advance(candle);
        Some(out)
    }

    fn advance(&mut self, candle: Candle) {
        // Update extreme point + AF if a new extreme is set.
        if self.long {
            if candle.high > self.ep {
                self.ep = candle.high;
                self.af = (self.af + self.af_step).min(self.af_max);
            }
        } else if candle.low < self.ep {
            self.ep = candle.low;
            self.af = (self.af + self.af_step).min(self.af_max);
        }

        // Compute next SAR.
        let mut next_sar = self.sar + self.af * (self.ep - self.sar);

        // Clamp: SAR must not penetrate the prior two lows (long) or highs
        // (short). We track only one prior bar in this v1, which is the
        // canonical Wilder formulation when the seed is short — accept the
        // single-bar clamp for simplicity.
        if self.long {
            next_sar = next_sar.min(self.prev_low).min(candle.low);
        } else {
            next_sar = next_sar.max(self.prev_high).max(candle.high);
        }

        self.sar = next_sar;
        self.prev_high = candle.high;
        self.prev_low = candle.low;
    }
}

impl Indicator<Candle, f64> for Sar {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 2)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - 1);
        for c in data {
            if let Some(v) = self.step(*c) {
                out.push(v);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        Ok(self.step(value))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Sar"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(n: usize, slope: f64) -> Vec<Candle> {
        (0..n)
            .map(|i| {
                let mid = i as f64 * slope;
                Candle {
                    timestamp: i as u64,
                    open: mid,
                    high: mid + 1.0,
                    low: mid - 1.0,
                    close: mid + 0.25,
                    volume: 1.0,
                }
            })
            .collect()
    }

    #[test]
    fn validates_factors() {
        assert!(Sar::new(0.0, 0.02, 0.20).is_err());
        assert!(Sar::new(0.02, 0.0, 0.20).is_err());
        assert!(Sar::new(0.02, 0.02, 0.0).is_err());
        assert!(Sar::new(0.30, 0.02, 0.20).is_err()); // start > max
        assert!(Sar::new(0.02, 0.30, 0.20).is_err()); // step > max
        assert!(Sar::new(0.02, 0.02, 0.20).is_ok());
    }

    #[test]
    fn first_bar_emits_nothing_second_emits() {
        let mut sar = Sar::default_params();
        let c0 = Candle {
            timestamp: 0,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1.0,
        };
        let c1 = Candle {
            timestamp: 1,
            open: 10.5,
            high: 12.0,
            low: 10.0,
            close: 11.5,
            volume: 1.0,
        };
        assert!(sar.next(c0).unwrap().is_none());
        assert!(sar.next(c1).unwrap().is_some());
    }

    #[test]
    fn uptrend_keeps_sar_below_price() {
        let mut sar = Sar::default_params();
        let candles = ramp(30, 1.0);
        let out = sar.calculate(&candles).unwrap();
        // For each emission past the seed, in a clean uptrend SAR <= bar low.
        // Skip the first emission (which seeds direction).
        for (i, &s) in out.iter().enumerate().skip(1) {
            // out[i] aligns with candles[i+1] because we drop the first seed.
            let c = candles[i + 1];
            assert!(s <= c.low + 1e-9, "bar {} SAR {} > low {}", i + 1, s, c.low);
        }
    }

    #[test]
    fn downtrend_keeps_sar_above_price() {
        let mut sar = Sar::default_params();
        let candles = ramp(30, -1.0);
        let out = sar.calculate(&candles).unwrap();
        for (i, &s) in out.iter().enumerate().skip(1) {
            let c = candles[i + 1];
            assert!(
                s >= c.high - 1e-9,
                "bar {} SAR {} < high {}",
                i + 1,
                s,
                c.high
            );
        }
    }

    #[test]
    fn reversal_flips_sar_to_other_side() {
        // Up-then-down: build an uptrend then drop sharply to trigger a flip.
        let mut up = ramp(15, 1.0);
        // Append a single sharp drop that should pierce the trailing SAR.
        for i in 0..10 {
            let mid = (15 - i) as f64 * 1.0 - 5.0;
            up.push(Candle {
                timestamp: (15 + i) as u64,
                open: mid,
                high: mid + 1.0,
                low: mid - 1.0,
                close: mid - 0.5,
                volume: 1.0,
            });
        }
        let mut sar = Sar::default_params();
        let out = sar.calculate(&up).unwrap();
        // The last few bars should have SAR above price (we're now short).
        let last = *out.last().unwrap();
        let last_close = up.last().unwrap().close;
        assert!(
            last > last_close,
            "expected SAR above price after flip, SAR={last} close={last_close}"
        );
    }
}
