use crate::indicators::utils::{calculate_sma, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Simple Moving Average (SMA) indicator
///
/// # Example with float values
///
/// ```
/// use rsta::indicators::trend::Sma;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period SMA
/// let mut sma = Sma::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate SMA values
/// let sma_values = sma.calculate(&prices).unwrap();
/// assert_eq!(sma_values.len(), 6);
/// assert_eq!(sma_values[0], 12.0);
/// ```
/// 
/// # Example with Candle data
/// 
/// ```
/// use rsta::indicators::trend::Sma;
/// use rsta::indicators::{Indicator, Candle};
/// 
/// // Create a 5-period SMA
/// let mut sma = Sma::new(5).unwrap();
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
/// // Calculate SMA values based on close prices
/// let sma_values = sma.calculate(&candles).unwrap();
/// assert_eq!(sma_values.len(), 6);
/// assert_eq!(sma_values[0], 12.0);
/// ```
#[derive(Debug)]
pub struct Sma {
    period: usize,
    buffer: VecDeque<f64>,
    sum: f64,
}

impl Sma {
    /// Create a new SMA indicator
    ///
    /// # Arguments
    /// * `period` - The period for SMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new SMA or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }
    
    /// Reset the SMA indicator state
    pub fn reset_state(&mut self) {
        self.buffer.clear();
        self.sum = 0.0;
    }
}

// Implementation for raw price values
impl Indicator<f64, f64> for Sma {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        calculate_sma(data, self.period)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.buffer.push_back(value);
        self.sum += value;

        if self.buffer.len() > self.period {
            if let Some(removed) = self.buffer.pop_front() {
                self.sum -= removed;
            }
        }

        if self.buffer.len() < self.period {
            return Ok(None);
        }

        Ok(Some(self.sum / self.period as f64))
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

// Implementation for candle data
impl Indicator<Candle, f64> for Sma {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        // Extract close prices from candles
        let close_prices: Vec<f64> = data.iter().map(|candle| candle.close).collect();
        calculate_sma(&close_prices, self.period)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        let close_price = candle.close;
        
        self.buffer.push_back(close_price);
        self.sum += close_price;

        if self.buffer.len() > self.period {
            if let Some(removed) = self.buffer.pop_front() {
                self.sum -= removed;
            }
        }

        if self.buffer.len() < self.period {
            return Ok(None);
        }

        Ok(Some(self.sum / self.period as f64))
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

#[cfg(test)]
mod tests {
    use super::Sma;
    use crate::indicators::{Candle, Indicator};
    
    #[test]
    fn test_sma_new() {
        // Valid period should work
        assert!(Sma::new(14).is_ok());

        // Invalid period should fail
        assert!(Sma::new(0).is_err());
    }

    // Tests for raw price values
    #[test]
    fn test_sma_calculation() {
        let mut sma = Sma::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = sma.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 4.0); // (2+4+6)/3
        assert_eq!(result[1], 6.0); // (4+6+8)/3
        assert_eq!(result[2], 8.0); // (6+8+10)/3
    }

    #[test]
    fn test_sma_next() {
        let mut sma = Sma::new(3).unwrap();

        // Initial values - not enough data yet
        assert_eq!(sma.next(2.0).unwrap(), None);
        assert_eq!(sma.next(4.0).unwrap(), None);

        // Third value - now we have an SMA
        assert_eq!(sma.next(6.0).unwrap(), Some(4.0));

        // More values - sliding window
        assert_eq!(sma.next(8.0).unwrap(), Some(6.0));
        assert_eq!(sma.next(10.0).unwrap(), Some(8.0));
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = Sma::new(3).unwrap();

        // Add some values
        sma.next(2.0).unwrap();
        sma.next(4.0).unwrap();
        sma.next(6.0).unwrap();

        // Reset
        sma.reset_state();

        // Should be back to initial state
        assert_eq!(sma.next(8.0).unwrap(), None);
    }
    
    // Tests for candle data
    #[test]
    fn test_sma_calculation_with_candles() {
        let mut sma = Sma::new(3).unwrap();
        
        // Create candles with specific close prices
        let candles = vec![
            Candle { timestamp: 1, open: 2.0, high: 2.5, low: 1.5, close: 2.0, volume: 1000.0 },
            Candle { timestamp: 2, open: 4.0, high: 4.5, low: 3.5, close: 4.0, volume: 1000.0 },
            Candle { timestamp: 3, open: 6.0, high: 6.5, low: 5.5, close: 6.0, volume: 1000.0 },
            Candle { timestamp: 4, open: 8.0, high: 8.5, low: 7.5, close: 8.0, volume: 1000.0 },
            Candle { timestamp: 5, open: 10.0, high: 10.5, low: 9.5, close: 10.0, volume: 1000.0 },
        ];

        let result = sma.calculate(&candles).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 4.0); // (2+4+6)/3
        assert_eq!(result[1], 6.0); // (4+6+8)/3
        assert_eq!(result[2], 8.0); // (6+8+10)/3
    }

    #[test]
    fn test_sma_next_with_candles() {
        let mut sma = Sma::new(3).unwrap();

        // Initial values - not enough data yet
        let candle1 = Candle { timestamp: 1, open: 2.0, high: 2.5, low: 1.5, close: 2.0, volume: 1000.0 };
        let candle2 = Candle { timestamp: 2, open: 4.0, high: 4.5, low: 3.5, close: 4.0, volume: 1000.0 };
        
        assert_eq!(sma.next(candle1).unwrap(), None);
        assert_eq!(sma.next(candle2).unwrap(), None);

        // Third value - now we have an SMA
        let candle3 = Candle { timestamp: 3, open: 6.0, high: 6.5, low: 5.5, close: 6.0, volume: 1000.0 };
        assert_eq!(sma.next(candle3).unwrap(), Some(4.0));

        // More values - sliding window
        let candle4 = Candle { timestamp: 4, open: 8.0, high: 8.5, low: 7.5, close: 8.0, volume: 1000.0 };
        let candle5 = Candle { timestamp: 5, open: 10.0, high: 10.5, low: 9.5, close: 10.0, volume: 1000.0 };
        
        assert_eq!(sma.next(candle4).unwrap(), Some(6.0));
        assert_eq!(sma.next(candle5).unwrap(), Some(8.0));
    }

    #[test]
    fn test_sma_reset_with_candles() {
        let mut sma = Sma::new(3).unwrap();

        // Add some values
        let candle1 = Candle { timestamp: 1, open: 2.0, high: 2.5, low: 1.5, close: 2.0, volume: 1000.0 };
        let candle2 = Candle { timestamp: 2, open: 4.0, high: 4.5, low: 3.5, close: 4.0, volume: 1000.0 };
        let candle3 = Candle { timestamp: 3, open: 6.0, high: 6.5, low: 5.5, close: 6.0, volume: 1000.0 };
        
        sma.next(candle1).unwrap();
        sma.next(candle2).unwrap();
        sma.next(candle3).unwrap();

        // Reset
        sma.reset_state();

        // Should be back to initial state
        let candle4 = Candle { timestamp: 4, open: 8.0, high: 8.5, low: 7.5, close: 8.0, volume: 1000.0 };
        assert_eq!(sma.next(candle4).unwrap(), None);
    }
    
    #[test]
    fn test_sma_implementations_produce_same_results() {
        let mut sma_f64 = Sma::new(3).unwrap();
        let mut sma_candle = Sma::new(3).unwrap();
        
        // Raw price data
        let prices = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        // Equivalent candle data
        let candles = vec![
            Candle { timestamp: 1, open: 2.0, high: 2.5, low: 1.5, close: 2.0, volume: 1000.0 },
            Candle { timestamp: 2, open: 4.0, high: 4.5, low: 3.5, close: 4.0, volume: 1000.0 },
            Candle { timestamp: 3, open: 6.0, high: 6.5, low: 5.5, close: 6.0, volume: 1000.0 },
            Candle { timestamp: 4, open: 8.0, high: 8.5, low: 7.5, close: 8.0, volume: 1000.0 },
            Candle { timestamp: 5, open: 10.0, high: 10.5, low: 9.5, close: 10.0, volume: 1000.0 },
        ];
        
        // Calculate using both implementations
        let result_f64 = sma_f64.calculate(&prices).unwrap();
        let result_candle = sma_candle.calculate(&candles).unwrap();
        
        // Results should be identical
        assert_eq!(result_f64.len(), result_candle.len());
        for (val_f64, val_candle) in result_f64.iter().zip(result_candle.iter()) {
            assert!((val_f64 - val_candle).abs() < 0.000001);
        }
    }
    
    #[test]
    fn test_sma_next_implementations_produce_same_results() {
        let mut sma_f64 = Sma::new(3).unwrap();
        let mut sma_candle = Sma::new(3).unwrap();
        
        // Test first value
        assert_eq!(
            sma_f64.next(2.0).unwrap(),
            sma_candle.next(Candle { timestamp: 1, open: 2.0, high: 2.5, low: 1.5, close: 2.0, volume: 1000.0 }).unwrap()
        );
        
        // Test second value
        assert_eq!(
            sma_f64.next(4.0).unwrap(),
            sma_candle.next(Candle { timestamp: 2, open: 4.0, high: 4.5, low: 3.5, close: 4.0, volume: 1000.0 }).unwrap()
        );
        
        // Test third value
        assert_eq!(
            sma_f64.next(6.0).unwrap(),
            sma_candle.next(Candle { timestamp: 3, open: 6.0, high: 6.5, low: 5.5, close: 6.0, volume: 1000.0 }).unwrap()
        );
    }
}