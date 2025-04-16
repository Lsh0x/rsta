use crate::Indicator;
use crate::IndicatorError;
use crate::indicators::utils::calculate_ema;
use crate::indicators::validate_period;

/// Exponential Moving Average (EMA) indicator
///
/// # Example
///
/// ```
/// use rsta::indicators::trend::EMA;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period EMA
/// let mut ema = EMA::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate EMA values
/// let ema_values = ema.calculate(&prices).unwrap();
/// ```
#[derive(Debug)]
pub struct EMA {
    period: usize,
    alpha: f64,
    current_ema: Option<f64>,
}

impl EMA {
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
}

impl Indicator<f64, f64> for EMA {
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
        self.current_ema = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_new() {
        // Valid period should work
        assert!(EMA::new(14).is_ok());

        // Invalid period should fail
        assert!(EMA::new(0).is_err());
    }

    #[test]
    fn test_ema_calculation() {
        let mut ema = EMA::new(3).unwrap();
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
        let mut ema = EMA::new(3).unwrap();
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
        let mut ema = EMA::new(3).unwrap();

        // Add some values
        ema.next(2.0).unwrap();
        ema.next(4.0).unwrap();

        // Reset
        ema.reset();

        // Should be back to initial state, next value becomes seed
        assert_eq!(ema.next(6.0).unwrap(), Some(6.0));
    }
}