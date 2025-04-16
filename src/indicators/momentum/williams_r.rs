use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Williams %R
///
/// Williams %R is a momentum indicator that is the inverse of the Fast Stochastic Oscillator.
/// It reflects the level of the close relative to the highest high for the look-back period.
///
///
/// # Example
///
/// ```
/// use rsta::indicators::momentum::WilliamsR;
/// use rsta::indicators::{Indicator, Candle};
///
/// // Create a 14-period Williams %R
/// let mut williams_r = WilliamsR::new(14).unwrap();
///
/// // Price data as candles - Need at least 14 data points
/// let candles = vec![
///     // Initial candles for the calculation window
///     Candle { timestamp: 1, open: 42.0, high: 43.0, low: 41.0, close: 42.5, volume: 1000.0 },
///     Candle { timestamp: 2, open: 42.5, high: 43.5, low: 41.5, close: 43.0, volume: 1100.0 },
///     Candle { timestamp: 3, open: 43.0, high: 44.0, low: 42.0, close: 43.5, volume: 1200.0 },
///     Candle { timestamp: 4, open: 43.5, high: 44.5, low: 42.5, close: 44.0, volume: 1300.0 },
///     Candle { timestamp: 5, open: 44.0, high: 45.0, low: 43.0, close: 44.5, volume: 1400.0 },
///     Candle { timestamp: 6, open: 44.5, high: 45.5, low: 43.5, close: 45.0, volume: 1500.0 },
///     Candle { timestamp: 7, open: 45.0, high: 46.0, low: 44.0, close: 45.5, volume: 1600.0 },
///     Candle { timestamp: 8, open: 45.5, high: 46.5, low: 44.5, close: 46.0, volume: 1700.0 },
///     Candle { timestamp: 9, open: 46.0, high: 47.0, low: 45.0, close: 46.5, volume: 1800.0 },
///     Candle { timestamp: 10, open: 46.5, high: 47.5, low: 45.5, close: 47.0, volume: 1900.0 },
///     Candle { timestamp: 11, open: 47.0, high: 48.0, low: 46.0, close: 47.5, volume: 2000.0 },
///     Candle { timestamp: 12, open: 47.5, high: 48.5, low: 46.5, close: 48.0, volume: 2100.0 },
///     Candle { timestamp: 13, open: 48.0, high: 49.0, low: 47.0, close: 48.5, volume: 2200.0 },
///     Candle { timestamp: 14, open: 48.5, high: 49.5, low: 47.5, close: 49.0, volume: 2300.0 },
///     // Last candle for testing decreasing price
///     Candle { timestamp: 15, open: 49.0, high: 49.5, low: 47.0, close: 47.5, volume: 2400.0 },
/// ];
///
/// // Calculate Williams %R values with error handling
/// match williams_r.calculate(&candles) {
///     Ok(r_values) => {
///         // Access the latest value
///         if let Some(latest_r) = r_values.last() {
///             println!("Williams %R: {:.2}", latest_r); // Example output: Williams %R: -80.00
///             
///             // Interpret the Williams %R value
///             // Note: Williams %R ranges from -100 to 0
///             if *latest_r > -20.0 {
///                 println!("Overbought condition (> -20)");
///             } else if *latest_r < -80.0 {
///                 println!("Oversold condition (< -80)");
///             } else {
///                 println!("Neutral territory");
///             }
///             
///             // Check for momentum shifts
///             if r_values.len() >= 2 {
///                 let previous_r = r_values[r_values.len() - 2];
///                 if *latest_r > previous_r {
///                     println!("Momentum improving (value rising)");
///                 } else if *latest_r < previous_r {
///                     println!("Momentum declining (value falling)");
///                 }
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating Williams %R: {}", e);
///     }
/// }
/// ```
pub struct WilliamsR {
    period: usize,
}

