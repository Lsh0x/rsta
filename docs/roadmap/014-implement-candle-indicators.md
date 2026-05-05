# Implement Indicator<Candle, T> for Each Indicator

## Description
Extend existing indicators to support working directly with Candle data, rather than just numerical (`f64`) values. This implementation will allow indicators to be calculated directly from candlestick data without preprocessing.

## Value
- Simplifies use of indicators with standard OHLCV datasets
- Enhances flexibility by allowing indicators to use different price components (open, high, low, close)
- Provides a more consistent API across all indicators
- Reduces code duplication in client applications by eliminating the need to extract price data

## Implementation Approach
1. **Target Indicators**: Implement `Indicator<Candle, T>` trait for the following indicators:
   - SimpleMovingAverage
   - ExponentialMovingAverage
   - STD (Standard Deviation)
   - Add more indicators incrementally as needed

2. **Implementation Details**:
   - Enable indicators to work directly with Candle input data
   - Use the `PriceDataAccessor` trait to access price components (open, high, low, close, volume)
   - Default to using the close price when appropriate
   - Add configuration options to allow users to specify which price component to use (close, open, high, low)
   - Ensure dual support for both `f64` and `Candle` input types

3. **Consistency**: Maintain consistency with existing indicators that already support Candle input (such as OnBalanceVolume, ChaikinMoneyFlow, and AccumulationDistributionLine)

4. **Documentation**:
   - Update documentation for all modified indicators
   - Add clear examples demonstrating how to use indicators with Candle data
   - Document configuration options for selecting price components

5. **Testing**:
   - Add comprehensive unit tests for the new Candle-based implementations
   - Cover edge cases and verify calculation accuracy
   - Compare results with existing f64-based calculations to ensure consistency

## Category
Technical Indicators

