/// # Technical Analysis Indicators
///
/// This module contains a comprehensive collection of technical analysis indicators
/// used in financial markets, organized by category and with a consistent interface.
///
/// ## Indicator Categories
///
/// The indicators are organized into four main categories:
///
/// - [`trend`]: Trend following indicators like Moving Averages
/// - [`momentum`]: Momentum indicators like RSI and Stochastic Oscillator
/// - [`volume`]: Volume-based indicators like OBV and A/D Line
/// - [`volatility`]: Volatility indicators like ATR and Bollinger Bands
///
/// ## Core Components
///
/// The library is built around these core components:
///
/// - [`Indicator`] trait: Common interface implemented by all indicators
/// - [`Candle`] struct: Represents OHLCV price data
/// - [`PriceDataAccessor`] trait: Provides uniform access to price data
/// - [`IndicatorError`] enum: Standardized error handling
///
/// ## Using Indicators
///
/// All indicators follow a common pattern:
///
/// 1. Create a new indicator instance with specific parameters
/// 2. Call `calculate()` with historical data to get a vector of values
/// 3. Or use `next()` for real-time updates with new data points
///
/// ```rust,no_run
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Sma;
///
/// // Create a new indicator instance
/// let mut sma = Sma::new(14).unwrap();
///
/// // Historical price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0];
///
/// // Batch calculation
/// let sma_values = sma.calculate(&prices).unwrap();
/// println!("SMA values: {:?}", sma_values);
///
/// // Or real-time updates
/// sma.reset(); // Reset the state first
/// for price in prices {
///     if let Some(value) = sma.next(price).unwrap() {
///         println!("New SMA value: {}", value);
///     }
/// }
/// ```
///
/// ## Working with OHLCV Data
///
/// Some indicators require OHLCV (Open, High, Low, Close, Volume) data:
///
/// ```rust,no_run
/// use rsta::indicators::Indicator;
/// use rsta::indicators::volatility::ATR;
/// use rsta::indicators::Candle;
///
/// // Create indicator
/// let mut atr = ATR::new(14).unwrap();
///
/// // Create OHLCV data
/// let candles = vec![
///     Candle { timestamp: 1, open: 10.0, high: 12.0, low: 9.0, close: 11.0, volume: 1000.0 },
///     Candle { timestamp: 2, open: 11.0, high: 13.0, low: 10.0, close: 12.0, volume: 1200.0 },
///     // Additional candles...
/// ];
///
/// // Calculate ATR values
/// let atr_values = atr.calculate(&candles).unwrap();
/// ```
///
/// ## Common Utilities
///
/// The [`utils`] module provides common calculations used across indicators.

// Ensure volatility module is accessible
pub mod volatility;

// Module declarations
pub mod candle;
pub mod error;
pub mod momentum;
pub mod traits;
pub mod trend;
pub mod utils;
pub mod volume;

// Re-export core traits and types
pub use self::candle::Candle;
pub use self::error::IndicatorError;
pub use self::traits::{Indicator, PriceDataAccessor};

// Re-export momentum indicators
pub use self::momentum::{
    RSI, StochasticOscillator, StochasticResult, WilliamsR,
};

// Re-export volatility indicators
pub use self::volatility::{
    ATR as Atr, BB, bb::BBResult, KeltnerChannels, KeltnerChannelsResult, STD as Std,
};

// Re-export trend indicators
pub use self::trend::{
    EMA as Ema, SMA as Sma
};

// Re-export volume indicators
pub use self::volume::{
    AccumulationDistributionLine,
    OnBalanceVolume,
    VolumeRateOfChange, // ChaikinMoneyFlow is not public in volume.rs
};

// Re-export utility functions
pub use self::utils::{
    calculate_ema, calculate_sma, rate_of_change, standard_deviation, validate_data_length,
    validate_period,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reexported_types() {
        // Create a candle using the re-exported type
        let candle = Candle {
            timestamp: 1618185600,
            open: 100.0,
            high: 105.0,
            low: 98.0,
            close: 103.0,
            volume: 1000.0,
        };

        // Verify candle fields are accessible
        assert_eq!(candle.open, 100.0);
        assert_eq!(candle.high, 105.0);
        assert_eq!(candle.low, 98.0);
        assert_eq!(candle.close, 103.0);
        assert_eq!(candle.volume, 1000.0);
    }

    #[test]
    fn test_sma_calculation() {
        // Test using SMA with the Indicator trait
        let mut sma = Sma::new(3).unwrap();

        // Sample price data
        let prices = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        // Calculate values using the Indicator trait method
        let result = sma.calculate(&prices).unwrap();

        // Verify results
        assert_eq!(result.len(), 3); // 5 prices - 3 period + 1 = 3 results
        assert_eq!(result[0], 4.0); // (2+4+6)/3
        assert_eq!(result[1], 6.0); // (4+6+8)/3
        assert_eq!(result[2], 8.0); // (6+8+10)/3
    }

    #[test]
    fn test_indicator_next_method() {
        // Test using the next method from the Indicator trait
        let mut sma = Sma::new(3).unwrap();

        // Add values one by one
        assert_eq!(sma.next(2.0).unwrap(), None); // Not enough data yet
        assert_eq!(sma.next(4.0).unwrap(), None); // Not enough data yet
        assert_eq!(sma.next(6.0).unwrap(), Some(4.0)); // First complete SMA
        assert_eq!(sma.next(8.0).unwrap(), Some(6.0)); // Second SMA
        assert_eq!(sma.next(10.0).unwrap(), Some(8.0)); // Third SMA
    }

    #[test]
    fn test_error_handling() {
        // Test error handling using re-exported error type
        let error_result = Sma::new(0); // Invalid period

        assert!(error_result.is_err());
        match error_result {
            Err(IndicatorError::InvalidParameter(msg)) => {
                assert!(msg.contains("Period must be greater than"));
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_utility_functions() {
        // Test a utility function
        let result = validate_period(10, 5);
        assert!(result.is_ok());

        let result = validate_period(4, 5);
        assert!(result.is_err());

        // Test SMA calculation
        let prices = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let sma_result = calculate_sma(&prices, 3).unwrap();
        assert_eq!(sma_result, vec![4.0, 6.0, 8.0]);
    }
}