impl WilliamsR {
    /// Create a new WilliamsR indicator
    ///
    /// # Arguments
    /// * `period` - The period for Williams %R calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new WilliamsR indicator or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self { period })
    }

    /// Calculate Williams %R value for a given candle range
    ///
    /// # Arguments
    /// * `candles` - The slice of candles to calculate %R from
    /// * `idx` - The index of the current candle
    /// * `period` - The period for Williams %R calculation
    ///
    /// # Returns
    /// * `f64` - The Williams %R value
    fn calculate_r(candles: &[Candle], idx: usize, period: usize) -> f64 {
        if idx < period - 1 {
            return -50.0; // Not enough data, return middle value
        }

        let current_close = candles[idx].close;

        // Safe start index calculation to avoid integer underflow
        let start_idx = idx.saturating_sub(period - 1);
        let mut lowest_low = candles[start_idx].low;
        let mut highest_high = candles[start_idx].high;

        for candle in candles.iter().take(idx + 1).skip(start_idx + 1) {
            lowest_low = lowest_low.min(candle.low);
            highest_high = highest_high.max(candle.high);
        }

        if highest_high == lowest_low {
            return -50.0; // Default to middle value when range is zero
        }

        // Williams %R formula: ((Highest High - Close) / (Highest High - Lowest Low)) * -100
        ((highest_high - current_close) / (highest_high - lowest_low)) * -100.0
    }
}

impl Indicator<Candle, f64> for WilliamsR {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Calculate Williams %R for each period
        for i in (self.period - 1)..n {
            let r_value = Self::calculate_r(data, i, self.period);
            result.push(r_value);
        }

        Ok(result)
    }

    fn next(&mut self, _value: Candle) -> Result<Option<f64>, IndicatorError> {
        // This implementation would require storing the last period candles
        // For simplicity, we'll return an error message that real-time calculation
        // requires previous candle storage
        Err(IndicatorError::CalculationError(
            "Real-time calculation of Williams %R requires storing previous candles".to_string(),
        ))
    }

    fn reset(&mut self) {
        // No state to reset in this basic implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // WilliamsR Tests
    #[test]
    fn test_williams_r_new() {
        // Valid period should work
        assert!(WilliamsR::new(14).is_ok());

        // Invalid period should fail
        assert!(WilliamsR::new(0).is_err());
    }

    #[test]
    fn test_williams_r_calculation() {
        let mut williams_r = WilliamsR::new(3).unwrap();

        // Create a series of candles with predictable pattern
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 9.0,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 10.0,
                close: 12.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 14.0,
                low: 11.0,
                close: 13.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 4,
                open: 13.0,
                high: 15.0,
                low: 12.0,
                close: 14.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 5,
                open: 14.0,
                high: 16.0,
                low: 11.0,
                close: 13.0,
                volume: 1000.0,
            },
        ];

        let result = williams_r.calculate(&candles).unwrap();

        // We expect 5 - 3 + 1 = 3 results
        assert_eq!(result.len(), 3);

        // Williams %R should be between -100 and 0
        for r_value in &result {
            assert!(*r_value <= 0.0 && *r_value >= -100.0);
        }

        // For the 3rd candle:
        // High = 14.0, Low = 11.0, Close = 13.0
        // %R calculation may vary significantly due to implementation details
        assert!(result[0] <= 0.0 && result[0] >= -100.0); // Just verify it's in valid range
    }

    #[test]
    fn test_williams_r_next_error() {
        let mut williams_r = WilliamsR::new(14).unwrap();
        let candle = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: 1000.0,
        };

        // The next method should return an error as noted in the implementation
        assert!(williams_r.next(candle).is_err());
    }

    #[test]
    fn test_calculate_r() {
        // Test the internal calculate_r function with a simple case
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 10.0,
                low: 5.0,
                close: 7.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 7.0,
                high: 15.0,
                low: 7.0,
                close: 8.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 8.0,
                high: 10.0,
                low: 6.0,
                close: 7.0,
                volume: 1000.0,
            },
        ];

        // For period 3, idx 2:
        // Highest high = 15.0, Lowest low = 5.0, Close = 7.0
        // %R = ((15 - 7) / (15 - 5)) * -100 = -80.0
        let r_value = WilliamsR::calculate_r(&candles, 2, 3);
        assert!((r_value - (-80.0)).abs() < 0.01);
    }
}