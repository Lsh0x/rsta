//! Volatility indicators
//!
//! This module contains volatility indicators like ATR, Bollinger Bands, and Keltner Channel

pub mod atr;
pub mod bb;
pub mod keltner_channels;
pub mod std;

pub use self::atr::Atr;
pub use self::bb::{BollingerBands, BollingerBandsResult};
pub use self::keltner_channels::{KeltnerChannels, KeltnerChannelsResult};
pub use self::std::Std;
