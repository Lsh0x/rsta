use crate::indicators::validate_data_length;
use crate::Candle;
use crate::Indicator;
use crate::IndicatorError;

/// Accumulation/Distribution Line (A/D Line) indicator
///
/// The Accumulation/Distribution Line is a volume-based indicator designed to measure
/// the cumulative flow of money into and out of a security. It assesses whether a
/// security is being accumulated (bought) or distributed (sold).
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::Adl;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create an A/D Line indicator
/// let mut adl = Adl::new();
///
/// // Create price data with OHLCV values
/// let candles = vec![
///     Candle { timestamp: 0, open: 10.0, high: 12.0, low: 9.0, close: 11.0, volume: 1000.0 },
///     // ... more candles ...
/// ];
///
/// // Calculate A/D Line values
/// let adl_values = adl.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct Adl {
    current_ad: f64,
}

impl Adl {
    /// Create a new Adl indicator
    pub fn new() -> Self {
        Self { current_ad: 0.0 }
    }

    /// Calculate Money Flow Multiplier (MFM) for a candle
    ///
    /// # Arguments
    /// * `candle` - The candle data to calculate MFM from
    ///
    /// # Returns
    /// * `f64` - The Money Flow Multiplier value
    fn money_flow_multiplier(candle: &Candle) -> Result<f64, IndicatorError> {
        let high = candle.high;
        let low = candle.low;
        let close = candle.close;

        let range = high - low;

        if range == 0.0 {
            return Err(IndicatorError::CalculationError(
                "Division by zero: high and low prices are equal".to_string(),
            ));
        }

        // Calculate Money Flow Multiplier
        // MFM = ((Close - Low) - (High - Close)) / (High - Low)
        // Simplified to: MFM = (2 * Close - High - Low) / (High - Low)
        Ok((2.0 * close - high - low) / range)
    }

    /// Calculate Money Flow Volume (MFV) for a candle
    ///
    /// # Arguments
    /// * `candle` - The candle data to calculate MFV from
    ///
    /// # Returns
    /// * `f64` - The Money Flow Volume value
    fn money_flow_volume(candle: &Candle) -> Result<f64, IndicatorError> {
        let mfm = Self::money_flow_multiplier(candle)?;
        let volume = candle.volume;

        // Money Flow Volume = Money Flow Multiplier * Volume
        Ok(mfm * volume)
    }
}

impl Default for Adl {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator<Candle, f64> for Adl {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n);

        // Reset state
        self.reset();

        // Calculate AD Line
        let mut ad_line = 0.0;

        for candle in data {
            let money_flow_volume = Self::money_flow_volume(candle)?;
            ad_line += money_flow_volume;
            result.push(ad_line);
        }

        self.current_ad = ad_line;

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        let money_flow_volume = Self::money_flow_volume(&value)?;
        self.current_ad += money_flow_volume;

        Ok(Some(self.current_ad))
    }

    fn reset(&mut self) {
        self.current_ad = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // Adl Tests
    #[test]
    fn test_adl_new() {
        // Adl has no parameters to validate
        let adl = Adl::new();
        // Verify fields are accessible
        assert_eq!(adl.current_ad, 0.0);
    }

    #[test]
    fn test_adl_calculation() {
        let mut adl = Adl::new();

        // Create candles with specific patterns for testing
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 14.0,
                low: 10.0,
                close: 11.0,
                volume: 800.0,
            },
        ];

        let result = adl.calculate(&candles).unwrap();

        // We get one ADL value for each candle
        assert_eq!(result.len(), 3);

        // First value: Money Flow Multiplier * Volume
        // MFM = ((11 - 8) - (12 - 11)) / (12 - 8) = (3 - 1) / 4 = 0.5
        // ADL = 0.5 * 1000 = 500
        assert!((result[0] - 500.0).abs() < 0.01);

        // Second value: Previous ADL + (MFM * Volume)
        // MFM = ((12 - 9) - (13 - 12)) / (13 - 9) = (3 - 1) / 4 = 0.5
        // ADL = 500 + (0.5 * 1200) = 500 + 600 = 1100
        assert!((result[1] - 1100.0).abs() < 0.01);
    }

    #[test]
    fn test_adl_next() {
        let mut adl = Adl::new();

        // First candle
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 8.0,
            close: 11.0,
            volume: 1000.0,
        };
        let result = adl.next(candle1).unwrap();
        assert!(result.is_some());

        // MFM = ((11 - 8) - (12 - 11)) / (12 - 8) = (3 - 1) / 4 = 0.5
        // ADL = 0.5 * 1000 = 500
        assert!((result.unwrap() - 500.0).abs() < 0.01);

        // Second candle
        let candle2 = Candle {
            timestamp: 2,
            open: 11.0,
            high: 13.0,
            low: 9.0,
            close: 12.0,
            volume: 1200.0,
        };
        let result = adl.next(candle2).unwrap();

        // MFM = ((12 - 9) - (13 - 12)) / (13 - 9) = (3 - 1) / 4 = 0.5
        // ADL = 500 + (0.5 * 1200) = 500 + 600 = 1100
        assert!((result.unwrap() - 1100.0).abs() < 0.01);
    }

    #[test]
    fn test_adl_reset() {
        let mut adl = Adl::new();

        // Add some values
        let candle = Candle {
            timestamp: 1,
            open: 10.0,
            high: 12.0,
            low: 8.0,
            close: 11.0,
            volume: 1000.0,
        };
        adl.next(candle).unwrap();

        // Reset
        adl.reset();

        // ADL should be reset to 0
        assert_eq!(adl.current_ad, 0.0);

        // After reset, next candle should be treated as first
        let candle2 = Candle {
            timestamp: 2,
            open: 11.0,
            high: 13.0,
            low: 9.0,
            close: 12.0,
            volume: 1200.0,
        };
        let result = adl.next(candle2).unwrap();

        // MFM = ((12 - 9) - (13 - 12)) / (13 - 9) = (3 - 1) / 4 = 0.5
        // ADL = 0.5 * 1200 = 600
        assert!((result.unwrap() - 600.0).abs() < 0.01);
    }

    #[test]
    fn test_adl_error_handling() {
        let mut adl = Adl::new();

        // Create a candle with equal high and low prices
        let candle = Candle {
            timestamp: 1,
            open: 10.0,
            high: 10.0,
            low: 10.0,
            close: 10.0,
            volume: 1000.0,
        };

        // Expect an error due to division by zero
        let result = adl.next(candle);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            IndicatorError::CalculationError(
                "Division by zero: high and low prices are equal".to_string()
            )
        );
    }

    #[test]
    fn test_adl_money_flow_multiplier_zero_range() {
        // Test with high-low range of zero (should error)
        let candle = Candle {
            timestamp: 1,
            open: 10.0,
            high: 10.0, // Same as low
            low: 10.0,  // Same as high
            close: 10.0,
            volume: 1000.0,
        };

        // Directly test the money_flow_multiplier function
        let result = Adl::money_flow_multiplier(&candle);
        assert!(result.is_err());

        // Verify it's the correct error type and message
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(
                msg.contains("division by zero") || msg.contains("high and low prices are equal")
            );
        } else {
            panic!("Expected CalculationError for zero range");
        }

        // Test error propagation in batch calculation
        let mut adl = Adl::new();
        let result = adl.calculate(&[candle]);
        assert!(result.is_err());

        // Test error propagation in streaming calculation
        let mut adl = Adl::new();
        let result = adl.next(candle);
        assert!(result.is_err());
    }
}
