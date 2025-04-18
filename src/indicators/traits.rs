//! Core traits for technical analysis indicators
//!
//! This module defines the core traits that form the foundation of the indicator system,
//! including the Indicator trait for implementing technical indicators and the
//! PriceDataAccessor trait for working with different price data formats.

use super::error::IndicatorError;

/// Base trait for all technical indicators
///
/// This trait defines the core interface implemented by all indicators in the library.
/// It is generic over the input type `T` (e.g., `f64` for price data or [`Candle`] for OHLCV data)
/// and the output type `O` (e.g., `f64` for simple indicators or custom result types for
/// more complex indicators like Bollinger Bands).
///
/// # Type Parameters
///
/// * `T` - The input data type (e.g., `f64` for price data, [`Candle`] for OHLCV data)
/// * `O` - The output data type (e.g., `f64` for simple indicators, custom struct for complex ones)
///
/// # Examples
///
/// Basic usage with a simple moving average:
///
/// ```rust,no_run
/// use rsta::indicators::Sma;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
/// // use of Indicator trait from this module
/// // Create a 14-period SMA
/// // Explicitly working with f64 data to avoid ambiguity
/// let mut sma = Sma::new(14).unwrap();
/// let _: Vec<f64> = Vec::new(); // Type hint to help compiler
///
/// // Historical price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0];
///
/// // Batch calculation
/// let sma_values = sma.calculate(&prices).unwrap();
///
/// // Real-time updates
/// // Using fully qualified syntax to resolve ambiguity between different Indicator implementations
/// <Sma as Indicator<f64, f64>>::reset(&mut sma);
/// for price in prices {
///     if let Ok(Some(value)) = sma.next(price) {
///         println!("New SMA value: {}", value);
///     }
/// }
/// ```
///
/// Using with a complex indicator like Bollinger Bands:
///
/// ```rust,no_run
/// use rsta::indicators::volatility::BollingerBands;
/// use rsta::indicators::Indicator;
/// // use of Indicator trait from this module
/// // Create Bollinger Bands with 20-period and 2 standard deviations
/// let mut bb = BollingerBands::new(20, 2.0).unwrap();
///
/// // Historical price data
/// let prices: Vec::<f64> = vec![/* price data */];
///
/// // Batch calculation
/// let bb_values = bb.calculate(&prices).unwrap();
///
/// // Access complex output type fields
/// for band in bb_values {
///     println!("Middle: {}, Upper: {}, Lower: {}",
///              band.middle, band.upper, band.lower);
/// }
/// ```
pub trait Indicator<T, O> {
    /// Calculate the indicator values based on input data
    ///
    /// This method performs batch calculation on a slice of historical data,
    /// returning a vector of output values. The length of the output vector
    /// may be smaller than the input data due to the lookback period required
    /// by most indicators.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of input data points
    ///
    /// # Returns
    ///
    /// * `Result<Vec<O>, IndicatorError>` - A vector of output values or an error
    fn calculate(&mut self, data: &[T]) -> Result<Vec<O>, IndicatorError>;

    /// Calculate the next value based on a new data point
    ///
    /// This method is designed for real-time updates, taking a single new data point
    /// and returning the latest indicator value, if available. It may return `None`
    /// until enough data points have been processed to produce a valid result.
    ///
    /// # Arguments
    ///
    /// * `value` - A single new data point
    ///
    /// # Returns
    ///
    /// * `Result<Option<O>, IndicatorError>` - The latest indicator value (if available) or an error
    fn next(&mut self, value: T) -> Result<Option<O>, IndicatorError>;

    /// Reset the indicator state
    ///
    /// This method clears the internal state of the indicator, returning it to its
    /// initial state as if newly created. This is useful when reusing the same indicator
    /// instance with different datasets.
    fn reset(&mut self);
}

/// Price data accessor trait
///
/// This trait provides a uniform interface for accessing price data components
/// (open, high, low, close, volume) regardless of the underlying data type.
/// It allows indicators to work with both simple price series (f64) and
/// complete OHLCV data through a common interface.
///
/// Implementors of this trait can work with price data in a generic way,
/// regardless of whether the data contains only closing prices or full OHLCV information.
///
/// # Type Parameters
///
/// * `T` - The price data type being accessed (e.g., `f64` or [`Candle`])
///
/// # Examples
///
/// Creating a function that works with any price data type:
///
/// ```rust,no_run
/// use rsta::indicators::Candle;
/// use rsta::indicators::PriceDataAccessor;
/// // use of PriceDataAccessor trait from this module
///
/// // Function that calculates the range (high - low) for any price data type
/// fn calculate_range<T, P: PriceDataAccessor<T>>(accessor: &P, data: &[T]) -> Vec<f64> {
///     data.iter()
///         .map(|item| accessor.get_high(item) - accessor.get_low(item))
///         .collect()
/// }
///
/// // Use with f64 data (high and low are the same, so range is 0)
/// let prices: Vec<f64> = vec![10.0, 20.0, 30.0];
/// let ranges = calculate_range(&10.0, &prices);
///
/// // Use with Candle data (actual high-low range)
/// let candles = vec![
///     Candle { timestamp: 1, open: 10.0, high: 15.0, low: 9.0, close: 14.0, volume: 1000.0 },
///     // More candles...
/// ];
/// let ranges = calculate_range(&candles[0], &candles);
/// ```
pub trait PriceDataAccessor<T> {
    /// Get the closing price from the data
    fn get_close(&self, data: &T) -> f64;

