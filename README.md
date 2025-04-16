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

## Quick Start

Here's a simple example calculating a Simple Moving Average (SMA):

```rust
use rsta::indicators::trend::{SimpleMovingAverage, Indicator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Price data
    let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];

    // Create a 5-period SMA
    let mut sma = SimpleMovingAverage::new(5)?;

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
use rsta::indicators::trend::{ExponentialMovingAverage, Indicator};

// Create a 14-period EMA
let mut ema = ExponentialMovingAverage::new(14)?;
let prices = vec![/* your price data */];
let ema_values = ema.calculate(&prices)?;
```

Available trend indicators:
- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)

### Momentum Indicators

Measure the rate of price changes to identify overbought or oversold conditions.

```rust
use rsta::indicators::momentum::{RSI, Indicator};

// Create a 14-period RSI
let mut rsi = RSI::new(14)?;
let prices = vec![/* your price data */];
let rsi_values = rsi.calculate(&prices)?;

// Identify overbought/oversold conditions
for value in rsi_values {
    if value > 70.0 {
        println!("Overbought: {}", value);
    } else if value < 30.0 {
        println!("Oversold: {}", value);
    }
}
```

Available momentum indicators:
- Relative Strength Index (RSI)
- Stochastic Oscillator
- Williams %R

### Volume Indicators

Analyze trading volume to confirm price movements.

```rust
use rsta::indicators::{Candle, volume::{OnBalanceVolume, Indicator}};

// Create an OBV indicator
let mut obv = OnBalanceVolume::new();

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

// Calculate OBV values
let obv_values = obv.calculate(&candles)?;
```

Available volume indicators:
- On Balance Volume (OBV)
- Volume Rate of Change
- Accumulation/Distribution Line
- Chaikin Money Flow

### Volatility Indicators

Measure market volatility and price dispersion.

```rust
use rsta::indicators::volatility::{BollingerBands, Indicator};

// Create Bollinger Bands with 20-period SMA and 2 standard deviations
let mut bb = BollingerBands::new(20, 2.0)?;
let prices = vec![/* your price data */];
let bb_values = bb.calculate(&prices)?;

// Access the bands
for band in bb_values {
    println!("Middle: {}, Upper: {}, Lower: {}", 
             band.middle, band.upper, band.lower);
             
    // Check if price is outside the bands
    if prices[prices.len() - 1] > band.upper {
        println!("Price above upper band!");
    } else if prices[prices.len() - 1] < band.lower {
        println!("Price below lower band!");
    }
}
```

Available volatility indicators:
- Average True Range (ATR)
- Standard Deviation
- Bollinger Bands
- Keltner Channels

## Usage Patterns and Best Practices

### Batch vs. Real-time Calculation

RSTA supports both batch calculation for historical data and real-time updates:

```rust
use rsta::indicators::trend::{SimpleMovingAverage, Indicator};

// Create indicator
let mut sma = SimpleMovingAverage::new(14)?;

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

Many trading strategies use multiple indicators together:

```rust
use rsta::indicators::momentum::{RSI, Indicator as MomentumIndicator};
use rsta::indicators::volatility::{BollingerBands, Indicator as VolatilityIndicator};

// Create indicators
let mut rsi = RSI::new(14)?;
let mut bb = BollingerBands::new(20, 2.0)?;

// Calculate indicators
let prices = vec![/* price data */];
let rsi_values = rsi.calculate(&prices)?;
let bb_values = bb.calculate(&prices)?;

// Find potential signals
for i in 0..rsi_values.len().min(bb_values.len()) {
    let price_idx = prices.len() - rsi_values.len() + i;
    
    // Potential buy signal: RSI oversold + price below lower band
    if rsi_values[i] < 30.0 && prices[price_idx] < bb_values[i].lower {
        println!("Potential buy signal at index {}", price_idx);
    }
    
    // Potential sell signal: RSI overbought + price above upper band
    if rsi_values[i] > 70.0 && prices[price_idx] > bb_values[i].upper {
        println!("Potential sell signal at index {}", price_idx);
    }
}
```

### Error Handling

All methods that might fail return a `Result` with detailed error information:

```rust
use rsta::indicators::trend::{SimpleMovingAverage, Indicator};
use rsta::indicators::IndicatorError;

// Handle errors explicitly
match SimpleMovingAverage::new(0) {
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
  - `trend`: Moving averages and trend-following indicators
  - `momentum`: Oscillators and momentum indicators
  - `volume`: Volume-based indicators
  - `volatility`: Volatility and dispersion measures

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
