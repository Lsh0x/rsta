use crate::indicators::utils::calculate_ema;
use crate::indicators::validate_period;
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Exponential Moving Average (EMA) indicator
///
/// # Example with float values
///
/// ```
/// use rsta::indicators::trend::Ema;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period EMA
/// let mut ema = Ema::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate EMA values
/// let ema_values = ema.calculate(&prices).unwrap();
/// ```
///
/// # Example with Candle data
///
/// ```
/// use rsta::indicators::trend::Ema;
/// use rsta::indicators::{Indicator, Candle};
///
/// // Create a 5-period EMA
/// let mut ema = Ema::new(5).unwrap();
///
/// // Create candle data
/// let mut candles = Vec::new();
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Convert prices to candles
/// for (i, &price) in prices.iter().enumerate() {
///     candles.push(Candle {
///         timestamp: i as u64,
///         open: price,
///         high: price,
///         low: price,
///         close: price,
///         volume: 1000.0,
///     });
/// }
///
/// // Calculate EMA values based on close prices
/// let ema_values = ema.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct Ema {
    period: usize,
    alpha: f64,
    current_ema: Option<f64>,
}

impl Ema {
    /// Create a new EMA indicator
    ///
    /// # Arguments
    /// * `period` - The period for EMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new EMA or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            alpha: 2.0 / (period as f64 + 1.0),
            current_ema: None,
        })
    }

    /// Set the initial EMA value
    ///
    /// # Arguments
    /// * `value` - Initial EMA value
    ///
    /// # Returns
    /// * `&mut Self` - Reference to self for method chaining
    pub fn with_initial_value(&mut self, value: f64) -> &mut Self {
        self.current_ema = Some(value);
        self
    }

    /// Reset the EMA indicator state
    pub fn reset_state(&mut self) {
        self.current_ema = None;
    }
}

// Implementation for raw price values
impl Indicator<f64, f64> for Ema {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        calculate_ema(data, self.period)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        if let Some(current) = self.current_ema {
            // Apply EMA formula: EMA_today = (Price_today * alpha) + (EMA_yesterday * (1 - alpha))
            let new_ema = (value * self.alpha) + (current * (1.0 - self.alpha));
            self.current_ema = Some(new_ema);
            Ok(Some(new_ema))
        } else {
            // First value becomes the initial EMA
            self.current_ema = Some(value);
            Ok(Some(value))
        }
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

// Implementation for candle data
impl Indicator<Candle, f64> for Ema {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        // Extract close prices from candles
        let close_prices: Vec<f64> = data.iter().map(|candle| candle.close).collect();
        calculate_ema(&close_prices, self.period)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        let close_price = candle.close;

        if let Some(current) = self.current_ema {
            // Apply EMA formula: EMA_today = (Price_today * alpha) + (EMA_yesterday * (1 - alpha))
            let new_ema = (close_price * self.alpha) + (current * (1.0 - self.alpha));
            self.current_ema = Some(new_ema);
            Ok(Some(new_ema))
        } else {
            // First value becomes the initial EMA
            self.current_ema = Some(close_price);
            Ok(Some(close_price))
        }
    }

    fn reset(&mut self) {
        self.current_ema = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_new() {
        // Valid period should work
        assert!(Ema::new(14).is_ok());

        // Invalid period should fail
        assert!(Ema::new(0).is_err());
    }

    // Tests for raw price values
    #[test]
    fn test_ema_calculation() {
        let mut ema = Ema::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = ema.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);

        // First EMA is SMA of first 3 values
        assert_eq!(result[0], 4.0); // (2+4+6)/3

        // Rest follow EMA formula with alpha = 2/(3+1) = 0.5
        let alpha = 0.5;
        let expected1 = 8.0 * alpha + 4.0 * (1.0 - alpha); // 6.0
        let expected2 = 10.0 * alpha + expected1 * (1.0 - alpha); // 8.0

        assert_eq!(result[1], expected1);
        assert_eq!(result[2], expected2);
    }

    #[test]
    fn test_ema_next() {
        let mut ema = Ema::new(3).unwrap();
        let alpha = 0.5; // alpha = 2/(3+1)

        // First value becomes the seed
        assert_eq!(ema.next(2.0).unwrap(), Some(2.0));

        // Next values follow EMA formula
        let expected1 = 4.0 * alpha + 2.0 * (1.0 - alpha); // 3.0
        assert_eq!(ema.next(4.0).unwrap(), Some(expected1));

        let expected2 = 6.0 * alpha + expected1 * (1.0 - alpha); // 4.5
        assert_eq!(ema.next(6.0).unwrap(), Some(expected2));
    }

    #[test]
    fn test_ema_reset() {
        let mut ema = Ema::new(3).unwrap();

        // Add some values
        ema.next(2.0).unwrap();
        ema.next(4.0).unwrap();

        // Reset
        ema.reset_state();

        // Should be back to initial state, next value becomes seed
        assert_eq!(ema.next(6.0).unwrap(), Some(6.0));
    }

