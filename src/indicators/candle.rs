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
/// use rsta::indicators::volatility::AverageTrueRange;
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
/// let mut atr = AverageTrueRange::new(3).unwrap();
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

/// Convert a series of regular OHLC candles into Heikin-Ashi form.
///
/// Heikin-Ashi (HA) candles smooth out noise by averaging the price action
/// across two consecutive bars. They are not an indicator in the streaming
/// sense — they are a transformation of the input series — but they are
/// commonly used as a *feed* for other indicators, so the function lives
/// here next to [`Candle`] itself.
///
/// Definition:
/// - `HA_close = (O + H + L + C) / 4`
/// - `HA_open  = (HA_open[prev] + HA_close[prev]) / 2`
///   (seeded with `(O[0] + C[0]) / 2` for the first bar)
/// - `HA_high  = max(H, HA_open, HA_close)`
/// - `HA_low   = min(L, HA_open, HA_close)`
/// - volume and timestamp are forwarded from the source candle.
///
/// Returns an empty `Vec` if `candles` is empty.
///
/// # Example
/// ```
/// use rsta::indicators::candle::{heikin_ashi, Candle};
///
/// let candles = vec![
///     Candle { timestamp: 0, open: 10.0, high: 11.0, low: 9.0, close: 10.5, volume: 1.0 },
///     Candle { timestamp: 1, open: 10.5, high: 12.0, low: 10.0, close: 11.5, volume: 1.0 },
/// ];
/// let ha = heikin_ashi(&candles);
/// assert_eq!(ha.len(), 2);
/// // First HA close = (10 + 11 + 9 + 10.5) / 4 = 10.125
/// assert!((ha[0].close - 10.125).abs() < 1e-12);
/// // First HA open seeded as (10 + 10.5) / 2 = 10.25
/// assert!((ha[0].open - 10.25).abs() < 1e-12);
/// ```
pub fn heikin_ashi(candles: &[Candle]) -> Vec<Candle> {
    if candles.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(candles.len());
    let first = &candles[0];
    let ha_close_0 = (first.open + first.high + first.low + first.close) / 4.0;
    let ha_open_0 = (first.open + first.close) / 2.0;
    out.push(Candle {
        timestamp: first.timestamp,
        open: ha_open_0,
        high: first.high.max(ha_open_0).max(ha_close_0),
        low: first.low.min(ha_open_0).min(ha_close_0),
        close: ha_close_0,
        volume: first.volume,
    });
    for c in &candles[1..] {
        let prev = out.last().unwrap();
        let ha_close = (c.open + c.high + c.low + c.close) / 4.0;
        let ha_open = (prev.open + prev.close) / 2.0;
        out.push(Candle {
            timestamp: c.timestamp,
            open: ha_open,
            high: c.high.max(ha_open).max(ha_close),
            low: c.low.min(ha_open).min(ha_close),
            close: ha_close,
            volume: c.volume,
        });
    }
    out
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
    fn test_heikin_ashi_empty() {
        let out = heikin_ashi(&[]);
        assert!(out.is_empty());
    }

    #[test]
    fn test_heikin_ashi_first_bar_seeded() {
        let candles = [Candle {
            timestamp: 0,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 1.0,
        }];
        let ha = heikin_ashi(&candles);
        // HA_close = (10 + 11 + 9 + 10.5) / 4 = 10.125
        assert!((ha[0].close - 10.125).abs() < 1e-12);
        // HA_open  = (10 + 10.5) / 2 = 10.25
        assert!((ha[0].open - 10.25).abs() < 1e-12);
        assert_eq!(ha[0].high, 11.0); // max(11, 10.25, 10.125)
        assert_eq!(ha[0].low, 9.0); // min(9, 10.25, 10.125)
        assert_eq!(ha[0].volume, 1.0);
    }

    #[test]
    fn test_heikin_ashi_recurrence_uses_prev_ha() {
        let candles = [
            Candle {
                timestamp: 0,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1.0,
            },
            Candle {
                timestamp: 1,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.5,
                volume: 1.0,
            },
        ];
        let ha = heikin_ashi(&candles);
        // HA[1].open = (HA[0].open + HA[0].close) / 2 = (10.25 + 10.125) / 2 = 10.1875
        assert!((ha[1].open - 10.1875).abs() < 1e-12);
        // HA[1].close = (10.5 + 12 + 10 + 11.5) / 4 = 11.0
        assert!((ha[1].close - 11.0).abs() < 1e-12);
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
