# TODO: Enhancements for Tars Library

## 1. Pattern Recognition Module

### Candlestick Pattern Recognition
**Description:** Implement algorithms to identify common candlestick patterns that signal potential market reversals or continuation.

**Value:** Candlestick patterns are widely used by traders to make decisions. Adding this functionality would enable users to automatically identify these patterns rather than visually scanning charts.

**Implementation approach:** Create a new module `patterns/candlestick.rs` with a trait-based system similar to the existing indicators. Each pattern (like Doji, Hammer, etc.) would implement a common trait with methods to detect the pattern in a series of candles.

**Examples:**
- Doji (open and close nearly equal)
- Hammer (small body, long lower wick, little/no upper wick)
- Shooting Star (small body, long upper wick, little/no lower wick)
- Engulfing patterns (bullish/bearish)
- Morning/Evening Star
- Harami

### Chart Pattern Detection
**Description:** Implement algorithms to identify larger technical chart patterns that form over many candles.

**Value:** Chart patterns often indicate major trend reversals or continuations and are key components of technical analysis. This would enhance the strategic capabilities of the library.

**Implementation approach:** Create a `patterns/chart.rs` module with algorithms that analyze price movements over longer periods. These would likely need more sophisticated detection methods using statistical techniques or possibly machine learning approaches for pattern recognition.

**Examples:**
- Head and Shoulders (and inverse)
- Double Top/Bottom
- Triangle patterns (ascending, descending, symmetrical)
- Flag and Pennant patterns
- Cup and Handle

## 2. Additional Indicators

### Ichimoku Cloud
**Description:** A comprehensive indicator system that includes multiple components (Tenkan-sen, Kijun-sen, Senkou Span A, Senkou Span B, and Chikou Span).

**Value:** Provides information about support/resistance, trend direction, momentum, and potential signals in one visualization. Popular in forex and cryptocurrency trading.

**Implementation approach:** Create a new indicator in `trend.rs` that calculates all five components and returns them as a composite result type.

### MACD (Moving Average Convergence Divergence)
**Description:** A trend-following momentum indicator showing the relationship between two moving averages of a security's price.

**Value:** One of the most popular indicators for identifying trend direction, strength, momentum, and potential reversals.

**Implementation approach:** Implement in `trend.rs`, building on the existing EMA functionality. MACD consists of the MACD line (difference between fast and slow EMAs), signal line (EMA of the MACD line), and histogram (difference between MACD and signal lines).

### Parabolic SAR
**Description:** A stop-and-reverse indicator used to determine potential reversals in market trend.

**Value:** Provides trailing stop levels that adjust with price movement, useful for setting stop-loss points and identifying trend changes.

**Implementation approach:** Implement in `trend.rs` using the classic Wilder's formula that accelerates the SAR value as the trend strengthens.

### Money Flow Index (MFI)
**Description:** A momentum indicator that incorporates both price and volume to measure buying and selling pressure.

**Value:** Often called "volume-weighted RSI," it helps identify overbought/oversold conditions while accounting for volume, which can indicate stronger signals than price-only indicators.

**Implementation approach:** Add to `momentum.rs` using calculation based on typical price multiplied by volume to determine positive and negative money flow over a period.

### Commodity Channel Index (CCI)
**Description:** An oscillator that measures the current price level relative to an average price level over a specified period.

**Value:** Identifies cyclical trends in commodities and securities. Can be used to spot overbought/oversold conditions and divergences.

**Implementation approach:** Implement in `momentum.rs` using the formula that compares typical price to a simple moving average of typical prices and scaling by a constant.

## 3. Signal Generation/Strategy Module

### Trading Signal Generation
**Description:** A framework for converting indicator values and patterns into actionable trading signals.

**Value:** Bridges the gap between technical analysis and automated trading decisions, allowing users to define clear rules for entry and exit points.

**Implementation approach:** Create a new module `signals/mod.rs` with a trait-based system for signal generators. These would take indicators as input and output standardized signal types (Buy, Sell, Neutral) with confidence levels.

### Cross-over Detection
**Description:** Utilities for detecting when indicator lines cross each other or predefined thresholds.

**Value:** Crossovers are among the most common signal generation techniques, used with MAs, MACD, Stochastic, and many other indicators.

**Implementation approach:** Implement in `signals/crossover.rs` with functions that can take any two series of indicator values and detect crossing points, returning the index and direction of crosses.

### Divergence Detection
**Description:** Tools to identify when price movement diverges from indicator movement.

**Value:** Divergences often signal potential trend reversals and are considered strong technical signals, particularly with momentum oscillators.

**Implementation approach:** Create `signals/divergence.rs` with algorithms that can compare price highs/lows with corresponding indicator highs/lows to detect regular and hidden divergences.

## 4. Backtesting Framework

### Basic Backtesting System
**Description:** A system to evaluate trading strategies on historical data.

**Value:** Essential for testing strategy effectiveness before deploying with real capital. Allows optimization of parameters and comparison of different approaches.

**Implementation approach:** Create a new module `backtest/mod.rs` with a framework for running strategies against historical data, tracking trade entry/exit points, and calculating performance.

### Position Sizing and Risk Management
**Description:** Utilities for determining optimal position sizes based on account size and risk tolerance.

**Value:** Proper position sizing is critical for long-term trading success and preventing account drawdowns.

**Implementation approach:** Add `backtest/position.rs` with functions implementing various position sizing methods (fixed fractional, Kelly criterion, etc.) and risk management techniques.

### Performance Metrics
**Description:** Calculations for standard trading performance statistics.

**Value:** Provides objective measures to evaluate and compare strategy performance beyond simple profit/loss.

**Implementation approach:** Create `backtest/performance.rs` with functions to calculate metrics like Sharpe ratio, maximum drawdown, win rate, profit factor, and expectancy from a series of trades.

## Implementation Priority

These enhancements are listed in rough order of priority, but MACD would be an excellent first addition as it's widely used and builds on the existing EMA functionality in the library.
