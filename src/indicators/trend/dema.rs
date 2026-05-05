use crate::indicators::trend::Ema;
use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Double Exponential Moving Average (DEMA).
///
/// `DEMA = 2 * EMA(price) - EMA(EMA(price))`. Reduces the lag of a plain
/// EMA while keeping smoothing.
///
/// First emission appears at the `2 * period - 1`-th input — before that
/// the chained EMA-of-EMA is biased toward its seed.
///
/// # Example
/// ```
/// use rsta::indicators::trend::Dema;
/// use rsta::indicators::Indicator;
///
/// let mut dema = Dema::new(5).unwrap();
/// let prices: Vec<f64> = (1..=20).map(|i| i as f64).collect();
/// let out = <Dema as Indicator<f64, f64>>::calculate(&mut dema, &prices).unwrap();
/// assert!(!out.is_empty());
/// ```
#[derive(Debug)]
pub struct Dema {
    period: usize,
    ema1: Ema,
    ema2: Ema,
    seen: usize,
}

impl Dema {
    /// Create a new DEMA with the given lookback. `period >= 1`.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            ema1: Ema::new(period)?,
            ema2: Ema::new(period)?,
            seen: 0,
        })
    }

    /// Reset internal state without dropping the configured period.
    pub fn reset_state(&mut self) {
        self.ema1.reset_state();
        self.ema2.reset_state();
        self.seen = 0;
    }

    fn step(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.seen += 1;
        let e1 = <Ema as Indicator<f64, f64>>::next(&mut self.ema1, value)?
            .expect("inner Ema always emits");
        let e2 = <Ema as Indicator<f64, f64>>::next(&mut self.ema2, e1)?
            .expect("inner Ema always emits");
        if self.seen < 2 * self.period - 1 {
            return Ok(None);
        }
        Ok(Some(2.0 * e1 - e2))
    }
}

impl Indicator<f64, f64> for Dema {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 2 * self.period - 1)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - 2 * (self.period - 1));
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
        "Dema"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl Indicator<Candle, f64> for Dema {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 2 * self.period - 1)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - 2 * (self.period - 1));
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
        "Dema"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_emission_at_2p_minus_1() {
        let mut dema = Dema::new(5).unwrap();
        // 2*period - 2 = 8 inputs produce nothing.
        for v in 1..=8 {
            assert!(
                <Dema as Indicator<f64, f64>>::next(&mut dema, v as f64)
                    .unwrap()
                    .is_none(),
                "premature emission at v={v}",
            );
        }
        assert!(<Dema as Indicator<f64, f64>>::next(&mut dema, 9.0)
            .unwrap()
            .is_some());
    }
}
