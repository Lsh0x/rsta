# RSTA - Rust Statistical Technical Analysis

A comprehensive Rust library for financial technical analysis indicators, providing efficient and type-safe implementations of popular indicators used in financial markets.

[![GitHub last commit](https://img.shields.io/github/last-commit/lsh0x/rsta)](https://github.com/lsh0x/rsta/commits/main)
[![CI](https://github.com/lsh0x/rsta/workflows/CI/badge.svg)](https://github.com/lsh0x/rsta/actions)
[![Codecov](https://codecov.io/gh/lsh0x/rsta/branch/main/graph/badge.svg)](https://codecov.io/gh/lsh0x/rsta)
[![Docs](https://docs.rs/rsta/badge.svg)](https://docs.rs/rsta)
[![Crates.io](https://img.shields.io/crates/v/rsta.svg)](https://crates.io/crates/rsta)
[![crates.io](https://img.shields.io/crates/d/rsta)](https://crates.io/crates/rsta)

## Overview

RSTA provides robust implementations of technical indicators used for analyzing financial markets and making trading decisions. The library is designed with a focus on performance, type safety, and ease of use.

### Features

- **Comprehensive Indicator Support**: Includes trend, momentum, volume, and volatility indicators
- **Type-Safe API**: Leverages Rust's type system to provide a safe API
- **Performance Optimized**: Efficient algorithms suitable for large datasets
- **Flexible Data Input**: Works with both simple price data and OHLCV (Open, High, Low, Close, Volume) candles
- **Real-time Updates**: Support for both batch calculations and real-time updates
- **Well Documented**: Extensive documentation and examples

## Installation

Add RSTA to your `Cargo.toml`:

```toml
[dependencies]
rsta = "0.0.2"
```

## API Reference

For complete API documentation, please visit [docs.rs/rsta](https://docs.rs/rsta).

Key components:

- **Core Traits**:
  - `Indicator<T, O>`: Common interface for all indicators
  - `PriceDataAccessor<T>`: Uniform access to price data

- **Data Types**:
  - `Candle`: OHLCV price data structure
  - `IndicatorError`: Error types for indicator operations

- **Indicator Categories**:
  - **Trend Indicators**:
    - Simple Moving Average (SMA)
    - Exponential Moving Average (EMA)
    - Moving Average Convergence Divergence (MACD)
  - **Momentum Indicators**:
    - Relative Strength Index (RSI)
    - Stochastic Oscillator
    - Williams %R

  - **Volume Indicators**:
    - On Balance Volume (OBV)
    - Chaikin Money Flow (CMF)
    - Accumulation/Distribution Line (ADL)
    - Volume Rate of Change (VROC)

  - **Volatility Indicators**:
    - Standard Deviation (STD)
    - Average True Range (ATR)
    - Bollinger Bands (BB)
    - Keltner Channels

## Quick Start

Here's a simple example calculating a Simple Moving Average (SMA):

```rust
use rsta::indicators::trend::Sma;
use rsta::indicators::Indicator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Price data
    let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];

    // Create a 5-period SMA
    let mut sma = Sma::new(5)?;

    // Calculate SMA values
    let sma_values = sma.calculate(&prices)?;

    println!("SMA values: {:?}", sma_values);
    Ok(())
}
```

## Indicator Categories

RSTA organizes indicators into four main categories:

### Trend Indicators

Track the direction of price movements over time.

```rust
use rsta::indicators::trend::Sma;
use rsta::indicators::Indicator;

// Create a 14-period SMA
let mut sma = Sma::new(14)?;
let prices = vec![/* your price data */];
let sma_values = sma.calculate(&prices)?;
```

Available trend indicators:
- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)
- Moving Average Convergence Divergence (MACD)

### Momentum Indicators

Measure the rate of price changes to identify overbought or oversold conditions.

```rust
use rsta::indicators::momentum::Rsi;
use rsta::indicators::Indicator;

// Create a 14-period RSI
let mut rsi = Rsi::new(14)?;
let prices = vec![/* your price data */];
let rsi_values = rsi.calculate(&prices)?;
```

Available momentum indicators:
- Relative Strength Index (RSI)
- Williams %R
- Stochastic Oscillator

### Volume Indicators

Analyze trading volume to confirm price movements.

```rust
use rsta::indicators::volume::Obv;
use rsta::indicators::Indicator;
use rsta::indicators::Candle;

// Create price data with OHLCV values
let candles = vec![
    Candle {
        timestamp: 1618185600,
        open: 100.0, high: 105.0, low: 99.0, close: 103.0, volume: 1000.0
    },
    Candle {
        timestamp: 1618272000,
        open: 103.0, high: 106.0, low: 102.0, close: 105.0, volume: 1200.0
    },
    // More candles...
];

// Create and calculate OBV
let mut obv = Obv::new()?;
let obv_values = obv.calculate(&candles)?;
```

Available volume indicators:
- On Balance Volume (OBV)
- Chaikin Money Flow (CMF)
- Accumulation/Distribution Line (ADL)
- Volume Rate of Change (VROC)

### Volatility Indicators

Measure market volatility and price dispersion.

```rust
use rsta::indicators::volatility::Std;
use rsta::indicators::Indicator;

// Create a 20-period Standard Deviation indicator
let mut std_dev = Std::new(20)?;
let prices = vec![/* your price data */];
let std_values = std_dev.calculate(&prices)?;

// Standard Deviation values
for value in std_values {
    println!("Standard Deviation: {}", value);
    
    // Check if volatility is high
    if value > 2.0 {
        println!("High volatility detected!");
    }
}
```

Available volatility indicators:
- Standard Deviation (STD)
- Average True Range (ATR)
- Bollinger Bands (BB)
- Keltner Channels

## Usage Patterns and Best Practices

### Batch vs. Real-time Calculation

RSTA supports both batch calculation for historical data and real-time updates:

```rust
use rsta::indicators::trend::Sma;
use rsta::indicators::Indicator;

// Create indicator
let mut sma = Sma::new(14)?;

// Batch calculation
let historical_prices = vec![/* historical data */];
let historical_sma = sma.calculate(&historical_prices)?;

// Reset state for real-time updates
sma.reset();

// Real-time updates
let new_price = 105.0;
if let Some(new_sma) = sma.next(new_price)? {
    println!("New SMA: {}", new_sma);
}
```

### Working with OHLCV Data

Some indicators require full OHLCV data using the `Candle` struct:

```rust
use rsta::indicators::Candle;

// Create a candle with OHLCV data
let candle = Candle {
    timestamp: 1618185600, // Unix timestamp
    open: 100.0,
    high: 105.0,
    low: 98.0,
    close: 103.0,
    volume: 1500.0,
};
```

### Combining Indicators for Trading Strategies

Many trading strategies use multiple indicators together. Once more indicators are implemented, 
you can combine them for complex trading strategies.

```rust
use rsta::indicators::trend::Sma;
use rsta::indicators::volatility::Std;
use rsta::indicators::Indicator;

// Create indicators
let mut sma = Sma::new(20)?;
let mut std_dev = Std::new(20)?;

// Calculate indicators
let prices = vec![/* price data */];
let sma_values = sma.calculate(&prices)?;
let std_values = std_dev.calculate(&prices)?;

// Analyze results (simple example)
for i in 0..sma_values.len().min(std_values.len()) {
    let price_idx = prices.len() - sma_values.len() + i;
    let current_price = prices[price_idx];
    
    // Simple volatility-based strategy
    if current_price > sma_values[i] && std_values[i] < 1.0 {
        println!("Low volatility uptrend at index {}", price_idx);
    } else if current_price < sma_values[i] && std_values[i] > 2.0 {
        println!("High volatility downtrend at index {}", price_idx);
    }
}
```

### Error Handling

All methods that might fail return a `Result` with detailed error information:

```rust
use rsta::indicators::trend::Sma;
use rsta::indicators::IndicatorError;

// Handle errors explicitly
match Sma::new(0) {
    Ok(sma) => {
        // Use the SMA
    },
    Err(IndicatorError::InvalidParameter(msg)) => {
        eprintln!("Invalid parameter: {}", msg);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Contributing

Contributions are welcome! Here's how you can help:

1. **Add New Indicators**: Implement additional technical indicators
2. **Improve Performance**: Optimize existing implementations
3. **Add Tests**: Increase test coverage and add test cases
4. **Enhance Documentation**: Improve examples and usage documentation
5. **Report Issues**: Report bugs or suggest features

Please follow these steps to contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-new-indicator`)
3. Commit your changes (`git commit -am 'Add a new indicator'`)
4. Push to the branch (`git push origin feature/my-new-indicator`)
5. Create a new Pull Request

## License

This project is licensed under the GPL-3.0 License - see the LICENSE file for details.
