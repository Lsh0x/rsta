//! Error types for technical indicators
//!
//! This module defines the common error types used by all indicator implementations
//! in the library.

use thiserror::Error;

/// Common error types for technical indicators
///
/// This enum represents the standard error types that can occur when working with
/// technical indicators. All indicator methods that can fail return a `Result`
/// with this error type.
///
/// # Examples
///
/// ```rust,no_run
/// use rsta::indicators::trend::SimpleMovingAverage;
/// use rsta::indicators::IndicatorError;
/// use rsta::indicators::Indicator;
///
/// // Handle a parameter validation error
/// match SimpleMovingAverage::new(0) {
///     Err(IndicatorError::InvalidParameter(msg)) => println!("Invalid parameter: {}", msg),
///     _ => println!("Unexpected result"),
/// }
/// ```
///
/// ```rust
/// use rsta::indicators::trend::SimpleMovingAverage;
/// use rsta::indicators::IndicatorError;
/// use rsta::indicators::Indicator;
///
/// // Handle an insufficient data error
/// let mut sma = SimpleMovingAverage::new(14).unwrap();
/// let prices = vec![1.0, 2.0]; // Not enough data
/// match sma.calculate(&prices) {
///     Err(IndicatorError::InsufficientData(msg)) => println!("Not enough data: {}", msg),
///     _ => println!("Unexpected result"),
/// }
/// ```
#[derive(Error, Debug)]
pub enum IndicatorError {
    /// Error for invalid parameters (e.g., negative period, invalid multiplier)
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Error for insufficient data to perform calculations
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    /// Error during calculation (e.g., division by zero)
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Debug, Display};

    #[test]
    fn test_invalid_parameter_error() {
        // Create the error
        let error = IndicatorError::InvalidParameter("Period must be greater than 0".to_string());

        // Test Debug trait
        assert!(format!("{:?}", error).contains("InvalidParameter"));
        assert!(format!("{:?}", error).contains("Period must be greater than 0"));

        // Test Display trait (error message)
        assert_eq!(
            format!("{}", error),
            "Invalid parameter: Period must be greater than 0"
        );
    }

    #[test]
    fn test_insufficient_data_error() {
        // Create the error
        let error = IndicatorError::InsufficientData("Need at least 14 data points".to_string());

        // Test Debug trait
        assert!(format!("{:?}", error).contains("InsufficientData"));
        assert!(format!("{:?}", error).contains("Need at least 14 data points"));

        // Test Display trait (error message)
        assert_eq!(
            format!("{}", error),
            "Insufficient data: Need at least 14 data points"
        );
    }

    #[test]
    fn test_calculation_error() {
        // Create the error
        let error = IndicatorError::CalculationError("Division by zero".to_string());

        // Test Debug trait
        assert!(format!("{:?}", error).contains("CalculationError"));
        assert!(format!("{:?}", error).contains("Division by zero"));

        // Test Display trait (error message)
        assert_eq!(format!("{}", error), "Calculation error: Division by zero");
    }

    #[test]
    fn test_error_conversion() {
        // Test that errors can be used with the ? operator
        fn returns_indicator_error() -> Result<(), IndicatorError> {
            Err(IndicatorError::InvalidParameter("Test error".to_string()))
        }

        fn propagates_error() -> Result<(), IndicatorError> {
            returns_indicator_error()?;
            Ok(())
        }

        let result = propagates_error();
        assert!(result.is_err());
        if let Err(error) = result {
            match error {
                IndicatorError::InvalidParameter(msg) => {
                    assert_eq!(msg, "Test error");
                }
                _ => panic!("Wrong error type"),
            }
        }
    }

    // Test that IndicatorError implements the required traits
    fn assert_traits<T: Debug + Display + std::error::Error>() {}

    #[test]
    fn test_error_traits() {
        // This will fail to compile if IndicatorError doesn't implement
        // the required traits
        assert_traits::<IndicatorError>();
    }
}
