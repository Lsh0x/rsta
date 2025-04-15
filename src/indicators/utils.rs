//! Utility functions for technical indicators

use crate::indicators::IndicatorError;

/// Validate period parameter
///
/// # Arguments
/// * `period` - The period to validate
/// * `min_period` - The minimum allowed period
///
/// # Returns
/// * `Result<(), IndicatorError>` - Ok if valid, Err otherwise
pub fn validate_period(period: usize, min_period: usize) -> Result<(), IndicatorError> {
    if period < min_period {
        return Err(IndicatorError::InvalidParameter(format!(
            "Period must be greater than or equal to {}",
            min_period
        )));
    }
    Ok(())
}

/// Validate data length against a minimum length
///
/// # Arguments
/// * `data` - The data slice to validate
/// * `min_length` - The minimum allowed length
///
/// # Returns
/// * `Result<(), IndicatorError>` - Ok if valid, Err otherwise
pub fn validate_data_length<T>(data: &[T], min_length: usize) -> Result<(), IndicatorError> {
    if data.len() < min_length {
        return Err(IndicatorError::InsufficientData(format!(
            "Input data length must be at least {}",
            min_length
        )));
    }
    Ok(())
}

/// Calculate Simple Moving Average (SMA)
///
/// # Arguments
/// * `data` - Data values
/// * `period` - Period for SMA calculation
///
/// # Returns
/// * `Result<Vec<f64>, IndicatorError>` - Vector of SMA values
pub fn calculate_sma(data: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validate_period(period, 1)?;
    validate_data_length(data, period)?;

    let n = data.len();
    let mut result = Vec::with_capacity(n - period + 1);

    // Calculate first SMA value
    let mut sum = data.iter().take(period).sum::<f64>();
    result.push(sum / period as f64);

    // Calculate the rest using the sliding window
    for i in period..n {
        sum = sum + data[i] - data[i - period];
        result.push(sum / period as f64);
    }

    Ok(result)
}

/// Calculate Exponential Moving Average (EMA)
///
/// # Arguments
/// * `data` - Data values
/// * `period` - Period for EMA calculation
///
/// # Returns
/// * `Result<Vec<f64>, IndicatorError>` - Vector of EMA values
pub fn calculate_ema(data: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validate_period(period, 1)?;
    validate_data_length(data, period)?;

    let n = data.len();
    let mut result = Vec::with_capacity(n - period + 1);

    // Calculate first EMA as SMA
    let first_sma = data.iter().take(period).sum::<f64>() / period as f64;
    result.push(first_sma);

    // EMA multiplier
    let multiplier = 2.0 / (period as f64 + 1.0);

    // Calculate the rest using the EMA formula
    for &value in data.iter().take(n).skip(period) {
        let ema = (value - result.last().unwrap()) * multiplier + result.last().unwrap();
        result.push(ema);
    }

    Ok(result)
}

/// Calculate standard deviation
///
/// # Arguments
/// * `data` - Data values
/// * `mean` - Mean value of the data (if None, will be calculated)
///
/// # Returns
/// * `Result<f64, IndicatorError>` - Standard deviation value
pub fn standard_deviation(data: &[f64], mean: Option<f64>) -> Result<f64, IndicatorError> {
    if data.is_empty() {
        return Err(IndicatorError::InsufficientData(
            "Cannot calculate standard deviation of empty dataset".to_string(),
        ));
    }

    let mean = mean.unwrap_or_else(|| data.iter().sum::<f64>() / data.len() as f64);

    let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;

    Ok(variance.sqrt())
}

