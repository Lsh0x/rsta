//! Trend following indicators
//!
//! This module contains trend following indicators like Moving Averages, MACD, and Bollinger Bands.

use crate::indicators::utils::{
    calculate_ema, calculate_sma, validate_data_length, validate_period,
};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// Simple Moving Average (SMA) indicator
///
/// # Example
///
/// ```
/// use rsta::indicators::trend::SimpleMovingAverage;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period SMA
/// let mut sma = SimpleMovingAverage::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate SMA values
/// let sma_values = sma.calculate(&prices).unwrap();
/// assert_eq!(sma_values.len(), 6);
/// assert_eq!(sma_values[0], 12.0);
/// ```
#[derive(Debug)]
pub struct SimpleMovingAverage {
    period: usize,
    buffer: VecDeque<f64>,
    sum: f64,
}

impl SimpleMovingAverage {
    /// Create a new SimpleMovingAverage indicator
    ///
    /// # Arguments
    /// * `period` - The period for SMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new SimpleMovingAverage or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }
}

impl Indicator<f64, f64> for SimpleMovingAverage {
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
        self.buffer.clear();
        self.sum = 0.0;
    }

    fn name(&self) -> &'static str {
        "SMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl SimpleMovingAverage {
    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

/// Exponential Moving Average (EMA) indicator
///
/// # Example
///
/// ```
/// use rsta::indicators::trend::ExponentialMovingAverage;
/// use rsta::indicators::Indicator;
///
/// // Create a 5-period EMA
/// let mut ema = ExponentialMovingAverage::new(5).unwrap();
///
/// // Price data
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
///
/// // Calculate EMA values
/// let ema_values = ema.calculate(&prices).unwrap();
/// ```
#[derive(Debug)]
pub struct ExponentialMovingAverage {
    period: usize,
    alpha: f64,
    current_ema: Option<f64>,
}

impl ExponentialMovingAverage {
    /// Create a new ExponentialMovingAverage indicator
    ///
    /// # Arguments
    /// * `period` - The period for EMA calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new ExponentialMovingAverage or an error
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

impl Indicator<f64, f64> for ExponentialMovingAverage {
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

    fn name(&self) -> &'static str {
        "EMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

impl ExponentialMovingAverage {
    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

/// Weighted Moving Average (WMA).
///
/// Linearly weighted: the most recent close gets weight `period`, the oldest
/// gets weight `1`. Weights sum to `period * (period + 1) / 2`.
///
/// # Example
/// ```
/// use rsta::indicators::trend::WeightedMovingAverage;
/// use rsta::indicators::Indicator;
///
/// let mut wma = WeightedMovingAverage::new(3).unwrap();
/// // Weights are 1, 2, 3 → (1*1 + 2*2 + 3*3) / 6 = 14/6 ≈ 2.333.
/// let out = wma.calculate(&[1.0, 2.0, 3.0]).unwrap();
/// assert!((out[0] - (14.0 / 6.0)).abs() < 1e-12);
/// ```
#[derive(Debug)]
pub struct WeightedMovingAverage {
    period: usize,
    buffer: VecDeque<f64>,
}

impl WeightedMovingAverage {
    /// Create a new WMA. `period >= 1`.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            buffer: VecDeque::with_capacity(period),
        })
    }

