//! Volume-based indicators
//!
//! This module contains volume-based indicators like OBV, Volume Rate of Change, and A/D Line.

use crate::indicators::utils::{validate_data_length, validate_period};
use crate::indicators::{Candle, Indicator, IndicatorError};
use std::collections::VecDeque;

/// On Balance Volume (OBV) indicator
///
/// OBV is a momentum indicator that uses volume flow to predict changes in stock price.
/// It accumulates volume on up days and subtracts volume on down days.
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::OnBalanceVolume;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create an OBV indicator
/// let mut obv = OnBalanceVolume::new();
///
/// // Create price data with close and volume values
/// let candles = vec![
///     Candle { timestamp: 0, open: 10.0, high: 12.0, low: 9.0, close: 11.0, volume: 1000.0 },
///     Candle { timestamp: 1, open: 11.0, high: 13.0, low: 10.0, close: 12.0, volume: 1500.0 },
///     Candle { timestamp: 2, open: 12.0, high: 15.0, low: 11.0, close: 11.5, volume: 2000.0 },
///     // ... more candles ...
/// ];
///
/// // Calculate OBV values
/// let obv_values = obv.calculate(&candles).unwrap();
/// ```
#[derive(Debug)]
pub struct OnBalanceVolume {
    prev_close: Option<f64>,
    current_obv: f64,
}

impl OnBalanceVolume {
    /// Create a new OnBalanceVolume indicator
    pub fn new() -> Self {
        Self {
            prev_close: None,
            current_obv: 0.0,
        }
    }
}

impl Default for OnBalanceVolume {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator<Candle, f64> for OnBalanceVolume {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n);

        // Reset state
        self.reset();

        // Set first OBV value
        self.current_obv = 0.0;
        result.push(self.current_obv);
        self.prev_close = Some(data[0].close);

        // Calculate OBV for each subsequent candle
        for candle in data.iter().take(n).skip(1) {
            let close = candle.close;
            let prev_close = self.prev_close.unwrap();
            let volume = candle.volume;

            if close > prev_close {
                // Up day
                self.current_obv += volume;
            } else if close < prev_close {
                // Down day
                self.current_obv -= volume;
            }
            // Equal days do not change OBV

            result.push(self.current_obv);
            self.prev_close = Some(close);
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        if let Some(prev_close) = self.prev_close {
            let close = value.close;
            let volume = value.volume;

            if close > prev_close {
                // Up day
                self.current_obv += volume;
            } else if close < prev_close {
                // Down day
                self.current_obv -= volume;
            }
            // Equal days do not change OBV

            self.prev_close = Some(close);
            Ok(Some(self.current_obv))
        } else {
            // First value just establishes the baseline
            self.prev_close = Some(value.close);
            self.current_obv = 0.0;
            Ok(Some(self.current_obv))
        }
    }

    fn reset(&mut self) {
        self.prev_close = None;
        self.current_obv = 0.0;
    }
}

/// Volume Rate of Change indicator
///
/// Volume Rate of Change measures the percentage change in volume over a given period.
/// This can be used to confirm price movements and identify potential reversals.
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::VolumeRateOfChange;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create a 14-period Volume Rate of Change indicator
/// let mut vroc = VolumeRateOfChange::new(14).unwrap();
///
/// // Create price data with volume values (need at least 15 candles for a 14-period calculation)
/// let candles = vec![
///     // Initial period (baseline volume)
///     Candle { timestamp: 1, open: 42.0, high: 43.0, low: 41.0, close: 42.5, volume: 1000.0 },
///     // Increasing volume trend
///     Candle { timestamp: 2, open: 42.5, high: 43.5, low: 41.5, close: 43.0, volume: 1050.0 },
///     Candle { timestamp: 3, open: 43.0, high: 44.0, low: 42.0, close: 43.5, volume: 1100.0 },
///     Candle { timestamp: 4, open: 43.5, high: 44.5, low: 42.5, close: 44.0, volume: 1200.0 },
///     Candle { timestamp: 5, open: 44.0, high: 45.0, low: 43.0, close: 44.5, volume: 1300.0 },
///     // Stable volume period
///     Candle { timestamp: 6, open: 44.5, high: 45.5, low: 43.5, close: 45.0, volume: 1320.0 },
///     Candle { timestamp: 7, open: 45.0, high: 46.0, low: 44.0, close: 45.5, volume: 1310.0 },
///     Candle { timestamp: 8, open: 45.5, high: 46.5, low: 44.5, close: 46.0, volume: 1330.0 },
///     // Volume surge (potential breakout)
///     Candle { timestamp: 9, open: 46.0, high: 47.0, low: 45.0, close: 46.8, volume: 2000.0 },
///     Candle { timestamp: 10, open: 46.8, high: 48.0, low: 46.5, close: 47.5, volume: 2200.0 },
///     // Volume declining (momentum fading)
///     Candle { timestamp: 11, open: 47.5, high: 48.0, low: 47.0, close: 47.8, volume: 1800.0 },
///     Candle { timestamp: 12, open: 47.8, high: 48.2, low: 47.2, close: 47.6, volume: 1500.0 },
///     Candle { timestamp: 13, open: 47.6, high: 48.0, low: 47.0, close: 47.4, volume: 1200.0 },
///     Candle { timestamp: 14, open: 47.4, high: 47.8, low: 46.8, close: 47.0, volume: 900.0 },
///     // Current candle (compared against first candle for 14-period calculation)
///     Candle { timestamp: 15, open: 47.0, high: 47.5, low: 46.5, close: 47.2, volume: 800.0 },
///     // Additional candle to see trend continuation
///     Candle { timestamp: 16, open: 47.2, high: 47.6, low: 46.8, close: 47.0, volume: 700.0 },
/// ];
///
/// // Calculate VROC values with error handling
/// match vroc.calculate(&candles) {
///     Ok(vroc_values) => {
///         // Access the latest VROC value
///         if let Some(latest_vroc) = vroc_values.last() {
///             println!("Volume Rate of Change: {:.2}%", latest_vroc); // Example output: -20.00%
///             
///             // Interpret the VROC value
///             if *latest_vroc > 0.0 {
///                 println!("Volume is higher than 14 periods ago");
///                 
///                 if *latest_vroc > 25.0 {
///                     println!("Significant volume increase - potential for trend continuation");
///                 } else if *latest_vroc > 10.0 {
///                     println!("Moderate volume increase - growing interest");
///                 } else {
///                     println!("Slight volume increase - maintain vigilance");
///                 }
///             } else if *latest_vroc < 0.0 {
///                 println!("Volume is lower than 14 periods ago");
///                 
///                 if *latest_vroc < -25.0 {
///                     println!("Significant volume decrease - waning interest");
///                 } else if *latest_vroc < -10.0 {
///                     println!("Moderate volume decrease - potential trend exhaustion");
///                 } else {
///                     println!("Slight volume decrease - monitor closely");
///                 }
///             } else {
///                 println!("Volume unchanged from 14 periods ago");
///             }
///             
///             // Check for volume divergence with price
///             if vroc_values.len() >= 2 {
///                 let previous_vroc = vroc_values[vroc_values.len() - 2];
///                 let current_close = candles.last().unwrap().close;
///                 let previous_close = candles[candles.len() - 2].close;
///                 
///                 // Potential bearish divergence: Price rising but volume falling
///                 if current_close > previous_close && *latest_vroc < previous_vroc {
///                     println!("Warning: Price rising but volume trend declining - potential weakness");
///                 }
///                 
///                 // Potential bullish confirmation: Price and volume both rising
///                 if current_close > previous_close && *latest_vroc > previous_vroc && *latest_vroc > 0.0 {
///                     println!("Bullish: Price and volume both increasing - strong trend confirmation");
///                 }
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating Volume Rate of Change: {}", e);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct VolumeRateOfChange {
    period: usize,
    volume_buffer: VecDeque<f64>,
}

