use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Average Directional Index (ADX) result.
///
/// Carries the two directional indicators alongside the ADX value so a single
/// emission gives the full trend-strength picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdxResult {
    /// +DI line — strength of upward movement (0..=100).
    pub plus_di: f64,
    /// -DI line — strength of downward movement (0..=100).
    pub minus_di: f64,
    /// ADX line — overall trend strength (0..=100, period-smoothed).
    pub adx: f64,
}

/// Average Directional Index (ADX) — Wilder's directional movement system.
///
/// Tracks +DM, -DM, and the True Range; applies Wilder smoothing over
/// `period` bars; then derives:
///
/// - `+DI = 100 * +DM_smoothed / ATR_smoothed`
/// - `-DI = 100 * -DM_smoothed / ATR_smoothed`
/// - `DX  = 100 * |+DI - -DI| / (+DI + -DI)`
/// - `ADX = Wilder-smoothed DX over period`
///
/// First emission appears at the `2 * period`-th candle (one seed candle,
/// then `period` directional samples to seed the sums, then `period - 1`
/// DX values to seed the ADX smoothing).
///
/// # Example
/// ```no_run
/// use rsta::indicators::trend::Adx;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut adx = Adx::new(14).unwrap();
/// let candles: Vec<Candle> = (0..50).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 2.0,
///     low: i as f64 - 1.0, close: i as f64 + 1.0, volume: 1000.0,
/// }).collect();
/// let values = adx.calculate(&candles).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct Adx {
    period: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    smooth_plus_dm: Option<f64>,
    smooth_minus_dm: Option<f64>,
    smooth_tr: Option<f64>,
    dx_buffer: VecDeque<f64>,
    smooth_adx: Option<f64>,
    seen: usize,
}

impl Adx {
    /// Create a new ADX with the given lookback (typically 14).
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            smooth_plus_dm: None,
            smooth_minus_dm: None,
            smooth_tr: None,
            dx_buffer: VecDeque::with_capacity(period),
            smooth_adx: None,
            seen: 0,
        })
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.prev_high = None;
        self.prev_low = None;
        self.prev_close = None;
        self.smooth_plus_dm = None;
        self.smooth_minus_dm = None;
        self.smooth_tr = None;
        self.dx_buffer.clear();
        self.smooth_adx = None;
        self.seen = 0;
    }

    fn step(&mut self, value: Candle) -> Option<AdxResult> {
        self.seen += 1;
        let (Some(prev_high), Some(prev_low), Some(prev_close)) =
            (self.prev_high, self.prev_low, self.prev_close)
        else {
            self.prev_high = Some(value.high);
            self.prev_low = Some(value.low);
            self.prev_close = Some(value.close);
            return None;
        };

        let up_move = value.high - prev_high;
        let down_move = prev_low - value.low;
        let plus_dm = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let minus_dm = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };

        let tr = (value.high - value.low)
            .max((value.high - prev_close).abs())
            .max((value.low - prev_close).abs());

        self.prev_high = Some(value.high);
        self.prev_low = Some(value.low);
        self.prev_close = Some(value.close);

        let n = self.period as f64;
        let samples = self.seen - 1;
        if samples == 1 {
            self.smooth_plus_dm = Some(plus_dm);
            self.smooth_minus_dm = Some(minus_dm);
            self.smooth_tr = Some(tr);
        } else {
            let prev_p = self.smooth_plus_dm.unwrap();
            let prev_m = self.smooth_minus_dm.unwrap();
            let prev_t = self.smooth_tr.unwrap();
            if samples <= self.period {
                // Wilder seeds with a raw sum over the first `period` samples.
                self.smooth_plus_dm = Some(prev_p + plus_dm);
                self.smooth_minus_dm = Some(prev_m + minus_dm);
                self.smooth_tr = Some(prev_t + tr);
            } else {
                self.smooth_plus_dm = Some(prev_p - prev_p / n + plus_dm);
                self.smooth_minus_dm = Some(prev_m - prev_m / n + minus_dm);
                self.smooth_tr = Some(prev_t - prev_t / n + tr);
            }
        }

        if samples < self.period {
            return None;
        }

        let p = self.smooth_plus_dm.unwrap();
        let m = self.smooth_minus_dm.unwrap();
        let t = self.smooth_tr.unwrap();
        if t == 0.0 {
            return Some(AdxResult {
                plus_di: 0.0,
                minus_di: 0.0,
                adx: 0.0,
            });
        }

        let plus_di = 100.0 * p / t;
        let minus_di = 100.0 * m / t;
        let denom = plus_di + minus_di;
        let dx = if denom == 0.0 {
            0.0
        } else {
            100.0 * (plus_di - minus_di).abs() / denom
        };

        match self.smooth_adx {
            None => {
                self.dx_buffer.push_back(dx);
                if self.dx_buffer.len() < self.period {
                    return None;
                }
                let seed = self.dx_buffer.iter().sum::<f64>() / n;
                self.smooth_adx = Some(seed);
                Some(AdxResult {
                    plus_di,
                    minus_di,
                    adx: seed,
                })
            }
            Some(prev_adx) => {
                let new_adx = (prev_adx * (n - 1.0) + dx) / n;
                self.smooth_adx = Some(new_adx);
                Some(AdxResult {
                    plus_di,
                    minus_di,
                    adx: new_adx,
                })
            }
        }
    }
}

impl Indicator<Candle, AdxResult> for Adx {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<AdxResult>, IndicatorError> {
        validate_data_length(data, 2 * self.period)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len().saturating_sub(2 * self.period - 1));
        for &c in data {
            if let Some(point) = self.step(c) {
                out.push(point);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: Candle) -> Result<Option<AdxResult>, IndicatorError> {
        Ok(self.step(value))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Adx"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candles(count: usize, trend: f64) -> Vec<Candle> {
        (0..count)
            .map(|i| {
                let mid = i as f64 * trend;
                Candle {
                    timestamp: i as u64,
                    open: mid,
                    high: mid + 1.5,
                    low: mid - 1.5,
                    close: mid + 0.5,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    #[test]
    fn validates_period() {
        assert!(Adx::new(0).is_err());
        assert!(Adx::new(14).is_ok());
    }

    #[test]
    fn emits_after_warmup() {
        let period = 3;
        let mut adx = Adx::new(period).unwrap();
        let candles = make_candles(20, 1.0);
        let mut emissions = 0;
        for c in &candles {
            if adx.next(*c).unwrap().is_some() {
                emissions += 1;
            }
        }
        assert_eq!(emissions, candles.len() - (2 * period - 1));
    }

    #[test]
    fn strong_uptrend_high_di_diff() {
        let mut adx = Adx::new(7).unwrap();
        let out = adx.calculate(&make_candles(40, 1.0)).unwrap();
        let last = out.last().unwrap();
        assert!(last.plus_di > last.minus_di);
        assert!(last.adx > 50.0, "got {}", last.adx);
    }
}
