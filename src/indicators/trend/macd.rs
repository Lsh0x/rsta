use crate::indicators::trend::Ema;
use crate::indicators::validate_period;
use crate::indicators::{Candle, Indicator, IndicatorError};

/// Moving Average Convergence Divergence (MACD) indicator
///
/// MACD is a trend-following momentum indicator that shows the relationship
/// between two moving averages of a security's price. It consists of three components:
/// - MACD Line: Difference between fast and slow EMAs
/// - Signal Line: EMA of the MACD Line
/// - Histogram: Difference between MACD Line and Signal Line
///
/// # Example with float values
///
/// ```
/// use rsta::indicators::trend::Macd;
/// use rsta::indicators::Indicator;
///
/// // Create a MACD with standard periods (12, 26, 9)
/// let mut macd = Macd::new(12, 26, 9).unwrap();
///
/// // Price data
/// let prices = vec![
///     10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///     20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///     30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0,
///     40.0, 41.0, 42.0, 43.0, 44.0, 45.0
/// ];
///
/// // Calculate MACD values
/// let macd_values = macd.calculate(&prices).unwrap();
/// ```
///
/// # Example with Candle data
///
/// ```
/// use rsta::indicators::trend::Macd;
/// use rsta::indicators::{Indicator, Candle};
///
/// // Create a MACD with standard periods (12, 26, 9)
/// let mut macd = Macd::new(12, 26, 9).unwrap();
///
/// // Create candle data
/// let mut candles = Vec::new();
/// let prices = vec![
///     10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,
///     20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0,
///     30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0,
///     40.0, 41.0, 42.0, 43.0, 44.0, 45.0
/// ];
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
/// // Calculate MACD values based on close prices
/// let macd_values = macd.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct Macd {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_ema: Ema,
    slow_ema: Ema,
    signal_ema: Ema,
    current_macd: Option<f64>,
    current_signal: Option<f64>,
    current_histogram: Option<f64>,
}

/// MACD result containing all three components
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacdResult {
    /// The MACD line value (fast EMA - slow EMA)
    pub macd: f64,
    /// The signal line value (EMA of MACD line)
    pub signal: f64,
    /// The histogram value (MACD line - signal line)
    pub histogram: f64,
}

impl Macd {
    /// Create a new MACD indicator
    ///
    /// # Arguments
    /// * `fast_period` - The period for the fast EMA (typically 12)
    /// * `slow_period` - The period for the slow EMA (typically 26)
    /// * `signal_period` - The period for the signal line EMA (typically 9)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new MACD or an error
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Result<Self, IndicatorError> {
        // Validate periods
        validate_period(fast_period, 1)?;
        validate_period(slow_period, 1)?;
        validate_period(signal_period, 1)?;

        // Slow period should be greater than fast period
        if fast_period >= slow_period {
            return Err(IndicatorError::InvalidParameter(
                "Slow period must be greater than fast period".to_string(),
            ));
        }

        Ok(Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: Ema::new(fast_period)?,
            slow_ema: Ema::new(slow_period)?,
            signal_ema: Ema::new(signal_period)?,
            current_macd: None,
            current_signal: None,
            current_histogram: None,
        })
    }

    /// Reset the MACD indicator state
    pub fn reset_state(&mut self) {
        // Use explicit type annotations to resolve ambiguity
        <Ema as Indicator<f64, f64>>::reset(&mut self.fast_ema);
        <Ema as Indicator<f64, f64>>::reset(&mut self.slow_ema);
        <Ema as Indicator<f64, f64>>::reset(&mut self.signal_ema);
        self.current_macd = None;
        self.current_signal = None;
        self.current_histogram = None;
    }
}

// Implementation for raw price values
impl Indicator<f64, MacdResult> for Macd {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<MacdResult>, IndicatorError> {
        if data.len() < self.slow_period + self.signal_period - 1 {
            return Err(IndicatorError::InsufficientData(format!(
                "At least {} data points required for MACD({},{},{})",
                self.slow_period + self.signal_period - 1,
                self.fast_period,
                self.slow_period,
                self.signal_period
            )));
        }

