use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Weighted Moving Average (WMA) indicator.
///
/// Linearly weighted: the most recent value gets weight `period`, the oldest
/// gets weight `1`. Weights sum to `period * (period + 1) / 2`.
///
/// # Example
/// ```
/// use rsta::indicators::trend::Wma;
/// use rsta::indicators::Indicator;
///
/// let mut wma = Wma::new(3).unwrap();
/// // Weights are 1, 2, 3 → (1*1 + 2*2 + 3*3) / 6 = 14/6 ≈ 2.333.
/// let out = wma.calculate(&[1.0_f64, 2.0, 3.0]).unwrap();
/// assert!((out[0] - (14.0 / 6.0)).abs() < 1e-12);
/// ```
#[derive(Debug)]
pub struct Wma {
    period: usize,
    buffer: VecDeque<f64>,
}

impl Wma {
    /// Create a new WMA. `period >= 1`.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
        })
    }

    /// Reset internal state without dropping the configured period.
    pub fn reset_state(&mut self) {
        self.buffer.clear();
    }

    fn weighted(buffer: &VecDeque<f64>, period: usize) -> f64 {
        let n = period as f64;
        let denom = n * (n + 1.0) / 2.0;
        let mut numer = 0.0;
        for (i, v) in buffer.iter().enumerate() {
            // Most-recent value (last in buffer) gets the highest weight.
            numer += (i as f64 + 1.0) * v;
        }
        numer / denom
    }

    fn step(&mut self, value: f64) -> Option<f64> {
        self.buffer.push_back(value);
        if self.buffer.len() > self.period {
            self.buffer.pop_front();
        }
        if self.buffer.len() < self.period {
            return None;
        }
        Some(Self::weighted(&self.buffer, self.period))
    }
}

impl Indicator<f64, f64> for Wma {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - self.period + 1);
        for &v in data {
            if let Some(x) = self.step(v) {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        Ok(self.step(value))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Wma"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl Indicator<Candle, f64> for Wma {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - self.period + 1);
        for c in data {
            if let Some(x) = self.step(c.close) {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        Ok(self.step(candle.close))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Wma"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_period() {
        assert!(Wma::new(0).is_err());
        assert!(Wma::new(1).is_ok());
    }

    #[test]
    fn weighting_is_linear() {
        let mut wma = Wma::new(3).unwrap();
        let out = <Wma as Indicator<f64, f64>>::calculate(&mut wma, &[1.0, 2.0, 3.0]).unwrap();
        assert!((out[0] - (14.0 / 6.0)).abs() < 1e-12);
    }

    #[test]
    fn batch_matches_streaming() {
        let prices: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let mut batch = Wma::new(5).unwrap();
        let batch_out = <Wma as Indicator<f64, f64>>::calculate(&mut batch, &prices).unwrap();
        let mut stream = Wma::new(5).unwrap();
        let stream_out: Vec<_> = prices
            .iter()
            .filter_map(|&p| <Wma as Indicator<f64, f64>>::next(&mut stream, p).unwrap())
            .collect();
        assert_eq!(batch_out, stream_out);
    }
}