    fn weighted(buffer: &VecDeque<f64>, period: usize) -> f64 {
        let n = period as f64;
        let denom = n * (n + 1.0) / 2.0;
        // Most-recent value has the highest weight.
        let mut numer = 0.0;
        for (i, v) in buffer.iter().enumerate() {
            numer += (i as f64 + 1.0) * v;
        }
        numer / denom
    }

    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

impl Indicator<f64, f64> for WeightedMovingAverage {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;
        self.reset();
        let mut out = Vec::with_capacity(data.len() - self.period + 1);
        for &v in data {
            if let Some(x) = self.next(v)? {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.buffer.push_back(value);
        if self.buffer.len() > self.period {
            self.buffer.pop_front();
        }
        if self.buffer.len() < self.period {
            return Ok(None);
        }
        Ok(Some(Self::weighted(&self.buffer, self.period)))
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn name(&self) -> &'static str {
        "WMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

/// Double Exponential Moving Average (DEMA).
///
/// `DEMA = 2 * EMA(price) - EMA(EMA(price))`. Reduces the lag of a plain EMA
/// while keeping smoothing.
#[derive(Debug)]
pub struct DoubleExponentialMovingAverage {
    period: usize,
    ema1: ExponentialMovingAverage,
    ema2: ExponentialMovingAverage,
    seen: usize,
}

impl DoubleExponentialMovingAverage {
    /// Create a new DEMA. `period >= 1`.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            ema1: ExponentialMovingAverage::new(period)?,
            ema2: ExponentialMovingAverage::new(period)?,
            seen: 0,
        })
    }

    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

impl Indicator<f64, f64> for DoubleExponentialMovingAverage {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        // The classic warmup for DEMA is 2*period - 1 samples (the EMA-of-EMA
        // needs `period` outputs from the inner EMA, which itself produces a
        // value on every input, so 2*period - 1 inputs are enough to stabilise).
        validate_data_length(data, 2 * self.period - 1)?;
        self.reset();
        let mut out = Vec::with_capacity(data.len() - 2 * (self.period - 1));
        for &v in data {
            if let Some(x) = self.next(v)? {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.seen += 1;
        let e1 = self.ema1.next(value)?.unwrap();
        let e2 = self.ema2.next(e1)?.unwrap();
        // Hold output until the inner EMA-of-EMA has had `period` samples to
        // stabilise; before that the chained EMA is biased toward the seed.
        if self.seen < 2 * self.period - 1 {
            return Ok(None);
        }
        Ok(Some(2.0 * e1 - e2))
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.seen = 0;
    }

    fn name(&self) -> &'static str {
        "DEMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

/// Triple Exponential Moving Average (TEMA).
///
/// `TEMA = 3 * EMA1 - 3 * EMA2 + EMA3` where each EMA chains the previous
/// one's output. Even less lag than DEMA at the cost of more warmup.
#[derive(Debug)]
pub struct TripleExponentialMovingAverage {
    period: usize,
    ema1: ExponentialMovingAverage,
    ema2: ExponentialMovingAverage,
    ema3: ExponentialMovingAverage,
    seen: usize,
}

impl TripleExponentialMovingAverage {
    /// Create a new TEMA. `period >= 1`.
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            ema1: ExponentialMovingAverage::new(period)?,
            ema2: ExponentialMovingAverage::new(period)?,
            ema3: ExponentialMovingAverage::new(period)?,
            seen: 0,
        })
    }

    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

impl Indicator<f64, f64> for TripleExponentialMovingAverage {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 3 * self.period - 2)?;
        self.reset();
        let mut out = Vec::with_capacity(data.len() - 3 * (self.period - 1));
        for &v in data {
            if let Some(x) = self.next(v)? {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        self.seen += 1;
        let e1 = self.ema1.next(value)?.unwrap();
        let e2 = self.ema2.next(e1)?.unwrap();
        let e3 = self.ema3.next(e2)?.unwrap();
        if self.seen < 3 * self.period - 2 {
            return Ok(None);
        }
        Ok(Some(3.0 * e1 - 3.0 * e2 + e3))
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
        self.seen = 0;
    }

    fn name(&self) -> &'static str {
        "TEMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

/// Hull Moving Average (HMA).
///
/// `HMA = WMA(2 * WMA(price, period/2) - WMA(price, period), sqrt(period))`.
/// Designed by Alan Hull to be both smooth and reactive.
#[derive(Debug)]
pub struct HullMovingAverage {
    period: usize,
    half: WeightedMovingAverage,
    full: WeightedMovingAverage,
    smooth: WeightedMovingAverage,
}

impl HullMovingAverage {
    /// Create a new HMA. `period >= 2` (we need a non-zero `period / 2`).
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 2)?;
        let half_p = period / 2;
        let smooth_p = (period as f64).sqrt().round() as usize;
        Ok(Self {
            period,
            half: WeightedMovingAverage::new(half_p)?,
            full: WeightedMovingAverage::new(period)?,
            smooth: WeightedMovingAverage::new(smooth_p.max(1))?,
        })
    }

    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(&mut self, candles: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, f64>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(&mut self, candle: Candle) -> Result<Option<f64>, IndicatorError> {
        <Self as Indicator<f64, f64>>::next(self, candle.close)
    }
}

impl Indicator<f64, f64> for HullMovingAverage {
    fn calculate(&mut self, data: &[f64]) -> Result<Vec<f64>, IndicatorError> {
        // Worst-case warmup: full WMA(period) + smooth WMA(sqrt(period)) - 1.
        let smooth_p = (self.period as f64).sqrt().round() as usize;
        let needed = self.period + smooth_p.max(1) - 1;
        validate_data_length(data, needed)?;
        self.reset();
        let mut out = Vec::with_capacity(data.len().saturating_sub(needed - 1));
        for &v in data {
            if let Some(x) = self.next(v)? {
                out.push(x);
            }
        }
        Ok(out)
    }

