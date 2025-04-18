use std::collections::VecDeque;

use crate::indicators::{validate_data_length, validate_period};
use crate::Candle;
use crate::Indicator;
use crate::IndicatorError;

/// Chaikin Money Flow indicator
///
/// Chaikin Money Flow measures the amount of Money Flow Volume over a specific period.
/// It provides insight into the buying and selling pressure during a given time period.
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::Cmf;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create a 20-period Chaikin Money Flow
/// let mut cmf = Cmf::new(20).unwrap();
///
/// // Price data with OHLCV values (need at least 20 candles for the period)
/// let candles = vec![
///     // Initial candles for accumulation phase (price rising, closing near highs)
///     Candle { timestamp: 1, open: 42.0, high: 43.0, low: 41.0, close: 42.8, volume: 1000.0 },
///     Candle { timestamp: 2, open: 42.8, high: 44.0, low: 42.5, close: 43.7, volume: 1200.0 },
///     Candle { timestamp: 3, open: 43.7, high: 44.5, low: 43.2, close: 44.3, volume: 1400.0 },
///     Candle { timestamp: 4, open: 44.3, high: 45.0, low: 44.0, close: 44.8, volume: 1600.0 },
///     Candle { timestamp: 5, open: 44.8, high: 45.5, low: 44.3, close: 45.2, volume: 1800.0 },
///     // Next candles for moderate accumulation (price still rising)
///     Candle { timestamp: 6, open: 45.2, high: 46.0, low: 45.0, close: 45.7, volume: 1700.0 },
///     Candle { timestamp: 7, open: 45.7, high: 46.5, low: 45.5, close: 46.3, volume: 1600.0 },
///     Candle { timestamp: 8, open: 46.3, high: 47.0, low: 46.0, close: 46.8, volume: 1500.0 },
///     Candle { timestamp: 9, open: 46.8, high: 47.5, low: 46.5, close: 47.2, volume: 1400.0 },
///     Candle { timestamp: 10, open: 47.2, high: 48.0, low: 47.0, close: 47.6, volume: 1300.0 },
///     // Transition to distribution phase (price peaking, closing away from highs)
///     Candle { timestamp: 11, open: 47.6, high: 48.5, low: 47.3, close: 47.9, volume: 1500.0 },
///     Candle { timestamp: 12, open: 47.9, high: 49.0, low: 47.5, close: 48.2, volume: 1700.0 },
///     Candle { timestamp: 13, open: 48.2, high: 49.5, low: 48.0, close: 48.6, volume: 1900.0 },
///     Candle { timestamp: 14, open: 48.6, high: 50.0, low: 48.4, close: 49.2, volume: 2100.0 },
///     Candle { timestamp: 15, open: 49.2, high: 50.5, low: 48.8, close: 49.5, volume: 2300.0 },
///     // Distribution phase begins (price falling, closing near lows)
///     Candle { timestamp: 16, open: 49.5, high: 50.0, low: 48.5, close: 48.7, volume: 2500.0 },
///     Candle { timestamp: 17, open: 48.7, high: 49.2, low: 47.8, close: 48.0, volume: 2700.0 },
///     Candle { timestamp: 18, open: 48.0, high: 48.5, low: 47.0, close: 47.2, volume: 2900.0 },
///     Candle { timestamp: 19, open: 47.2, high: 47.7, low: 46.5, close: 46.7, volume: 3100.0 },
///     Candle { timestamp: 20, open: 46.7, high: 47.0, low: 45.8, close: 46.0, volume: 3300.0 },
///     // Additional candles to see trend change
///    Candle { timestamp: 21, open: 46.0, high: 46.5, low: 45.0, close: 45.2, volume: 3500.0 },
///     Candle { timestamp: 22, open: 45.2, high: 46.0, low: 44.5, close: 44.8, volume: 3700.0 }];
/// // Calculate CMF values with error handling
/// match cmf.calculate(&candles) {
///     Ok(cmf_values) => {
///         // Access the latest CMF value
///         if let Some(latest_cmf) = cmf_values.last() {
///             println!("CMF value: {:.2}", latest_cmf);     
///             // Interpret the value
///             if *latest_cmf > 0.0 {
///                 println!("Accumulation phase - money flow into the security");
///             } else {
///                 println!("Distribution phase - money flow out of the security");
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating CMF: {}", e);
///     }
/// }
///```

#[derive(Debug)]
pub struct Cmf {
    period: usize,
    mfv_buffer: VecDeque<f64>,
    volume_buffer: VecDeque<f64>,
}

