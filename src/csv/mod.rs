//! # CSV Module for Technical Analysis
//!
//! This module provides tools for loading OHLCV price data from CSV files,
//! computing technical indicators in batch, and exporting the augmented
//! dataset back to CSV.
//!
//! It is gated behind the `csv` feature flag and pulls in `csv`, `serde` and
//! `chrono` as optional dependencies.
//!
//! ## Example
//!
//! ```no_run
//! use rsta::csv::CsvFormatter;
//! use rsta::indicators::trend::SimpleMovingAverage;
//! use rsta::indicators::momentum::RelativeStrengthIndex;
//!
//! let mut formatter = CsvFormatter::new();
//! formatter.load_from_file("price_data.csv").unwrap();
//!
//! formatter
//!     .add_close_indicator("SMA20", Box::new(SimpleMovingAverage::new(20).unwrap()))
//!     .add_close_indicator("RSI14", Box::new(RelativeStrengthIndex::new(14).unwrap()));
//!
//! formatter.calculate_and_export("enhanced_data.csv").unwrap();
//! ```
//!
//! ## Scope
//!
//! Only scalar-valued indicators are supported in this module:
//!
//! - close-price indicators (`Indicator<f64, f64>`) — SMA, EMA, RSI, StdDev …
//! - candle-based indicators (`Indicator<Candle, f64>`) — ATR, OBV, ADL, VROC,
//!   CMF, Williams %R …
//!
//! Multi-output indicators (Bollinger Bands, Keltner Channels, Stochastic) will
//! be supported in a follow-up: each output channel will be exposed as its own
//! column.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};

use crate::indicators::{Candle, Indicator, IndicatorError};

/// Configuration for CSV import/export operations.
#[derive(Debug, Clone)]
pub struct CsvConfig {
    /// Date format for parsing dates from the input CSV (e.g. `"%Y-%m-%d"`).
    pub date_format: String,
    /// Whether the input CSV has a header row.
    pub has_header: bool,
    /// Field delimiter (default `b','`).
    pub delimiter: u8,
    /// Column indices for OHLCV data (date, open, high, low, close, volume).
    pub column_indices: OhlcvColumnIndices,
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            date_format: "%Y-%m-%d".to_string(),
            has_header: true,
            delimiter: b',',
            column_indices: OhlcvColumnIndices::default(),
        }
    }
}

/// Column indices for OHLCV data in CSV files.
#[derive(Debug, Clone, Copy)]
pub struct OhlcvColumnIndices {
    /// Index of the date column.
    pub date: usize,
    /// Index of the open price column.
    pub open: usize,
    /// Index of the high price column.
    pub high: usize,
    /// Index of the low price column.
    pub low: usize,
    /// Index of the close price column.
    pub close: usize,
    /// Index of the volume column.
    pub volume: usize,
}

impl Default for OhlcvColumnIndices {
    fn default() -> Self {
        Self {
            date: 0,
            open: 1,
            high: 2,
            low: 3,
            close: 4,
            volume: 5,
        }
    }
}

/// A single row of OHLCV data, keeping the original date string alongside the
/// parsed timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhlcvData {
    /// Original date string from the CSV (preserved for round-tripping).
    pub date: String,
    /// Unix timestamp (seconds since epoch) parsed from `date`.
    pub timestamp: u64,
    /// Opening price.
    pub open: f64,
    /// Highest price during the period.
    pub high: f64,
    /// Lowest price during the period.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Trading volume.
    pub volume: f64,
}

impl OhlcvData {
    /// Build a [`Candle`] from this row (used as input for indicators).
    pub fn to_candle(&self) -> Candle {
        Candle {
            timestamp: self.timestamp,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
        }
    }
}

/// Errors emitted by the CSV module.
#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    /// Underlying I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from the underlying `csv` crate.
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Error parsing a field (price, volume, date, …).
    #[error("Parse error: {0}")]
    Parse(String),

    /// Indicator-level error during calculation.
    #[error("Indicator error: {0}")]
    Indicator(#[from] IndicatorError),

    /// Triggered when calculation is requested without any loaded data.
    #[error("Missing data for indicator calculation")]
    MissingData,
}

/// Indicator that consumes close prices and produces a scalar value per period.
type CloseIndicator = Box<dyn Indicator<f64, f64> + Send>;
/// Indicator that consumes [`Candle`] data and produces a scalar value per period.
type CandleIndicator = Box<dyn Indicator<Candle, f64> + Send>;