    fn next(&mut self, value: f64) -> Result<Option<f64>, IndicatorError> {
        let h = self.half.next(value)?;
        let f = self.full.next(value)?;
        let raw = match (h, f) {
            (Some(h), Some(f)) => 2.0 * h - f,
            _ => return Ok(None),
        };
        self.smooth.next(raw)
    }

    fn reset(&mut self) {
        self.half.reset();
        self.full.reset();
        self.smooth.reset();
    }

    fn name(&self) -> &'static str {
        "HMA"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

/// MACD (Moving Average Convergence/Divergence) result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingAverageConvergenceDivergenceResult {
    /// MACD line: `EMA(fast) - EMA(slow)`.
    pub macd: f64,
    /// Signal line: EMA of the MACD line.
    pub signal: f64,
    /// Histogram: `macd - signal`. Positive = bullish momentum.
    pub histogram: f64,
}

/// Moving Average Convergence/Divergence (MACD) indicator.
///
/// MACD is a trend-following momentum indicator that shows the relationship
/// between two exponential moving averages of an asset's price. It is composed
/// of three series:
///
/// - **MACD line** = EMA(fast_period) − EMA(slow_period)
/// - **Signal line** = EMA(signal_period) of the MACD line
/// - **Histogram** = MACD − signal
///
/// Standard parameters are 12/26/9.
///
/// # Example
///
/// ```no_run
/// use rsta::indicators::trend::MovingAverageConvergenceDivergence;
/// use rsta::indicators::Indicator;
///
/// let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
/// let prices: Vec<f64> = (1..=60).map(|i| i as f64).collect();
/// let values = macd.calculate(&prices).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct MovingAverageConvergenceDivergence {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    fast_ema: ExponentialMovingAverage,
    slow_ema: ExponentialMovingAverage,
    signal_ema: ExponentialMovingAverage,
    /// Number of close prices observed so far via `next()`.
    seen: usize,
}

impl MovingAverageConvergenceDivergence {
    /// Create a new MACD indicator.
    ///
    /// # Errors
    /// Returns `IndicatorError::InvalidParameter` if any period is `0` or if
    /// `fast_period >= slow_period` (otherwise the MACD line collapses).
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Result<Self, IndicatorError> {
        validate_period(fast_period, 1)?;
        validate_period(slow_period, 1)?;
        validate_period(signal_period, 1)?;
        if fast_period >= slow_period {
            return Err(IndicatorError::InvalidParameter(
                "fast_period must be strictly less than slow_period".to_string(),
            ));
        }
        Ok(Self {
            fast_period,
            slow_period,
            signal_period,
            fast_ema: ExponentialMovingAverage::new(fast_period)?,
            slow_ema: ExponentialMovingAverage::new(slow_period)?,
            signal_ema: ExponentialMovingAverage::new(signal_period)?,
            seen: 0,
        })
    }

    /// Fast EMA period.
    pub fn fast_period(&self) -> usize {
        self.fast_period
    }
    /// Slow EMA period.
    pub fn slow_period(&self) -> usize {
        self.slow_period
    }
    /// Signal EMA period.
    pub fn signal_period(&self) -> usize {
        self.signal_period
    }

    /// Convenience: feed candles by extracting the close price.
    pub fn calculate_candles(
        &mut self,
        candles: &[Candle],
    ) -> Result<Vec<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        <Self as Indicator<f64, MovingAverageConvergenceDivergenceResult>>::calculate(self, &closes)
    }

    /// Convenience: streaming update with the close price of a candle.
    pub fn next_candle(
        &mut self,
        candle: Candle,
    ) -> Result<Option<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        <Self as Indicator<f64, MovingAverageConvergenceDivergenceResult>>::next(self, candle.close)
    }
}

