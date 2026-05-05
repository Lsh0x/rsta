use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Money Flow Index (MFI) — volume-weighted RSI.
///
/// MFI uses both price and volume to gauge overbought/oversold conditions.
/// Bounded 0..=100; readings above 80 are commonly considered overbought
/// and below 20 oversold.
///
/// Algorithm (Wilder's classic, period typically 14):
/// 1. Typical Price `TP = (high + low + close) / 3`
/// 2. Raw Money Flow `RMF = TP * volume`
/// 3. Sum positive (TP up) and negative (TP down) RMFs over the lookback.
/// 4. `MFI = 100 - (100 / (1 + positive_sum / negative_sum))`. If
///    `negative_sum == 0`, MFI is `100`. If both are 0, MFI is `50`.
///
/// # Example
/// ```no_run
/// use rsta::indicators::volume::Mfi;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut mfi = Mfi::new(14).unwrap();
/// let candles: Vec<Candle> = (0..30).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 1.0,
///     low: i as f64 - 1.0, close: i as f64, volume: 1000.0 + i as f64,
/// }).collect();
/// let values = mfi.calculate(&candles).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct Mfi {
    period: usize,
    /// (signed_raw_money_flow, direction). Direction: +1 up, -1 down, 0 unchanged.
    flow_buffer: VecDeque<(f64, i8)>,
    prev_tp: Option<f64>,
}

impl Mfi {
    /// Create a new MFI indicator. Standard period is 14.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            flow_buffer: VecDeque::with_capacity(period),
            prev_tp: None,
        })
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.flow_buffer.clear();
        self.prev_tp = None;
    }

    fn step(&mut self, value: Candle) -> Option<f64> {
        let tp = (value.high + value.low + value.close) / 3.0;
        let rmf = tp * value.volume;
        let Some(prev) = self.prev_tp else {
            self.prev_tp = Some(tp);
            return None;
        };
        let direction: i8 = if tp > prev {
            1
        } else if tp < prev {
            -1
        } else {
            0
        };
        self.prev_tp = Some(tp);

        self.flow_buffer.push_back((rmf, direction));
        if self.flow_buffer.len() > self.period {
            self.flow_buffer.pop_front();
        }
        if self.flow_buffer.len() < self.period {
            return None;
        }

        let mut positive = 0.0f64;
        let mut negative = 0.0f64;
        for &(rmf, dir) in &self.flow_buffer {
            match dir {
                1 => positive += rmf,
                -1 => negative += rmf,
                _ => {}
            }
        }
        if negative == 0.0 {
            if positive == 0.0 {
                return Some(50.0);
            }
            return Some(100.0);
        }
        let ratio = positive / negative;
        Some(100.0 - 100.0 / (1.0 + ratio))
    }
}

impl Indicator<Candle, f64> for Mfi {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period + 1)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - self.period);
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
        "Mfi"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp_candles(count: usize, vol: f64) -> Vec<Candle> {
        (0..count)
            .map(|i| Candle {
                timestamp: i as u64,
                open: i as f64,
                high: i as f64 + 1.0,
                low: i as f64 - 1.0,
                close: i as f64,
                volume: vol,
            })
            .collect()
    }

    #[test]
    fn validates_period() {
        assert!(Mfi::new(0).is_err());
        assert!(Mfi::new(14).is_ok());
    }

    #[test]
    fn pure_uptrend_saturates_to_100() {
        let mut mfi = Mfi::new(5).unwrap();
        let out = mfi.calculate(&ramp_candles(20, 1000.0)).unwrap();
        assert!(out.iter().all(|&v| v == 100.0));
    }

    #[test]
    fn batch_matches_streaming() {
        let candles = ramp_candles(30, 1500.0);
        let mut batch = Mfi::new(14).unwrap();
        let batch_out = batch.calculate(&candles).unwrap();
        let mut stream = Mfi::new(14).unwrap();
        let stream_out: Vec<f64> = candles
            .iter()
            .filter_map(|c| stream.next(*c).unwrap())
            .collect();
        assert_eq!(batch_out, stream_out);
    }
}