        // Calculate EMAs
        let fast_ema_values = self.fast_ema.calculate(data)?;
        let slow_ema_values = self.slow_ema.calculate(data)?;

        // Calculate MACD line values (fast EMA - slow EMA)
        let mut macd_line = Vec::new();

        // The MACD line starts at the index where both fast and slow EMAs are available
        let start_idx = self.slow_period - 1;
        for i in start_idx..data.len() {
            let fast_idx = i - (self.slow_period - self.fast_period);
            macd_line.push(fast_ema_values[fast_idx] - slow_ema_values[i - start_idx]);
        }

        // Calculate signal line (EMA of MACD line)
        let signal_values = self.signal_ema.calculate(&macd_line)?;

        // Calculate histogram (MACD line - signal line)
        let mut result = Vec::new();
        let signal_start_idx = self.signal_period - 1;
        for i in signal_start_idx..macd_line.len() {
            let macd = macd_line[i];
            let signal = signal_values[i - signal_start_idx];
            let histogram = macd - signal;

            result.push(MacdResult {
                macd,
                signal,
                histogram,
            });
        }

        // Update current values
        if let Some(last) = result.last() {
            self.current_macd = Some(last.macd);
            self.current_signal = Some(last.signal);
            self.current_histogram = Some(last.histogram);
        }

        Ok(result)
    }

    fn next(&mut self, value: f64) -> Result<Option<MacdResult>, IndicatorError> {
        // Calculate new EMA values
        let fast_ema = self.fast_ema.next(value)?.unwrap_or(value);
        let slow_ema = self.slow_ema.next(value)?.unwrap_or(value);

        // Calculate new MACD line value
        let macd = fast_ema - slow_ema;
        self.current_macd = Some(macd);

        // Calculate new signal line value
        let signal = if let Some(signal_value) = self.signal_ema.next(macd)? {
            signal_value
        } else {
            // If there's no signal value yet, use MACD as the initial value
            macd
        };
        self.current_signal = Some(signal);

        // Calculate histogram
        let histogram = macd - signal;
        self.current_histogram = Some(histogram);

        // Only return complete MACD output when all components are available
        if self.current_macd.is_some()
            && self.current_signal.is_some()
            && self.current_histogram.is_some()
        {
            return Ok(Some(MacdResult {
                macd,
                signal,
                histogram,
            }));
        }

        Ok(None)
    }

    fn reset(&mut self) {
        self.reset_state();
    }
}

// Implementation for candle data
impl Indicator<Candle, MacdResult> for Macd {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<MacdResult>, IndicatorError> {
        // Extract close prices from candles
        let close_prices: Vec<f64> = data.iter().map(|candle| candle.close).collect();
        self.calculate(&close_prices)
    }

