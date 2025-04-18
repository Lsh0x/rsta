use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Williams %R
///
/// Williams %R is a momentum indicator that is the inverse of the Fast Stochastic Oscillator.
/// It reflects the level of the close relative to the highest high for the look-back period.
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
///     // ... more candles ...
/// ];
///
/// // Calculate Williams %R values with error handling
/// match williams_r.calculate(&candles) {
///     Ok(r_values) => {
///         // Access the latest value
///         if let Some(latest_r) = r_values.last() {
///             println!("Williams %R: {:.2}", latest_r);
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating Williams %R: {}", e);
///     }
/// }
/// ```
pub struct WilliamsR {
    period: usize,
    history: Vec<Candle>, // Added history to store candles for real-time calculation
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

        Ok(Self {
            period,
            history: Vec::with_capacity(period), // Initialize history with capacity
        })
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

        // Update history with the most recent candles for future next() calls
        self.history.clear();
        if n >= self.period {
            self.history.extend_from_slice(&data[n - self.period..]);
        } else {
            self.history.extend_from_slice(data);
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        // Add the new candle to history
        self.history.push(value);

        // If we have more candles than needed, remove the oldest one
        if self.history.len() > self.period {
            self.history.remove(0);
        }

        // If we don't have enough data yet, return None
        if self.history.len() < self.period {
            return Ok(None);
        }

        // Calculate Williams %R using the history
        let r_value = Self::calculate_r(&self.history, self.history.len() - 1, self.period);
        Ok(Some(r_value))
    }

    fn reset(&mut self) {
        // Clear the history
        self.history.clear();
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
    }

    #[test]
    fn test_williams_r_next() {
        let mut williams_r = WilliamsR::new(3).unwrap();

        // Add initial candles one by one
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: 1000.0,
        };
        let candle2 = Candle {
            timestamp: 2,
            open: 11.0,
            high: 13.0,
            low: 10.0,
            close: 12.0,
            volume: 1000.0,
        };

        // First two candles should return None (not enough data)
        assert_eq!(williams_r.next(candle1).unwrap(), None);
        assert_eq!(williams_r.next(candle2).unwrap(), None);

        // Third candle should return a value
        let candle3 = Candle {
            timestamp: 3,
            open: 12.0,
            high: 14.0,
            low: 11.0,
            close: 13.0,
            volume: 1000.0,
        };
        let result3 = williams_r.next(candle3).unwrap();
        assert!(result3.is_some());
        let r_value3 = result3.unwrap();
        assert!((-100.0..=0.0).contains(&r_value3));

        // Fourth candle should return a value and maintain sliding window
        let candle4 = Candle {
            timestamp: 4,
            open: 13.0,
            high: 15.0,
            low: 12.0,
            close: 14.0,
            volume: 1000.0,
        };
        let result4 = williams_r.next(candle4).unwrap();
        assert!(result4.is_some());
        let r_value4 = result4.unwrap();
        assert!((-100.0..=0.0).contains(&r_value4));

        // Verify reset clears history
        williams_r.reset();
        let candle5 = Candle {
            timestamp: 5,
            open: 14.0,
            high: 16.0,
            low: 11.0,
            close: 13.0,
            volume: 1000.0,
        };
        assert_eq!(williams_r.next(candle5).unwrap(), None);
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
