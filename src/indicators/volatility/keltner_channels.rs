use std::collections::VecDeque;

use crate::indicators::volatility::ATR;
use crate::indicators::{Candle, Indicator, IndicatorError};
use crate::indicators::utils::{calculate_ema, validate_data_length, validate_period};

/// Keltner Channels indicator
///
/// Keltner Channels are volatility-based bands that use the Average True Range (ATR)
/// to set channel distance. The channels are typically set two ATR values above and below
/// an Exponential Moving Average (EMA) of the price.
///
/// # Example
///
/// ```
/// use rsta::indicators::volatility::{KeltnerChannels, KeltnerChannelsResult};
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create a Keltner Channels indicator with EMA period 20, ATR period 10, and multiplier 2.0
/// let mut keltner = KeltnerChannels::new(20, 10, 2.0).unwrap();
///
/// // Create price data with OHLC values (need at least 20 candles for EMA period)
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
///     Candle { timestamp: 15, open: 49.0, high: 50.0, low: 48.0, close: 49.5, volume: 2400.0 },
///     Candle { timestamp: 16, open: 49.5, high: 50.5, low: 48.5, close: 50.0, volume: 2500.0 },
///     Candle { timestamp: 17, open: 50.0, high: 51.0, low: 49.0, close: 50.5, volume: 2600.0 },
///     Candle { timestamp: 18, open: 50.5, high: 51.5, low: 49.5, close: 51.0, volume: 2700.0 },
///     Candle { timestamp: 19, open: 51.0, high: 52.0, low: 50.0, close: 51.5, volume: 2800.0 },
///     Candle { timestamp: 20, open: 51.5, high: 52.5, low: 50.5, close: 52.0, volume: 2900.0 },
///     // Additional candles for testing
///     Candle { timestamp: 21, open: 52.0, high: 54.0, low: 51.0, close: 53.5, volume: 3000.0 }, // Volatility increases
///     Candle { timestamp: 22, open: 53.5, high: 54.5, low: 52.5, close: 53.0, volume: 3100.0 }, // Price drops
/// ];
///
/// // Calculate Keltner Channels with error handling
/// match keltner.calculate(&candles) {
///     Ok(channels) => {
///         // Access the latest Keltner Channels values
///         if let Some(latest) = channels.last() {
///             println!("Middle band (EMA): {:.2}", latest.middle); // Example output: Middle band (EMA): 52.50
///             println!("Upper band: {:.2}", latest.upper);         // Example output: Upper band: 56.80
///             println!("Lower band: {:.2}", latest.lower);         // Example output: Lower band: 48.20
///             println!("Bandwidth: {:.4}", latest.bandwidth);      // Example output: Bandwidth: 0.1638
///             
///             // Interpret the values
///             let current_close = candles.last().unwrap().close;
///             
///             if current_close > latest.upper {
///                 println!("Price is above upper band - potential overbought condition");
///             } else if current_close < latest.lower {
///                 println!("Price is below lower band - potential oversold condition");
///             } else {
///                 println!("Price is within the Keltner Channels");
///             }
///             
///             // Trend analysis
///             if channels.len() >= 2 {
///                 let previous = channels[channels.len() - 2];
///                 
///                 if latest.bandwidth > previous.bandwidth {
///                     println!("Bandwidth increasing - volatility is rising");
///                 } else if latest.bandwidth < previous.bandwidth {
///                     println!("Bandwidth decreasing - volatility is falling");
///                 }
///                 
///                 if latest.middle > previous.middle {
///                     println!("Middle band rising - uptrend continues");
///                 } else if latest.middle < previous.middle {
///                     println!("Middle band falling - downtrend continues");
///                 }
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating Keltner Channels: {}", e);
///     }
/// }
/// ```
/// 
/// 

/// Keltner Channels indicator result
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeltnerChannelsResult {
    /// Middle band (usually EMA)
    pub middle: f64,
    /// Upper band (middle + multiplier * ATR)
    pub upper: f64,
    /// Lower band (middle - multiplier * ATR)
    pub lower: f64,
    /// Width of the channels ((upper - lower) / middle)
    pub bandwidth: f64,
}


#[derive(Debug)]
pub struct KeltnerChannels {
    ema_period: usize,
    atr_period: usize,
    multiplier: f64,
    candle_buffer: VecDeque<Candle>,
    current_ema: Option<f64>,
    current_atr: Option<f64>,
}

impl KeltnerChannels {
    /// Create a new KeltnerChannels indicator
    ///
    /// # Arguments
    /// * `ema_period` - The period for EMA calculation (must be at least 1)
    /// * `atr_period` - The period for ATR calculation (must be at least 1)
    /// * `multiplier` - The multiplier for the ATR to determine channel width (typical: 1.5-2.5)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new KeltnerChannels or an error
    pub fn new(
        ema_period: usize,
        atr_period: usize,
        multiplier: f64,
    ) -> Result<Self, IndicatorError> {
        validate_period(ema_period, 1)?;
        validate_period(atr_period, 1)?;

        if multiplier <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "ATR multiplier must be positive".to_string(),
            ));
        }

        Ok(Self {
            ema_period,
            atr_period,
            multiplier,
            candle_buffer: VecDeque::with_capacity(ema_period.max(atr_period)),
            current_ema: None,
            current_atr: None,
        })
    }
}

impl Indicator<Candle, KeltnerChannelsResult> for KeltnerChannels {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<KeltnerChannelsResult>, IndicatorError> {
        // Need enough data for both EMA and ATR
        let min_data_len = self.ema_period.max(self.atr_period);
        validate_data_length(data, min_data_len)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - min_data_len + 1);

