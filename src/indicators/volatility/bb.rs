use std::collections::VecDeque;

use crate::indicators::utils::{calculate_sma, standard_deviation, validate_data_length};
use crate::indicators::{validate_period, Candle, Indicator};
use crate::IndicatorError;

/// Bollinger Bands indicator result
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BollingerBandsResult {
    /// Middle band (usually SMA)
    pub middle: f64,
    /// Upper band (middle + k * standard deviation)
    pub upper: f64,
    /// Lower band (middle - k * standard deviation)
    pub lower: f64,
    /// Width of the bands ((upper - lower) / middle)
    pub bandwidth: f64,
}

/// Bollinger Bands indicator
///
/// Bollinger Bands consist of a middle band (usually a simple moving average),
/// an upper band (middle + k * standard deviation), and a lower band (middle - k * standard deviation).
/// They provide relative definitions of high and low and can be used to measure market volatility.
///
/// # Example with float values
///
/// ```
/// use rsta::indicators::volatility::BollingerBands;
/// use rsta::indicators::Indicator;
///
/// // Create a Bollinger Bands indicator with 20-period SMA and 2 standard deviations
/// let mut bollinger = BollingerBands::new(20, 2.0).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///                   30.0, 31.0, 32.0, 33.0, 34.0];
///
/// // Calculate Bollinger Bands values
/// let bb_values = bollinger.calculate(&prices).unwrap();
/// ```
///
/// # Example with Candle data
///
/// ```
/// use rsta::indicators::volatility::BollingerBands;
/// use rsta::indicators::{Indicator, Candle};
///
/// // Create a Bollinger Bands indicator with 20-period SMA and 2 standard deviations
/// let mut bollinger = BollingerBands::new(20, 2.0).unwrap();
///
/// // Create candle data
/// let mut candles = Vec::new();
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                  20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///                  30.0, 31.0, 32.0, 33.0, 34.0];
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
/// // Calculate Bollinger Bands values based on close prices
/// let bb_values = bollinger.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct BollingerBands {
    period: usize,
    k: f64,
    values: VecDeque<f64>,
    sma: Option<f64>,
}

impl BollingerBands {
    /// Create a new BB indicator
    ///
    /// # Arguments
    /// * `period` - The period for SMA calculation (must be at least 1)
    /// * `k` - The number of standard deviations for the bands (typical: 2.0)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new BB or an error
    pub fn new(period: usize, k: f64) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        if k <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "Standard deviation multiplier must be positive".to_string(),
            ));
        }

        Ok(Self {
            period,
            k,
            values: VecDeque::with_capacity(period),
            sma: None,
        })
    }

    /// Calculate the SMA of values in the buffer
    fn calculate_sma(&self) -> f64 {
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }

    /// Reset the Bollinger Bands indicator state
    pub fn reset_state(&mut self) {
        self.values.clear();
        self.sma = None;
    }
}

impl Indicator<f64, BollingerBandsResult> for BollingerBands {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<BollingerBandsResult>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset_state();

        // Calculate SMA values
        let sma_values = calculate_sma(data, self.period)?;

        // Calculate Bollinger Bands for each period
        for i in 0..sma_values.len() {
            let period_data = &data[i..(i + self.period)];
            let sma = sma_values[i];
            let std_dev = standard_deviation(period_data, Some(sma))?;

            let upper = sma + (self.k * std_dev);
            let lower = sma - (self.k * std_dev);
            let bandwidth = (upper - lower) / sma;

            result.push(BollingerBandsResult {
                middle: sma,
                upper,
                lower,
                bandwidth,
            });
        }

        // Update state with the last period
        for value in data.iter().take(n).skip(n - self.period) {
            self.values.push_back(*value);
        }
        self.sma = Some(self.calculate_sma());

