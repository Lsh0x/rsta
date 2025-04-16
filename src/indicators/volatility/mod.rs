//! Volatility indicators
//!
//! This module contains volatility indicators like ATR, Bollinger Bands, and Keltner Channel

pub mod atr;
pub mod bb;
pub mod std;
pub mod keltner_channels;

pub use self::atr::ATR;
pub use self::bb::BB;
pub use self::std::STD;
pub use self::keltner_channels::{KeltnerChannels, KeltnerChannelsResult};