//! Candlestick and chart pattern recognition.
//!
//! Currently exposes [`candlestick`] — geometric detection of common
//! 1-, 2-, and 3-bar candle patterns (Doji, Hammer, Engulfing, Morning
//! Star, Three White Soldiers, …).
//!
//! Chart pattern detection (head & shoulders, triangles, flags) is on
//! the roadmap but not yet implemented; see
//! [`todo/002-chart-pattern-detection.md`](https://github.com/Lsh0x/rsta/blob/main/todo/002-chart-pattern-detection.md).

pub mod candlestick;
