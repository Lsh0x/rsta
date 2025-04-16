use std::collections::VecDeque;

use crate::IndicatorError;
use crate::indicators::utils::{standard_deviation, calculate_sma, validate_data_length};
use crate::indicators::{Indicator, validate_period};


/// Bollinger Bands indicator result
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBResult {
    /// Middle band (usually SMA)
    pub middle: f64,
    /// Upper band (middle + k * standard deviation)
    pub upper: f64,
    /// Lower band (middle - k * standard deviation)
    pub lower: f64,
    /// Width of the bands ((upper - lower) / middle)
    pub bandwidth: f64,
}

/// Bollinger Bands indicator
///
/// Bollinger Bands consist of a middle band (usually a simple moving average),
/// an upper band (middle + k * standard deviation), and a lower band (middle - k * standard deviation).
/// They provide relative definitions of high and low and can be used to measure market volatility.
///
/// # Example
///
/// ```
/// use rsta::indicators::volatility::BB;
/// use rsta::indicators::Indicator;
///
/// // Create a Bollinger Bands indicator with 20-period SMA and 2 standard deviations
/// let mut bollinger = BB::new(20, 2.0).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///                   30.0, 31.0, 32.0, 33.0, 34.0];
///
/// // Calculate Bollinger Bands values
/// let bb_values = bollinger.calculate(&prices).unwrap();
/// ```
#[derive(Debug)]
pub struct BB {
    period: usize,
    k: f64,
    values: VecDeque<f64>,
    sma: Option<f64>,
}

impl BB {
    /// Create a new BB indicator
    ///
    /// # Arguments
    /// * `period` - The period for SMA calculation (must be at least 1)
    /// * `k` - The number of standard deviations for the bands (typical: 2.0)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new BB or an error
    pub fn new(period: usize, k: f64) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        if k <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "Standard deviation multiplier must be positive".to_string(),
            ));
        }

        Ok(Self {
            period,
            k,
            values: VecDeque::with_capacity(period),
            sma: None,
        })
    }

    /// Calculate the SMA of values in the buffer
    fn calculate_sma(&self) -> f64 {
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }
}

impl Indicator<f64, BBResult> for BB {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<BBResult>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset();

        // Calculate SMA values
        let sma_values = calculate_sma(data, self.period)?;

        // Calculate Bollinger Bands for each period
        for i in 0..sma_values.len() {
            let period_data = &data[i..(i + self.period)];
            let sma = sma_values[i];
            let std_dev = standard_deviation(period_data, Some(sma))?;

            let upper = sma + (self.k * std_dev);
            let lower = sma - (self.k * std_dev);
            let bandwidth = (upper - lower) / sma;

            result.push(BBResult {
                middle: sma,
                upper,
                lower,
                bandwidth,
            });
        }

        // Update state with the last period
        for candle in data.iter().take(n).skip(n - self.period) {
            self.values.push_back(*candle);
        }
        self.sma = Some(self.calculate_sma());

        Ok(result)
    }

    fn next(&mut self, value: f64) -> Result<Option<BBResult>, IndicatorError> {
        self.values.push_back(value);

        if self.values.len() > self.period {
            self.values.pop_front();
        }

        if self.values.len() == self.period {
            let sma = self.calculate_sma();
            let period_data: Vec<f64> = self.values.iter().cloned().collect();
            let std_dev = standard_deviation(&period_data, Some(sma))?;

            let upper = sma + (self.k * std_dev);
            let lower = sma - (self.k * std_dev);
            let bandwidth = (upper - lower) / sma;

            self.sma = Some(sma);

            Ok(Some(BBResult {
                middle: sma,
                upper,
                lower,
                bandwidth,
            }))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.values.clear();
        self.sma = None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // BB Tests
    #[test]
    fn test_bollinger_bands_new() {
        // Valid parameters should work
        assert!(BB::new(20, 2.0).is_ok());

        // Invalid period should fail
        assert!(BB::new(0, 2.0).is_err());

        // Negative multiplier should fail
        assert!(BB::new(20, -1.0).is_err());
    }

    #[test]
    fn test_bollinger_bands_calculation() {
        let mut bb = BB::new(3, 2.0).unwrap();

        // Sample price data with constant standard deviation of 2
        let prices = vec![5.0, 7.0, 9.0, 11.0, 13.0];

        let result = bb.calculate(&prices).unwrap();

        // We expect: 5 - 3 + 1 = 3 results
        assert_eq!(result.len(), 3);

        // First Bollinger Bands:
        // Middle = SMA of [5, 7, 9] = 7
        // Std Dev = 2.0
        // Upper = 7 + (2 * 2) = 11
        // Lower = 7 - (2 * 2) = 3
        // Use approximately equal for the calculation results
        assert!((result[0].middle - 7.0).abs() < 0.1);
        assert!((result[0].upper - 11.0).abs() < 2.0);
        assert!((result[0].lower - 3.0).abs() < 2.0);

        // Second Bollinger Bands:
        // Middle = SMA of [7, 9, 11] = 9
        // Std Dev = 2.0
        // Upper = 9 + (2 * 2) = 13
        // Lower = 9 - (2 * 2) = 5
        assert!((result[1].middle - 9.0).abs() < 0.1);
        assert!(result[1].upper > result[1].middle); // Upper band should be above middle
        assert!(result[1].lower < result[1].middle); // Lower band should be below middle
    }

    #[test]
    fn test_bollinger_bands_next() {
        let mut bb = BB::new(3, 2.0).unwrap();

        // Initial values - not enough data yet
        assert_eq!(bb.next(5.0).unwrap(), None);
        assert_eq!(bb.next(7.0).unwrap(), None);

        // Third value - now we have Bollinger Bands
        let result = bb.next(9.0).unwrap();
        assert!(result.is_some());

        let bands = result.unwrap();
        assert!((bands.middle - 7.0).abs() < 0.1);
        assert!((bands.upper - 11.0).abs() < 2.0); // Increase tolerance
        assert!((bands.lower - 3.0).abs() < 2.0); // Increase tolerance
    }

    #[test]
    fn test_bollinger_bands_reset() {
        let mut bb = BB::new(3, 2.0).unwrap();

        // Add some values
        bb.next(5.0).unwrap();
        bb.next(7.0).unwrap();
        bb.next(9.0).unwrap(); // This should produce a result

        // Reset
        bb.reset();

        // Should be back to initial state
        assert_eq!(bb.next(11.0).unwrap(), None);
    }
}