/// Loads OHLCV data, runs registered indicators against it and exports the
/// augmented dataset back to CSV.
///
/// `BTreeMap` is used internally to keep indicator columns in a deterministic
/// (alphabetical) order in the exported CSV.
pub struct CsvFormatter {
    config: CsvConfig,
    data: Vec<OhlcvData>,
    close_indicators: BTreeMap<String, CloseIndicator>,
    candle_indicators: BTreeMap<String, CandleIndicator>,
    calculated_values: BTreeMap<String, Vec<Option<f64>>>,
}

impl Default for CsvFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl CsvFormatter {
    /// Create a new formatter with default configuration.
    pub fn new() -> Self {
        Self::with_config(CsvConfig::default())
    }

    /// Create a new formatter with a custom configuration.
    pub fn with_config(config: CsvConfig) -> Self {
        Self {
            config,
            data: Vec::new(),
            close_indicators: BTreeMap::new(),
            candle_indicators: BTreeMap::new(),
            calculated_values: BTreeMap::new(),
        }
    }

    /// Load OHLCV data from a CSV file at `path`.
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        let file = File::open(path)?;
        self.load_from_reader(file)
    }

    /// Load OHLCV data from any reader.
    pub fn load_from_reader<R: Read>(&mut self, reader: R) -> Result<(), CsvError> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(self.config.has_header)
            .delimiter(self.config.delimiter)
            .from_reader(reader);

        self.data.clear();
        self.calculated_values.clear();

        let ci = self.config.column_indices;
        for result in rdr.records() {
            let record = result?;

            if record.len() <= ci.volume {
                return Err(CsvError::Parse(
                    "Record has fewer columns than expected".to_string(),
                ));
            }

            let date = record.get(ci.date).unwrap_or("").to_string();
            let timestamp = parse_timestamp(&date, &self.config.date_format)?;
            let open = parse_f64(record.get(ci.open), "open")?;
            let high = parse_f64(record.get(ci.high), "high")?;
            let low = parse_f64(record.get(ci.low), "low")?;
            let close = parse_f64(record.get(ci.close), "close")?;
            let volume = parse_f64(record.get(ci.volume), "volume")?;

            self.data.push(OhlcvData {
                date,
                timestamp,
                open,
                high,
                low,
                close,
                volume,
            });
        }

        Ok(())
    }

    /// Register an indicator that consumes close prices and outputs a scalar.
    ///
    /// Returns `&mut self` to allow chaining.
    pub fn add_close_indicator(&mut self, name: &str, indicator: CloseIndicator) -> &mut Self {
        self.close_indicators.insert(name.to_string(), indicator);
        self
    }

    /// Register an indicator that consumes [`Candle`] data and outputs a scalar.
    ///
    /// Returns `&mut self` to allow chaining.
    pub fn add_candle_indicator(&mut self, name: &str, indicator: CandleIndicator) -> &mut Self {
        self.candle_indicators.insert(name.to_string(), indicator);
        self
    }

    /// Calculate every registered indicator on the loaded data.
    pub fn calculate_indicators(&mut self) -> Result<(), CsvError> {
        if self.data.is_empty() {
            return Err(CsvError::MissingData);
        }

        let len = self.data.len();
        let candles: Vec<Candle> = self.data.iter().map(OhlcvData::to_candle).collect();
        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        for (name, indicator) in self.close_indicators.iter_mut() {
            let values = indicator.calculate(&closes)?;
            self.calculated_values
                .insert(name.clone(), align_to_len(values, len));
        }

        for (name, indicator) in self.candle_indicators.iter_mut() {
            let values = indicator.calculate(&candles)?;
            self.calculated_values
                .insert(name.clone(), align_to_len(values, len));
        }

        Ok(())
    }

    /// Calculate indicators and export the augmented data to `path`.
    pub fn calculate_and_export<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CsvError> {
        self.calculate_indicators()?;
        self.export_to_file(path)
    }

    /// Export the augmented data (raw OHLCV + indicator columns) to `path`.
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CsvError> {
        let file = File::create(path)?;
        self.export_to_writer(file)
    }

    /// Export the augmented data to any writer.
    pub fn export_to_writer<W: Write>(&self, writer: W) -> Result<(), CsvError> {
        let mut wtr = WriterBuilder::new()
            .delimiter(self.config.delimiter)
            .from_writer(writer);

        let mut header = vec![
            "Date".to_string(),
            "Open".to_string(),
            "High".to_string(),
            "Low".to_string(),
            "Close".to_string(),
            "Volume".to_string(),
        ];
        for name in self.calculated_values.keys() {
            header.push(name.clone());
        }
        wtr.write_record(&header)?;

        for (i, data) in self.data.iter().enumerate() {
            let mut row = vec![
                data.date.clone(),
                data.open.to_string(),
                data.high.to_string(),
                data.low.to_string(),
                data.close.to_string(),
                data.volume.to_string(),
            ];
            for values in self.calculated_values.values() {
                row.push(match values.get(i).copied().flatten() {
                    Some(v) => v.to_string(),
                    None => String::new(),
                });
            }
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        Ok(())
    }

    /// Borrow the loaded OHLCV data.
    pub fn data(&self) -> &[OhlcvData] {
        &self.data
    }

    /// Borrow the calculated values for the indicator registered as `name`.
    pub fn indicator_values(&self, name: &str) -> Option<&Vec<Option<f64>>> {
        self.calculated_values.get(name)
    }
}

