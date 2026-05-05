//! Trend following indicators
//!
//! This module contains trend following indicators like Moving Averages, MACD, and Bollinger Bands.

use crate::indicators::utils::{
    calculate_ema, calculate_sma, validate_data_length, validate_period,
};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Simple Moving Average (SMA) indicator
///
/// # Example
///
/// ```
/// use rsta::indicators::trend::SimpleMovingAverage;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period SMA
/// let mut sma = SimpleMovingAverage::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate SMA values
/// let sma_values = sma.calculate(&prices).unwrap();
/// assert_eq!(sma_values.len(), 6);
/// assert_eq!(sma_values[0], 12.0);
/// ```
#[derive(Debug)]
pub struct SimpleMovingAverage {
    period: usize,
    buffer: VecDeque<f64>,
    sum: f64,
}

impl SimpleMovingAverage {
    /// Create a new SimpleMovingAverage indicator
    ///
    /// # Arguments
    /// * `period` - The period for SMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new SimpleMovingAverage or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }
}

impl Indicator<f64, f64> for SimpleMovingAverage {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        calculate_sma(data, self.period)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.buffer.push_back(value);
        self.sum += value;

        if self.buffer.len() > self.period {
            if let Some(removed) = self.buffer.pop_front() {
                self.sum -= removed;
            }
        }

        if self.buffer.len() < self.period {
            return Ok(None);
        }

        Ok(Some(self.sum / self.period as f64))
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.sum = 0.0;
    }

    fn name(&self) -> &'static str {
        "SMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl SimpleMovingAverage {
    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

/// Exponential Moving Average (EMA) indicator
///
/// # Example
///
/// ```
/// use rsta::indicators::trend::ExponentialMovingAverage;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period EMA
/// let mut ema = ExponentialMovingAverage::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate EMA values
/// let ema_values = ema.calculate(&prices).unwrap();
/// ```
#[derive(Debug)]
pub struct ExponentialMovingAverage {
    period: usize,
    alpha: f64,
    current_ema: Option<f64>,
}

impl ExponentialMovingAverage {
    /// Create a new ExponentialMovingAverage indicator
    ///
    /// # Arguments
    /// * `period` - The period for EMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new ExponentialMovingAverage or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            alpha: 2.0 / (period as f64 + 1.0),
            current_ema: None,
        })
    }

    /// Set the initial EMA value
    ///
    /// # Arguments
    /// * `value` - Initial EMA value
    ///
    /// # Returns
    /// * `&mut Self` - Reference to self for method chaining
    pub fn with_initial_value(&mut self, value: f64) -> &mut Self {
        self.current_ema = Some(value);
        self
    }
}

impl Indicator<f64, f64> for ExponentialMovingAverage {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        calculate_ema(data, self.period)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        if let Some(current) = self.current_ema {
            // Apply EMA formula: EMA_today = (Price_today * alpha) + (EMA_yesterday * (1 - alpha))
            let new_ema = (value * self.alpha) + (current * (1.0 - self.alpha));
            self.current_ema = Some(new_ema);
            Ok(Some(new_ema))
        } else {
            // First value becomes the initial EMA
            self.current_ema = Some(value);
            Ok(Some(value))
        }
    }

    fn reset(&mut self) {
        self.current_ema = None;
    }

    fn name(&self) -> &'static str {
        "EMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl ExponentialMovingAverage {
    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

/// MACD (Moving Average Convergence/Divergence) result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingAverageConvergenceDivergenceResult {
    /// MACD line: `EMA(fast) - EMA(slow)`.
    pub macd: f64,
    /// Signal line: EMA of the MACD line.
    pub signal: f64,
    /// Histogram: `macd - signal`. Positive = bullish momentum.
    pub histogram: f64,
}

/// Moving Average Convergence/Divergence (MACD) indicator.
///
/// MACD is a trend-following momentum indicator that shows the relationship
/// between two exponential moving averages of an asset's price. It is composed
/// of three series:
///
/// - **MACD line** = EMA(fast_period) − EMA(slow_period)
/// - **Signal line** = EMA(signal_period) of the MACD line
/// - **Histogram** = MACD − signal
///
/// Standard parameters are 12/26/9.
///
/// # Example
///
/// ```no_run
/// use rsta::indicators::trend::MovingAverageConvergenceDivergence;
/// use rsta::indicators::Indicator;
///
/// let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
/// let prices: Vec<f64> = (1..=60).map(|i| i as f64).collect();
/// let values = macd.calculate(&prices).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct MovingAverageConvergenceDivergence {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_ema: ExponentialMovingAverage,
    slow_ema: ExponentialMovingAverage,
    signal_ema: ExponentialMovingAverage,
    /// Number of close prices observed so far via `next()`.
    seen: usize,
}