    /// Get the highest price from the data
    fn get_high(&self, data: &T) -> f64;

    /// Get the lowest price from the data
    fn get_low(&self, data: &T) -> f64;

    /// Get the opening price from the data
    fn get_open(&self, data: &T) -> f64;

    /// Get the volume from the data
    fn get_volume(&self, data: &T) -> f64;
}

/// Default implementation for f64 price data
impl PriceDataAccessor<f64> for f64 {
    fn get_close(&self, data: &f64) -> f64 {
        *data
    }
    fn get_high(&self, _: &f64) -> f64 {
        *self
    }
    fn get_low(&self, _: &f64) -> f64 {
        *self
    }
    fn get_open(&self, _: &f64) -> f64 {
        *self
    }
    fn get_volume(&self, _: &f64) -> f64 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::IndicatorError;

    // Simple mock struct for testing PriceDataAccessor trait
    #[derive(Debug, Clone, Copy)]
    struct MockCandle {
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    }

    // Implement PriceDataAccessor for MockCandle
    impl PriceDataAccessor<MockCandle> for MockCandle {
        fn get_open(&self, data: &MockCandle) -> f64 {
            data.open
        }
        fn get_high(&self, data: &MockCandle) -> f64 {
            data.high
        }
        fn get_low(&self, data: &MockCandle) -> f64 {
            data.low
        }
        fn get_close(&self, data: &MockCandle) -> f64 {
            data.close
        }
        fn get_volume(&self, data: &MockCandle) -> f64 {
            data.volume
        }
    }

    // Simple mock indicator that calculates the average of values
    struct MockAverageIndicator {
        values: Vec<f64>,
    }

    impl MockAverageIndicator {
        fn new() -> Self {
            Self { values: Vec::new() }
        }
    }

    impl Indicator<f64, f64> for MockAverageIndicator {
        fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
            if data.is_empty() {
                return Err(IndicatorError::InsufficientData("Empty data".to_string()));
            }

            let average = data.iter().sum::<f64>() / data.len() as f64;
            Ok(vec![average])
        }

        fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
            self.values.push(value);
            let avg = self.values.iter().sum::<f64>() / self.values.len() as f64;
            Ok(Some(avg))
        }

        fn reset(&mut self) {
            self.values.clear();
        }
    }

    #[test]
    fn test_price_data_accessor_f64() {
        let price = 42.0;

        // Test the default implementation for f64
        assert_eq!(price.get_close(&price), 42.0);
        assert_eq!(price.get_high(&price), 42.0);
        assert_eq!(price.get_low(&price), 42.0);
        assert_eq!(price.get_open(&price), 42.0);
        assert_eq!(price.get_volume(&price), 0.0); // Volume is 0 for f64
    }

    #[test]
    fn test_price_data_accessor_mock_candle() {
        let candle = MockCandle {
            open: 10.0,
            high: 15.0,
            low: 9.0,
            close: 14.0,
            volume: 1000.0,
        };

        // Test our implementation for MockCandle
        assert_eq!(candle.get_open(&candle), 10.0);
        assert_eq!(candle.get_high(&candle), 15.0);
        assert_eq!(candle.get_low(&candle), 9.0);
        assert_eq!(candle.get_close(&candle), 14.0);
        assert_eq!(candle.get_volume(&candle), 1000.0);
    }

    #[test]
    fn test_indicator_calculate() {
        let mut indicator = MockAverageIndicator::new();

        // Test with some data
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = indicator.calculate(&data).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 3.0); // Average of 1-5 is 3

        // Test with empty data
        let empty: Vec<f64> = Vec::new();
        let error_result = indicator.calculate(&empty);
        assert!(error_result.is_err());

        if let Err(IndicatorError::InsufficientData(msg)) = error_result {
            assert!(msg.contains("Empty"));
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_indicator_next() {
        let mut indicator = MockAverageIndicator::new();

        // First value
        assert_eq!(indicator.next(10.0).unwrap(), Some(10.0));

        // Second value
        assert_eq!(indicator.next(20.0).unwrap(), Some(15.0)); // Avg of 10 and 20

        // Third value
        assert_eq!(indicator.next(30.0).unwrap(), Some(20.0)); // Avg of 10, 20, and 30

        // Reset and start again
        indicator.reset();
        assert_eq!(indicator.next(100.0).unwrap(), Some(100.0));
    }

    #[test]
    fn test_trait_usage_with_generic_function() {
        // Define a generic function that works with any PriceDataAccessor
        fn calculate_range<T, P: PriceDataAccessor<T>>(data: &[T], accessor: &P) -> f64 {
            if data.is_empty() {
                return 0.0;
            }

            let high = accessor.get_high(&data[0]);
            let low = accessor.get_low(&data[0]);
            high - low
        }

        // Test with f64
        let prices = vec![42.0];
        let range = calculate_range(&prices, &prices[0]);
        assert_eq!(range, 0.0); // High and low are the same for f64

        // Test with MockCandle
        let candles = vec![MockCandle {
            open: 10.0,
            high: 15.0,
            low: 9.0,
            close: 14.0,
            volume: 1000.0,
        }];
        let range = calculate_range(&candles, &candles[0]);
        assert_eq!(range, 6.0); // 15 - 9 = 6
    }
}