fn parse_f64(value: Option<&str>, field: &str) -> Result<f64, CsvError> {
    value
        .unwrap_or("0")
        .parse::<f64>()
        .map_err(|e| CsvError::Parse(format!("Failed to parse {field}: {e}")))
}

fn parse_timestamp(date: &str, format: &str) -> Result<u64, CsvError> {
    if date.is_empty() {
        return Ok(0);
    }
    let parsed = NaiveDate::parse_from_str(date, format)
        .map_err(|e| CsvError::Parse(format!("Failed to parse date '{date}': {e}")))?;
    let dt = parsed
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| CsvError::Parse(format!("Invalid date '{date}'")))?;
    Ok(dt.and_utc().timestamp() as u64)
}

/// Right-align indicator output with the source data: indicators that need a
/// warmup period emit fewer values than the input length, so we left-pad with
/// `None`.
fn align_to_len(values: Vec<f64>, len: usize) -> Vec<Option<f64>> {
    let pad = len.saturating_sub(values.len());
    let mut out = Vec::with_capacity(len);
    out.extend(std::iter::repeat_n(None, pad));
    out.extend(values.into_iter().map(Some));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indicators::trend::SimpleMovingAverage;
    use crate::indicators::volatility::AverageTrueRange;

    fn sample_csv() -> &'static str {
        "Date,Open,High,Low,Close,Volume\n\
         2024-01-01,10,12,9,11,1000\n\
         2024-01-02,11,13,10,12,1100\n\
         2024-01-03,12,14,11,13,1200\n\
         2024-01-04,13,15,12,14,1300\n\
         2024-01-05,14,16,13,15,1400\n"
    }

    #[test]
    fn loads_data_and_parses_timestamp() {
        let mut f = CsvFormatter::new();
        f.load_from_reader(sample_csv().as_bytes()).unwrap();
        assert_eq!(f.data().len(), 5);
        assert!(f.data()[0].timestamp > 0);
        assert_eq!(f.data()[0].close, 11.0);
    }

    #[test]
    fn calculates_close_and_candle_indicators() {
        let mut f = CsvFormatter::new();
        f.load_from_reader(sample_csv().as_bytes()).unwrap();
        f.add_close_indicator("SMA3", Box::new(SimpleMovingAverage::new(3).unwrap()))
            .add_candle_indicator("ATR3", Box::new(AverageTrueRange::new(3).unwrap()));
        f.calculate_indicators().unwrap();

        let sma = f.indicator_values("SMA3").unwrap();
        assert_eq!(sma.len(), 5);
        assert!(sma[0].is_none() && sma[1].is_none());
        assert_eq!(sma[2], Some(12.0));
        assert_eq!(sma[3], Some(13.0));
        assert_eq!(sma[4], Some(14.0));

        let atr = f.indicator_values("ATR3").unwrap();
        assert_eq!(atr.len(), 5);
        assert!(atr.last().unwrap().is_some());
    }

    #[test]
    fn calculate_without_data_errors() {
        let mut f = CsvFormatter::new();
        let err = f.calculate_indicators().unwrap_err();
        assert!(matches!(err, CsvError::MissingData));
    }

    #[test]
    fn round_trips_through_export() {
        let mut f = CsvFormatter::new();
        f.load_from_reader(sample_csv().as_bytes()).unwrap();
        f.add_close_indicator("SMA3", Box::new(SimpleMovingAverage::new(3).unwrap()));
        f.calculate_indicators().unwrap();

        let mut out = Vec::new();
        f.export_to_writer(&mut out).unwrap();
        let exported = String::from_utf8(out).unwrap();
        assert!(exported.contains("SMA3"));
        assert!(exported.contains("2024-01-03"));
    }
}