        // Reset state
        self.reset();

        // Extract close prices for EMA calculation
        let close_prices: Vec<f64> = data.iter().map(|c| c.close).collect();

        // Calculate EMA values using close prices
        let ema_values = calculate_ema(&close_prices, self.ema_period)?;

        // Calculate ATR values
        let _start_idx = (self.ema_period - 1).max(self.atr_period - 1);
        let ema_offset = self.atr_period.saturating_sub(self.ema_period);

        let atr_offset = self.ema_period.saturating_sub(self.atr_period);

        // Calculate ATR values
        let mut atr = ATR::new(self.atr_period)?;
        let atr_values = atr.calculate(data)?;

        // Calculate Keltner Channels for each period where we have both EMA and ATR
        for i in 0..atr_values.len().min(ema_values.len() - ema_offset) {
            let ema = ema_values[i + ema_offset];
            let atr = atr_values[i + atr_offset];

            let upper = ema + (self.multiplier * atr);
            let lower = ema - (self.multiplier * atr);
            let bandwidth = (upper - lower) / ema;

            result.push(KeltnerChannelsResult {
                middle: ema,
                upper,
                lower,
                bandwidth,
            });
        }

        // Update state with the last values
        self.current_ema = Some(*ema_values.last().unwrap());
        self.current_atr = Some(*atr_values.last().unwrap());

        for candle in data.iter().take(n).skip(n - min_data_len) {
            self.candle_buffer.push_back(*candle);
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<KeltnerChannelsResult>, IndicatorError> {
        self.candle_buffer.push_back(value);

        let min_data_len = self.ema_period.max(self.atr_period);

        if self.candle_buffer.len() > min_data_len {
            self.candle_buffer.pop_front();
        }

        if self.candle_buffer.len() < min_data_len {
            return Ok(None);
        }

        // Real-time update of EMA
        if let Some(current_ema) = self.current_ema {
            let alpha = 2.0 / (self.ema_period as f64 + 1.0);
            let new_ema = (value.close - current_ema) * alpha + current_ema;
            self.current_ema = Some(new_ema);
        } else {
            // Initial EMA calculation
            let close_prices: Vec<f64> = self.candle_buffer.iter().map(|c| c.close).collect();
            let ema_values = calculate_ema(&close_prices, self.ema_period)?;
            self.current_ema = Some(*ema_values.last().unwrap());
        }

        // Real-time update of ATR
        let mut atr = ATR::new(self.atr_period)?;
        let candles: Vec<Candle> = self.candle_buffer.iter().cloned().collect();
        let atr_values = atr.calculate(&candles)?;
        self.current_atr = Some(*atr_values.last().unwrap());

        // Create result
        let ema = self.current_ema.unwrap();
        let atr = self.current_atr.unwrap();

        let upper = ema + (self.multiplier * atr);
        let lower = ema - (self.multiplier * atr);
        let bandwidth = (upper - lower) / ema;

        Ok(Some(KeltnerChannelsResult {
            middle: ema,
            upper,
            lower,
            bandwidth,
        }))
    }

    fn reset(&mut self) {
        self.candle_buffer.clear();
        self.current_ema = None;
        self.current_atr = None;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // KeltnerChannels Tests
    #[test]
    fn test_keltner_channels_new() {
        // Valid parameters should work
        assert!(KeltnerChannels::new(20, 10, 2.0).is_ok());

        // Invalid period should fail
        assert!(KeltnerChannels::new(0, 10, 2.0).is_err());

        // Negative multiplier should fail
        assert!(KeltnerChannels::new(20, 10, -1.0).is_err());
    }

    #[test]
    fn test_keltner_channels_calculation() {
        let mut kc = KeltnerChannels::new(3, 3, 2.0).unwrap();

        // Create test candles with predictable pattern
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 13.0,
                low: 7.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 13.0,
                low: 7.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 13.0,
                low: 7.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 13.0,
                low: 7.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 13.0,
                low: 7.0,
                close: 10.0,
                volume: 1000.0,
            },
        ];

        let result = kc.calculate(&candles).unwrap();

        // We need at least period candles for the first result
        assert_eq!(result.len(), 3);

        // For these candles, the TR is always 6 (high-low) and EMA is 10 (all closes are 10)
        // First Keltner:
        // Middle = EMA of closes = 10
        // ATR = 6
        // Upper = 10 + (2 * 6) = 22
        // Lower = 10 - (2 * 6) = -2
        assert_eq!(result[0].middle, 10.0);
        assert!((result[0].upper - 22.0).abs() < 0.1);
        assert!((result[0].lower - (-2.0)).abs() < 0.1);
    }

    #[test]
    fn test_keltner_channels_next_error() {
        let mut kc = KeltnerChannels::new(14, 10, 2.0).unwrap();
        let candle = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: 1000.0,
        };

        // The next method should return an error as it's not implemented

        // Test that the next method works with a candle
        let result = kc.next(candle);

        // Since we haven't accumulated enough candles yet, we should get None
        assert!(result.is_ok());
        if let Ok(value) = result {
            assert!(value.is_none());
        }
    }

    #[test]
    fn test_keltner_channels_reset() {
        let mut kc = KeltnerChannels::new(3, 3, 2.0).unwrap();

        // Add some candles
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 9.0,
            close: 11.0,
            volume: 1000.0,
        };
        kc.next(candle1).unwrap();

        // Reset
        kc.reset();

        // After reset, we should have cleared state
        assert!(kc.current_ema.is_none());
        assert!(kc.current_atr.is_none());
    }
}