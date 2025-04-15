# Cross-over Detection

## Description
Utilities for detecting when indicator lines cross each other or predefined thresholds.

## Value
Crossovers are among the most common signal generation techniques, used with MAs, MACD, Stochastic, and many other indicators.

## Implementation Approach
Implement in `signals/crossover.rs` with functions that can take any two series of indicator values and detect crossing points, returning the index and direction of crosses.

## Category
Signal Generation/Strategy Module
