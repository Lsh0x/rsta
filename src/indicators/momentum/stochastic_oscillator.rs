use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Stochastic Oscillator
///
/// The Stochastic Oscillator is a momentum indicator that shows the location of the close
/// relative to the high-low range over a set number of periods.
///
/// # Example
///
/// ```
/// use rsta::indicators::momentum::{StochasticOscillator, StochasticResult};
/// use rsta::indicators::{Indicator, Candle};
///
/// // Create a Stochastic Oscillator with %K period of 14 and %D period of 3
/// let mut stoch = StochasticOscillator::new(14, 3).unwrap();
///
/// // Create price data with OHLC values (need at least 16 candles for 14 %K period + 3 %D period - 1)
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
///     // Additional candles to calculate %D (3-period SMA of %K)
///     Candle { timestamp: 15, open: 49.0, high: 50.0, low: 48.0, close: 49.5, volume: 2400.0 },
///     Candle { timestamp: 16, open: 49.5, high: 50.5, low: 48.5, close: 50.0, volume: 2500.0 },
///     Candle { timestamp: 17, open: 50.0, high: 51.0, low: 49.0, close: 49.0, volume: 2600.0 }, // Price drop
/// ];
///
/// // Calculate Stochastic values
/// match stoch.calculate(&candles) {
///     Ok(stoch_values) => {
///         // Access the latest Stochastic values
///         if let Some(latest) = stoch_values.last() {
///             println!("%K (Fast): {:.2}", latest.k); // Example output: %K (Fast): 50.00
///             println!("%D (Slow): {:.2}", latest.d); // Example output: %D (Slow): 66.67
///             
///             // Interpret the values
///             if latest.k > 80.0 && latest.d > 80.0 {
///                 println!("Overbought condition");
///             } else if latest.k < 20.0 && latest.d < 20.0 {
///                 println!("Oversold condition");
///             }
///             
///             // Check for crossovers
///             if stoch_values.len() >= 2 {
///                 let previous = stoch_values[stoch_values.len() - 2];
///                 if latest.k > latest.d && previous.k <= previous.d {
///                     println!("Bullish crossover: %K crossed above %D");
///                 } else if latest.k < latest.d && previous.k >= previous.d {
///                     println!("Bearish crossover: %K crossed below %D");
///                 }
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating Stochastic: {}", e);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct StochasticOscillator {
    k_period: usize,
    d_period: usize,
    k_buffer: VecDeque<f64>,
}

impl StochasticOscillator {
    /// Create a new StochasticOscillator
    ///
    /// # Arguments
    /// * `k_period` - The %K period (typically 14) - must be at least 1
    /// * `d_period` - The %D period (typically 3) - must be at least 1
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new StochasticOscillator or an error
    pub fn new(k_period: usize, d_period: usize) -> Result<Self, IndicatorError> {
        validate_period(k_period, 1)?;
        validate_period(d_period, 1)?;

        Ok(Self {
            k_period,
            d_period,
            k_buffer: VecDeque::with_capacity(d_period),
        })
    }

    /// Calculate %K value for a given candle
    ///
    /// # Arguments
    /// * `candles` - The slice of candles to calculate %K from
    /// * `idx` - The index of the current candle
    ///
    /// # Returns
    /// * `f64` - The %K value
    // StochasticOscillator fix - line 291
    fn calculate_k(candles: &[Candle], idx: usize, period: usize) -> f64 {
        if idx < period - 1 {
            return 50.0; // Not enough data, return middle value
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
            return 50.0; // Default to middle value when range is zero
        }

        ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
    }
}

/// Stochastic indicator result
#[derive(Debug, Clone, Copy)]
pub struct StochasticResult {
    /// %K value (fast stochastic)
    pub k: f64,
    /// %D value (slow stochastic - SMA of %K)
    pub d: f64,
}

impl Indicator<Candle, StochasticResult> for StochasticOscillator {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<StochasticResult>, IndicatorError> {
        validate_data_length(data, self.k_period + self.d_period - 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.k_period - self.d_period + 2);

        // Reset state
        self.reset();

        // Calculate %K values
        let mut k_values = Vec::with_capacity(n);
        for i in 0..n {
            k_values.push(Self::calculate_k(data, i, self.k_period));
        }

        // We can only start calculating %D once we have k_period values
        // We can only start calculating %D once we have k_period values
        let k_start_idx = self.k_period - 1;
        for (i, &k_value) in k_values.iter().enumerate().skip(k_start_idx) {
            // Add to buffer
            self.k_buffer.push_back(k_value);

            if self.k_buffer.len() > self.d_period {
                self.k_buffer.pop_front();
            }

            if self.k_buffer.len() == self.d_period {
                // Calculate %D (SMA of %K)
                let d = self.k_buffer.iter().sum::<f64>() / self.d_period as f64;

                result.push(StochasticResult { k: k_values[i], d });
            }
        }
        Ok(result)
    }

    fn next(&mut self, _value: Candle) -> Result<Option<StochasticResult>, IndicatorError> {
        // Implementation would require storing the last k_period candles
        // For simplicity, we're leaving this as an exercise
        Err(IndicatorError::CalculationError(
            "Real-time calculation of Stochastic Oscillator requires storing previous candles"
                .to_string(),
        ))
    }

    fn reset(&mut self) {
        self.k_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // StochasticOscillator Tests
    #[test]
    fn test_stochastic_new() {
        // Valid periods should work
        assert!(StochasticOscillator::new(14, 3).is_ok());

        // Invalid periods should fail
        assert!(StochasticOscillator::new(0, 3).is_err());
        assert!(StochasticOscillator::new(14, 0).is_err());
    }

    #[test]
    fn test_stochastic_calculation() {
        let mut stoch = StochasticOscillator::new(3, 2).unwrap();

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

        let result = stoch.calculate(&candles).unwrap();

        // We expect: 3 candles for k_period + 2 candles for d_period - 1 = 4 candles needed
        // So we get 5 - 4 + 1 = 2 results
        assert_eq!(result.len(), 2);

        // Verify the StochasticResult struct has valid values
        for stoch_result in &result {
            assert!(stoch_result.k >= 0.0 && stoch_result.k <= 100.0);
            assert!(stoch_result.d >= 0.0 && stoch_result.d <= 100.0);
        }
    }

    #[test]
    fn test_stochastic_next_error() {
        let mut stoch = StochasticOscillator::new(14, 3).unwrap();
        let candle = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: 1000.0,
        };

        // The next method should return an error as noted in the implementation
        assert!(stoch.next(candle).is_err());
    }

    #[test]
    fn test_stochastic_reset() {
        let mut stoch = StochasticOscillator::new(14, 3).unwrap();

        // Create data
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
        ];

        // Calculate something
        let _ = stoch.calculate(&candles);

        // Reset
        stoch.reset();

        // k_buffer should be empty after reset
        // We can't directly test the internal state, but we can test the behavior
        // by doing a calculation that requires an empty state
    }
}