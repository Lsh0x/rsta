
use crate::indicators::utils::{validate_data_length, validate_period, standard_deviation};
use crate::indicators::IndicatorError;
use crate::indicators::traits::Indicator;
use std::collections::VecDeque;

/// Standard Deviation (STD) indicator
///
/// Measures the dispersion of a dataset relative to its mean over a specific period.
/// Standard deviation is commonly used to measure market volatility. Higher values indicate
/// greater price volatility, while lower values suggest more stable prices.
///
/// The STD indicator can be particularly useful for:
/// - Identifying periods of high vs low volatility
/// - Setting dynamic stop-loss levels
/// - Determining position sizing based on market volatility
///
/// # Formula
///
/// The standard deviation is calculated as:
/// ```text
/// STD = √(Σ(x - μ)² / n)
/// 
/// where:
/// x = each value in the dataset
/// μ = mean of the dataset
/// n = number of values
/// ```
///
/// # Example
///
/// ```
/// use rsta::indicators::volatility::STD;
/// use rsta::indicators::Indicator;
///
/// // Create a 20-period Standard Deviation indicator
/// let mut std_dev = STD::new(20).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///                   20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///                   30.0, 31.0, 32.0, 33.0, 34.0];
///
/// // Calculate Standard Deviation values
/// let std_values = std_dev.calculate(&prices).unwrap();
/// ```
#[derive(Debug)]
pub struct STD {
    period: usize,
    values: VecDeque<f64>,
}

impl STD {
    /// Create a new STD indicator
    ///
    /// # Arguments
    /// * `period` - The period for Standard Deviation calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new STD instance or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            values: VecDeque::with_capacity(period),
        })
    }

}

impl Indicator<f64, f64> for STD {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset();

        // Calculate standard deviation for each period
        for i in 0..=(n - self.period) {
            let period_data = &data[i..(i + self.period)];
            let std_dev = standard_deviation(period_data, None)?;
            result.push(std_dev);
        }

        // Update state with the last period
        for &value in data.iter().take(n).skip(n - self.period) {
            self.values.push_back(value);
        }

        Ok(result)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.values.push_back(value);

        if self.values.len() > self.period {
            self.values.pop_front();
        }

        if self.values.len() == self.period {
            standard_deviation(&self.values.make_contiguous(), None).map(Some)
        } else {
            Ok(None)
        }
    }
    fn reset(&mut self) {
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    const FLOAT_EPSILON: f64 = 1e-10;

    // Helper function to compare floating point values
#[test]
fn test_std_calculation_basic() {
    let mut std = STD::new(3).unwrap();
    let data = vec![2.0, 4.0, 6.0];
    
    let result = std.calculate(&data).unwrap();
    assert_eq!(result.len(), 1);
    
    // Mean = (2 + 4 + 6) / 3 = 4
    // Variance = ((2-4)² + (4-4)² + (6-4)²) / 3 = (4 + 0 + 4) / 3 = 8/3
    // STD = √(8/3) ≈ 1.632993161855452
    assert_float_eq(result[0], 1.632993161855452);
}

#[test]
fn test_std_calculation_multiple_periods() {
    let mut std = STD::new(2).unwrap();
    let data = vec![1.0, 2.0, 3.0];
    
    let result = std.calculate(&data).unwrap();
    assert_eq!(result.len(), 2);
    
    // First window [1.0, 2.0]: Mean = 1.5, Variance = ((1-1.5)² + (2-1.5)²) / 2 = 0.25
    // STD = √0.25 = 0.5
    assert_float_eq(result[0], 0.5);
    
    // Second window [2.0, 3.0]: Mean = 2.5, Variance = ((2-2.5)² + (3-2.5)²) / 2 = 0.25
    // STD = √0.25 = 0.5
    assert_float_eq(result[1], 0.5);
}

    // Helper function to compare floating point values
    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < FLOAT_EPSILON, "{} != {}", a, b);
    }
    #[test]
    fn test_std_with_decimal_values() {
        let mut std = STD::new(4).unwrap();
        let data = vec![1.5, 2.5, 3.5, 4.5];
        
        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 1);
        // Mean = 3.0
        // STD = √(((1.5-3)² + (2.5-3)² + (3.5-3)² + (4.5-3)²)/4) = √(5/4) ≈ 1.118033988749895
        assert_float_eq(result[0], 1.118033988749895);
    }

    #[test]
    fn test_std_edge_cases() {
        // Test period of 1
        let mut std = STD::new(1).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 5);
        // For period=1, all standard deviations should be 0
        for value in result {
            assert_float_eq(value, 0.0);
        }

        // Test with constant values
        let mut std = STD::new(3).unwrap();
        let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        
        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);
        // STD should be 0 for constant values
        for value in result {
            assert_float_eq(value, 0.0);
        }
    }

    #[test]
    fn test_std_next_value() {
        let mut std = STD::new(3).unwrap();

        // First two values should return None
        assert_eq!(std.next(2.0).unwrap(), None);
        assert_eq!(std.next(4.0).unwrap(), None);

        // Third value should give us our first STD
        let result = std.next(6.0).unwrap().unwrap();
        // Mean = 4.0
        // STD ≈ 1.632993161855452
        assert_float_eq(result, 1.632993161855452);

        // Next value should maintain window of 3
        let result = std.next(8.0).unwrap().unwrap();
        // Window now contains [4.0, 6.0, 8.0]
        assert_float_eq(result, 1.632993161855452);
    }

    #[test]
    fn test_std_with_market_pattern() {
        let mut std = STD::new(5).unwrap();
        // Simulated market pattern: trending up with increasing volatility
        let data = vec![
            100.0, 101.0, 101.5, 102.0, 103.0,  // low volatility trend
            105.0, 104.0, 106.0, 103.0, 107.0,  // increasing volatility
        ];
        
        let result = std.calculate(&data).unwrap();
        assert_eq!(result.len(), 6);
        
        // The standard deviation should increase as volatility increases
        assert!(result[0] < result[result.len() - 1]);
    }

    #[test]
    fn test_std_error_handling() {
        let mut std = STD::new(5).unwrap();
        
        // Test with insufficient data
        let data = vec![1.0, 2.0, 3.0];
        assert!(matches!(
            std.calculate(&data),
            Err(IndicatorError::InsufficientData(_))
        ));

        // Test with empty data
        let data: Vec<f64> = vec![];
        assert!(matches!(
            std.calculate(&data),
            Err(IndicatorError::InsufficientData(_))
        ));
        
        // Test valid period initialization
        assert!(STD::new(100).is_ok());
    }
    #[test]
    fn test_std_reset() {
        let mut std = STD::new(3).unwrap();
        
        // Add some values
        std.next(1.0).unwrap();
        std.next(2.0).unwrap();
        std.next(3.0).unwrap();
        
        // Reset the indicator
        std.reset();
        
        // Next value after reset should return None
        assert_eq!(std.next(4.0).unwrap(), None);
    }
}
