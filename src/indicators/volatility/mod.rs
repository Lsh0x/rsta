//! Volatility indicators
//!
//! This module contains volatility indicators like ATR, Bollinger Bands,
//! Keltner Channels, Donchian Channels, and Standard Deviation.

pub mod atr;
pub mod bb;
pub mod donchian;
pub mod keltner_channels;
pub mod std;

pub use self::atr::Atr;
pub use self::bb::{BollingerBands, BollingerBandsResult};
pub use self::donchian::{Donchian, DonchianResult};
pub use self::keltner_channels::{KeltnerChannels, KeltnerChannelsResult};
pub use self::std::Std;