impl Indicator<f64, MovingAverageConvergenceDivergenceResult>
    for MovingAverageConvergenceDivergence
{
    fn calculate(
        &mut self,
        data: &[f64],
    ) -> Result<Vec<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        // We need at least slow_period prices for the MACD line, plus
        // signal_period values of the MACD line for the signal line. The
        // signal line seeds on its first value, so the very first MACD
        // datapoint produces signal = macd and histogram = 0.
        let needed = self.slow_period;
        validate_data_length(data, needed)?;

        self.reset();

        let mut out = Vec::with_capacity(data.len().saturating_sub(needed - 1));
        for &price in data {
            if let Some(point) = self.next(price)? {
                out.push(point);
            }
        }
        Ok(out)
    }

    fn next(
        &mut self,
        value: f64,
    ) -> Result<Option<MovingAverageConvergenceDivergenceResult>, IndicatorError> {
        self.seen += 1;
        let fast = self.fast_ema.next(value)?;
        let slow = self.slow_ema.next(value)?;

        // Emit nothing until both EMAs have stabilised over their full
        // window, which by convention is `slow_period` close prices.
        if self.seen < self.slow_period {
            return Ok(None);
        }

        match (fast, slow) {
            (Some(f), Some(s)) => {
                let macd = f - s;
                let signal = self
                    .signal_ema
                    .next(macd)?
                    .expect("signal EMA always emits on first value");
                Ok(Some(MovingAverageConvergenceDivergenceResult {
                    macd,
                    signal,
                    histogram: macd - signal,
                }))
            }
            _ => Ok(None),
        }
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.seen = 0;
    }

    fn name(&self) -> &'static str {
        "MACD"
    }

    fn period(&self) -> Option<usize> {
        // MACD has three periods; surfacing only the slow one would be
        // misleading.
        None
    }
}

/// Average Directional Index (ADX) result.
///
/// Carries the two directional indicators alongside the ADX value so a single
/// emission gives the full trend-strength picture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AverageDirectionalIndexResult {
    /// +DI line — strength of upward movement (0..=100).
    pub plus_di: f64,
    /// -DI line — strength of downward movement (0..=100).
    pub minus_di: f64,
    /// ADX line — overall trend strength (0..=100, period-smoothed).
    pub adx: f64,
}

/// Average Directional Index (ADX) indicator.
///
/// Wilder's directional movement system: tracks +DM/-DM and the True Range,
/// applies Wilder smoothing over `period` bars, then derives:
///
/// - `+DI = 100 * +DM_smoothed / ATR_smoothed`
/// - `-DI = 100 * -DM_smoothed / ATR_smoothed`
/// - `DX  = 100 * |+DI - -DI| / (+DI + -DI)`
/// - `ADX = Wilder-smoothed DX over period`
///
/// The first ADX value is emitted after `2 * period` candles (one period to
/// initialise +DI/-DI, then `period` DX values to seed the ADX smoothing).
///
/// # Example
/// ```no_run
/// use rsta::indicators::trend::AverageDirectionalIndex;
/// use rsta::indicators::{Indicator, Candle};
///
/// let mut adx = AverageDirectionalIndex::new(14).unwrap();
/// let candles: Vec<Candle> = (0..50)
///     .map(|i| Candle {
///         timestamp: i, open: i as f64, high: i as f64 + 2.0,
///         low: i as f64 - 1.0, close: i as f64 + 1.0, volume: 1000.0,
///     })
///     .collect();
/// let values = adx.calculate(&candles).unwrap();
/// assert!(!values.is_empty());
/// ```
#[derive(Debug)]
pub struct AverageDirectionalIndex {
    period: usize,
    prev_high: Option<f64>,
    prev_low: Option<f64>,
    prev_close: Option<f64>,
    /// Wilder-smoothed +DM, -DM and TR (`None` until period+1 candles seen).
    smooth_plus_dm: Option<f64>,
    smooth_minus_dm: Option<f64>,
    smooth_tr: Option<f64>,
    /// Buffer of DX values until we have `period` of them to seed ADX.
    dx_buffer: std::collections::VecDeque<f64>,
    smooth_adx: Option<f64>,
    /// Number of candles processed.
    seen: usize,
}

impl AverageDirectionalIndex {
    /// Create a new ADX indicator with the given lookback (typically 14).
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;
        Ok(Self {
            period,
            prev_high: None,
            prev_low: None,
            prev_close: None,
            smooth_plus_dm: None,
            smooth_minus_dm: None,
            smooth_tr: None,
            dx_buffer: std::collections::VecDeque::with_capacity(period),
            smooth_adx: None,
            seen: 0,
        })
    }
}

