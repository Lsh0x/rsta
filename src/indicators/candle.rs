//! OHLCV Price Data Structures
//!
//! This module contains structures and traits for working with OHLCV (Open, High, Low, Close, Volume)
//! price data in technical analysis calculations.

use super::traits::PriceDataAccessor;

/// Price data with OHLCV components
///
/// This struct represents a single candlestick in a price chart, containing
/// Open, High, Low, Close prices and Volume data, along with a timestamp.
/// It is used by indicators that require more than just closing prices.
///
/// # Examples
///
/// Creating and using candle data:
///
/// ```
/// use rsta::indicators::volatility::ATR;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create a series of candlesticks
/// let candles = vec![
///     Candle {
///         timestamp: 1618185600, // Unix timestamp (seconds since epoch)
///         open: 100.0,
///         high: 105.0,
///         low: 98.0,
///         close: 103.0,
///         volume: 1000.0
///     },
///     Candle {
///         timestamp: 1618272000, // Next day
///         open: 103.0,
///         high: 107.0,
///         low: 101.0,
///         close: 105.0,
///         volume: 1200.0
///     },
///     Candle {
///         timestamp: 1618358400, // Two days later
///         open: 105.0,
///         high: 108.0,
///         low: 102.0,
///         close: 106.0,
///         volume: 1100.0
///     },
///     Candle {
///         timestamp: 1618444800, // Three days later
///         open: 106.0,
///         high: 110.0,
///         low: 104.0,
///         close: 109.0,
///         volume: 1300.0
///     },
///     Candle {
///         timestamp: 1618531200, // Four days later
///         open: 109.0,
///         high: 112.0,
///         low: 107.0,
///         close: 110.0,
///         volume: 1400.0
///     }
/// ];
///
/// // Use with an indicator that requires OHLCV data
/// let mut atr = ATR::new(3).unwrap();
/// let atr_values = atr.calculate(&candles).unwrap();
///
/// // The ATR values can be inspected
/// println!("ATR value: {}", atr_values[0]); // First ATR value
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    /// Timestamp (typically Unix timestamp in seconds)
    pub timestamp: u64,
    /// Opening price
    pub open: f64,
    /// Highest price during the period
    pub high: f64,
    /// Lowest price during the period
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
}

/// Default implementation for Candle price data
impl PriceDataAccessor<Candle> for Candle {
    fn get_close(&self, data: &Candle) -> f64 {
        data.close
    }
    fn get_high(&self, data: &Candle) -> f64 {
        data.high
    }
    fn get_low(&self, data: &Candle) -> f64 {
        data.low
    }
    fn get_open(&self, data: &Candle) -> f64 {
        data.open
    }
    fn get_volume(&self, data: &Candle) -> f64 {
        data.volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_creation_and_access() {
        // Create a sample candle
        let candle = Candle {
            timestamp: 1618185600,
            open: 100.0,
            high: 105.0,
            low: 98.0,
            close: 103.0,
            volume: 1000.0,
        };

        // Verify that fields are correctly accessible
        assert_eq!(candle.timestamp, 1618185600);
        assert_eq!(candle.open, 100.0);
        assert_eq!(candle.high, 105.0);
        assert_eq!(candle.low, 98.0);
        assert_eq!(candle.close, 103.0);
        assert_eq!(candle.volume, 1000.0);
    }

    #[test]
    fn test_price_data_accessor_impl() {
        // Create a sample candle
        let candle = Candle {
            timestamp: 1618185600,
            open: 100.0,
            high: 105.0,
            low: 98.0,
            close: 103.0,
            volume: 1000.0,
        };

        // Test that PriceDataAccessor methods return correct values
        assert_eq!(candle.get_open(&candle), 100.0);
        assert_eq!(candle.get_high(&candle), 105.0);
        assert_eq!(candle.get_low(&candle), 98.0);
        assert_eq!(candle.get_close(&candle), 103.0);
        assert_eq!(candle.get_volume(&candle), 1000.0);
    }

    #[test]
    fn test_candle_copy_and_clone() {
        // Create a sample candle
        let candle1 = Candle {
            timestamp: 1618185600,
            open: 100.0,
            high: 105.0,
            low: 98.0,
            close: 103.0,
            volume: 1000.0,
        };

        // Test copy
        let candle2 = candle1;
        assert_eq!(candle1.timestamp, candle2.timestamp);
        assert_eq!(candle1.open, candle2.open);
        assert_eq!(candle1.high, candle2.high);
        assert_eq!(candle1.low, candle2.low);
        assert_eq!(candle1.close, candle2.close);
        assert_eq!(candle1.volume, candle2.volume);

        // Test clone
        let candle3 = candle1;
        assert_eq!(candle1.timestamp, candle3.timestamp);
        assert_eq!(candle1.open, candle3.open);
        assert_eq!(candle1.high, candle3.high);
        assert_eq!(candle1.low, candle3.low);
        assert_eq!(candle1.close, candle3.close);
        assert_eq!(candle1.volume, candle3.volume);
    }
}
