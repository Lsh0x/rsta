use crate::indicators::trend::Wma;
use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Hull Moving Average (HMA).
///
/// `HMA = WMA(2 * WMA(price, period/2) - WMA(price, period), sqrt(period))`.
/// Designed by Alan Hull to be both smooth and reactive.
#[derive(Debug)]
pub struct Hma {
    period: usize,
    half: Wma,
    full: Wma,
    smooth: Wma,
}

impl Hma {
    /// Create a new HMA. `period >= 2` (we need a non-zero `period / 2`).
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 2)?;
        let half_p = period / 2;
        let smooth_p = (period as f64).sqrt().round() as usize;
        Ok(Self {
            period,
            half: Wma::new(half_p)?,
            full: Wma::new(period)?,
            smooth: Wma::new(smooth_p.max(1))?,
        })
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.half.reset_state();
        self.full.reset_state();
        self.smooth.reset_state();
    }

    fn step(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        let h = <Wma as Indicator<f64, f64>>::next(&mut self.half, value)?;
        let f = <Wma as Indicator<f64, f64>>::next(&mut self.full, value)?;
        let raw = match (h, f) {
            (Some(h), Some(f)) => 2.0 * h - f,
            _ => return Ok(None),
        };
        <Wma as Indicator<f64, f64>>::next(&mut self.smooth, raw)
    }
}

impl Indicator<f64, f64> for Hma {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        let smooth_p = (self.period as f64).sqrt().round() as usize;
        let needed = self.period + smooth_p.max(1) - 1;
        validate_data_length(data, needed)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len().saturating_sub(needed - 1));
        for &v in data {
            if let Some(x) = self.step(v)? {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.step(value)
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Hma"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl Indicator<Candle, f64> for Hma {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let smooth_p = (self.period as f64).sqrt().round() as usize;
        let needed = self.period + smooth_p.max(1) - 1;
        validate_data_length(data, needed)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len().saturating_sub(needed - 1));
        for c in data {
            if let Some(x) = self.step(c.close)? {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        self.step(candle.close)
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Hma"
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
        assert!(Hma::new(1).is_err());
        assert!(Hma::new(2).is_ok());
    }

    #[test]
    fn emits_after_warmup() {
        let mut hma = Hma::new(9).unwrap();
        let prices: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let out = <Hma as Indicator<f64, f64>>::calculate(&mut hma, &prices).unwrap();
        assert!(!out.is_empty());
    }
}
