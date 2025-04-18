use crate::indicators::utils::validate_data_length;
use crate::indicators::{Candle, Indicator, IndicatorError};

/// On Balance Volume (OBV) indicator
///
/// OBV is a momentum indicator that uses volume flow to predict changes in stock price.
/// It accumulates volume on up days and subtracts volume on down days.
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::Obv;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create an OBV indicator
/// let mut obv = Obv::new();
///
/// // Create price data with close and volume values
/// let candles = vec![
///     Candle { timestamp: 0, open: 10.0, high: 12.0, low: 9.0, close: 11.0, volume: 1000.0 },
///     Candle { timestamp: 1, open: 11.0, high: 13.0, low: 10.0, close: 12.0, volume: 1500.0 },
///     Candle { timestamp: 2, open: 12.0, high: 15.0, low: 11.0, close: 11.5, volume: 2000.0 },
///     // ... more candles ...
/// ];
///
/// // Calculate OBV values
/// let obv_values = obv.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct Obv {
    prev_close: Option<f64>,
    current_obv: f64,
}

impl Obv {
    /// Create a new Obv indicator
    pub fn new() -> Self {
        Self {
            prev_close: None,
            current_obv: 0.0,
        }
    }
}

impl Default for Obv {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator<Candle, f64> for Obv {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n);

        // Reset state
        self.reset();

        // Set first OBV value
        self.current_obv = 0.0;
        result.push(self.current_obv);
        self.prev_close = Some(data[0].close);

