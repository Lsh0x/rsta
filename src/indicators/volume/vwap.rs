use crate::indicators::{Candle, Indicator, IndicatorError};

/// Volume Weighted Average Price (VWAP) indicator.
///
/// Cumulative `Σ(TP * volume) / Σ(volume)` where the typical price is
/// `TP = (high + low + close) / 3`.
///
/// VWAP is **session-based** in real trading — every new session resets the
/// accumulators. Call [`Vwap::reset_state`] (or [`Indicator::reset`]) at each
/// session boundary (daily candles, opening bell, etc.).
///
/// # Example
/// ```no_run
/// use rsta::indicators::volume::Vwap;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut vwap = Vwap::new();
/// let candles: Vec<Candle> = (0..20).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 1.0,
///     low: i as f64 - 1.0, close: i as f64, volume: 1000.0 + i as f64,
/// }).collect();
/// let values = vwap.calculate(&candles).unwrap();
/// assert_eq!(values.len(), candles.len());
/// ```
#[derive(Debug, Default)]
pub struct Vwap {
    cumulative_tp_volume: f64,
    cumulative_volume: f64,
}

impl Vwap {
    /// Create a new VWAP indicator with empty session accumulators.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset session accumulators (call at each new session start).
    pub fn reset_state(&mut self) {
        self.cumulative_tp_volume = 0.0;
        self.cumulative_volume = 0.0;
    }

    fn step(&mut self, value: Candle) -> f64 {
        let tp = (value.high + value.low + value.close) / 3.0;
        self.cumulative_tp_volume += tp * value.volume;
        self.cumulative_volume += value.volume;
        if self.cumulative_volume == 0.0 {
            return tp;
        }
        self.cumulative_tp_volume / self.cumulative_volume
    }
}

impl Indicator<Candle, f64> for Vwap {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        if data.is_empty() {
            return Err(IndicatorError::InsufficientData(
                "VWAP requires at least one candle".to_string(),
            ));
        }
        self.reset_state();
        let mut out = Vec::with_capacity(data.len());
        for c in data {
            out.push(self.step(*c));
        }
        Ok(out)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        Ok(Some(self.step(value)))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Vwap"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_value_equals_tp() {
        let mut vwap = Vwap::new();
        let c = Candle {
            timestamp: 0,
            open: 10.0,
            high: 12.0,
            low: 8.0,
            close: 10.0,
            volume: 1000.0,
        };
        // TP = 10.0; volume cancels in (TP*V)/V.
        assert_eq!(vwap.next(c).unwrap(), Some(10.0));
    }

    #[test]
    fn weighted_average() {
        let mut vwap = Vwap::new();
        let candles = [
            Candle {
                timestamp: 0,
                open: 9.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 100.0,
            },
            Candle {
                timestamp: 1,
                open: 19.0,
                high: 21.0,
                low: 19.0,
                close: 20.0,
                volume: 300.0,
            },
        ];
        let out = vwap.calculate(&candles).unwrap();
        assert_eq!(out[0], 10.0);
        // (10*100 + 20*300) / (100 + 300) = 7000/400 = 17.5
        assert!((out[1] - 17.5).abs() < 1e-12);
    }

    #[test]
    fn reset_starts_new_session() {
        let mut vwap = Vwap::new();
        let early = [Candle {
            timestamp: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1.0,
        }];
        vwap.calculate(&early).unwrap();
        vwap.reset_state();
        let new = Candle {
            timestamp: 1,
            open: 50.0,
            high: 50.0,
            low: 50.0,
            close: 50.0,
            volume: 1.0,
        };
        assert_eq!(vwap.next(new).unwrap(), Some(50.0));
    }
}