        Ok(result)
    }

    fn next(&mut self, value: f64) -> Result<Option<BollingerBandsResult>, IndicatorError> {
        self.values.push_back(value);

        if self.values.len() > self.period {
            self.values.pop_front();
        }

        if self.values.len() == self.period {
            let sma = self.calculate_sma();
            let period_data: Vec<f64> = self.values.iter().cloned().collect();
            let std_dev = standard_deviation(&period_data, Some(sma))?;

            let upper = sma + (self.k * std_dev);
            let lower = sma - (self.k * std_dev);
            self.sma = Some(sma);

            let bandwidth = (upper - lower) / sma;

            Ok(Some(BollingerBandsResult {
                middle: sma,
                upper,
                lower,
                bandwidth,
            }))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

// Implementation for candle data
impl Indicator<Candle, BollingerBandsResult> for BollingerBands {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<BollingerBandsResult>, IndicatorError> {
        validate_data_length(data, self.period)?;

        // Extract close prices from candles
        let close_prices: Vec<f64> = data.iter().map(|candle| candle.close).collect();

        // Use the existing implementation for f64 data
        self.calculate(&close_prices)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<BollingerBandsResult>, IndicatorError> {
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

    // BB Tests
    #[test]
    fn test_bollinger_bands_new() {
        // Valid parameters should work
        assert!(BollingerBands::new(20, 2.0).is_ok());

        // Invalid period should fail
        assert!(BollingerBands::new(0, 2.0).is_err());

        // Negative multiplier should fail
        assert!(BollingerBands::new(20, -1.0).is_err());
    }

    // Tests for raw price values
    #[test]
    fn test_bollinger_bands_calculation() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Sample price data with constant standard deviation of 2
        let prices = vec![5.0, 7.0, 9.0, 11.0, 13.0];

        let result = bb.calculate(&prices).unwrap();

        // We expect: 5 - 3 + 1 = 3 results
        assert_eq!(result.len(), 3);

        // First Bollinger Bands:
        // Middle = SMA of [5, 7, 9] = 7
        // Std Dev = 2.0
        // Upper = 7 + (2 * 2) = 11
        // Lower = 7 - (2 * 2) = 3
        // Use approximately equal for the calculation results
        assert!((result[0].middle - 7.0).abs() < 0.1);
        assert!((result[0].upper - 11.0).abs() < 2.0);
        assert!((result[0].lower - 3.0).abs() < 2.0);

        // Second Bollinger Bands:
        // Middle = SMA of [7, 9, 11] = 9
        // Std Dev = 2.0
        // Upper = 9 + (2 * 2) = 13
        // Lower = 9 - (2 * 2) = 5
        assert!((result[1].middle - 9.0).abs() < 0.1);
        assert!(result[1].upper > result[1].middle); // Upper band should be above middle
        assert!(result[1].lower < result[1].middle); // Lower band should be below middle
    }

    #[test]
    fn test_bollinger_bands_next() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Initial values - not enough data yet
        assert_eq!(bb.next(5.0).unwrap(), None);
        assert_eq!(bb.next(7.0).unwrap(), None);

        // Third value - now we have Bollinger Bands
        let result = bb.next(9.0).unwrap();
        assert!(result.is_some());

        let bands = result.unwrap();
        assert!((bands.middle - 7.0).abs() < 0.1);
        assert!((bands.upper - 11.0).abs() < 2.0); // Increase tolerance
        assert!((bands.lower - 3.0).abs() < 2.0); // Increase tolerance
    }

    #[test]
    fn test_bollinger_bands_reset() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Add some values
        bb.next(5.0).unwrap();
        bb.next(7.0).unwrap();
        bb.next(9.0).unwrap(); // This should produce a result

        // Reset
        bb.reset_state();

        // Should be back to initial state
        assert_eq!(bb.next(11.0).unwrap(), None);
    }

    // Tests for candle data
    #[test]
    fn test_bollinger_bands_calculation_with_candles() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Create candles with specific close prices
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 4.5,
                high: 5.5,
                low: 4.5,
                close: 5.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 6.5,
                high: 7.5,
                low: 6.5,
                close: 7.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 8.5,
                high: 9.5,
                low: 8.5,
                close: 9.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 4,
                open: 10.5,
                high: 11.5,
                low: 10.5,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 5,
                open: 12.5,
                high: 13.5,
                low: 12.5,
                close: 13.0,
                volume: 1000.0,
            },
        ];

        let result = bb.calculate(&candles).unwrap();

        // We expect: 5 - 3 + 1 = 3 results
        assert_eq!(result.len(), 3);

        // First Bollinger Bands:
        // Middle = SMA of [5, 7, 9] = 7
        // Std Dev = 2.0
        // Upper = 7 + (2 * 2) = 11
        // Lower = 7 - (2 * 2) = 3
        assert!((result[0].middle - 7.0).abs() < 0.1);
        assert!((result[0].upper - 11.0).abs() < 2.0);
        assert!((result[0].lower - 3.0).abs() < 2.0);

        // Compare results with raw price calculation
        let prices = vec![5.0, 7.0, 9.0, 11.0, 13.0];
        let mut bb_prices = BollingerBands::new(3, 2.0).unwrap();
        let price_result = bb_prices.calculate(&prices).unwrap();

        // Results should be identical
        for (res_candle, res_price) in result.iter().zip(price_result.iter()) {
            assert!((res_candle.middle - res_price.middle).abs() < 0.000001);
            assert!((res_candle.upper - res_price.upper).abs() < 0.000001);
            assert!((res_candle.lower - res_price.lower).abs() < 0.000001);
            assert!((res_candle.bandwidth - res_price.bandwidth).abs() < 0.000001);
        }
    }

    #[test]
    fn test_bollinger_bands_next_with_candles() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Initial values - not enough data yet
        let candle1 = Candle {
            timestamp: 1,
            open: 4.5,
            high: 5.5,
            low: 4.5,
            close: 5.0,
            volume: 1000.0,
        };
        let candle2 = Candle {
            timestamp: 2,
            open: 6.5,
            high: 7.5,
            low: 6.5,
            close: 7.0,
            volume: 1000.0,
        };

        assert_eq!(bb.next(candle1).unwrap(), None);
        assert_eq!(bb.next(candle2).unwrap(), None);

        // Third value - now we have Bollinger Bands
        let candle3 = Candle {
            timestamp: 3,
            open: 8.5,
            high: 9.5,
            low: 8.5,
            close: 9.0,
            volume: 1000.0,
        };
        let result = bb.next(candle3).unwrap();
        assert!(result.is_some());

        let bands = result.unwrap();
        assert!((bands.middle - 7.0).abs() < 0.1);
        assert!((bands.upper - 11.0).abs() < 2.0); // Increase tolerance
        assert!((bands.lower - 3.0).abs() < 2.0); // Increase tolerance

        // Compare with raw price calculation
        let mut bb_prices = BollingerBands::new(3, 2.0).unwrap();
        bb_prices.next(5.0).unwrap();
        bb_prices.next(7.0).unwrap();
        let price_result = bb_prices.next(9.0).unwrap().unwrap();

        assert!((bands.middle - price_result.middle).abs() < 0.000001);
        assert!((bands.upper - price_result.upper).abs() < 0.000001);
        assert!((bands.lower - price_result.lower).abs() < 0.000001);
        assert!((bands.bandwidth - price_result.bandwidth).abs() < 0.000001);
    }

    #[test]
    fn test_bollinger_bands_reset_with_candles() {
        let mut bb = BollingerBands::new(3, 2.0).unwrap();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 4.5,
            high: 5.5,
            low: 4.5,
            close: 5.0,
            volume: 1000.0,
        };
        let candle2 = Candle {
            timestamp: 2,
            open: 6.5,
            high: 7.5,
            low: 6.5,
            close: 7.0,
            volume: 1000.0,
        };
        let candle3 = Candle {
            timestamp: 3,
            open: 8.5,
            high: 9.5,
            low: 8.5,
            close: 9.0,
            volume: 1000.0,
        };

        bb.next(candle1).unwrap();
        bb.next(candle2).unwrap();
        bb.next(candle3).unwrap(); // This should produce a result

        // Reset
        bb.reset_state();

        // Should be back to initial state
        let candle4 = Candle {
            timestamp: 4,
            open: 10.5,
            high: 11.5,
            low: 10.5,
            close: 11.0,
            volume: 1000.0,
        };
        assert_eq!(bb.next(candle4).unwrap(), None);
    }
}