        // Calculate OBV for each subsequent candle
        for candle in data.iter().take(n).skip(1) {
            let close = candle.close;
            let prev_close = self.prev_close.unwrap();
            let volume = candle.volume;

            if close > prev_close {
                // Up day
                self.current_obv += volume;
            } else if close < prev_close {
                // Down day
                self.current_obv -= volume;
            }
            // Equal days do not change OBV

            result.push(self.current_obv);
            self.prev_close = Some(close);
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        if let Some(prev_close) = self.prev_close {
            let close = value.close;
            let volume = value.volume;

            if close > prev_close {
                // Up day
                self.current_obv += volume;
            } else if close < prev_close {
                // Down day
                self.current_obv -= volume;
            }
            // Equal days do not change OBV

            self.prev_close = Some(close);
            Ok(Some(self.current_obv))
        } else {
            // First value just establishes the baseline
            self.prev_close = Some(value.close);
            self.current_obv = 0.0;
            Ok(Some(self.current_obv))
        }
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.current_obv = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // Obv Tests
    #[test]
    fn test_obv_new() {
        // Obv has no parameters to validate
        let obv = Obv::new();
        assert!(obv.current_obv == 0.0);
    }

    #[test]
    fn test_obv_calculation() {
        let mut obv = Obv::new();

        // Create test candles with predictable pattern
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 1200.0,
            }, // Price up
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 10.0,
                close: 10.2,
                volume: 800.0,
            }, // Price down
            Candle {
                timestamp: 4,
                open: 10.2,
                high: 11.0,
                low: 10.0,
                close: 10.8,
                volume: 900.0,
            }, // Price up
            Candle {
                timestamp: 5,
                open: 10.8,
                high: 11.0,
                low: 10.0,
                close: 10.8,
                volume: 700.0,
            }, // Price unchanged
        ];

        let result = obv.calculate(&candles).unwrap();

        // We get one OBV value for each candle
        assert_eq!(result.len(), 5);

        // First value is set to 0 by the OBV implementation
        assert_eq!(result[0], 0.0);

        // Second value: previous OBV + second volume (price up)
        assert_eq!(result[1], 1200.0);

        // Third value: previous OBV - volume (price down)
        assert_eq!(result[2], 400.0);

        // Fourth value: previous OBV + volume (price up)
        assert_eq!(result[3], 1300.0);

        // Fifth value: unchanged OBV (price unchanged)
        assert_eq!(result[4], 1300.0);
    }

    #[test]
    fn test_obv_next() {
        let mut obv = Obv::new();

        // First candle - sets initial OBV
        // First candle - sets initial OBV to 0
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 1000.0,
        };
        assert_eq!(obv.next(candle1).unwrap(), Some(0.0));

        // Next candle - price up, add volume
        let candle2 = Candle {
            timestamp: 2,
            open: 10.5,
            high: 12.0,
            low: 10.0,
            close: 11.0,
            volume: 1200.0,
        };
        assert_eq!(obv.next(candle2).unwrap(), Some(1200.0));

        // Next candle - price down, subtract volume
        let candle3 = Candle {
            timestamp: 3,
            open: 11.0,
            high: 11.5,
            low: 10.0,
            close: 10.2,
            volume: 800.0,
        };
        assert_eq!(obv.next(candle3).unwrap(), Some(400.0));
    }

    #[test]
    fn test_obv_reset() {
        let mut obv = Obv::new();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 1000.0,
        };
        obv.next(candle1).unwrap();

        // Reset
        obv.reset();

        // OBV should be reset to 0
        assert_eq!(obv.current_obv, 0.0);
        assert_eq!(obv.prev_close, None);

        // After reset, next candle should be treated as first
        // After reset, next candle should be treated as first (OBV starts at 0)
        let candle2 = Candle {
            timestamp: 2,
            open: 10.5,
            high: 12.0,
            low: 10.0,
            close: 11.0,
            volume: 1200.0,
        };
        assert_eq!(obv.next(candle2).unwrap(), Some(0.0));
    }

    #[test]
    fn test_obv_zero_volume() {
        let mut obv = Obv::new();

        // Create test candles with zero volume
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial candle with volume
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 0.0,
            }, // Price up but zero volume
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 10.0,
                close: 10.0,
                volume: 0.0,
            }, // Price down but zero volume
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // Second value should be unchanged from first because volume is 0
        assert_eq!(result[1], 0.0);

        // Third value should be unchanged from second because volume is 0
        assert_eq!(result[2], 0.0);

        // Test with streaming calculation too
        obv.reset();
        assert_eq!(obv.next(candles[0]).unwrap(), Some(0.0));
        assert_eq!(obv.next(candles[1]).unwrap(), Some(0.0)); // Zero volume should not change OBV
        assert_eq!(obv.next(candles[2]).unwrap(), Some(0.0)); // Zero volume should not change OBV
    }

    #[test]
    fn test_obv_extreme_volume_values() {
        let mut obv = Obv::new();

        // Create test candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 1_000_000_000.0, // Extremely large volume
            }, // Price up
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 10.0,
                close: 10.0,
                volume: 500_000_000.0, // Another large volume
            }, // Price down
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // Second value: add the large volume (price up)
        assert_eq!(result[1], 1_000_000_000.0);

        // Third value: subtract the large volume (price down)
        assert_eq!(result[2], 1_000_000_000.0 - 500_000_000.0);
    }

    #[test]
    fn test_obv_identical_closing_prices() {
        let mut obv = Obv::new();

        // Create test candles with identical closing prices
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 10.5, // Same close as previous
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.5,
                high: 11.5,
                low: 10.0,
                close: 10.5, // Same close again
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 10.5,
                high: 11.0,
                low: 10.0,
                close: 10.5, // Same close again
                volume: 900.0,
            },
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // All subsequent values should remain at 0 since prices are unchanged
        // and OBV should not change when close prices are identical
        assert_eq!(result[1], 0.0);
        assert_eq!(result[2], 0.0);
        assert_eq!(result[3], 0.0);
    }

    #[test]
    fn test_obv_consecutive_up_down_sequences() {
        let mut obv = Obv::new();

        // Create test candles with alternating up/down patterns
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 12.0,
                low: 10.0,
                close: 11.0, // Up
                volume: 500.0,
            },
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 9.5,
                close: 10.0, // Down
                volume: 300.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 10.5,
                low: 9.0,
                close: 10.5, // Up
                volume: 700.0,
            },
            Candle {
                timestamp: 5,
                open: 10.5,
                high: 11.0,
                low: 9.5,
                close: 9.5, // Down
                volume: 400.0,
            },
            Candle {
                timestamp: 6,
                open: 9.5,
                high: 10.5,
                low: 9.0,
                close: 10.0, // Up
                volume: 600.0,
            },
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // Check the pattern matches our expectation:
        // Initial: 0
        // Up: +500 => 500
        // Down: -300 => 200
        // Up: +700 => 900
        // Down: -400 => 500
        // Up: +600 => 1100
        assert_eq!(result[1], 500.0); // +500
        assert_eq!(result[2], 200.0); // +500-300
        assert_eq!(result[3], 900.0); // +500-300+700
        assert_eq!(result[4], 500.0); // +500-300+700-400
        assert_eq!(result[5], 1100.0); // +500-300+700-400+600

        // Test streaming calculation matches batch calculation
        let mut streaming_obv = Obv::new();
        for (i, candle) in candles.iter().enumerate() {
            let obv_value = streaming_obv.next(*candle).unwrap().unwrap();
            assert_eq!(
                obv_value, result[i],
                "Streaming calculation mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_obv_insufficient_data() {
        let mut obv = Obv::new();

        // Test with empty data
        let empty: Vec<Candle> = vec![];
        let result = obv.calculate(&empty);

        // Should error due to insufficient data (require at least 1 data point)
        assert!(result.is_err());
        if let Err(IndicatorError::InsufficientData(_)) = result {
            // Expected error
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_obv_batch_vs_streaming_consistency() {
        let mut batch_obv = Obv::new();
        let mut streaming_obv = Obv::new();

        // Create test data with different patterns
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
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 9.5,
                close: 10.0,
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 10.5,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 11.0,
                low: 9.5,
                close: 10.5,
                volume: 2000.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_obv.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::with_capacity(candles.len());
        for candle in &candles {
            let value = streaming_obv.next(*candle).unwrap().unwrap();
            streaming_result.push(value);
        }

        // Compare results - they should be identical
        assert_eq!(batch_result.len(), streaming_result.len());

        for i in 0..batch_result.len() {
            assert_eq!(
                batch_result[i], streaming_result[i],
                "Batch and streaming results differ at index {}",
                i
            );
        }
    }
}