impl VolumeRateOfChange {
    /// Create a new VolumeRateOfChange indicator
    ///
    /// # Arguments
    /// * `period` - The period for VROC calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new VolumeRateOfChange or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            volume_buffer: VecDeque::with_capacity(period + 1),
        })
    }
}

impl Indicator<Candle, f64> for VolumeRateOfChange {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period + 1)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period);

        // Reset state
        self.reset();

        // Cannot calculate until we have period + 1 values
        for i in self.period..n {
            let current_volume = data[i].volume;
            let past_volume = data[i - self.period].volume;

            if past_volume == 0.0 {
                return Err(IndicatorError::CalculationError(
                    "Division by zero: past volume is zero".to_string(),
                ));
            }

            let vroc = (current_volume - past_volume) / past_volume * 100.0;
            result.push(vroc);
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        self.volume_buffer.push_back(value.volume);

        if self.volume_buffer.len() > self.period + 1 {
            self.volume_buffer.pop_front();
        }

        if self.volume_buffer.len() == self.period + 1 {
            let current_volume = self.volume_buffer.back().unwrap();
            let past_volume = self.volume_buffer.front().unwrap();

            if *past_volume == 0.0 {
                return Err(IndicatorError::CalculationError(
                    "Division by zero: past volume is zero".to_string(),
                ));
            }

            let vroc = (current_volume - past_volume) / past_volume * 100.0;
            Ok(Some(vroc))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.volume_buffer.clear();
    }
}

/// Accumulation/Distribution Line (A/D Line) indicator
///
/// The Accumulation/Distribution Line is a volume-based indicator designed to measure
/// the cumulative flow of money into and out of a security. It assesses whether a
/// security is being accumulated (bought) or distributed (sold).
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::AccumulationDistributionLine;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create an A/D Line indicator
/// let mut adl = AccumulationDistributionLine::new();
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
pub struct AccumulationDistributionLine {
    current_ad: f64,
}