impl MovingAverageConvergenceDivergence {
    /// Create a new MACD indicator.
    ///
    /// # Errors
    /// Returns `IndicatorError::InvalidParameter` if any period is `0` or if
    /// `fast_period >= slow_period` (otherwise the MACD line collapses).
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Result<Self, IndicatorError> {
        validate_period(fast_period, 1)?;
        validate_period(slow_period, 1)?;
        validate_period(signal_period, 1)?;
        if fast_period >= slow_period {
            return Err(IndicatorError::InvalidParameter(
                "fast_period must be strictly less than slow_period".to_string(),
            ));
        }
        Ok(Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: ExponentialMovingAverage::new(fast_period)?,
            slow_ema: ExponentialMovingAverage::new(slow_period)?,
            signal_ema: ExponentialMovingAverage::new(signal_period)?,
            seen: 0,
        })
    }

    /// Fast EMA period.
    pub fn fast_period(&self) -> usize {
        self.fast_period
    }
    /// Slow EMA period.
    pub fn slow_period(&self) -> usize {
        self.slow_period
    }
    /// Signal EMA period.
    pub fn signal_period(&self) -> usize {
        self.signal_period
    }

    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(
        &mut self,
        candles: &[Candle],
    ) -> Result<Vec<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, MovingAverageConvergenceDivergenceResult>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(
        &mut self,
        candle: Candle,
    ) -> Result<Option<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        <Self as Indicator<f64, MovingAverageConvergenceDivergenceResult>>::next(self, candle.close)
    }
}