impl Cmf {
    /// Create a new Cmf indicator
    ///
    /// # Arguments
    /// * `period` - The period for CMF calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new Cmf or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            mfv_buffer: VecDeque::with_capacity(period),
            volume_buffer: VecDeque::with_capacity(period),
        })
    }

    /// Calculate Money Flow Multiplier (MFM) for a candle
    ///
    /// # Arguments
    /// * `candle` - The candle data to calculate MFM from
    ///
    /// # Returns
    /// * `f64` - The Money Flow Multiplier value
    fn money_flow_multiplier(candle: &Candle) -> Result<f64, IndicatorError> {
        let high = candle.high;
        let low = candle.low;
        let close = candle.close;

        let range = high - low;

        if range == 0.0 {
            return Err(IndicatorError::CalculationError(
                "Division by zero: high and low prices are equal".to_string(),
            ));
        }

        // Calculate Money Flow Multiplier
        // MFM = ((Close - Low) - (High - Close)) / (High - Low)
        // Simplified to: MFM = (2 * Close - High - Low) / (High - Low)
        Ok((2.0 * close - high - low) / range)
    }

    /// Calculate Money Flow Volume (MFV) for a candle
    ///
    /// # Arguments
    /// * `candle` - The candle data to calculate MFV from
    ///
    /// # Returns
    /// * `f64` - The Money Flow Volume value
    fn money_flow_volume(candle: &Candle) -> Result<f64, IndicatorError> {
        let mfm = Self::money_flow_multiplier(candle)?;
        let volume = candle.volume;

        // Money Flow Volume = Money Flow Multiplier * Volume
        Ok(mfm * volume)
    }
}

impl Indicator<Candle, f64> for Cmf {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset();

        for candle in data.iter().take(n) {
            let mfv = Self::money_flow_volume(candle)?;
            self.mfv_buffer.push_back(mfv);
            self.volume_buffer.push_back(candle.volume);

            if self.mfv_buffer.len() > self.period {
                self.mfv_buffer.pop_front();
                self.volume_buffer.pop_front();
            }

            if self.mfv_buffer.len() == self.period {
                let sum_mfv: f64 = self.mfv_buffer.iter().sum();
                let sum_volume: f64 = self.volume_buffer.iter().sum();

                if sum_volume == 0.0 {
                    return Err(IndicatorError::CalculationError(
                        "Division by zero: sum of volumes is zero".to_string(),
                    ));
                }

                let cmf = sum_mfv / sum_volume;
                result.push(cmf);
            }
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        let mfv = Self::money_flow_volume(&value)?;

        self.mfv_buffer.push_back(mfv);
        self.volume_buffer.push_back(value.volume);

        if self.mfv_buffer.len() > self.period {
            self.mfv_buffer.pop_front();
            self.volume_buffer.pop_front();
        }

        if self.mfv_buffer.len() == self.period {
            let sum_mfv: f64 = self.mfv_buffer.iter().sum();
            let sum_volume: f64 = self.volume_buffer.iter().sum();

            if sum_volume == 0.0 {
                return Err(IndicatorError::CalculationError(
                    "Division by zero: sum of volumes is zero".to_string(),
                ));
            }

            let cmf = sum_mfv / sum_volume;
            Ok(Some(cmf))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.mfv_buffer.clear();
        self.volume_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    #[test]
    fn test_cmf_new() {
        // Valid period should work
        assert!(Cmf::new(14).is_ok());

        // Invalid period should fail
        assert!(Cmf::new(0).is_err());
    }

    #[test]
    fn test_cmf_calculation() {
        let mut cmf = Cmf::new(2).unwrap();

        // Create candles with specific patterns for testing
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 14.0,
                low: 10.0,
                close: 11.0,
                volume: 800.0,
            },
        ];

        let result = cmf.calculate(&candles).unwrap();

        // We need at least period (2) candles
        assert_eq!(result.len(), 2);

        // Verify the CMF values are between -1 and 1
        for cmf_value in &result {
            assert!(*cmf_value >= -1.0 && *cmf_value <= 1.0);
        }

        // For the first period (candles 1-2):
        // First candle: MFM = (2*11 - 12 - 8)/(12 - 8) = 0.5, MFV = 0.5 * 1000 = 500
        // Second candle: MFM = (2*12 - 13 - 9)/(13 - 9) = 0.5, MFV = 0.5 * 1200 = 600
        // Sum of MFV = 500 + 600 = 1100
        // Sum of Volume = 1000 + 1200 = 2200
        // CMF = 1100 / 2200 = 0.5
        assert!((result[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cmf_zero_volume_sum() {
        let mut cmf = Cmf::new(2).unwrap();

        // Create candles with zero volume
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 0.0, // Zero volume
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 0.0, // Zero volume
            },
        ];

        // Should error with division by zero
        let result = cmf.calculate(&candles);
        assert!(result.is_err());

        // Verify it's the correct error type and message
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(msg.contains("division by zero") || msg.contains("sum of volumes is zero"));
        } else {
            panic!("Expected CalculationError for zero volume sum");
        }

        // Test streaming calculation
        cmf.reset();
        assert_eq!(cmf.next(candles[0]).unwrap(), None); // Not enough data yet
        let result = cmf.next(candles[1]);

        assert!(result.is_err());
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(msg.contains("division by zero") || msg.contains("sum of volumes is zero"));
        } else {
            panic!("Expected CalculationError for zero volume sum in streaming mode");
        }
    }