impl AccumulationDistributionLine {
    /// Create a new AccumulationDistributionLine indicator
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

impl Default for AccumulationDistributionLine {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator<Candle, f64> for AccumulationDistributionLine {
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

/// Chaikin Money Flow indicator
///
/// Chaikin Money Flow measures the amount of Money Flow Volume over a specific period.
/// It provides insight into the buying and selling pressure during a given time period.
///
/// # Example
///
/// ```
/// use rsta::indicators::volume::ChaikinMoneyFlow;
/// use rsta::indicators::Indicator;
/// use rsta::indicators::Candle;
///
/// // Create a 20-period Chaikin Money Flow
/// let mut cmf = ChaikinMoneyFlow::new(20).unwrap();
///
/// // Price data with OHLCV values (need at least 20 candles for the period)
/// let candles = vec![
///     // Initial candles for accumulation phase (price rising, closing near highs)
///     Candle { timestamp: 1, open: 42.0, high: 43.0, low: 41.0, close: 42.8, volume: 1000.0 },
///     Candle { timestamp: 2, open: 42.8, high: 44.0, low: 42.5, close: 43.7, volume: 1200.0 },
///     Candle { timestamp: 3, open: 43.7, high: 44.5, low: 43.2, close: 44.3, volume: 1400.0 },
///     Candle { timestamp: 4, open: 44.3, high: 45.0, low: 44.0, close: 44.8, volume: 1600.0 },
///     Candle { timestamp: 5, open: 44.8, high: 45.5, low: 44.3, close: 45.2, volume: 1800.0 },
///     // Next candles for moderate accumulation (price still rising)
///     Candle { timestamp: 6, open: 45.2, high: 46.0, low: 45.0, close: 45.7, volume: 1700.0 },
///     Candle { timestamp: 7, open: 45.7, high: 46.5, low: 45.5, close: 46.3, volume: 1600.0 },
///     Candle { timestamp: 8, open: 46.3, high: 47.0, low: 46.0, close: 46.8, volume: 1500.0 },
///     Candle { timestamp: 9, open: 46.8, high: 47.5, low: 46.5, close: 47.2, volume: 1400.0 },
///     Candle { timestamp: 10, open: 47.2, high: 48.0, low: 47.0, close: 47.6, volume: 1300.0 },
///     // Transition to distribution phase (price peaking, closing away from highs)
///     Candle { timestamp: 11, open: 47.6, high: 48.5, low: 47.3, close: 47.9, volume: 1500.0 },
///     Candle { timestamp: 12, open: 47.9, high: 49.0, low: 47.5, close: 48.2, volume: 1700.0 },
///     Candle { timestamp: 13, open: 48.2, high: 49.5, low: 48.0, close: 48.6, volume: 1900.0 },
///     Candle { timestamp: 14, open: 48.6, high: 50.0, low: 48.4, close: 49.2, volume: 2100.0 },
///     Candle { timestamp: 15, open: 49.2, high: 50.5, low: 48.8, close: 49.5, volume: 2300.0 },
///     // Distribution phase begins (price falling, closing near lows)
///     Candle { timestamp: 16, open: 49.5, high: 50.0, low: 48.5, close: 48.7, volume: 2500.0 },
///     Candle { timestamp: 17, open: 48.7, high: 49.2, low: 47.8, close: 48.0, volume: 2700.0 },
///     Candle { timestamp: 18, open: 48.0, high: 48.5, low: 47.0, close: 47.2, volume: 2900.0 },
///     Candle { timestamp: 19, open: 47.2, high: 47.7, low: 46.5, close: 46.7, volume: 3100.0 },
///     Candle { timestamp: 20, open: 46.7, high: 47.0, low: 45.8, close: 46.0, volume: 3300.0 },
///     // Additional candles to see trend change
///    Candle { timestamp: 21, open: 46.0, high: 46.5, low: 45.0, close: 45.2, volume: 3500.0 },
///     Candle { timestamp: 22, open: 45.2, high: 46.0, low: 44.5, close: 44.8, volume: 3700.0 }];
/// // Calculate CMF values with error handling
/// match cmf.calculate(&candles) {
///     Ok(cmf_values) => {
///         // Access the latest CMF value
///         if let Some(latest_cmf) = cmf_values.last() {
///             println!("CMF value: {:.2}", latest_cmf);     
///             // Interpret the value
///             if *latest_cmf > 0.0 {
///                 println!("Accumulation phase - money flow into the security");
///             } else {
///                 println!("Distribution phase - money flow out of the security");
///             }
///         }
///     },
///     Err(e) => {
///         eprintln!("Error calculating CMF: {}", e);
///     }
/// }
///```

#[derive(Debug)]
pub struct ChaikinMoneyFlow {
    period: usize,
    mfv_buffer: VecDeque<f64>,
    volume_buffer: VecDeque<f64>,
}

impl ChaikinMoneyFlow {
    /// Create a new ChaikinMoneyFlow indicator
    ///
    /// # Arguments
    /// * `period` - The period for CMF calculation (must be at least 1)
    ///
    /// # Returns
    /// * `Result<Self, IndicatorError>` - A new ChaikinMoneyFlow or an error
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        validate_period(period, 1)?;

        Ok(Self {
            period,
            mfv_buffer: VecDeque::with_capacity(period),
            volume_buffer: VecDeque::with_capacity(period),
        })
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

impl Indicator<Candle, f64> for ChaikinMoneyFlow {
    fn calculate(&mut self, data: &[Candle]) -> Result<Vec<f64>, IndicatorError> {
        validate_data_length(data, self.period)?;

        let n = data.len();
        let mut result = Vec::with_capacity(n - self.period + 1);

        // Reset state
        self.reset();

        for candle in data.iter().take(n) {
            let mfv = Self::money_flow_volume(candle)?;
            self.mfv_buffer.push_back(mfv);
            self.volume_buffer.push_back(candle.volume);

            if self.mfv_buffer.len() > self.period {
                self.mfv_buffer.pop_front();
                self.volume_buffer.pop_front();
            }

            if self.mfv_buffer.len() == self.period {
                let sum_mfv: f64 = self.mfv_buffer.iter().sum();
                let sum_volume: f64 = self.volume_buffer.iter().sum();

                if sum_volume == 0.0 {
                    return Err(IndicatorError::CalculationError(
                        "Division by zero: sum of volumes is zero".to_string(),
                    ));
                }

                let cmf = sum_mfv / sum_volume;
                result.push(cmf);
            }
        }

        Ok(result)
    }

    fn next(&mut self, value: Candle) -> Result<Option<f64>, IndicatorError> {
        let mfv = Self::money_flow_volume(&value)?;

        self.mfv_buffer.push_back(mfv);
        self.volume_buffer.push_back(value.volume);

        if self.mfv_buffer.len() > self.period {
            self.mfv_buffer.pop_front();
            self.volume_buffer.pop_front();
        }

        if self.mfv_buffer.len() == self.period {
            let sum_mfv: f64 = self.mfv_buffer.iter().sum();
            let sum_volume: f64 = self.volume_buffer.iter().sum();

            if sum_volume == 0.0 {
                return Err(IndicatorError::CalculationError(
                    "Division by zero: sum of volumes is zero".to_string(),
                ));
            }

            let cmf = sum_mfv / sum_volume;
            Ok(Some(cmf))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.mfv_buffer.clear();
        self.volume_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::Candle;

    // OnBalanceVolume Tests
    #[test]
    fn test_obv_new() {
        // OnBalanceVolume has no parameters to validate
        let obv = OnBalanceVolume::new();
        assert!(obv.current_obv == 0.0);
    }

    #[test]
    fn test_obv_calculation() {
        let mut obv = OnBalanceVolume::new();

        // Create test candles with predictable pattern
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 1200.0,
            }, // Price up
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 10.0,
                close: 10.2,
                volume: 800.0,
            }, // Price down
            Candle {
                timestamp: 4,
                open: 10.2,
                high: 11.0,
                low: 10.0,
                close: 10.8,
                volume: 900.0,
            }, // Price up
            Candle {
                timestamp: 5,
                open: 10.8,
                high: 11.0,
                low: 10.0,
                close: 10.8,
                volume: 700.0,
            }, // Price unchanged
        ];

