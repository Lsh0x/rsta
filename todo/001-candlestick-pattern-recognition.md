# Candlestick Pattern Recognition

## Description
Implement algorithms to identify common candlestick patterns that signal potential market reversals or continuation.

## Value
Candlestick patterns are widely used by traders to make decisions. Adding this functionality would enable users to automatically identify these patterns rather than visually scanning charts.

## Implementation Approach
Create a new module `patterns/candlestick.rs` with a trait-based system similar to the existing indicators. Each pattern (like Doji, Hammer, etc.) would implement a common trait with methods to detect the pattern in a series of candles.

## Examples
- Doji (open and close nearly equal)
- Hammer (small body, long lower wick, little/no upper wick)
- Shooting Star (small body, long upper wick, little/no lower wick)
- Engulfing patterns (bullish/bearish)
- Morning/Evening Star
- Harami

## Category
Pattern Recognition Module