impl Indicator<f64, MovingAverageConvergenceDivergenceResult>
    for MovingAverageConvergenceDivergence
{
    fn calculate(
        &mut self,
        data: &[f64],
    ) -> Result<Vec<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        // We need at least slow_period prices for the MACD line, plus
        // signal_period values of the MACD line for the signal line. The
        // signal line seeds on its first value, so the very first MACD
        // datapoint produces signal = macd and histogram = 0.
        let needed = self.slow_period;
        validate_data_length(data, needed)?;

        self.reset();

        let mut out = Vec::with_capacity(data.len().saturating_sub(needed - 1));
        for &price in data {
            if let Some(point) = self.next(price)? {
                out.push(point);
            }
        }
        Ok(out)
    }

    fn next(
        &mut self,
        value: f64,
    ) -> Result<Option<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        self.seen += 1;
        let fast = self.fast_ema.next(value)?;
        let slow = self.slow_ema.next(value)?;

        // Emit nothing until both EMAs have stabilised over their full
        // window, which by convention is `slow_period` close prices.
        if self.seen < self.slow_period {
            return Ok(None);
        }

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd = f - s;
                let signal = self
                    .signal_ema
                    .next(macd)?
                    .expect("signal EMA always emits on first value");
                Ok(Some(MovingAverageConvergenceDivergenceResult {
                    macd,
                    signal,
                    histogram: macd - signal,
                }))
            }
            _ => Ok(None),
        }
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.seen = 0;
    }

    fn name(&self) -> &'static str {
        "MACD"
    }

    fn period(&self) -> Option<usize> {
        // MACD has three periods; surfacing only the slow one would be
        // misleading.
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_new() {
        // Valid period should work
        assert!(SimpleMovingAverage::new(14).is_ok());

        // Invalid period should fail
        assert!(SimpleMovingAverage::new(0).is_err());
    }

    #[test]
    fn test_sma_calculation() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = sma.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 4.0); // (2+4+6)/3
        assert_eq!(result[1], 6.0); // (4+6+8)/3
        assert_eq!(result[2], 8.0); // (6+8+10)/3
    }

    #[test]
    fn test_sma_next() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();

        // Initial values - not enough data yet
        assert_eq!(sma.next(2.0).unwrap(), None);
        assert_eq!(sma.next(4.0).unwrap(), None);

        // Third value - now we have an SMA
        assert_eq!(sma.next(6.0).unwrap(), Some(4.0));

        // More values - sliding window
        assert_eq!(sma.next(8.0).unwrap(), Some(6.0));
        assert_eq!(sma.next(10.0).unwrap(), Some(8.0));
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();

        // Add some values
        sma.next(2.0).unwrap();
        sma.next(4.0).unwrap();
        sma.next(6.0).unwrap();

        // Reset
        sma.reset();

        // Should be back to initial state
        assert_eq!(sma.next(8.0).unwrap(), None);
    }

    #[test]
    fn test_ema_new() {
        // Valid period should work
        assert!(ExponentialMovingAverage::new(14).is_ok());

        // Invalid period should fail
        assert!(ExponentialMovingAverage::new(0).is_err());
    }

    #[test]
    fn test_ema_calculation() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = ema.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);

        // First EMA is SMA of first 3 values
        assert_eq!(result[0], 4.0); // (2+4+6)/3

        // Rest follow EMA formula with alpha = 2/(3+1) = 0.5
        let alpha = 0.5;
        let expected1 = 8.0 * alpha + 4.0 * (1.0 - alpha); // 6.0
        let expected2 = 10.0 * alpha + expected1 * (1.0 - alpha); // 8.0

        assert_eq!(result[1], expected1);
        assert_eq!(result[2], expected2);
    }

    #[test]
    fn test_ema_next() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();
        let alpha = 0.5; // alpha = 2/(3+1)

        // First value becomes the seed
        assert_eq!(ema.next(2.0).unwrap(), Some(2.0));

        // Next values follow EMA formula
        let expected1 = 4.0 * alpha + 2.0 * (1.0 - alpha); // 3.0
        assert_eq!(ema.next(4.0).unwrap(), Some(expected1));

        let expected2 = 6.0 * alpha + expected1 * (1.0 - alpha); // 4.5
        assert_eq!(ema.next(6.0).unwrap(), Some(expected2));
    }

    #[test]
    fn test_ema_reset() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();

        // Add some values
        ema.next(2.0).unwrap();
        ema.next(4.0).unwrap();

        // Reset
        ema.reset();

        // Should be back to initial state, next value becomes seed
        assert_eq!(ema.next(6.0).unwrap(), Some(6.0));
    }

    #[test]
    fn test_macd_construction_validates_periods() {
        // fast must be strictly less than slow
        assert!(MovingAverageConvergenceDivergence::new(26, 26, 9).is_err());
        assert!(MovingAverageConvergenceDivergence::new(30, 26, 9).is_err());
        // any zero period is rejected
        assert!(MovingAverageConvergenceDivergence::new(0, 26, 9).is_err());
        assert!(MovingAverageConvergenceDivergence::new(12, 0, 9).is_err());
        assert!(MovingAverageConvergenceDivergence::new(12, 26, 0).is_err());
        // standard parameters work
        assert!(MovingAverageConvergenceDivergence::new(12, 26, 9).is_ok());
    }

    #[test]
    fn test_macd_emits_after_warmup() {
        let mut macd = MovingAverageConvergenceDivergence::new(3, 6, 2).unwrap();
        // Slow period is 6, so the first 5 closes produce no output.
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
            assert!(macd.next(v).unwrap().is_none(), "premature emission");
        }
        let first = macd.next(6.0).unwrap().expect("should emit at slow_period");
        // First emission: signal seeded with current MACD → histogram == 0.
        assert!(first.histogram.abs() < 1e-12);
        assert!((first.macd - first.signal).abs() < 1e-12);
    }

    #[test]
    fn test_macd_calculate_matches_streaming() {
        let prices: Vec<f64> = (1..=40).map(|i| i as f64).collect();
        let mut batch = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        let batch_out = batch.calculate(&prices).unwrap();

        let mut stream = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        let stream_out: Vec<_> = prices
            .iter()
            .filter_map(|&p| stream.next(p).unwrap())
            .collect();

        assert_eq!(batch_out, stream_out);
    }

    #[test]
    fn test_macd_histogram_definition() {
        let prices: Vec<f64> = (1..=40).map(|i| i as f64).collect();
        let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        let out = macd.calculate(&prices).unwrap();
        for v in &out {
            assert!((v.histogram - (v.macd - v.signal)).abs() < 1e-12);
        }
    }
}
