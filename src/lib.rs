//! # Tars
//!
//! A Rust library for technical analysis and trading strategies.
//!
//! ## Features
//!
//! - Technical indicators for price analysis
//! - Support for both price and volume-based indicators
//! - Customizable parameters for each indicator
//! - Error handling for invalid parameters and insufficient data
//! - Current version: 0.0.1
//!
//! ## Quick Start
//!
//! ```rust
//! use tars::indicators::Indicator;
//! use tars::indicators::Candle;
//! use tars::indicators::trend::SimpleMovingAverage;
//!
//! // Create a Simple Moving Average indicator
//! let mut sma = SimpleMovingAverage::new(5).unwrap();
//!
//! // Create some price data
//! let prices = vec![42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0];
//!
//! // Calculate SMA values
//! let sma_values = sma.calculate(&prices).unwrap();
//! println!("SMA values: {:?}", sma_values);
//! ```
//!
//! For more examples and detailed documentation, please refer to the individual indicator modules.

/// Re-exports all indicator modules
pub mod indicators;

// Re-export key types for convenience
pub use indicators::Candle;
pub use indicators::Indicator;
pub use indicators::IndicatorError;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