        let result = obv.calculate(&candles).unwrap();

        // We get one OBV value for each candle
        assert_eq!(result.len(), 5);

        // First value is set to 0 by the OBV implementation
        assert_eq!(result[0], 0.0);

        // Second value: previous OBV + second volume (price up)
        assert_eq!(result[1], 1200.0);

        // Third value: previous OBV - volume (price down)
        assert_eq!(result[2], 400.0);

        // Fourth value: previous OBV + volume (price up)
        assert_eq!(result[3], 1300.0);

        // Fifth value: unchanged OBV (price unchanged)
        assert_eq!(result[4], 1300.0);
    }

    #[test]
    fn test_obv_next() {
        let mut obv = OnBalanceVolume::new();

        // First candle - sets initial OBV
        // First candle - sets initial OBV to 0
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 1000.0,
        };
        assert_eq!(obv.next(candle1).unwrap(), Some(0.0));

        // Next candle - price up, add volume
        let candle2 = Candle {
            timestamp: 2,
            open: 10.5,
            high: 12.0,
            low: 10.0,
            close: 11.0,
            volume: 1200.0,
        };
        assert_eq!(obv.next(candle2).unwrap(), Some(1200.0));

        // Next candle - price down, subtract volume
        let candle3 = Candle {
            timestamp: 3,
            open: 11.0,
            high: 11.5,
            low: 10.0,
            close: 10.2,
            volume: 800.0,
        };
        assert_eq!(obv.next(candle3).unwrap(), Some(400.0));
    }

    #[test]
    fn test_obv_reset() {
        let mut obv = OnBalanceVolume::new();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.5,
            volume: 1000.0,
        };
        obv.next(candle1).unwrap();

        // Reset
        obv.reset();

        // OBV should be reset to 0
        assert_eq!(obv.current_obv, 0.0);
        assert_eq!(obv.prev_close, None);

        // After reset, next candle should be treated as first
        // After reset, next candle should be treated as first (OBV starts at 0)
        let candle2 = Candle {
            timestamp: 2,
            open: 10.5,
            high: 12.0,
            low: 10.0,
            close: 11.0,
            volume: 1200.0,
        };
        assert_eq!(obv.next(candle2).unwrap(), Some(0.0));
    }

    #[test]
    fn test_obv_zero_volume() {
        let mut obv = OnBalanceVolume::new();

        // Create test candles with zero volume
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial candle with volume
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 0.0,
            }, // Price up but zero volume
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 10.0,
                close: 10.0,
                volume: 0.0,
            }, // Price down but zero volume
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // Second value should be unchanged from first because volume is 0
        assert_eq!(result[1], 0.0);

        // Third value should be unchanged from second because volume is 0
        assert_eq!(result[2], 0.0);