    #[test]
    fn test_cmf_boundary_conditions() {
        let mut cmf = Cmf::new(3).unwrap();

        // Create candles that should produce CMF values close to boundaries
        // For CMF near +1: High MFM (close near high) with consistent volume
        let max_candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.9, // Close near high
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.9,
                high: 14.0,
                low: 11.0,
                close: 13.9, // Close near high
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 13.9,
                high: 16.0,
                low: 13.0,
                close: 15.9, // Close near high
                volume: 1000.0,
            },
        ];

        // For CMF near -1: Low MFM (close near low) with consistent volume
        let min_candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 8.1, // Close near low
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 8.1,
                high: 10.0,
                low: 7.0,
                close: 7.1, // Close near low
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 7.1,
                high: 9.0,
                low: 6.0,
                close: 6.1, // Close near low
                volume: 1000.0,
            },
        ];

        // Test near maximum value
        let max_result = cmf.calculate(&max_candles).unwrap();
        assert_eq!(max_result.len(), 1);
        assert!(
            max_result[0] > 0.9,
            "CMF value should be close to +1, got {}",
            max_result[0]
        );
        assert!(
            max_result[0] <= 1.0,
            "CMF value should not exceed +1, got {}",
            max_result[0]
        );

        // Test near minimum value
        cmf.reset();
        let min_result = cmf.calculate(&min_candles).unwrap();
        assert_eq!(min_result.len(), 1);
        assert!(
            min_result[0] < -0.9,
            "CMF value should be close to -1, got {}",
            min_result[0]
        );
        assert!(
            min_result[0] >= -1.0,
            "CMF value should not be less than -1, got {}",
            min_result[0]
        );
    }

    #[test]
    fn test_cmf_minimum_period() {
        // Test with period = 1 (minimum valid period)
        let mut cmf = Cmf::new(1).unwrap();

        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 14.0,
                low: 10.0,
                close: 11.0,
                volume: 800.0,
            },
        ];

        let result = cmf.calculate(&candles).unwrap();

        // With period = 1, we should get result for each candle
        assert_eq!(result.len(), 3);

        // First candle: MFM = (2*11 - 12 - 8)/(12 - 8) = 0.5, CMF = 0.5
        assert!((result[0] - 0.5).abs() < 0.001);

        // Second candle: MFM = (2*12 - 13 - 9)/(13 - 9) = 0.5, CMF = 0.5
        assert!((result[1] - 0.5).abs() < 0.001);

        // Third candle: MFM = (2*11 - 14 - 10)/(14 - 10) = -0.5, CMF = -0.5
        assert!((result[2] - (-0.5)).abs() < 0.001);

        // Test streaming calculation with minimum period
        cmf.reset();
        assert_eq!(cmf.next(candles[0]).unwrap().unwrap(), 0.5);
        assert_eq!(cmf.next(candles[1]).unwrap().unwrap(), 0.5);
        assert!((cmf.next(candles[2]).unwrap().unwrap() - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_cmf_reset_partial_data() {
        let mut cmf = Cmf::new(3).unwrap();

        // Create test candles
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 14.0,
                low: 10.0,
                close: 11.0,
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 5,
                open: 12.0,
                high: 15.0,
                low: 11.0,
                close: 14.0,
                volume: 2000.0,
            },
        ];

        // Process first two candles
        cmf.next(candles[0]).unwrap();
        cmf.next(candles[1]).unwrap();

        // Reset and process different candles
        cmf.reset();

        cmf.next(candles[2]).unwrap();
        cmf.next(candles[3]).unwrap();

        // We need one more candle to get a result with period = 3
        let result = cmf.next(candles[4]).unwrap();
        assert!(result.is_some());

        // Verify that CMF calculation after reset uses only the new data
        // This should be based on candles 2, 3, and 4, not include candles 0 and 1

        // Calculate expected result from batch calculation for verification
        cmf.reset();
        let expected = cmf.calculate(&candles[2..5]).unwrap()[0];

        assert!((result.unwrap() - expected).abs() < 0.001);
    }

    #[test]
    fn test_cmf_batch_vs_streaming() {
        let period = 3;
        let mut batch_cmf = Cmf::new(period).unwrap();
        let mut streaming_cmf = Cmf::new(period).unwrap();

        // Create test data
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 14.0,
                low: 10.0,
                close: 11.0,
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 5,
                open: 12.0,
                high: 15.0,
                low: 11.0,
                close: 14.0,
                volume: 2000.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_cmf.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::new();
        for candle in &candles {
            if let Some(value) = streaming_cmf.next(*candle).unwrap() {
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
    fn test_cmf_extreme_volume_values() {
        let mut cmf = Cmf::new(2).unwrap();

        // Create candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1_000_000_000.0, // Very large volume
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 2_000_000_000.0, // Another large volume
            },
        ];

        let result = cmf.calculate(&candles).unwrap();

        // We should get one result
        assert_eq!(result.len(), 1);

        // The value should still be constrained between -1 and 1
        assert!(
            result[0] >= -1.0 && result[0] <= 1.0,
            "CMF with extreme volumes should still be between -1 and 1, got: {}",
            result[0]
        );
    }
} // Close the test modu
