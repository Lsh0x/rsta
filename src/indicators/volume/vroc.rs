use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Volume Rate of Change indicator
///
/// Volume Rate of Change measures the percentage change in volume over a given period.
/// This can be used to confirm price movements and identify potential reversals.
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::Vroc;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create a 14-period Volume Rate of Change indicator
/// let mut vroc = Vroc::new(14).unwrap();
///
/// // Create price data with volume values (need at least 15 candles for a 14-period calculation)
/// let candles = vec![
///     // Initial period (baseline volume)
///     Candle { timestamp: 1, open: 42.0, high: 43.0, low: 41.0, close: 42.5, volume: 1000.0 },
///     // Increasing volume trend
///     Candle { timestamp: 2, open: 42.5, high: 43.5, low: 41.5, close: 43.0, volume: 1050.0 },
///     Candle { timestamp: 3, open: 43.0, high: 44.0, low: 42.0, close: 43.5, volume: 1100.0 },
///     Candle { timestamp: 4, open: 43.5, high: 44.5, low: 42.5, close: 44.0, volume: 1200.0 },
///     Candle { timestamp: 5, open: 44.0, high: 45.0, low: 43.0, close: 44.5, volume: 1300.0 },
///     // Stable volume period
///     Candle { timestamp: 6, open: 44.5, high: 45.5, low: 43.5, close: 45.0, volume: 1320.0 },
///     Candle { timestamp: 7, open: 45.0, high: 46.0, low: 44.0, close: 45.5, volume: 1310.0 },
///     Candle { timestamp: 8, open: 45.5, high: 46.5, low: 44.5, close: 46.0, volume: 1330.0 },
///     // Volume surge (potential breakout)
///     Candle { timestamp: 9, open: 46.0, high: 47.0, low: 45.0, close: 46.8, volume: 2000.0 },
///     Candle { timestamp: 10, open: 46.8, high: 48.0, low: 46.5, close: 47.5, volume: 2200.0 },
///     // Volume declining (momentum fading)
///     Candle { timestamp: 11, open: 47.5, high: 48.0, low: 47.0, close: 47.8, volume: 1800.0 },
///     Candle { timestamp: 12, open: 47.8, high: 48.2, low: 47.2, close: 47.6, volume: 1500.0 },
///     Candle { timestamp: 13, open: 47.6, high: 48.0, low: 47.0, close: 47.4, volume: 1200.0 },
///     Candle { timestamp: 14, open: 47.4, high: 47.8, low: 46.8, close: 47.0, volume: 900.0 },
///     // Current candle (compared against first candle for 14-period calculation)
///     Candle { timestamp: 15, open: 47.0, high: 47.5, low: 46.5, close: 47.2, volume: 800.0 },
///     // Additional candle to see trend continuation
///     Candle { timestamp: 16, open: 47.2, high: 47.6, low: 46.8, close: 47.0, volume: 700.0 },
/// ];
///
/// // Calculate VROC values with error handling
/// match vroc.calculate(&candles) {
///     Ok(vroc_values) => {
///         // Access the latest VROC value
///         if let Some(latest_vroc) = vroc_values.last() {
///             println!("Volume Rate of Change: {:.2}%", latest_vroc); // Example output: -20.00%
///             
///             // Interpret the VROC value
///             if *latest_vroc > 0.0 {
///                 println!("Volume is higher than 14 periods ago");
///                 
///                 if *latest_vroc > 25.0 {
///                     println!("Significant volume increase - potential for trend continuation");
///                 } else if *latest_vroc > 10.0 {
///                     println!("Moderate volume increase - growing interest");
///                 } else {
///                     println!("Slight volume increase - maintain vigilance");
///                 }
///             } else if *latest_vroc < 0.0 {
///                 println!("Volume is lower than 14 periods ago");
///                 
///                 if *latest_vroc < -25.0 {
///                     println!("Significant volume decrease - waning interest");
///                 } else if *latest_vroc < -10.0 {
///                     println!("Moderate volume decrease - potential trend exhaustion");
///                 } else {
///                     println!("Slight volume decrease - monitor closely");
///                 }
///             } else {
///                 println!("Volume unchanged from 14 periods ago");
///             }
///             
///             // Check for volume divergence with price
///             if vroc_values.len() >= 2 {
///                 let previous_vroc = vroc_values[vroc_values.len() - 2];
///                 let current_close = candles.last().unwrap().close;
///                 let previous_close = candles[candles.len() - 2].close;
///                 
///                 // Potential bearish divergence: Price rising but volume falling
///                 if current_close > previous_close && *latest_vroc < previous_vroc {
///                     println!("Warning: Price rising but volume trend declining - potential weakness");
///                 }
///                 
///                 // Potential bullish confirmation: Price and volume both rising
///                 if current_close > previous_close && *latest_vroc > previous_vroc && *latest_vroc > 0.0 {
///                     println!("Bullish: Price and volume both increasing - strong trend confirmation");
///                 }
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating Volume Rate of Change: {}", e);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Vroc {
    period: usize,
    volume_buffer: VecDeque<f64>,
}