        // Test with streaming calculation too
        obv.reset();
        assert_eq!(obv.next(candles[0]).unwrap(), Some(0.0));
        assert_eq!(obv.next(candles[1]).unwrap(), Some(0.0)); // Zero volume should not change OBV
        assert_eq!(obv.next(candles[2]).unwrap(), Some(0.0)); // Zero volume should not change OBV
    }

    #[test]
    fn test_obv_extreme_volume_values() {
        let mut obv = OnBalanceVolume::new();

        // Create test candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 1_000_000_000.0, // Extremely large volume
            }, // Price up
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 10.0,
                close: 10.0,
                volume: 500_000_000.0, // Another large volume
            }, // Price down
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // Second value: add the large volume (price up)
        assert_eq!(result[1], 1_000_000_000.0);

        // Third value: subtract the large volume (price down)
        assert_eq!(result[2], 1_000_000_000.0 - 500_000_000.0);
    }

    #[test]
    fn test_obv_identical_closing_prices() {
        let mut obv = OnBalanceVolume::new();

        // Create test candles with identical closing prices
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.5,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.5,
                high: 12.0,
                low: 10.0,
                close: 10.5, // Same close as previous
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.5,
                high: 11.5,
                low: 10.0,
                close: 10.5, // Same close again
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 10.5,
                high: 11.0,
                low: 10.0,
                close: 10.5, // Same close again
                volume: 900.0,
            },
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // All subsequent values should remain at 0 since prices are unchanged
        // and OBV should not change when close prices are identical
        assert_eq!(result[1], 0.0);
        assert_eq!(result[2], 0.0);
        assert_eq!(result[3], 0.0);
    }

    #[test]
    fn test_obv_consecutive_up_down_sequences() {
        let mut obv = OnBalanceVolume::new();

        // Create test candles with alternating up/down patterns
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            }, // Initial price
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 12.0,
                low: 10.0,
                close: 11.0, // Up
                volume: 500.0,
            },
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 9.5,
                close: 10.0, // Down
                volume: 300.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 10.5,
                low: 9.0,
                close: 10.5, // Up
                volume: 700.0,
            },
            Candle {
                timestamp: 5,
                open: 10.5,
                high: 11.0,
                low: 9.5,
                close: 9.5, // Down
                volume: 400.0,
            },
            Candle {
                timestamp: 6,
                open: 9.5,
                high: 10.5,
                low: 9.0,
                close: 10.0, // Up
                volume: 600.0,
            },
        ];

        let result = obv.calculate(&candles).unwrap();

        // First value is 0 by OBV definition
        assert_eq!(result[0], 0.0);

        // Check the pattern matches our expectation:
        // Initial: 0
        // Up: +500 => 500
        // Down: -300 => 200
        // Up: +700 => 900
        // Down: -400 => 500
        // Up: +600 => 1100
        assert_eq!(result[1], 500.0); // +500
        assert_eq!(result[2], 200.0); // +500-300
        assert_eq!(result[3], 900.0); // +500-300+700
        assert_eq!(result[4], 500.0); // +500-300+700-400
        assert_eq!(result[5], 1100.0); // +500-300+700-400+600

        // Test streaming calculation matches batch calculation
        let mut streaming_obv = OnBalanceVolume::new();
        for (i, candle) in candles.iter().enumerate() {
            let obv_value = streaming_obv.next(*candle).unwrap().unwrap();
            assert_eq!(
                obv_value, result[i],
                "Streaming calculation mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_obv_insufficient_data() {
        let mut obv = OnBalanceVolume::new();

        // Test with empty data
        let empty: Vec<Candle> = vec![];
        let result = obv.calculate(&empty);

        // Should error due to insufficient data (require at least 1 data point)
        assert!(result.is_err());
        if let Err(IndicatorError::InsufficientData(_)) = result {
            // Expected error
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_obv_batch_vs_streaming_consistency() {
        let mut batch_obv = OnBalanceVolume::new();
        let mut streaming_obv = OnBalanceVolume::new();

        // Create test data with different patterns
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 12.0,
                low: 10.0,
                close: 11.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 3,
                open: 11.0,
                high: 11.5,
                low: 9.5,
                close: 10.0,
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 10.5,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 11.0,
                low: 9.5,
                close: 10.5,
                volume: 2000.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_obv.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::with_capacity(candles.len());
        for candle in &candles {
            let value = streaming_obv.next(*candle).unwrap().unwrap();
            streaming_result.push(value);
        }

        // Compare results - they should be identical
        assert_eq!(batch_result.len(), streaming_result.len());

        for i in 0..batch_result.len() {
            assert_eq!(
                batch_result[i], streaming_result[i],
                "Batch and streaming results differ at index {}",
                i
            );
        }
    }

    // AccumulationDistributionLine Tests
    #[test]
    fn test_adl_new() {
        // AccumulationDistributionLine has no parameters to validate
        let adl = AccumulationDistributionLine::new();
        // Verify fields are accessible
        assert_eq!(adl.current_ad, 0.0);
    }

    #[test]
    fn test_adl_calculation() {
        let mut adl = AccumulationDistributionLine::new();

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
        let mut adl = AccumulationDistributionLine::new();

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
        let mut adl = AccumulationDistributionLine::new();

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
        let result = AccumulationDistributionLine::money_flow_multiplier(&candle);
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
        let mut adl = AccumulationDistributionLine::new();
        let result = adl.calculate(&[candle]);
        assert!(result.is_err());

        // Test error propagation in streaming calculation
        let mut adl = AccumulationDistributionLine::new();
        let result = adl.next(candle);
        assert!(result.is_err());
    }

    #[test]
    fn test_adl_consecutive_identical_prices() {
        let mut adl = AccumulationDistributionLine::new();

        // Create candles with identical prices (but different high/low)
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0, // Same close as previous
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0, // Same close as previous
                volume: 800.0,
            },
        ];

        let result = adl.calculate(&candles).unwrap();

        // Verify the ADL values for each candle
        assert_eq!(result.len(), 3);

        // Calculate expected values
        // First candle: MFM = (2*10 - 11 - 9)/(11 - 9) = 0.0, MFV = 0.0 * 1000 = 0
        // Second candle: MFM = (2*10 - 11 - 9)/(11 - 9) = 0.0, MFV = 0.0 * 1200 = 0
        // Third candle: MFM = (2*10 - 11 - 9)/(11 - 9) = 0.0, MFV = 0.0 * 800 = 0

        // Since all MFMs are 0, the ADL should remain at 0
        assert!((result[0] - 0.0).abs() < 0.001);
        assert!((result[1] - 0.0).abs() < 0.001);
        assert!((result[2] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_adl_extreme_volume_scenarios() {
        let mut adl = AccumulationDistributionLine::new();

        // Create candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 10_000_000.0, // Very large volume
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 0.1, // Very small volume
            },
            Candle {
                timestamp: 3,
                open: 12.0,
                high: 15.0,
                low: 10.0,
                close: 11.0,
                volume: 999_999_999.0, // Extremely large volume
            },
        ];

        let result = adl.calculate(&candles).unwrap();
        assert_eq!(result.len(), 3);

        // First candle: MFM = (2*11 - 12 - 8)/(12 - 8) = 0.5, MFV = 0.5 * 10_000_000 = 5_000_000
        assert!((result[0] - 5_000_000.0).abs() < 0.1);

        // Second candle: MFM = (2*12 - 13 - 9)/(13 - 9) = 0.5, MFV = 0.5 * 0.1 = 0.05
        // ADL = previous + 0.05 = 5_000_000.05
        assert!((result[1] - 5_000_000.05).abs() < 0.1);

        // Third candle: MFM = (2*11 - 15 - 10)/(15 - 10) = -0.6, MFV = -0.6 * 999_999_999 = -599_999_999.4
        // ADL = previous + (-599_999_999.4) = -594_999_999.35
        assert!((result[2] - (-594_999_999.35)).abs() < 0.1);
    }

    #[test]
    fn test_adl_batch_vs_streaming_consistency() {
        let mut batch_adl = AccumulationDistributionLine::new();
        let mut streaming_adl = AccumulationDistributionLine::new();

        // Create test data with different patterns
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
                high: 14.0,
                low: 10.0,
                close: 13.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 3,
                open: 13.0,
                high: 15.0,
                low: 11.0,
                close: 12.0,
                volume: 800.0,
            },
            Candle {
                timestamp: 4,
                open: 12.0,
                high: 13.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_adl.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::with_capacity(candles.len());
        for candle in &candles {
            let value = streaming_adl.next(*candle).unwrap().unwrap();
            streaming_result.push(value);
        }

        // Compare results - they should be identical
        assert_eq!(batch_result.len(), streaming_result.len());

        for i in 0..batch_result.len() {
            assert!(
                (batch_result[i] - streaming_result[i]).abs() < 0.001,
                "Batch and streaming results differ at index {}: batch={}, streaming={}",
                i,
                batch_result[i],
                streaming_result[i]
            );
        }
    }

    #[test]
    fn test_adl_insufficient_data() {
        let mut adl = AccumulationDistributionLine::new();

        // Test with empty data
        let empty: Vec<Candle> = vec![];
        let result = adl.calculate(&empty);

        // Should error with insufficient data
        assert!(result.is_err());
        if let Err(IndicatorError::InsufficientData(_)) = result {
            // Expected error
        } else {
            panic!("Expected InsufficientData error");
        }
    }

    #[test]
    fn test_adl_edge_cases() {
        let mut adl = AccumulationDistributionLine::new();

        // Test with zero volume (should not cause errors, just have no effect on ADL)
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1000.0, // Normal volume
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 0.0, // Zero volume
            },
        ];

        let result = adl.calculate(&candles).unwrap();
        assert_eq!(result.len(), 2);

        // First value: MFM = (2*11 - 12 - 8)/(12 - 8) = 0.5, MFV = 0.5 * 1000 = 500
        assert!((result[0] - 500.0).abs() < 0.01);

        // Second value: MFM = (2*12 - 13 - 9)/(13 - 9) = 0.5, MFV = 0.5 * 0 = 0
        // ADL = previous + 0 = 500
        assert!((result[1] - 500.0).abs() < 0.01);
    }

    // VolumeRateOfChange Tests
    #[test]
    fn test_vroc_new() {
        // Valid period should work
        assert!(VolumeRateOfChange::new(14).is_ok());

        // Invalid period should fail
        assert!(VolumeRateOfChange::new(0).is_err());
    }

    #[test]
    fn test_vroc_calculation() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Create candles with known volumes
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 900.0,
            },
        ];

        let result = vroc.calculate(&candles).unwrap();

        // We need at least period+1 candles, and we get n-period results
        assert_eq!(result.len(), 2);

        // First VROC: (1500 - 1000) / 1000 * 100 = 50%
        assert!((result[0] - 50.0).abs() < 0.01);

        // Second VROC: (900 - 1200) / 1200 * 100 = -25%
        assert!((result[1] - (-25.0)).abs() < 0.01);
    }

    #[test]
    fn test_vroc_next() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Initial values - not enough data yet
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1000.0,
        };
        assert_eq!(vroc.next(candle1).unwrap(), None);

        let candle2 = Candle {
            timestamp: 2,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1200.0,
        };
        assert_eq!(vroc.next(candle2).unwrap(), None);

        // Third value - now we have enough data
        let candle3 = Candle {
            timestamp: 3,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1500.0,
        };
        let result = vroc.next(candle3).unwrap();

        // VROC: (1500 - 1000) / 1000 * 100 = 50%
        assert!((result.unwrap() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_vroc_reset() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Add some values
        let candle1 = Candle {
            timestamp: 1,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1000.0,
        };
        vroc.next(candle1).unwrap();
        let candle2 = Candle {
            timestamp: 2,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1200.0,
        };
        vroc.next(candle2).unwrap();

        // Reset
        vroc.reset();

        // Volume buffer should be cleared
        let candle3 = Candle {
            timestamp: 3,
            open: 10.0,
            high: 11.0,
            low: 9.0,
            close: 10.0,
            volume: 1500.0,
        };
        assert_eq!(vroc.next(candle3).unwrap(), None);
    }

    #[test]
    fn test_vroc_past_volume_zero() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Create candles with zero volume at the reference point
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 0.0, // Zero volume at reference point
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
        ];

        // Should return an error for division by zero
        let result = vroc.calculate(&candles);
        assert!(result.is_err());

        // Verify it's the correct error type and message
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(msg.contains("division by zero") || msg.contains("zero"));
        } else {
            panic!("Expected CalculationError");
        }

        // Test with streaming calculation too
        vroc.reset();
        assert_eq!(vroc.next(candles[0]).unwrap(), None); // Not enough data yet
        assert_eq!(vroc.next(candles[1]).unwrap(), None); // Not enough data yet

        // This should error due to division by zero
        let next_result = vroc.next(candles[2]);
        assert!(next_result.is_err());

        // Verify it's the correct error type
        if let Err(IndicatorError::CalculationError(msg)) = next_result {
            assert!(msg.contains("division by zero") || msg.contains("zero"));
        } else {
            panic!("Expected CalculationError");
        }
    }

    #[test]
    fn test_vroc_minimum_period() {
        // Test with period = 1 (minimum valid period)
        let mut vroc = VolumeRateOfChange::new(1).unwrap();

        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 2000.0,
            },
        ];

        let result = vroc.calculate(&candles).unwrap();

        // With period = 1, we should get n-1 results
        assert_eq!(result.len(), 2);

        // First VROC: (1500 - 1000) / 1000 * 100 = 50%
        assert!((result[0] - 50.0).abs() < 0.001);

        // Second VROC: (2000 - 1500) / 1500 * 100 = 33.33%
        assert!((result[1] - 33.33).abs() < 0.01);
    }

    #[test]
    fn test_vroc_large_period() {
        // Test with period close to data length
        let data_length = 10;
        let period = data_length - 1; // Use period = 9 for data length of 10

        let mut vroc = VolumeRateOfChange::new(period).unwrap();

        // Create 10 candles with sequential volumes
        let mut candles = Vec::with_capacity(data_length);
        for i in 0..data_length {
            candles.push(Candle {
                timestamp: i as u64,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0 * (i + 1) as f64, // Volumes: 1000, 2000, 3000, ...
            });
        }

        let result = vroc.calculate(&candles).unwrap();

        // With period = 9 and data length = 10, we should get 1 result
        assert_eq!(result.len(), 1);

        // VROC: (10000 - 1000) / 1000 * 100 = 900%
        assert!((result[0] - 900.0).abs() < 0.001);
        // Test boundary cases with data length
        // For VolumeRateOfChange, we need at least period+1 data points
        // With 10 candles:
        // - period = 9 works (needs 10 data points, we have 10)
        // - period = 10 doesn't work (needs 11 data points, we only have 10)

        // Test with period = 9 (should work with 10 data points)
        let mut vroc_large = VolumeRateOfChange::new(9).unwrap();
        let result = vroc_large.calculate(&candles);
        assert!(result.is_ok()); // Should have enough data

        // Test with period = 10 (should fail with only 10 data points since we need period+1)
        let mut vroc_too_large = VolumeRateOfChange::new(10).unwrap();
        let result = vroc_too_large.calculate(&candles);
        assert!(result.is_err()); // Should not be enough data
        assert!(result.is_err()); // Should not be enough data
    }

    #[test]
    fn test_vroc_reset_streaming() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Create test candles
        let candles = [
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1800.0,
            },
        ];

        // Process first three candles
        vroc.next(candles[0]).unwrap();
        vroc.next(candles[1]).unwrap();
        let first_result = vroc.next(candles[2]).unwrap().unwrap();

        // Reset indicator
        vroc.reset();

        // Process the candles again in a different order
        vroc.next(candles[1]).unwrap();
        vroc.next(candles[2]).unwrap();
        let second_result = vroc.next(candles[3]).unwrap().unwrap();

        // Results should be different as we've processed different candles
        // First: (1500-1000)/1000*100 = 50%
        // Second: (1800-1200)/1200*100 = 50%
        assert_eq!(first_result, 50.0);
        assert_eq!(second_result, 50.0);

        // Now reset and verify we need to process 3 candles again
        vroc.reset();
        assert_eq!(vroc.next(candles[0]).unwrap(), None);
        assert_eq!(vroc.next(candles[1]).unwrap(), None);
        assert!(vroc.next(candles[2]).unwrap().is_some());
    }

    #[test]
    fn test_vroc_batch_vs_streaming() {
        let period = 3;
        let mut batch_vroc = VolumeRateOfChange::new(period).unwrap();
        let mut streaming_vroc = VolumeRateOfChange::new(period).unwrap();

        // Create test data
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1200.0,
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 4,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1800.0,
            },
            Candle {
                timestamp: 5,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 2100.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_vroc.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::new();
        for candle in &candles {
            if let Some(value) = streaming_vroc.next(*candle).unwrap() {
                streaming_result.push(value);
            }
        }

        // Compare results - they should be identical
        assert_eq!(batch_result.len(), streaming_result.len());

        for i in 0..batch_result.len() {
            assert!(
                (batch_result[i] - streaming_result[i]).abs() < 0.001,
                "Batch and streaming results differ at index {}: batch={}, streaming={}",
                i,
                batch_result[i],
                streaming_result[i]
            );
        }
    }

    #[test]
    fn test_vroc_extreme_volume_values() {
        let mut vroc = VolumeRateOfChange::new(2).unwrap();

        // Create candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 100.0, // Small volume
            },
            Candle {
                timestamp: 2,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 1_000_000_000.0, // Very large volume
            },
            Candle {
                timestamp: 3,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 5_000_000_000.0, // Extremely large volume
            },
        ];

        let result = vroc.calculate(&candles).unwrap();

        // We need at least period+1 candles, and we get n-period results
        assert_eq!(result.len(), 1);

        // VROC: (5_000_000_000.0 - 100.0) / 100.0 * 100 = 4,999,999,900%
        assert!(
            result[0] > 4_000_000_000.0,
            "Extreme VROC value not calculated correctly"
        );

        // Test with streaming calculation
        vroc.reset();
        assert_eq!(vroc.next(candles[0]).unwrap(), None); // Not enough data yet
        assert_eq!(vroc.next(candles[1]).unwrap(), None); // Not enough data yet

        let streaming_result = vroc.next(candles[2]).unwrap().unwrap();
        assert!((streaming_result - result[0]).abs() < 0.001);
    }
    #[test]
    fn test_cmf_new() {
        // Valid period should work
        assert!(ChaikinMoneyFlow::new(14).is_ok());

        // Invalid period should fail
        assert!(ChaikinMoneyFlow::new(0).is_err());
    }

    #[test]
    fn test_cmf_calculation() {
        let mut cmf = ChaikinMoneyFlow::new(2).unwrap();

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

        let result = cmf.calculate(&candles).unwrap();

        // We need at least period (2) candles
        assert_eq!(result.len(), 2);

        // Verify the CMF values are between -1 and 1
        for cmf_value in &result {
            assert!(*cmf_value >= -1.0 && *cmf_value <= 1.0);
        }

        // For the first period (candles 1-2):
        // First candle: MFM = (2*11 - 12 - 8)/(12 - 8) = 0.5, MFV = 0.5 * 1000 = 500
        // Second candle: MFM = (2*12 - 13 - 9)/(13 - 9) = 0.5, MFV = 0.5 * 1200 = 600
        // Sum of MFV = 500 + 600 = 1100
        // Sum of Volume = 1000 + 1200 = 2200
        // CMF = 1100 / 2200 = 0.5
        assert!((result[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_cmf_zero_volume_sum() {
        let mut cmf = ChaikinMoneyFlow::new(2).unwrap();

        // Create candles with zero volume
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 0.0, // Zero volume
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 0.0, // Zero volume
            },
        ];

        // Should error with division by zero
        let result = cmf.calculate(&candles);
        assert!(result.is_err());

        // Verify it's the correct error type and message
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(msg.contains("division by zero") || msg.contains("sum of volumes is zero"));
        } else {
            panic!("Expected CalculationError for zero volume sum");
        }

        // Test streaming calculation
        cmf.reset();
        assert_eq!(cmf.next(candles[0]).unwrap(), None); // Not enough data yet
        let result = cmf.next(candles[1]);

        assert!(result.is_err());
        if let Err(IndicatorError::CalculationError(msg)) = result {
            assert!(msg.contains("division by zero") || msg.contains("sum of volumes is zero"));
        } else {
            panic!("Expected CalculationError for zero volume sum in streaming mode");
        }
    }

    #[test]
    fn test_cmf_boundary_conditions() {
        let mut cmf = ChaikinMoneyFlow::new(3).unwrap();

        // Create candles that should produce CMF values close to boundaries
        // For CMF near +1: High MFM (close near high) with consistent volume
        let max_candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.9, // Close near high
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 11.9,
                high: 14.0,
                low: 11.0,
                close: 13.9, // Close near high
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 13.9,
                high: 16.0,
                low: 13.0,
                close: 15.9, // Close near high
                volume: 1000.0,
            },
        ];

        // For CMF near -1: Low MFM (close near low) with consistent volume
        let min_candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 8.1, // Close near low
                volume: 1000.0,
            },
            Candle {
                timestamp: 2,
                open: 8.1,
                high: 10.0,
                low: 7.0,
                close: 7.1, // Close near low
                volume: 1000.0,
            },
            Candle {
                timestamp: 3,
                open: 7.1,
                high: 9.0,
                low: 6.0,
                close: 6.1, // Close near low
                volume: 1000.0,
            },
        ];

        // Test near maximum value
        let max_result = cmf.calculate(&max_candles).unwrap();
        assert_eq!(max_result.len(), 1);
        assert!(
            max_result[0] > 0.9,
            "CMF value should be close to +1, got {}",
            max_result[0]
        );
        assert!(
            max_result[0] <= 1.0,
            "CMF value should not exceed +1, got {}",
            max_result[0]
        );

        // Test near minimum value
        cmf.reset();
        let min_result = cmf.calculate(&min_candles).unwrap();
        assert_eq!(min_result.len(), 1);
        assert!(
            min_result[0] < -0.9,
            "CMF value should be close to -1, got {}",
            min_result[0]
        );
        assert!(
            min_result[0] >= -1.0,
            "CMF value should not be less than -1, got {}",
            min_result[0]
        );
    }

    #[test]
    fn test_cmf_minimum_period() {
        // Test with period = 1 (minimum valid period)
        let mut cmf = ChaikinMoneyFlow::new(1).unwrap();

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

        let result = cmf.calculate(&candles).unwrap();

        // With period = 1, we should get result for each candle
        assert_eq!(result.len(), 3);

        // First candle: MFM = (2*11 - 12 - 8)/(12 - 8) = 0.5, CMF = 0.5
        assert!((result[0] - 0.5).abs() < 0.001);

        // Second candle: MFM = (2*12 - 13 - 9)/(13 - 9) = 0.5, CMF = 0.5
        assert!((result[1] - 0.5).abs() < 0.001);

        // Third candle: MFM = (2*11 - 14 - 10)/(14 - 10) = -0.5, CMF = -0.5
        assert!((result[2] - (-0.5)).abs() < 0.001);

        // Test streaming calculation with minimum period
        cmf.reset();
        assert_eq!(cmf.next(candles[0]).unwrap().unwrap(), 0.5);
        assert_eq!(cmf.next(candles[1]).unwrap().unwrap(), 0.5);
        assert!((cmf.next(candles[2]).unwrap().unwrap() - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_cmf_reset_partial_data() {
        let mut cmf = ChaikinMoneyFlow::new(3).unwrap();

        // Create test candles
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
            Candle {
                timestamp: 4,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 5,
                open: 12.0,
                high: 15.0,
                low: 11.0,
                close: 14.0,
                volume: 2000.0,
            },
        ];

        // Process first two candles
        cmf.next(candles[0]).unwrap();
        cmf.next(candles[1]).unwrap();

        // Reset and process different candles
        cmf.reset();

        cmf.next(candles[2]).unwrap();
        cmf.next(candles[3]).unwrap();

        // We need one more candle to get a result with period = 3
        let result = cmf.next(candles[4]).unwrap();
        assert!(result.is_some());

        // Verify that CMF calculation after reset uses only the new data
        // This should be based on candles 2, 3, and 4, not include candles 0 and 1

        // Calculate expected result from batch calculation for verification
        cmf.reset();
        let expected = cmf.calculate(&candles[2..5]).unwrap()[0];

        assert!((result.unwrap() - expected).abs() < 0.001);
    }

    #[test]
    fn test_cmf_batch_vs_streaming() {
        let period = 3;
        let mut batch_cmf = ChaikinMoneyFlow::new(period).unwrap();
        let mut streaming_cmf = ChaikinMoneyFlow::new(period).unwrap();

        // Create test data
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
            Candle {
                timestamp: 4,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 1500.0,
            },
            Candle {
                timestamp: 5,
                open: 12.0,
                high: 15.0,
                low: 11.0,
                close: 14.0,
                volume: 2000.0,
            },
        ];

        // Calculate batch result
        let batch_result = batch_cmf.calculate(&candles).unwrap();

        // Calculate streaming result
        let mut streaming_result = Vec::new();
        for candle in &candles {
            if let Some(value) = streaming_cmf.next(*candle).unwrap() {
                streaming_result.push(value);
            }
        }

        // Compare results - they should be identical
        assert_eq!(batch_result.len(), streaming_result.len());

        for i in 0..batch_result.len() {
            assert!(
                (batch_result[i] - streaming_result[i]).abs() < 0.001,
                "Batch and streaming results differ at index {}: batch={}, streaming={}",
                i,
                batch_result[i],
                streaming_result[i]
            );
        }
    }

    #[test]
    fn test_cmf_extreme_volume_values() {
        let mut cmf = ChaikinMoneyFlow::new(2).unwrap();

        // Create candles with extreme volume values
        let candles = vec![
            Candle {
                timestamp: 1,
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 1_000_000_000.0, // Very large volume
            },
            Candle {
                timestamp: 2,
                open: 11.0,
                high: 13.0,
                low: 9.0,
                close: 12.0,
                volume: 2_000_000_000.0, // Another large volume
            },
        ];

        let result = cmf.calculate(&candles).unwrap();

        // We should get one result
        assert_eq!(result.len(), 1);

        // The value should still be constrained between -1 and 1
        assert!(
            result[0] >= -1.0 && result[0] <= 1.0,
            "CMF with extreme volumes should still be between -1 and 1, got: {}",
            result[0]
        );
    }
} // Close the test module