    // Tests for candle data
    #[test]
    fn test_ema_calculation_with_candles() {
        let mut ema = Ema::new(3).unwrap();

        // Create candles with specific close prices
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 2.0,
                high: 2.5,
                low: 1.5,
                close: 2.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 4.0,
                high: 4.5,
                low: 3.5,
                close: 4.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 6.0,
                high: 6.5,
                low: 5.5,
                close: 6.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 4,
                open: 8.0,
                high: 8.5,
                low: 7.5,
                close: 8.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 10.5,
                low: 9.5,
                close: 10.0,
                volume: 1000.0,
            },
        ];

        let result = ema.calculate(&candles).unwrap();
        assert_eq!(result.len(), 3);

        // First EMA is SMA of first 3 close prices
        assert_eq!(result[0], 4.0); // (2+4+6)/3

        // Rest follow EMA formula with alpha = 2/(3+1) = 0.5
        let alpha = 0.5;
        let expected1 = 8.0 * alpha + 4.0 * (1.0 - alpha); // 6.0
        let expected2 = 10.0 * alpha + expected1 * (1.0 - alpha); // 8.0

        assert_eq!(result[1], expected1);
        assert_eq!(result[2], expected2);
    }

    #[test]
    fn test_ema_next_with_candles() {
        let mut ema = Ema::new(3).unwrap();
        let alpha = 0.5; // alpha = 2/(3+1)

        // First value becomes the seed
        let candle1 = Candle {
            timestamp: 1,
            open: 2.0,
            high: 2.5,
            low: 1.5,
            close: 2.0,
            volume: 1000.0,
        };
        assert_eq!(ema.next(candle1).unwrap(), Some(2.0));

        // Next values follow EMA formula
        let candle2 = Candle {
            timestamp: 2,
            open: 4.0,
            high: 4.5,
            low: 3.5,
            close: 4.0,
            volume: 1000.0,
        };
        let expected1 = 4.0 * alpha + 2.0 * (1.0 - alpha); // 3.0
        assert_eq!(ema.next(candle2).unwrap(), Some(expected1));

        let candle3 = Candle {
            timestamp: 3,
            open: 6.0,
            high: 6.5,
            low: 5.5,
            close: 6.0,
            volume: 1000.0,
        };
        let expected2 = 6.0 * alpha + expected1 * (1.0 - alpha); // 4.5
        assert_eq!(ema.next(candle3).unwrap(), Some(expected2));
    }

    #[test]
    fn test_ema_reset_with_candles() {
        let mut ema = Ema::new(3).unwrap();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 2.0,
            high: 2.5,
            low: 1.5,
            close: 2.0,
            volume: 1000.0,
        };
        let candle2 = Candle {
            timestamp: 2,
            open: 4.0,
            high: 4.5,
            low: 3.5,
            close: 4.0,
            volume: 1000.0,
        };

        ema.next(candle1).unwrap();
        ema.next(candle2).unwrap();

        // Reset
        ema.reset_state();

        // Should be back to initial state, next value becomes seed
        let candle3 = Candle {
            timestamp: 3,
            open: 6.0,
            high: 6.5,
            low: 5.5,
            close: 6.0,
            volume: 1000.0,
        };
        assert_eq!(ema.next(candle3).unwrap(), Some(6.0));
    }

    #[test]
    fn test_ema_implementations_produce_same_results() {
        let mut ema_f64 = Ema::new(3).unwrap();
        let mut ema_candle = Ema::new(3).unwrap();

        // Raw price data
        let prices = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        // Equivalent candle data
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 2.0,
                high: 2.5,
                low: 1.5,
                close: 2.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 4.0,
                high: 4.5,
                low: 3.5,
                close: 4.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 6.0,
                high: 6.5,
                low: 5.5,
                close: 6.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 4,
                open: 8.0,
                high: 8.5,
                low: 7.5,
                close: 8.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 10.5,
                low: 9.5,
                close: 10.0,
                volume: 1000.0,
            },
        ];

        // Calculate using both implementations
        let result_f64 = ema_f64.calculate(&prices).unwrap();
        let result_candle = ema_candle.calculate(&candles).unwrap();

        // Results should be identical
        assert_eq!(result_f64.len(), result_candle.len());
        for (val_f64, val_candle) in result_f64.iter().zip(result_candle.iter()) {
            assert!((val_f64 - val_candle).abs() < 0.000001);
        }
    }

    #[test]
    fn test_ema_next_implementations_produce_same_results() {
        let mut ema_f64 = Ema::new(3).unwrap();
        let mut ema_candle = Ema::new(3).unwrap();

        // Test first value
        assert_eq!(
            ema_f64.next(2.0).unwrap(),
            ema_candle
                .next(Candle {
                    timestamp: 1,
                    open: 2.0,
                    high: 2.5,
                    low: 1.5,
                    close: 2.0,
                    volume: 1000.0
                })
                .unwrap()
        );

        // Test second value
        assert_eq!(
            ema_f64.next(4.0).unwrap(),
            ema_candle
                .next(Candle {
                    timestamp: 2,
                    open: 4.0,
                    high: 4.5,
                    low: 3.5,
                    close: 4.0,
                    volume: 1000.0
                })
                .unwrap()
        );

        // Test third value
        assert_eq!(
            ema_f64.next(6.0).unwrap(),
            ema_candle
                .next(Candle {
                    timestamp: 3,
                    open: 6.0,
                    high: 6.5,
                    low: 5.5,
                    close: 6.0,
                    volume: 1000.0
                })
                .unwrap()
        );
    }
}