impl Vroc {
    /// Create a new Vroc indicator
    ///
    /// # Arguments
    /// * `period` - The period for VROC calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new Vroc or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            volume_buffer: VecDeque::with_capacity(period + 1),
        })
    }
}

impl Indicator<Candle, f64> for Vroc {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period + 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period);

        // Reset state
        self.reset();

        // Cannot calculate until we have period + 1 values
        for i in self.period..n {
            let current_volume = data[i].volume;
            let past_volume = data[i - self.period].volume;

            if past_volume == 0.0 {
                return Err(IndicatorError::CalculationError(
                    "Division by zero: past volume is zero".to_string(),
                ));
            }

            let vroc = (current_volume - past_volume) / past_volume * 100.0;
            result.push(vroc);
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        self.volume_buffer.push_back(value.volume);

        if self.volume_buffer.len() > self.period + 1 {
            self.volume_buffer.pop_front();
        }

        if self.volume_buffer.len() == self.period + 1 {
            let current_volume = self.volume_buffer.back().unwrap();
            let past_volume = self.volume_buffer.front().unwrap();

            if *past_volume == 0.0 {
                return Err(IndicatorError::CalculationError(
                    "Division by zero: past volume is zero".to_string(),
                ));
            }

            let vroc = (current_volume - past_volume) / past_volume * 100.0;
            Ok(Some(vroc))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.volume_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // Vroc Tests
    #[test]
    fn test_vroc_new() {
        // Valid period should work
        assert!(Vroc::new(14).is_ok());

        // Invalid period should fail
        assert!(Vroc::new(0).is_err());
    }

    #[test]
    fn test_vroc_calculation() {
        let mut vroc = Vroc::new(2).unwrap();

        // Create candles with known volumes
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 900.0,
            },
        ];

        let result = vroc.calculate(&candles).unwrap();

        // We need at least period+1 candles, and we get n-period results
        assert_eq!(result.len(), 2);

        // First VROC: (1500 - 1000) / 1000 * 100 = 50%
        assert!((result[0] - 50.0).abs() < 0.01);

        // Second VROC: (900 - 1200) / 1200 * 100 = -25%
        assert!((result[1] - (-25.0)).abs() < 0.01);
    }

    #[test]
    fn test_vroc_next() {
        let mut vroc = Vroc::new(2).unwrap();

        // Initial values - not enough data yet
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1000.0,
        };
        assert_eq!(vroc.next(candle1).unwrap(), None);

        let candle2 = Candle {
            timestamp: 2,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1200.0,
        };
        assert_eq!(vroc.next(candle2).unwrap(), None);

        // Third value - now we have enough data
        let candle3 = Candle {
            timestamp: 3,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1500.0,
        };
        let result = vroc.next(candle3).unwrap();

        // VROC: (1500 - 1000) / 1000 * 100 = 50%
        assert!((result.unwrap() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_vroc_reset() {
        let mut vroc = Vroc::new(2).unwrap();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1000.0,
        };
        vroc.next(candle1).unwrap();
        let candle2 = Candle {
            timestamp: 2,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1200.0,
        };
        vroc.next(candle2).unwrap();

        // Reset
        vroc.reset();

        // Volume buffer should be cleared
        let candle3 = Candle {
            timestamp: 3,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1500.0,
        };
        assert_eq!(vroc.next(candle3).unwrap(), None);
    }

    #[test]
    fn test_vroc_past_volume_zero() {
        let mut vroc = Vroc::new(2).unwrap();

        // Create candles with zero volume at the reference point
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 0.0, // Zero volume at reference point
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
        ];

        // Should return an error for division by zero
        let result = vroc.calculate(&candles);
        assert!(result.is_err());

        // Verify it's the correct error type and message
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(msg.contains("division by zero") || msg.contains("zero"));
        } else {
            panic!("Expected CalculationError");
        }

        // Test with streaming calculation too
        vroc.reset();
        assert_eq!(vroc.next(candles[0]).unwrap(), None); // Not enough data yet
        assert_eq!(vroc.next(candles[1]).unwrap(), None); // Not enough data yet

        // This should error due to division by zero
        let next_result = vroc.next(candles[2]);
        assert!(next_result.is_err());

        // Verify it's the correct error type
        if let Err(IndicatorError::CalculationError(msg)) = next_result {
            assert!(msg.contains("division by zero") || msg.contains("zero"));
        } else {
            panic!("Expected CalculationError");
        }
    }

    #[test]
    fn test_vroc_minimum_period() {
        // Test with period = 1 (minimum valid period)
        let mut vroc = Vroc::new(1).unwrap();

        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 2000.0,
            },
        ];

        let result = vroc.calculate(&candles).unwrap();

        // With period = 1, we should get n-1 results
        assert_eq!(result.len(), 2);

        // First VROC: (1500 - 1000) / 1000 * 100 = 50%
        assert!((result[0] - 50.0).abs() < 0.001);

        // Second VROC: (2000 - 1500) / 1500 * 100 = 33.33%
        assert!((result[1] - 33.33).abs() < 0.01);
    }

    #[test]
    fn test_vroc_large_period() {
        // Test with period close to data length
        let data_length = 10;
        let period = data_length - 1; // Use period = 9 for data length of 10

        let mut vroc = Vroc::new(period).unwrap();

        // Create 10 candles with sequential volumes
        let mut candles = Vec::with_capacity(data_length);
        for i in 0..data_length {
            candles.push(Candle {
                timestamp: i as u64,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0 * (i + 1) as f64, // Volumes: 1000, 2000, 3000, ...
            });
        }

        let result = vroc.calculate(&candles).unwrap();

        // With period = 9 and data length = 10, we should get 1 result
        assert_eq!(result.len(), 1);

        // VROC: (10000 - 1000) / 1000 * 100 = 900%
        assert!((result[0] - 900.0).abs() < 0.001);
        // Test boundary cases with data length
        // For Vroc, we need at least period+1 data points
        // With 10 candles:
        // - period = 9 works (needs 10 data points, we have 10)
        // - period = 10 doesn't work (needs 11 data points, we only have 10)

        // Test with period = 9 (should work with 10 data points)
        let mut vroc_large = Vroc::new(9).unwrap();
        let result = vroc_large.calculate(&candles);
        assert!(result.is_ok()); // Should have enough data

        // Test with period = 10 (should fail with only 10 data points since we need period+1)
        let mut vroc_too_large = Vroc::new(10).unwrap();
        let result = vroc_too_large.calculate(&candles);
        assert!(result.is_err()); // Should not be enough data
        assert!(result.is_err()); // Should not be enough data
    }

    #[test]
    fn test_vroc_reset_streaming() {
        let mut vroc = Vroc::new(2).unwrap();

        // Create test candles
        let candles = [
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1800.0,
            },
        ];

        // Process first three candles
        vroc.next(candles[0]).unwrap();
        vroc.next(candles[1]).unwrap();
        let first_result = vroc.next(candles[2]).unwrap().unwrap();

        // Reset indicator
        vroc.reset();

        // Process the candles again in a different order
        vroc.next(candles[1]).unwrap();
        vroc.next(candles[2]).unwrap();
        let second_result = vroc.next(candles[3]).unwrap().unwrap();

        // Results should be different as we've processed different candles
        // First: (1500-1000)/1000*100 = 50%
        // Second: (1800-1200)/1200*100 = 50%
        assert_eq!(first_result, 50.0);
        assert_eq!(second_result, 50.0);

        // Now reset and verify we need to process 3 candles again
        vroc.reset();
        assert_eq!(vroc.next(candles[0]).unwrap(), None);
        assert_eq!(vroc.next(candles[1]).unwrap(), None);
        assert!(vroc.next(candles[2]).unwrap().is_some());
    }

    #[test]
    fn test_vroc_batch_vs_streaming() {
        let period = 3;
        let mut batch_vroc = Vroc::new(period).unwrap();
        let mut streaming_vroc = Vroc::new(period).unwrap();

        // Create test data
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1800.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 2100.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_vroc.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::new();
        for candle in &candles {
            if let Some(value) = streaming_vroc.next(*candle).unwrap() {
                streaming_result.push(value);
            }
        }

        // Compare results - they should be identical
        assert_eq!(batch_result.len(), streaming_result.len());

        for i in 0..batch_result.len() {
            assert!(
                (batch_result[i] - streaming_result[i]).abs() < 0.001,
                "Batch and streaming results differ at index {}: batch={}, streaming={}",
                i,
                batch_result[i],
                streaming_result[i]
            );
        }
    }

    #[test]
    fn test_vroc_extreme_volume_values() {
        let mut vroc = Vroc::new(2).unwrap();

        // Create candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 100.0, // Small volume
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1_000_000_000.0, // Very large volume
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 5_000_000_000.0, // Extremely large volume
            },
        ];

        let result = vroc.calculate(&candles).unwrap();

        // We need at least period+1 candles, and we get n-period results
        assert_eq!(result.len(), 1);

        // VROC: (5_000_000_000.0 - 100.0) / 100.0 * 100 = 4,999,999,900%
        assert!(
            result[0] > 4_000_000_000.0,
            "Extreme VROC value not calculated correctly"
        );

        // Test with streaming calculation
        vroc.reset();
        assert_eq!(vroc.next(candles[0]).unwrap(), None); // Not enough data yet
        assert_eq!(vroc.next(candles[1]).unwrap(), None); // Not enough data yet

        let streaming_result = vroc.next(candles[2]).unwrap().unwrap();
        assert!((streaming_result - result[0]).abs() < 0.001);
    }
}
