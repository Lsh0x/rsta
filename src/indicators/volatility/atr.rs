use crate::indicators::traits::Indicator;
use crate::indicators::utils::validate_data_length;
use crate::indicators::utils::validate_period;
use crate::indicators::{Candle, IndicatorError};
use std::collections::VecDeque;

/// Average True Range (Atr) indicator
///
/// Measures market volatility by decomposing the entire range of an asset price for a period.
/// The ATR is particularly useful for:
/// - Measuring market volatility
/// - Position sizing
/// - Setting stop-loss levels
/// - Identifying potential breakout points
///
/// # Formula
///
/// The ATR is calculated using the following steps:
///
/// 1. True Range (TR) is the greatest of:
///    - Current High - Current Low
///    - |Current High - Previous Close|
///    - |Current Low - Previous Close|
///
/// 2. Initial ATR = Simple moving average of TR for the first n periods
///
/// 3. Subsequent ATR values use Wilder's smoothing:
///    ATR = ((Previous ATR * (n-1)) + Current TR) / n
///
/// where n is the period length
///
/// # Example
///
/// ```rust,no_run
/// use rsta::indicators::volatility::Atr;
/// use rsta::indicators::Indicator;
/// use rsta::Candle;
///
/// // Create a 14-period ATR
/// let mut atr = Atr::new(14).unwrap();
///
/// // Example candle data
/// let candles = vec![
///     Candle { timestamp: 0, open: 10.0, high: 12.0, low: 9.0, close: 11.0, volume: 1000.0 },
///     Candle { timestamp: 1, open: 11.0, high: 13.0, low: 10.0, close: 12.0, volume: 1000.0 },
///     // ... more candles ...
/// ];
///
/// // Calculate ATR values
/// let atr_values = atr.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct Atr {
    period: usize,
    prev_close: Option<f64>,
    current_atr: Option<f64>,
    tr_values: VecDeque<f64>,
}

impl Atr {
    /// Create a new ATR indicator
    ///
    /// # Arguments
    /// * `period` - The period for ATR calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new ATR instance or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            prev_close: None,
            current_atr: None,
            tr_values: VecDeque::with_capacity(period),
        })
    }

    /// Calculate True Range for a single candle
    ///
    /// # Arguments
    /// * `candle` - Current price candle
    /// * `prev_close` - Previous candle's closing price (if available)
    ///
    /// # Returns
    /// * `f64` - The True Range value
    fn true_range(candle: &Candle, prev_close: Option<f64>) -> f64 {
        let high_low = candle.high - candle.low;

        match prev_close {
            Some(prev_close) => {
                let high_close = (candle.high - prev_close).abs();
                let low_close = (candle.low - prev_close).abs();
                high_low.max(high_close).max(low_close)
            }
            None => high_low,
        }
    }

    /// Calculate initial ATR value using simple moving average
    ///
    /// # Arguments
    /// * `tr_values` - Slice of True Range values
    ///
    /// # Returns
    /// * `f64` - The initial ATR value
    fn initial_atr(tr_values: &[f64]) -> f64 {
        tr_values.iter().sum::<f64>() / tr_values.len() as f64
    }

    /// Apply Wilder's smoothing to calculate the next ATR value
    ///
    /// # Arguments
    /// * `prev_atr` - Previous ATR value
    /// * `current_tr` - Current True Range value
    ///
    /// # Returns
    /// * `f64` - The smoothed ATR value
    fn smooth_atr(&self, prev_atr: f64, current_tr: f64) -> f64 {
        ((prev_atr * (self.period - 1) as f64) + current_tr) / self.period as f64
    }
}

impl Indicator<Candle, f64> for Atr {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset();

        // Calculate True Range values
        let mut tr_values = Vec::with_capacity(n);
        let mut prev_close = None;

        for candle in data {
            let tr = Self::true_range(candle, prev_close);
            tr_values.push(tr);
            prev_close = Some(candle.close);
        }

        // Calculate initial ATR using simple moving average
        let initial_atr = Self::initial_atr(&tr_values[0..self.period]);
        result.push(initial_atr);
        let mut current_atr = initial_atr;

        // Calculate subsequent ATR values using Wilder's smoothing
        for tr in tr_values.iter().skip(self.period) {
            current_atr = self.smooth_atr(current_atr, *tr);
            result.push(current_atr);
        }

        // Update state with the last values
        self.prev_close = Some(data[n - 1].close);
        self.current_atr = Some(current_atr);
        self.tr_values = tr_values.into_iter().skip(n - self.period).collect();

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        let tr = Self::true_range(&value, self.prev_close);
        self.tr_values.push_back(tr);
        self.prev_close = Some(value.close);

        if self.tr_values.len() > self.period {
            self.tr_values.pop_front();
        }