impl Indicator<Candle, AverageDirectionalIndexResult> for AverageDirectionalIndex {
    fn calculate(
        &mut self,
        data: &[Candle],
    ) -> Result<Vec<AverageDirectionalIndexResult>, IndicatorError> {
        // First emission appears at the 2*period-th candle: one seed, `period`
        // samples to fill the smoothing sums, then `period - 1` more DX values
        // to seed the ADX smoothing.
        validate_data_length(data, 2 * self.period)?;
        self.reset();
        let mut out = Vec::with_capacity(data.len().saturating_sub(2 * self.period - 1));
        for &candle in data {
            if let Some(point) = self.next(candle)? {
                out.push(point);
            }
        }
        Ok(out)
    }

    fn next(
        &mut self,
        value: Candle,
    ) -> Result<Option<AverageDirectionalIndexResult>, IndicatorError> {
        self.seen += 1;
        let (Some(prev_high), Some(prev_low), Some(prev_close)) =
            (self.prev_high, self.prev_low, self.prev_close)
        else {
            // First candle: seed the prev_* state and emit nothing.
            self.prev_high = Some(value.high);
            self.prev_low = Some(value.low);
            self.prev_close = Some(value.close);
            return Ok(None);
        };

        // Directional movement.
        let up_move = value.high - prev_high;
        let down_move = prev_low - value.low;
        let plus_dm = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let minus_dm = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };

        // True Range.
        let tr = (value.high - value.low)
            .max((value.high - prev_close).abs())
            .max((value.low - prev_close).abs());

        // Update state for next call.
        self.prev_high = Some(value.high);
        self.prev_low = Some(value.low);
        self.prev_close = Some(value.close);

        let n = self.period as f64;
        // `samples` = number of completed directional movement pairs
        // (the very first candle is a seed, contributing nothing).
        let samples = self.seen - 1;
        if samples == 1 {
            self.smooth_plus_dm = Some(plus_dm);
            self.smooth_minus_dm = Some(minus_dm);
            self.smooth_tr = Some(tr);
        } else {
            let prev_p = self.smooth_plus_dm.unwrap();
            let prev_m = self.smooth_minus_dm.unwrap();
            let prev_t = self.smooth_tr.unwrap();
            if samples <= self.period {
                // Wilder seeds the smoothed values with raw sums over the
                // first `period` directional samples.
                self.smooth_plus_dm = Some(prev_p + plus_dm);
                self.smooth_minus_dm = Some(prev_m + minus_dm);
                self.smooth_tr = Some(prev_t + tr);
            } else {
                // Wilder smoothing: x_new = x_prev - x_prev / n + raw.
                self.smooth_plus_dm = Some(prev_p - prev_p / n + plus_dm);
                self.smooth_minus_dm = Some(prev_m - prev_m / n + minus_dm);
                self.smooth_tr = Some(prev_t - prev_t / n + tr);
            }
        }

        if samples < self.period {
            return Ok(None);
        }

        let p = self.smooth_plus_dm.unwrap();
        let m = self.smooth_minus_dm.unwrap();
        let t = self.smooth_tr.unwrap();
        if t == 0.0 {
            // Pathological flat market: emit zeros to avoid division blow-up.
            return Ok(Some(AverageDirectionalIndexResult {
                plus_di: 0.0,
                minus_di: 0.0,
                adx: 0.0,
            }));
        }

        let plus_di = 100.0 * p / t;
        let minus_di = 100.0 * m / t;
        let denom = plus_di + minus_di;
        let dx = if denom == 0.0 {
            0.0
        } else {
            100.0 * (plus_di - minus_di).abs() / denom
        };

