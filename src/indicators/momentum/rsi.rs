use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Indicator, IndicatorError};
use std::collections::VecDeque;

/// Relative Strength Index (RSI) indicator
///
/// RSI measures the magnitude of recent price changes to evaluate
/// overbought or oversold conditions. The RSI ranges from 0 to 100.
/// Traditionally, RSI values of 70 or above indicate overbought conditions,
/// while values of 30 or below indicate oversold conditions.
///
/// # Example
///
/// ```
/// use rsta::indicators::momentum::RSI;
/// use rsta::indicators::Indicator;
///
/// // Create a 14-period RSI
/// let mut rsi = RSI::new(14).unwrap();
///
/// // Price data
/// let prices = vec![44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42,
///                   45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28, 46.00,
///                   46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 45.71, 46.45];
///
/// // Calculate RSI values
/// let rsi_values = rsi.calculate(&prices).unwrap();
/// ```
#[derive(Debug)]
pub struct RSI {
    period: usize,
    prev_price: Option<f64>,
    gains: VecDeque<f64>,
    losses: VecDeque<f64>,
    avg_gain: Option<f64>,
    avg_loss: Option<f64>,
}

impl RSI {
    /// Create a new RSI indicator
    ///
    /// # Arguments
    /// * `period` - The period for RSI calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new RSI or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            prev_price: None,
            gains: VecDeque::with_capacity(period),
            losses: VecDeque::with_capacity(period),
            avg_gain: None,
            avg_loss: None,
        })
    }

    /// Calculate a single RSI value from average gain and loss
    ///
    /// # Arguments
    /// * `avg_gain` - The average gain over the period
    /// * `avg_loss` - The average loss over the period
    ///
    /// # Returns
    /// * `f64` - The RSI value
    fn calculate_rsi(avg_gain: f64, avg_loss: f64) -> f64 {
        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
}

impl Indicator<f64, f64> for RSI {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period + 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period);

        // Reset state
        self.reset();

        // First, calculate price changes
        let mut price_changes = Vec::with_capacity(n - 1);
        for i in 1..n {
            price_changes.push(data[i] - data[i - 1]);
        }

        // Calculate initial gains and losses
        let mut gains = Vec::with_capacity(self.period);
        let mut losses = Vec::with_capacity(self.period);

        for &change in price_changes.iter().take(self.period) {
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        // Calculate first average gain and loss
        let mut avg_gain = gains.iter().sum::<f64>() / self.period as f64;
        let mut avg_loss = losses.iter().sum::<f64>() / self.period as f64;

        // Calculate first RSI
        result.push(Self::calculate_rsi(avg_gain, avg_loss));

        // Calculate the rest using the smoothed method
        for change in price_changes.iter().skip(self.period).copied() {
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            // Use Wilder's smoothing method
            avg_gain = (avg_gain * (self.period - 1) as f64 + gain) / self.period as f64;
            avg_loss = (avg_loss * (self.period - 1) as f64 + loss) / self.period as f64;

            result.push(Self::calculate_rsi(avg_gain, avg_loss));
        }

        Ok(result)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        if let Some(prev) = self.prev_price {
            let change = value - prev;
            let gain = if change > 0.0 { change } else { 0.0 };
            let loss = if change < 0.0 { -change } else { 0.0 };

            self.gains.push_back(gain);
            self.losses.push_back(loss);

            if self.gains.len() > self.period {
                self.gains.pop_front();
                self.losses.pop_front();
            }

            if self.gains.len() < self.period {
                self.avg_gain = None;
                self.avg_loss = None;
                self.prev_price = Some(value);
                return Ok(None);
            }

            // Calculate/update average gain and loss
            if let (Some(avg_gain), Some(avg_loss)) = (self.avg_gain, self.avg_loss) {
                // Use Wilder's smoothing method for ongoing calculations
                self.avg_gain = Some((avg_gain * (self.period - 1) as f64 + gain) / self.period as f64);
                self.avg_loss = Some((avg_loss * (self.period - 1) as f64 + loss) / self.period as f64);
            } else {
                // Initial average calculation
                self.avg_gain = Some(self.gains.iter().sum::<f64>() / self.period as f64);
                self.avg_loss = Some(self.losses.iter().sum::<f64>() / self.period as f64);
            }

            let rsi = Self::calculate_rsi(self.avg_gain.unwrap(), self.avg_loss.unwrap());

            self.prev_price = Some(value);
            Ok(Some(rsi))
        } else {
            self.prev_price = Some(value);
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.prev_price = None;
        self.gains.clear();
        self.losses.clear();
        self.avg_gain = None;
        self.avg_loss = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RSI Tests
    #[test]
    fn test_rsi_new() {
        // Valid period should work
        assert!(RSI::new(14).is_ok());

        // Invalid period should fail
        assert!(RSI::new(0).is_err());
    }

    #[test]
    fn test_rsi_calculation() {

        let mut rsi = RSI::new(3).unwrap();

        // Sample price data
        let prices = vec![10.0, 11.0, 10.5, 11.5, 12.0, 11.0, 11.5];

        let result = rsi.calculate(&prices).unwrap();
        assert_eq!(result.len(), 4); // 7 prices - 3 period = 4 results

        // First RSI: price changes = [1.0, -0.5, 1.0]
        // Average gain = (1.0 + 0.0 + 1.0) / 3 = 0.6667
        // Average loss = (0.0 + 0.5 + 0.0) / 3 = 0.1667
        // RS = 0.6667 / 0.1667 = 4.0
        // RSI = 100 - (100 / (1 + 4.0)) = 100 - 20 = 80.0
        assert!((result[0] - 80.0).abs() < 0.01);

        // Check final value is correct
        // Last price change is 0.5, a gain
        // Using Wilder's smoothing method
        let last_value = result.last().unwrap();
        assert!(*last_value >= 0.0 && *last_value <= 100.0);
    }

    #[test]
    fn test_rsi_next() {
        let mut rsi = RSI::new(3).unwrap();

        // Initial values - not enough data yet
        assert_eq!(rsi.next(10.0).unwrap(), None);
        assert_eq!(rsi.next(11.0).unwrap(), None);
        assert_eq!(rsi.next(10.5).unwrap(), None);

        // Fourth value - now we have RSI
        let first_rsi = rsi.next(11.5).unwrap();
        assert!(first_rsi.is_some());
        let first_rsi_value = first_rsi.unwrap();
        assert!((0.0..=100.0).contains(&first_rsi_value));

        // More values - should keep producing results
        assert!(rsi.next(12.0).unwrap().is_some());
        assert!(rsi.next(11.0).unwrap().is_some());
    }

    #[test]
    fn test_rsi_reset() {
        let mut rsi = RSI::new(3).unwrap();

        // Add some values
        rsi.next(10.0).unwrap();
        rsi.next(11.0).unwrap();
        rsi.next(10.5).unwrap();
        rsi.next(11.5).unwrap(); // This should produce a result

        // Reset
        rsi.reset();

        // Should be back to initial state
        assert_eq!(rsi.next(12.0).unwrap(), None);
    }
}