        if self.tr_values.len() == self.period {
            let atr = match self.current_atr {
                Some(prev_atr) => self.smooth_atr(prev_atr, tr),
                None => Self::initial_atr(self.tr_values.make_contiguous()),
            };
            self.current_atr = Some(atr);
            Ok(Some(atr))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.current_atr = None;
        self.tr_values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_EPSILON: f64 = 1e-10;

    fn assert_float_eq(a: f64, b: f64) {
        assert!(
            (a - b).abs() < FLOAT_EPSILON,
            "Expected {} but got {}",
            b,
            a
        );
    }

    fn create_test_candle(timestamp: u64, open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    #[test]
    fn test_atr_new() {
        assert!(Atr::new(1).is_ok());
        assert!(Atr::new(14).is_ok());
        assert!(Atr::new(100).is_ok());
        assert!(Atr::new(0).is_err());
    }

    #[test]
    fn test_true_range_calculation() {
        // Test case 1: Simple high-low range
        let candle1 = create_test_candle(0, 10.0, 15.0, 8.0, 12.0);
        assert_float_eq(Atr::true_range(&candle1, None), 7.0); // high - low = 15 - 8 = 7

        // Test case 2: Previous close creates larger range
        let candle2 = create_test_candle(1, 11.0, 13.0, 9.0, 10.0);
        assert_float_eq(Atr::true_range(&candle2, Some(12.0)), 4.0); // max(4, 3, 1)

        // Test case 3: Gap down scenario
        let candle3 = create_test_candle(2, 8.0, 9.0, 7.0, 8.0);
        assert_float_eq(Atr::true_range(&candle3, Some(10.0)), 3.0); // max(2, 1, 3)
    }

    #[test]
    fn test_atr_calculation_basic() {
        let mut atr = Atr::new(3).unwrap();
        let candles = vec![
            create_test_candle(0, 10.0, 12.0, 9.0, 11.0),  // TR = 3
            create_test_candle(1, 11.0, 14.0, 10.0, 13.0), // TR = 4
            create_test_candle(2, 13.0, 15.0, 11.0, 14.0), // TR = 4
            create_test_candle(3, 14.0, 16.0, 12.0, 15.0), // TR = 4
        ];

        let result = atr.calculate(&candles).unwrap();
        assert_eq!(result.len(), 2);

        // First ATR is simple average of first 3 TRs: (3 + 4 + 4) / 3 = 3.666...
        assert_float_eq(result[0], 3.666666666666667);

        // Second ATR uses Wilder's smoothing: ((3.666... * 2) + 4) / 3 = 3.777...
        assert_float_eq(result[1], 3.777777777777778);
    }

    #[test]
    fn test_atr_with_gaps() {
        let mut atr = Atr::new(3).unwrap();
        let candles = vec![
            create_test_candle(0, 10.0, 12.0, 9.0, 11.0),  // TR = 3
            create_test_candle(1, 11.0, 14.0, 10.0, 13.0), // TR = 4
            create_test_candle(2, 15.0, 17.0, 14.0, 16.0), // TR = 4 (gap up)
            create_test_candle(3, 12.0, 13.0, 11.0, 12.0), // TR = 5 (gap down)
        ];

        let result = atr.calculate(&candles).unwrap();
        assert_eq!(result.len(), 2);

        // Values should reflect the increased volatility from the gaps
        assert!(result[1] > result[0]);
    }

    #[test]
    fn test_atr_next_value() {
        let mut atr = Atr::new(3).unwrap();

        // First two values should return None
        assert_eq!(
            atr.next(create_test_candle(0, 10.0, 12.0, 9.0, 11.0))
                .unwrap(),
            None
        );
        assert_eq!(
            atr.next(create_test_candle(1, 11.0, 14.0, 10.0, 13.0))
                .unwrap(),
            None
        );

        // Third value should give us our first ATR
        let result = atr
            .next(create_test_candle(2, 13.0, 15.0, 11.0, 14.0))
            .unwrap()
            .unwrap();
        assert_float_eq(result, 3.666666666666667);

        // Fourth value should use Wilder's smoothing
        let result = atr
            .next(create_test_candle(3, 14.0, 16.0, 12.0, 15.0))
            .unwrap()
            .unwrap();
        assert_float_eq(result, 3.777777777777778);
    }

    #[test]
    fn test_atr_error_handling() {
        let mut atr = Atr::new(5).unwrap();

        // Test with insufficient data
        let data = vec![
            create_test_candle(0, 10.0, 12.0, 9.0, 11.0),
            create_test_candle(1, 11.0, 14.0, 10.0, 13.0),
            create_test_candle(2, 13.0, 15.0, 11.0, 14.0),
        ];

        assert!(matches!(
            atr.calculate(&data),
            Err(IndicatorError::InsufficientData(_))
        ));

        // Test with empty data
        let data = vec![];
        assert!(matches!(
            atr.calculate(&data),
            Err(IndicatorError::InsufficientData(_))
        ));
    }

    #[test]
    fn test_atr_reset() {
        let mut atr = Atr::new(3).unwrap();

        // Add some values
        atr.next(create_test_candle(0, 10.0, 12.0, 9.0, 11.0))
            .unwrap();
        atr.next(create_test_candle(1, 11.0, 14.0, 10.0, 13.0))
            .unwrap();
        atr.next(create_test_candle(2, 13.0, 15.0, 11.0, 14.0))
            .unwrap();

        // Reset the indicator
        atr.reset();

        // Next value after reset should return None
        assert_eq!(
            atr.next(create_test_candle(3, 14.0, 16.0, 12.0, 15.0))
                .unwrap(),
            None
        );
    }
}
