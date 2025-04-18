use crate::indicators::traits::Indicator;
use crate::indicators::utils::{standard_deviation, validate_data_length, validate_period};
use crate::indicators::{Candle, IndicatorError};
use std::collections::VecDeque;

/// Standard Deviation (Std) indicator
///
/// Measures the dispersion of a dataset relative to its mean over a specific period.
/// Standard deviation is commonly used to measure market volatility. Higher values indicate
/// greater price volatility, while lower values suggest more stable prices.
///
/// The STD indicator can be particularly useful for:
/// - Identifying periods of high vs low volatility
/// - Setting dynamic stop-loss levels
/// - Determining position sizing based on market volatility
///
/// # Formula
///
/// The standard deviation is calculated as:
/// ```text
/// STD = √(Σ(x - μ)² / n)
///
/// where:
/// x = each value in the dataset
/// μ = mean of the dataset
/// n = number of values
/// ```
///
/// # Example with float values
///
/// ```
/// use rsta::indicators::volatility::Std;
/// use rsta::indicators::Indicator;
///
/// // Create a 20-period Standard Deviation indicator
/// let mut std_dev = Std::new(20).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///                   30.0, 31.0, 32.0, 33.0, 34.0];
///
/// // Calculate Standard Deviation values
/// let std_values = std_dev.calculate(&prices).unwrap();
/// ```
///
/// # Example with Candle data
///
/// ```
/// use rsta::indicators::volatility::Std;
/// use rsta::indicators::{Indicator, Candle};
///
/// // Create a 20-period Standard Deviation indicator
/// let mut std_dev = Std::new(20).unwrap();
///
/// // Create candle data
/// let mut candles = Vec::new();
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///                   30.0, 31.0, 32.0, 33.0, 34.0];
///
/// // Convert prices to candles
/// for (i, &price) in prices.iter().enumerate() {
///     candles.push(Candle {
///         timestamp: i as u64,
///         open: price - 0.5,
///         high: price + 0.5,
///         low: price - 0.5,
///         close: price,
///         volume: 1000.0,
///     });
/// }
///
/// // Calculate Standard Deviation values based on close prices
/// let std_values = std_dev.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct Std {
    period: usize,
    values: VecDeque<f64>,
}

impl Std {
    /// Create a new STD indicator
    ///
    /// # Arguments
    /// * `period` - The period for Standard Deviation calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new STD instance or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            values: VecDeque::with_capacity(period),
        })
    }

    /// Reset the Standard Deviation indicator state
    pub fn reset_state(&mut self) {
        self.values.clear();
    }
}

// Implementation for raw price values
impl Indicator<f64, f64> for Std {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset_state();

        // Calculate standard deviation for each period
        for i in 0..=(n - self.period) {
            let period_data = &data[i..(i + self.period)];
            let std_dev = standard_deviation(period_data, None)?;
            result.push(std_dev);
        }

        // Update state with the last period
        for &value in data.iter().take(n).skip(n - self.period) {
            self.values.push_back(value);
        }

        Ok(result)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.values.push_back(value);

        if self.values.len() > self.period {
            self.values.pop_front();
        }

        if self.values.len() == self.period {
            standard_deviation(self.values.make_contiguous(), None).map(Some)
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

// Implementation for candle data
impl Indicator<Candle, f64> for Std {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        // Extract close prices from candles
        let close_prices: Vec<f64> = data.iter().map(|candle| candle.close).collect();

        // Use the existing implementation for f64 data
        self.calculate(&close_prices)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        // Use the close price for the calculation
        let close_price = candle.close;
        self.next(close_price)
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_EPSILON: f64 = 1e-10;

    // Helper function to compare floating point values
    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < FLOAT_EPSILON, "{} != {}", a, b);
    }

