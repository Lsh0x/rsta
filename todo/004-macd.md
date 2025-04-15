# MACD (Moving Average Convergence Divergence)

## Description
A trend-following momentum indicator showing the relationship between two moving averages of a security's price.

## Value
One of the most popular indicators for identifying trend direction, strength, momentum, and potential reversals.

## Implementation Approach
Implement in `trend.rs`, building on the existing EMA functionality. MACD consists of the MACD line (difference between fast and slow EMAs), signal line (EMA of the MACD line), and histogram (difference between MACD and signal lines).

## Priority
High priority - recommended as first implementation due to building on existing EMA functionality.

## Category
Additional Indicators