        // Seed and update ADX.
        match self.smooth_adx {
            None => {
                self.dx_buffer.push_back(dx);
                if self.dx_buffer.len() < self.period {
                    return Ok(None);
                }
                let seed = self.dx_buffer.iter().sum::<f64>() / n;
                self.smooth_adx = Some(seed);
                Ok(Some(AverageDirectionalIndexResult {
                    plus_di,
                    minus_di,
                    adx: seed,
                }))
            }
            Some(prev_adx) => {
                let new_adx = (prev_adx * (n - 1.0) + dx) / n;
                self.smooth_adx = Some(new_adx);
                Ok(Some(AverageDirectionalIndexResult {
                    plus_di,
                    minus_di,
                    adx: new_adx,
                }))
            }
        }
    }

    fn reset(&mut self) {
        self.prev_high = None;
        self.prev_low = None;
        self.prev_close = None;
        self.smooth_plus_dm = None;
        self.smooth_minus_dm = None;
        self.smooth_tr = None;
        self.dx_buffer.clear();
        self.smooth_adx = None;
        self.seen = 0;
    }

    fn name(&self) -> &'static str {
        "ADX"
    }

    fn period(&self) -> Option<usize> {
        Some(self.period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_new() {
        // Valid period should work
        assert!(SimpleMovingAverage::new(14).is_ok());

        // Invalid period should fail
        assert!(SimpleMovingAverage::new(0).is_err());
    }

    #[test]
    fn test_sma_calculation() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let result = sma.calculate(&data).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 4.0); // (2+4+6)/3
        assert_eq!(result[1], 6.0); // (4+6+8)/3
        assert_eq!(result[2], 8.0); // (6+8+10)/3
    }

    #[test]
    fn test_sma_next() {
        let mut sma = SimpleMovingAverage::new(3).unwrap();

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
        let mut sma = SimpleMovingAverage::new(3).unwrap();

        // Add some values
        sma.next(2.0).unwrap();
        sma.next(4.0).unwrap();
        sma.next(6.0).unwrap();

        // Reset
        sma.reset();

        // Should be back to initial state
        assert_eq!(sma.next(8.0).unwrap(), None);
    }

    #[test]
    fn test_ema_new() {
        // Valid period should work
        assert!(ExponentialMovingAverage::new(14).is_ok());

        // Invalid period should fail
        assert!(ExponentialMovingAverage::new(0).is_err());
    }

    #[test]
    fn test_ema_calculation() {
        let mut ema = ExponentialMovingAverage::new(3).unwrap();
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
        let mut ema = ExponentialMovingAverage::new(3).unwrap();
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
        let mut ema = ExponentialMovingAverage::new(3).unwrap();

        // Add some values
        ema.next(2.0).unwrap();
        ema.next(4.0).unwrap();

        // Reset
        ema.reset();

        // Should be back to initial state, next value becomes seed
        assert_eq!(ema.next(6.0).unwrap(), Some(6.0));
    }

    #[test]
    fn test_macd_construction_validates_periods() {
        // fast must be strictly less than slow
        assert!(MovingAverageConvergenceDivergence::new(26, 26, 9).is_err());
        assert!(MovingAverageConvergenceDivergence::new(30, 26, 9).is_err());
        // any zero period is rejected
        assert!(MovingAverageConvergenceDivergence::new(0, 26, 9).is_err());
        assert!(MovingAverageConvergenceDivergence::new(12, 0, 9).is_err());
        assert!(MovingAverageConvergenceDivergence::new(12, 26, 0).is_err());
        // standard parameters work
        assert!(MovingAverageConvergenceDivergence::new(12, 26, 9).is_ok());
    }

    #[test]
    fn test_macd_emits_after_warmup() {
        let mut macd = MovingAverageConvergenceDivergence::new(3, 6, 2).unwrap();
        // Slow period is 6, so the first 5 closes produce no output.
        for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
            assert!(macd.next(v).unwrap().is_none(), "premature emission");
        }
        let first = macd.next(6.0).unwrap().expect("should emit at slow_period");
        // First emission: signal seeded with current MACD → histogram == 0.
        assert!(first.histogram.abs() < 1e-12);
        assert!((first.macd - first.signal).abs() < 1e-12);
    }

    #[test]
    fn test_macd_calculate_matches_streaming() {
        let prices: Vec<f64> = (1..=40).map(|i| i as f64).collect();
        let mut batch = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        let batch_out = batch.calculate(&prices).unwrap();

        let mut stream = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        let stream_out: Vec<_> = prices
            .iter()
            .filter_map(|&p| stream.next(p).unwrap())
            .collect();

        assert_eq!(batch_out, stream_out);
    }

    #[test]
    fn test_macd_histogram_definition() {
        let prices: Vec<f64> = (1..=40).map(|i| i as f64).collect();
        let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
        let out = macd.calculate(&prices).unwrap();
        for v in &out {
            assert!((v.histogram - (v.macd - v.signal)).abs() < 1e-12);
        }
    }

    #[test]
    fn test_wma_basic_weighting() {
        // Weights are 1, 2, 3 → (1*1 + 2*2 + 3*3) / 6 = 14/6.
        let mut wma = WeightedMovingAverage::new(3).unwrap();
        let out = wma.calculate(&[1.0, 2.0, 3.0]).unwrap();
        assert!((out[0] - (14.0 / 6.0)).abs() < 1e-12);
    }

    #[test]
    fn test_wma_calculate_matches_streaming() {
        let prices: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let mut batch = WeightedMovingAverage::new(5).unwrap();
        let batch_out = batch.calculate(&prices).unwrap();
        let mut stream = WeightedMovingAverage::new(5).unwrap();
        let stream_out: Vec<_> = prices
            .iter()
            .filter_map(|&p| stream.next(p).unwrap())
            .collect();
        assert_eq!(batch_out, stream_out);
    }

    #[test]
    fn test_dema_construction_and_warmup() {
        let mut dema = DoubleExponentialMovingAverage::new(5).unwrap();
        // First 2*period - 2 = 8 inputs produce nothing.
        for v in 1..=8 {
            assert!(dema.next(v as f64).unwrap().is_none(), "premature {v}");
        }
        let first = dema.next(9.0).unwrap();
        assert!(first.is_some());
    }

    #[test]
    fn test_tema_warmup() {
        let mut tema = TripleExponentialMovingAverage::new(3).unwrap();
        // 3*period - 2 = 7 → first emission at the 7th input.
        for v in 1..=6 {
            assert!(tema.next(v as f64).unwrap().is_none());
        }
        assert!(tema.next(7.0).unwrap().is_some());
    }

    #[test]
    fn test_hma_emits_and_validates_period() {
        assert!(HullMovingAverage::new(1).is_err());
        let mut hma = HullMovingAverage::new(9).unwrap();
        let prices: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let out = hma.calculate(&prices).unwrap();
        assert!(!out.is_empty());
    }

    fn make_candles(count: usize, trend: f64) -> Vec<Candle> {
        (0..count)
            .map(|i| {
                let mid = i as f64 * trend;
                Candle {
                    timestamp: i as u64,
                    open: mid,
                    high: mid + 1.5,
                    low: mid - 1.5,
                    close: mid + 0.5,
                    volume: 1000.0,
                }
            })
            .collect()
    }

    #[test]
    fn test_adx_construction_validates_period() {
        assert!(AverageDirectionalIndex::new(0).is_err());
        assert!(AverageDirectionalIndex::new(14).is_ok());
    }

    #[test]
    fn test_adx_emits_after_warmup() {
        let period = 3;
        let mut adx = AverageDirectionalIndex::new(period).unwrap();
        let candles = make_candles(20, 1.0);
        let mut emissions = 0;
        for c in &candles {
            if adx.next(*c).unwrap().is_some() {
                emissions += 1;
            }
        }
        // First ADX appears at the 2*period-th candle (one seed + `period`
        // directional samples to seed sums + `period - 1` DX values to seed
        // the ADX smoothing → emission at candle index 2*period - 1).
        assert_eq!(emissions, candles.len() - (2 * period - 1));
    }

    #[test]
    fn test_adx_strong_uptrend_high_di_diff() {
        // A clean uptrend should give +DI >> -DI and a high ADX.
        let mut adx = AverageDirectionalIndex::new(7).unwrap();
        let out = adx.calculate(&make_candles(40, 1.0)).unwrap();
        let last = out.last().expect("at least one emission");
        assert!(
            last.plus_di > last.minus_di,
            "+DI {} should exceed -DI {}",
            last.plus_di,
            last.minus_di,
        );
        assert!(
            last.adx > 50.0,
            "uptrend ADX should be high, got {}",
            last.adx
        );
    }

    #[test]
    fn test_adx_calculate_matches_streaming() {
        let candles = make_candles(50, 1.0);
        let mut batch = AverageDirectionalIndex::new(14).unwrap();
        let batch_out = batch.calculate(&candles).unwrap();

        let mut stream = AverageDirectionalIndex::new(14).unwrap();
        let stream_out: Vec<_> = candles
            .iter()
            .filter_map(|c| stream.next(*c).unwrap())
            .collect();
        assert_eq!(batch_out, stream_out);
    }
}