    fn next(&mut self, candle: Candle) -> Result<Option<MacdResult>, IndicatorError> {
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

    #[test]
    fn test_macd_new() {
        // Valid parameters should work
        assert!(Macd::new(12, 26, 9).is_ok());

        // Fast period should be less than slow period
        assert!(Macd::new(26, 12, 9).is_err());

        // Invalid periods should fail
        assert!(Macd::new(0, 26, 9).is_err());
        assert!(Macd::new(12, 0, 9).is_err());
        assert!(Macd::new(12, 26, 0).is_err());
    }

    #[test]
    fn test_macd_calculation() {
        let mut macd = Macd::new(3, 6, 2).unwrap();

        // Create price data with a clear trend
        let prices: Vec<f64> = (1..=20).map(|i| i as f64 * 2.0).collect();

        // We need at least slow_period + signal_period - 1 data points
        assert!(prices.len() >= macd.slow_period + macd.signal_period - 1);

        let result = macd.calculate(&prices).unwrap();

        // Check output length: data_len - slow_period - signal_period + 2
        // Because: data_len - slow_period + 1 MACD points, then signal starts at signal_period
        let expected_len = prices.len() - macd.slow_period - macd.signal_period + 2;
        assert_eq!(result.len(), expected_len);

        // Verify MACD values are all non-zero (since we have a clear trend)
        for output in &result {
            assert!(output.macd != 0.0);
            assert!(output.signal != 0.0);
            // In a consistent trend, histogram should stabilize around non-zero values
        }

        // In an uptrend with consistent price increases, MACD should be positive
        assert!(result.last().unwrap().macd > 0.0);
    }

    #[test]
    fn test_macd_next() {
        let mut macd = Macd::new(3, 6, 2).unwrap();

        // Add increasingly higher prices to simulate an uptrend
        for i in 1..=15 {
            let price = i as f64 * 2.0;
            macd.next(price).unwrap();
        }

        // After sufficient data points, we should have valid MACD values
        let result = macd.next(32.0).unwrap();

        // Verify we got a result
        assert!(result.is_some());

        let output = result.unwrap();

        // In a consistent uptrend, MACD and Signal should be positive
        assert!(output.macd > 0.0);
        assert!(output.signal > 0.0);
    }

    #[test]
    fn test_macd_reset() {
        let mut macd = Macd::new(3, 6, 2).unwrap();

        // Add some values
        for i in 1..=10 {
            macd.next(i as f64 * 2.0).unwrap();
        }

        // Reset state
        macd.reset_state();

        // After reset, internal state should be cleared
        assert!(macd.current_macd.is_none());
        assert!(macd.current_signal.is_none());
        assert!(macd.current_histogram.is_none());
    }

    #[test]
    fn test_macd_with_candles() {
        let mut macd = Macd::new(3, 6, 2).unwrap();

        // Create candles with uptrending prices
        let mut candles = Vec::new();
        for i in 1..=20 {
            let price = i as f64 * 2.0;
            candles.push(Candle {
                timestamp: i as u64,
                open: price - 0.5,
                high: price + 1.0,
                low: price - 1.0,
                close: price,
                volume: 1000.0,
            });
        }

        let result = macd.calculate(&candles).unwrap();

        // Check output length
        let expected_len = candles.len() - macd.slow_period - macd.signal_period + 2;
        assert_eq!(result.len(), expected_len);

        // In an uptrend, MACD should be positive
        assert!(result.last().unwrap().macd > 0.0);
    }

    #[test]
    fn test_macd_implementations_produce_same_results() {
        let mut macd_f64 = Macd::new(3, 6, 2).unwrap();
        let mut macd_candle = Macd::new(3, 6, 2).unwrap();

        // Raw price data
        let prices: Vec<f64> = (1..=20).map(|i| i as f64 * 2.0).collect();

        // Equivalent candle data
        let candles: Vec<Candle> = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| Candle {
                timestamp: i as u64,
                open: price - 0.5,
                high: price + 1.0,
                low: price - 1.0,
                close: price,
                volume: 1000.0,
            })
            .collect();

        // Calculate using both implementations
        let result_f64 = macd_f64.calculate(&prices).unwrap();
        let result_candle = macd_candle.calculate(&candles).unwrap();

        // Results should be identical
        assert_eq!(result_f64.len(), result_candle.len());
        for (i, (out_f64, out_candle)) in result_f64.iter().zip(result_candle.iter()).enumerate() {
            assert!(
                (out_f64.macd - out_candle.macd).abs() < 0.000001,
                "MACD values differ at index {}: {} vs {}",
                i,
                out_f64.macd,
                out_candle.macd
            );
            assert!(
                (out_f64.signal - out_candle.signal).abs() < 0.000001,
                "Signal values differ at index {}: {} vs {}",
                i,
                out_f64.signal,
                out_candle.signal
            );
            assert!(
                (out_f64.histogram - out_candle.histogram).abs() < 0.000001,
                "Histogram values differ at index {}: {} vs {}",
                i,
                out_f64.histogram,
                out_candle.histogram
            );
        }
    }
}
