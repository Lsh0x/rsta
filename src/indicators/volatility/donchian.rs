use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Donchian Channels result: rolling max high, min low, and their midpoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DonchianResult {
    /// Highest high over the lookback period.
    pub upper: f64,
    /// Midpoint: `(upper + lower) / 2`.
    pub middle: f64,
    /// Lowest low over the lookback period.
    pub lower: f64,
}

/// Donchian Channels indicator.
///
/// Tracks the highest high and lowest low over the last `period` candles.
/// Foundational breakout filter — the original Turtle Trading rules use a
/// 20-period Donchian breakout.
///
/// # Example
/// ```no_run
/// use rsta::indicators::volatility::Donchian;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut dc = Donchian::new(20).unwrap();
/// let candles: Vec<Candle> = (0..40).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 2.0,
///     low: i as f64 - 1.0, close: i as f64 + 1.0, volume: 1000.0,
/// }).collect();
/// let bands = dc.calculate(&candles).unwrap();
/// assert!(!bands.is_empty());
/// ```
#[derive(Debug)]
pub struct Donchian {
    period: usize,
    buffer: VecDeque<(f64, f64)>,
}

impl Donchian {
    /// Create a new Donchian Channels indicator. Typical period is 20.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
        })
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.buffer.clear();
    }

    fn step(&mut self, value: Candle) -> Option<DonchianResult> {
        self.buffer.push_back((value.high, value.low));
        if self.buffer.len() > self.period {
            self.buffer.pop_front();
        }
        if self.buffer.len() < self.period {
            return None;
        }
        let upper = self
            .buffer
            .iter()
            .map(|&(h, _)| h)
            .fold(f64::NEG_INFINITY, f64::max);
        let lower = self
            .buffer
            .iter()
            .map(|&(_, l)| l)
            .fold(f64::INFINITY, f64::min);
        Some(DonchianResult {
            upper,
            middle: (upper + lower) / 2.0,
            lower,
        })
    }
}

impl Indicator<Candle, DonchianResult> for Donchian {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<DonchianResult>, IndicatorError> {
        validate_data_length(data, self.period)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - self.period + 1);
        for c in data {
            if let Some(v) = self.step(*c) {
                out.push(v);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: Candle) -> Result<Option<DonchianResult>, IndicatorError> {
        Ok(self.step(value))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Donchian"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn donchian_candles() -> Vec<Candle> {
        vec![
            Candle {
                timestamp: 0,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1.0,
            },
            Candle {
                timestamp: 1,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1.0,
            },
            Candle {
                timestamp: 2,
                open: 12.0,
                high: 15.0,
                low: 10.0,
                close: 13.0,
                volume: 1.0,
            },
            Candle {
                timestamp: 3,
                open: 13.0,
                high: 14.0,
                low: 11.0,
                close: 12.0,
                volume: 1.0,
            },
            Candle {
                timestamp: 4,
                open: 12.0,
                high: 16.0,
                low: 11.0,
                close: 15.0,
                volume: 1.0,
            },
        ]
    }

    #[test]
    fn validates_period() {
        assert!(Donchian::new(0).is_err());
        assert!(Donchian::new(20).is_ok());
    }

    #[test]
    fn returns_max_high_min_low() {
        let mut dc = Donchian::new(3).unwrap();
        let out = dc.calculate(&donchian_candles()).unwrap();
        assert_eq!(out.len(), 3);
        // First emission covers candles 0..3: highs 12,13,15 → 15; lows 8,9,10 → 8.
        assert_eq!(out[0].upper, 15.0);
        assert_eq!(out[0].lower, 8.0);
        assert_eq!(out[0].middle, 11.5);
        // Last emission covers candles 2..5: highs 15,14,16 → 16; lows 10,11,11 → 10.
        assert_eq!(out[2].upper, 16.0);
        assert_eq!(out[2].lower, 10.0);
        assert_eq!(out[2].middle, 13.0);
    }
}
