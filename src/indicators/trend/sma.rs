use crate::indicators::utils::{calculate_sma, validate_period};
use crate::indicators::{Indicator, IndicatorError};
use std::collections::VecDeque;

/// Simple Moving Average (SMA) indicator
///
/// # Example
///
/// ```
/// use rsta::indicators::trend::Sma;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period SMA
/// let mut sma = Sma::new(5).unwrap();
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
pub struct Sma {
    period: usize,
    buffer: VecDeque<f64>,
    sum: f64,
}

impl Sma {
    /// Create a new SMA indicator
    ///
    /// # Arguments
    /// * `period` - The period for SMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new SMA or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }
}

impl Indicator<f64, f64> for Sma {
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

#[cfg(test)]
mod tests {
    use super::Sma;
    use crate::indicators::Indicator;
    #[test]
    fn test_sma_new() {
        // Valid period should work
        assert!(Sma::new(14).is_ok());

        // Invalid period should fail
        assert!(Sma::new(0).is_err());
    }

    #[test]
    fn test_sma_calculation() {
        let mut sma = Sma::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = sma.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 4.0); // (2+4+6)/3
        assert_eq!(result[1], 6.0); // (4+6+8)/3
        assert_eq!(result[2], 8.0); // (6+8+10)/3
    }

    #[test]
    fn test_sma_next() {
        let mut sma = Sma::new(3).unwrap();

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
        let mut sma = Sma::new(3).unwrap();

        // Add some values
        sma.next(2.0).unwrap();
        sma.next(4.0).unwrap();
        sma.next(6.0).unwrap();

        // Reset
        sma.reset();

        // Should be back to initial state
        assert_eq!(sma.next(8.0).unwrap(), None);
    }
}
