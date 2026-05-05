use crate::indicators::trend::Ema;
use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Triple Exponential Moving Average (TEMA).
///
/// `TEMA = 3 * EMA1 - 3 * EMA2 + EMA3` where each EMA chains the previous
/// one's output. Even less lag than DEMA at the cost of more warmup.
///
/// First emission appears at the `3 * period - 2`-th input.
#[derive(Debug)]
pub struct Tema {
    period: usize,
    ema1: Ema,
    ema2: Ema,
    ema3: Ema,
    seen: usize,
}

impl Tema {
    /// Create a new TEMA with the given lookback. `period >= 1`.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            ema1: Ema::new(period)?,
            ema2: Ema::new(period)?,
            ema3: Ema::new(period)?,
            seen: 0,
        })
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.ema1.reset_state();
        self.ema2.reset_state();
        self.ema3.reset_state();
        self.seen = 0;
    }

    fn step(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.seen += 1;
        let e1 = <Ema as Indicator<f64, f64>>::next(&mut self.ema1, value)?
            .expect("inner Ema always emits");
        let e2 = <Ema as Indicator<f64, f64>>::next(&mut self.ema2, e1)?
            .expect("inner Ema always emits");
        let e3 = <Ema as Indicator<f64, f64>>::next(&mut self.ema3, e2)?
            .expect("inner Ema always emits");
        if self.seen < 3 * self.period - 2 {
            return Ok(None);
        }
        Ok(Some(3.0 * e1 - 3.0 * e2 + e3))
    }
}

impl Indicator<f64, f64> for Tema {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 3 * self.period - 2)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - 3 * (self.period - 1));
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
        "Tema"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl Indicator<Candle, f64> for Tema {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 3 * self.period - 2)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - 3 * (self.period - 1));
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
        "Tema"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_emission_at_3p_minus_2() {
        let mut tema = Tema::new(3).unwrap();
        for v in 1..=6 {
            assert!(<Tema as Indicator<f64, f64>>::next(&mut tema, v as f64)
                .unwrap()
                .is_none());
        }
        assert!(<Tema as Indicator<f64, f64>>::next(&mut tema, 7.0)
            .unwrap()
            .is_some());
    }
}
