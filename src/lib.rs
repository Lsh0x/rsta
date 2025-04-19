//! # RSTA
//!
//! A Rust library for technical analysis and trading strategies.
//!
//! ## Features
//!
//! - Momentum indicators: Rsi, StochasticOscillator, WilliamsR
//! - Trend indicators: Ema, Sma
//! - Volatility indicators: Atr, BollingerBands, KeltnerChannels, Std
//! - Volume indicators: Adl, Cmf, Obv, Vroc
//! - Consistent naming and export patterns across all indicators
//! - Comprehensive documentation and examples
//! - Error handling for invalid parameters and insufficient data
//! - Current version: 0.0.2
//!
//! ## Quick Start
//!
//! ```rust
//! use rsta::indicators::Indicator;
//! use rsta::indicators::Candle;
//! use rsta::indicators::Sma;
//!
//! // Create a Simple Moving Average indicator
//! let mut sma = Sma::new(5).unwrap();
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
