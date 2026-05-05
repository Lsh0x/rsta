use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Ichimoku Cloud output for a single bar.
///
/// `senkou_a` / `senkou_b` are conventionally plotted `kijun_period` bars in
/// the future (the "leading" projection); this struct simply carries their
/// value as computed from the current bar's window. It is up to the consumer
/// to shift them when rendering or signalling.
///
/// `chikou` is the close of the current bar, intended to be plotted
/// `kijun_period` bars in the past.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IchimokuResult {
    /// Tenkan-sen (Conversion Line): midpoint of the last `tenkan_period` highs/lows.
    pub tenkan: f64,
    /// Kijun-sen (Base Line): midpoint of the last `kijun_period` highs/lows.
    pub kijun: f64,
    /// Senkou Span A (Leading Span A): `(tenkan + kijun) / 2`. Plot `kijun_period` bars ahead.
    pub senkou_a: f64,
    /// Senkou Span B (Leading Span B): midpoint of the last `senkou_b_period` highs/lows.
    pub senkou_b: f64,
    /// Chikou Span (Lagging Span): the current close, intended to be plotted `kijun_period` bars behind.
    pub chikou: f64,
}

/// Ichimoku Kinkō Hyō ("one-glance equilibrium chart") — Goichi Hosoda.
///
/// Five-component system that summarises support, resistance, momentum and
/// trend direction. Standard parameters are 9 / 26 / 52 (tenkan / kijun /
/// senkou_b).
///
/// First emission appears once enough bars have accumulated for the longest
/// component (`senkou_b_period`).
///
/// # Example
/// ```no_run
/// use rsta::indicators::trend::Ichimoku;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut ichi = Ichimoku::new(9, 26, 52).unwrap();
/// let candles: Vec<Candle> = (0..120).map(|i| Candle {
///     timestamp: i, open: i as f64, high: i as f64 + 1.0,
///     low: i as f64 - 1.0, close: i as f64, volume: 1.0,
/// }).collect();
/// let values = ichi.calculate(&candles).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct Ichimoku {
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    /// Rolling buffer of `(high, low)` covering at least `senkou_b_period`.
    buffer: VecDeque<(f64, f64)>,
}

impl Ichimoku {
    /// Create a new Ichimoku with the given periods.
    ///
    /// Standard parameters are `(9, 26, 52)`. Constraints: each period >= 1
    /// and `tenkan_period <= kijun_period <= senkou_b_period`.
    pub fn new(
        tenkan_period: usize,
        kijun_period: usize,
        senkou_b_period: usize,
    ) -> Result<Self, IndicatorError> {
        validate_period(tenkan_period, 1)?;
        validate_period(kijun_period, 1)?;
        validate_period(senkou_b_period, 1)?;
        if tenkan_period > kijun_period || kijun_period > senkou_b_period {
            return Err(IndicatorError::InvalidParameter(
                "Ichimoku periods must satisfy tenkan <= kijun <= senkou_b".to_string(),
            ));
        }
        Ok(Self {
            tenkan_period,
            kijun_period,
            senkou_b_period,
            buffer: VecDeque::with_capacity(senkou_b_period),
        })
    }

    /// Default Ichimoku with the canonical (9, 26, 52) parameters.
    pub fn default_params() -> Self {
        Self::new(9, 26, 52).expect("canonical params are valid")
    }

    /// Reset internal state.
    pub fn reset_state(&mut self) {
        self.buffer.clear();
    }

    /// Midpoint of the highest high and lowest low over the last `n` entries.
    fn midpoint(buffer: &VecDeque<(f64, f64)>, n: usize) -> f64 {
        let start = buffer.len().saturating_sub(n);
        let slice = buffer.iter().skip(start);
        let mut hi = f64::NEG_INFINITY;
        let mut lo = f64::INFINITY;
        for &(h, l) in slice {
            if h > hi {
                hi = h;
            }
            if l < lo {
                lo = l;
            }
        }
        (hi + lo) / 2.0
    }

    fn step(&mut self, candle: Candle) -> Option<IchimokuResult> {
        self.buffer.push_back((candle.high, candle.low));
        if self.buffer.len() > self.senkou_b_period {
            self.buffer.pop_front();
        }
        if self.buffer.len() < self.senkou_b_period {
            return None;
        }
        let tenkan = Self::midpoint(&self.buffer, self.tenkan_period);
        let kijun = Self::midpoint(&self.buffer, self.kijun_period);
        let senkou_a = (tenkan + kijun) / 2.0;
        let senkou_b = Self::midpoint(&self.buffer, self.senkou_b_period);
        Some(IchimokuResult {
            tenkan,
            kijun,
            senkou_a,
            senkou_b,
            chikou: candle.close,
        })
    }
}

impl Indicator<Candle, IchimokuResult> for Ichimoku {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<IchimokuResult>, IndicatorError> {
        validate_data_length(data, self.senkou_b_period)?;
        self.reset_state();
        let mut out = Vec::with_capacity(data.len() - self.senkou_b_period + 1);
        for c in data {
            if let Some(v) = self.step(*c) {
                out.push(v);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: Candle) -> Result<Option<IchimokuResult>, IndicatorError> {
        Ok(self.step(value))
    }

    fn reset(&mut self) {
        self.reset_state();
    }

    fn name(&self) -> &'static str {
        "Ichimoku"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear_candles(n: usize) -> Vec<Candle> {
        (0..n)
            .map(|i| Candle {
                timestamp: i as u64,
                open: i as f64,
                high: i as f64 + 1.0,
                low: i as f64 - 1.0,
                close: i as f64,
                volume: 1.0,
            })
            .collect()
    }

    #[test]
    fn validates_period_ordering() {
        assert!(Ichimoku::new(0, 26, 52).is_err());
        assert!(Ichimoku::new(26, 9, 52).is_err()); // tenkan > kijun
        assert!(Ichimoku::new(9, 52, 26).is_err()); // kijun > senkou_b
        assert!(Ichimoku::new(9, 26, 52).is_ok());
    }

    #[test]
    fn first_emission_at_senkou_b_period() {
        let mut ichi = Ichimoku::new(2, 4, 6).unwrap();
        let candles = linear_candles(20);
        let mut emissions = 0;
        for c in &candles {
            if ichi.next(*c).unwrap().is_some() {
                emissions += 1;
            }
        }
        // First emission at the 6th bar; 20 - 6 + 1 = 15 emissions.
        assert_eq!(emissions, 15);
    }

    #[test]
    fn senkou_a_is_average_of_tenkan_and_kijun() {
        let mut ichi = Ichimoku::default_params();
        let candles = linear_candles(120);
        let out = ichi.calculate(&candles).unwrap();
        for v in &out {
            assert!((v.senkou_a - (v.tenkan + v.kijun) / 2.0).abs() < 1e-12);
        }
    }

    #[test]
    fn chikou_is_current_close() {
        let mut ichi = Ichimoku::default_params();
        let candles = linear_candles(120);
        let out = ichi.calculate(&candles).unwrap();
        // First emission is at index senkou_b_period - 1 = 51.
        for (offset, v) in out.iter().enumerate() {
            let bar_idx = offset + 51;
            assert_eq!(v.chikou, candles[bar_idx].close);
        }
    }
}
