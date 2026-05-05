use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Commodity Channel Index (CCI) indicator.
///
/// CCI measures the deviation of the typical price from its moving average,
/// scaled by the mean absolute deviation. It oscillates around 0; readings
/// above +100 traditionally signal overbought and below −100 oversold.
///
/// `CCI = (TP - SMA(TP, n)) / (0.015 * MeanDeviation)`
///
/// where `TP = (high + low + close) / 3` and `0.015` is Lambert's scaling
/// factor putting roughly 70-80% of values in [−100, 100].
///
/// # Example
/// ```no_run
/// use rsta::indicators::momentum::Cci;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut cci = Cci::new(20).unwrap();
/// let candles: Vec<Candle> = (0..40).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 1.0,
///     low: i as f64 - 1.0, close: i as f64, volume: 1000.0,
/// }).collect();
/// let values = cci.calculate(&candles).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct Cci {
    period: usize,
    tp_buffer: VecDeque<f64>,
}

impl Cci {
    /// Create a new CCI. Typical period is 20.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            tp_buffer: VecDeque::with_capacity(period),
        })
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.tp_buffer.clear();
    }

    fn typical_price(c: &Candle) -> f64 {
        (c.high + c.low + c.close) / 3.0
    }

    fn step(&mut self, c: Candle) -> Option<f64> {
        let tp = Self::typical_price(&c);
        self.tp_buffer.push_back(tp);
        if self.tp_buffer.len() > self.period {
            self.tp_buffer.pop_front();
        }
        if self.tp_buffer.len() < self.period {
            return None;
        }
        let n = self.period as f64;
        let sma: f64 = self.tp_buffer.iter().sum::<f64>() / n;
        let mean_dev: f64 = self.tp_buffer.iter().map(|x| (x - sma).abs()).sum::<f64>() / n;
        if mean_dev == 0.0 {
            return Some(0.0);
        }
        Some((tp - sma) / (0.015 * mean_dev))
    }
}

impl Indicator<Candle, f64> for Cci {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
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

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        Ok(self.step(value))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Cci"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cci_candles(count: usize) -> Vec<Candle> {
        (0..count)
            .map(|i| {
                let mid = (i as f64) * 0.5 + (i as f64 % 5.0);
                Candle {
                    timestamp: i as u64,
                    open: mid,
                    high: mid + 1.0,
                    low: mid - 1.0,
                    close: mid + 0.25,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    #[test]
    fn validates_period() {
        assert!(Cci::new(0).is_err());
        assert!(Cci::new(20).is_ok());
    }

    #[test]
    fn batch_matches_streaming() {
        let candles = cci_candles(40);
        let mut batch = Cci::new(20).unwrap();
        let batch_out = batch.calculate(&candles).unwrap();
        assert_eq!(batch_out.len(), candles.len() - 19);
        let mut stream = Cci::new(20).unwrap();
        let stream_out: Vec<f64> = candles
            .iter()
            .filter_map(|c| stream.next(*c).unwrap())
            .collect();
        assert_eq!(batch_out, stream_out);
    }

    #[test]
    fn flat_market_emits_zero() {
        let mut cci = Cci::new(5).unwrap();
        let flat = vec![
            Candle {
                timestamp: 0,
                open: 10.0,
                high: 10.0,
                low: 10.0,
                close: 10.0,
                volume: 1.0
            };
            10
        ];
        let out = cci.calculate(&flat).unwrap();
        assert!(out.iter().all(|&v| v == 0.0));
    }
}