/// Calculate the rate of change (ROC)
///
/// # Arguments
/// * `data` - Data values
/// * `period` - Period for ROC calculation
///
/// # Returns
/// * `Result<Vec<f64>, IndicatorError>` - Vector of ROC values
pub fn rate_of_change(data: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    validate_period(period, 1)?;
    validate_data_length(data, period + 1)?;

    let n = data.len();
    let mut result = Vec::with_capacity(n - period);

    // Creating iterator pairs of (current value, past value) separated by period
    for i in period..n {
        let current = data[i];
        let past = data[i - period];
        let roc = (current - past) / past * 100.0;
        result.push(roc);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_period() {
        assert!(validate_period(10, 5).is_ok());
        assert!(validate_period(5, 5).is_ok());
        assert!(validate_period(1, 1).is_ok());

        let result = validate_period(4, 5);
        assert!(result.is_err());
        if let Err(IndicatorError::InvalidParameter(msg)) = result {
            assert!(msg.contains("5"));
        } else {
            panic!("Expected InvalidParameter error");
        }
    }

    #[test]
    fn test_validate_data_length() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(validate_data_length(&data, 5).is_ok());
        assert!(validate_data_length(&data, 3).is_ok());

        let result = validate_data_length(&data, 6);
        assert!(result.is_err());
        if let Err(IndicatorError::InsufficientData(msg)) = result {
            assert!(msg.contains("6"));
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_calculate_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        // SMA with period 3
        let sma_result = calculate_sma(&data, 3).unwrap();
        assert_eq!(sma_result.len(), 8);
        assert_eq!(sma_result[0], (1.0 + 2.0 + 3.0) / 3.0);
        assert_eq!(sma_result[1], (2.0 + 3.0 + 4.0) / 3.0);
        assert_eq!(sma_result[7], (8.0 + 9.0 + 10.0) / 3.0);

        // SMA with period 5
        let sma_result = calculate_sma(&data, 5).unwrap();
        assert_eq!(sma_result.len(), 6);
        assert_eq!(sma_result[0], (1.0 + 2.0 + 3.0 + 4.0 + 5.0) / 5.0);
        assert_eq!(sma_result[5], (6.0 + 7.0 + 8.0 + 9.0 + 10.0) / 5.0);

        // Error case - period too large
        let result = calculate_sma(&data, 11);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_ema() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        // EMA with period 3
        let ema_result = calculate_ema(&data, 3).unwrap();
        assert_eq!(ema_result.len(), 8);

        // First value should be SMA
        assert_eq!(ema_result[0], 2.0);

        // Manual calculation of second value
        // multiplier = 2 / (3 + 1) = 0.5
        // EMA = (4 - 2) * 0.5 + 2 = 3.0
        assert_eq!(ema_result[1], 3.0);

        // Error case - period too large
        let result = calculate_ema(&data, 11);
        assert!(result.is_err());
    }

    #[test]
    fn test_standard_deviation() {
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        // Mean = 6.0
        // Variance = ((2-6)² + (4-6)² + (6-6)² + (8-6)² + (10-6)²) / 5 = (16 + 4 + 0 + 4 + 16) / 5 = 40 / 5 = 8
        // Standard deviation = √8 ≈ 2.828
        let std_dev = standard_deviation(&data, Some(6.0)).unwrap();
        assert!((std_dev - 2.828427).abs() < 0.000001);

        // Auto-calculate mean
        let std_dev = standard_deviation(&data, None).unwrap();
        assert!((std_dev - 2.828427).abs() < 0.000001);

        // Error case - empty data
        let result = standard_deviation(&[] as &[f64], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_of_change() {
        let data = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0];

        // ROC with period 1
        let roc_result = rate_of_change(&data, 1).unwrap();
        assert_eq!(roc_result.len(), 5);
        // (11 - 10) / 10 * 100 = 10%
        assert_eq!(roc_result[0], 10.0);
        // (15 - 14) / 14 * 100 = 7.142857%
        assert!((roc_result[4] - 7.142857).abs() < 0.000001);

        // ROC with period 3
        let roc_result = rate_of_change(&data, 3).unwrap();
        assert_eq!(roc_result.len(), 3);
        // (13 - 10) / 10 * 100 = 30%
        assert_eq!(roc_result[0], 30.0);

        // Error case - period too large
        let result = rate_of_change(&data, 6);
        assert!(result.is_err());
    }
}