    #[test]
    fn test_std_calculation_basic() {
        let mut std = Std::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0];

        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 1);

        // Mean = (2 + 4 + 6) / 3 = 4
        // Variance = ((2-4)² + (4-4)² + (6-4)²) / 3 = (4 + 0 + 4) / 3 = 8/3
        // STD = √(8/3) ≈ 1.632993161855452
        assert_float_eq(result[0], 1.632993161855452);
    }

    #[test]
    fn test_std_calculation_multiple_periods() {
        let mut std = Std::new(2).unwrap();
        let data = vec![1.0, 2.0, 3.0];

        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 2);

        // First window [1.0, 2.0]: Mean = 1.5, Variance = ((1-1.5)² + (2-1.5)²) / 2 = 0.25
        // STD = √0.25 = 0.5
        assert_float_eq(result[0], 0.5);

        // Second window [2.0, 3.0]: Mean = 2.5, Variance = ((2-2.5)² + (3-2.5)²) / 2 = 0.25
        // STD = √0.25 = 0.5
        assert_float_eq(result[1], 0.5);
    }

    #[test]
    fn test_std_with_decimal_values() {
        let mut std = Std::new(4).unwrap();
        let data = vec![1.5, 2.5, 3.5, 4.5];

        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 1);
        // Mean = 3.0
        // STD = √(((1.5-3)² + (2.5-3)² + (3.5-3)² + (4.5-3)²)/4) = √(5/4) ≈ 1.118033988749895
        assert_float_eq(result[0], 1.118033988749895);
    }

    #[test]
    fn test_std_edge_cases() {
        // Test period of 1
        let mut std = Std::new(1).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 5);
        // For period=1, all standard deviations should be 0
        for value in result {
            assert_float_eq(value, 0.0);
        }

        // Test with constant values
        let mut std = Std::new(3).unwrap();
        let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];

        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);
        // STD should be 0 for constant values
        for value in result {
            assert_float_eq(value, 0.0);
        }
    }

    #[test]
    fn test_std_next_value() {
        let mut std = Std::new(3).unwrap();

        // First two values should return None
        assert_eq!(std.next(2.0).unwrap(), None);
        assert_eq!(std.next(4.0).unwrap(), None);

        // Third value should give us our first STD
        let result = std.next(6.0).unwrap().unwrap();
        // Mean = 4.0
        // STD ≈ 1.632993161855452
        assert_float_eq(result, 1.632993161855452);

        // Next value should maintain window of 3
        let result = std.next(8.0).unwrap().unwrap();
        // Window now contains [4.0, 6.0, 8.0]
        assert_float_eq(result, 1.632993161855452);
    }

    #[test]
    fn test_std_with_market_pattern() {
        let mut std = Std::new(5).unwrap();
        // Simulated market pattern: trending up with increasing volatility
        let data = vec![
            100.0, 101.0, 101.5, 102.0, 103.0, // low volatility trend
            105.0, 104.0, 106.0, 103.0, 107.0, // increasing volatility
        ];

        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 6);

        // The standard deviation should increase as volatility increases
        assert!(result[0] < result[result.len() - 1]);
    }

    #[test]
    fn test_std_error_handling() {
        let mut std = Std::new(5).unwrap();

        // Test with insufficient data
        let data = vec![1.0, 2.0, 3.0];
        assert!(matches!(
            std.calculate(&data),
            Err(IndicatorError::InsufficientData(_))
        ));

        // Test with empty data
        let data: Vec<f64> = vec![];
        assert!(matches!(
            std.calculate(&data),
            Err(IndicatorError::InsufficientData(_))
        ));

        // Test valid period initialization
        assert!(Std::new(100).is_ok());
    }

    #[test]
    fn test_std_reset() {
        let mut std = Std::new(3).unwrap();

        // Add some values
        std.next(1.0).unwrap();
        std.next(2.0).unwrap();
        std.next(3.0).unwrap();

        // Reset the indicator
        std.reset_state();

        // Next value after reset should return None
        assert_eq!(std.next(4.0).unwrap(), None);
    }

    // Tests for candle data
    #[test]
    fn test_std_calculation_with_candles() {
        let mut std = Std::new(3).unwrap();

        // Create candles with specific close prices
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 1.5,
                high: 2.5,
                low: 1.5,
                close: 2.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 3.5,
                high: 4.5,
                low: 3.5,
                close: 4.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 5.5,
                high: 6.5,
                low: 5.5,
                close: 6.0,
                volume: 1000.0,
            },
        ];

        let result = std.calculate(&candles).unwrap();
        assert_eq!(result.len(), 1);

        // Mean = (2 + 4 + 6) / 3 = 4
        // Variance = ((2-4)² + (4-4)² + (6-4)²) / 3 = (4 + 0 + 4) / 3 = 8/3
        // STD = √(8/3) ≈ 1.632993161855452
        assert_float_eq(result[0], 1.632993161855452);

        // Compare with raw price calculation
        let prices = vec![2.0, 4.0, 6.0];
        let mut std_prices = Std::new(3).unwrap();
        let price_result = std_prices.calculate(&prices).unwrap();

        assert_eq!(result.len(), price_result.len());
        for (res_candle, res_price) in result.iter().zip(price_result.iter()) {
            assert_float_eq(*res_candle, *res_price);
        }
    }

    #[test]
    fn test_std_next_with_candles() {
        let mut std = Std::new(3).unwrap();

        // First two values should return None
        let candle1 = Candle {
            timestamp: 1,
            open: 1.5,
            high: 2.5,
            low: 1.5,
            close: 2.0,
            volume: 1000.0,
        };
        let candle2 = Candle {
            timestamp: 2,
            open: 3.5,
            high: 4.5,
            low: 3.5,
            close: 4.0,
            volume: 1000.0,
        };

        assert_eq!(std.next(candle1).unwrap(), None);
        assert_eq!(std.next(candle2).unwrap(), None);

        // Third value should give us our first STD
        let candle3 = Candle {
            timestamp: 3,
            open: 5.5,
            high: 6.5,
            low: 5.5,
            close: 6.0,
            volume: 1000.0,
        };
        let result = std.next(candle3).unwrap().unwrap();

        // Mean = 4.0
        // STD ≈ 1.632993161855452
        assert_float_eq(result, 1.632993161855452);

        // Next value should maintain window of 3
        let candle4 = Candle {
            timestamp: 4,
            open: 7.5,
            high: 8.5,
            low: 7.5,
            close: 8.0,
            volume: 1000.0,
        };
        let result = std.next(candle4).unwrap().unwrap();

        // Window now contains [4.0, 6.0, 8.0]
        assert_float_eq(result, 1.632993161855452);

        // Compare with raw price calculation
        let mut std_prices = Std::new(3).unwrap();
        std_prices.next(2.0).unwrap();
        std_prices.next(4.0).unwrap();
        std_prices.next(6.0).unwrap();
        let price_result = std_prices.next(8.0).unwrap().unwrap();

        assert_float_eq(result, price_result);
    }

    #[test]
    fn test_std_reset_with_candles() {
        let mut std = Std::new(3).unwrap();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 0.5,
            high: 1.5,
            low: 0.5,
            close: 1.0,
            volume: 1000.0,
        };
        let candle2 = Candle {
            timestamp: 2,
            open: 1.5,
            high: 2.5,
            low: 1.5,
            close: 2.0,
            volume: 1000.0,
        };
        let candle3 = Candle {
            timestamp: 3,
            open: 2.5,
            high: 3.5,
            low: 2.5,
            close: 3.0,
            volume: 1000.0,
        };

        std.next(candle1).unwrap();
        std.next(candle2).unwrap();
        std.next(candle3).unwrap();

        // Reset the indicator
        std.reset_state();

        // Next value after reset should return None
        let candle4 = Candle {
            timestamp: 4,
            open: 3.5,
            high: 4.5,
            low: 3.5,
            close: 4.0,
            volume: 1000.0,
        };
        assert_eq!(std.next(candle4).unwrap(), None);
    }

    #[test]
    fn test_std_with_market_pattern_candles() {
        let mut std = Std::new(5).unwrap();
        // Simulated market pattern: trending up with increasing volatility
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 99.0,
                high: 101.0,
                low: 99.0,
                close: 100.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 100.0,
                high: 102.0,
                low: 100.0,
                close: 101.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 100.5,
                high: 102.5,
                low: 100.5,
                close: 101.5,
                volume: 1000.0,
            },
            Candle {
                timestamp: 4,
                open: 101.0,
                high: 103.0,
                low: 101.0,
                close: 102.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 5,
                open: 102.0,
                high: 104.0,
                low: 102.0,
                close: 103.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 6,
                open: 104.0,
                high: 106.0,
                low: 104.0,
                close: 105.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 7,
                open: 103.0,
                high: 105.0,
                low: 103.0,
                close: 104.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 8,
                open: 105.0,
                high: 107.0,
                low: 105.0,
                close: 106.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 9,
                open: 102.0,
                high: 104.0,
                low: 102.0,
                close: 103.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 10,
                open: 106.0,
                high: 108.0,
                low: 106.0,
                close: 107.0,
                volume: 1000.0,
            },
        ];

        let result = std.calculate(&candles).unwrap();
        assert_eq!(result.len(), 6);

        // The standard deviation should increase as volatility increases
        assert!(result[0] < result[result.len() - 1]);

        // Compare with raw price calculation
        let prices = vec![
            100.0, 101.0, 101.5, 102.0, 103.0, 105.0, 104.0, 106.0, 103.0, 107.0,
        ];
        let mut std_prices = Std::new(5).unwrap();
        let price_result = std_prices.calculate(&prices).unwrap();

        assert_eq!(result.len(), price_result.len());
        for (res_candle, res_price) in result.iter().zip(price_result.iter()) {
            assert_float_eq(*res_candle, *res_price);
        }
    }
}
