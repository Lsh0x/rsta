//! Volatility indicators
//!
//! This module contains volatility indicators like ATR, Bollinger Bands, and Keltner Channel

pub mod atr;
pub mod bb;
pub mod keltner_channels;
pub mod std;

pub use self::atr::ATR;
pub use self::bb::BB;
pub use self::keltner_channels::{KeltnerChannels, KeltnerChannelsResult};
pub use self::std::STD;
