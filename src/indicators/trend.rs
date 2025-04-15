//! Trend following indicators
//!
//! This module contains trend following indicators like Moving Averages, MACD, and Bollinger Bands.

use crate::indicators::utils::{calculate_ema, calculate_sma, validate_period};
use crate::indicators::{Indicator, IndicatorError};
use std::collections::VecDeque;

/// Simple Moving Average (SMA) indicator
///
/// # Example
///
/// ```
/// use tars::indicators::{SimpleMovingAverage, Indicator};
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
}

/// Exponential Moving Average (EMA) indicator
///
/// # Example
///
/// ```
/// use tars::indicators::{ExponentialMovingAverage, Indicator};
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
}
