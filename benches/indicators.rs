//! Microbenchmarks for individual indicators on a 100k-bar synthetic
//! series. Reports throughput as bars/second.
//!
//! Run with:
//! ```text
//! cargo bench --bench indicators
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rsta::indicators::momentum::{Cci, Rsi, StochasticOscillator};
use rsta::indicators::trend::{Adx, Ema, Macd, Sma};
use rsta::indicators::volatility::{Atr, BollingerBands};
use rsta::indicators::volume::{Mfi, Obv};
use rsta::indicators::{Candle, Indicator};

const N: usize = 100_000;

fn synthetic_closes(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let t = i as f64;
            // Smooth wave + linear drift — gives the indicators something
            // non-trivial to chew on without dragging in `rand`.
            100.0 + (t * 0.01).sin() * 20.0 + t * 0.001
        })
        .collect()
}

fn synthetic_candles(n: usize) -> Vec<Candle> {
    let closes = synthetic_closes(n);
    closes
        .iter()
        .enumerate()
        .map(|(i, &c)| Candle {
            timestamp: i as u64,
            open: c - 0.25,
            high: c + 0.5,
            low: c - 0.5,
            close: c,
            volume: 1_000.0 + (i as f64).cos() * 100.0,
        })
        .collect()
}

fn close_indicators(c: &mut Criterion) {
    let closes = synthetic_closes(N);
    let mut group = c.benchmark_group("close_indicators");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("sma_20", |b| {
        b.iter(|| {
            let mut sma = Sma::new(20).unwrap();
            black_box(<Sma as Indicator<f64, f64>>::calculate(&mut sma, &closes).unwrap())
        })
    });
    group.bench_function("ema_20", |b| {
        b.iter(|| {
            let mut ema = Ema::new(20).unwrap();
            black_box(<Ema as Indicator<f64, f64>>::calculate(&mut ema, &closes).unwrap())
        })
    });
    group.bench_function("rsi_14", |b| {
        b.iter(|| {
            let mut rsi = Rsi::new(14).unwrap();
            black_box(rsi.calculate(&closes).unwrap())
        })
    });
    group.bench_function("bb_20_2", |b| {
        b.iter(|| {
            let mut bb = BollingerBands::new(20, 2.0).unwrap();
            black_box(bb.calculate(&closes).unwrap())
        })
    });
    group.bench_function("macd_12_26_9", |b| {
        b.iter(|| {
            let mut macd = Macd::new(12, 26, 9).unwrap();
            black_box(macd.calculate(&closes).unwrap())
        })
    });
    group.bench_function("cci_20", |b| {
        b.iter(|| {
            // Cci needs candles; reuse the synthetic dataset.
            let candles = synthetic_candles(N);
            let mut cci = Cci::new(20).unwrap();
            black_box(cci.calculate(&candles).unwrap())
        })
    });
    group.finish();
}

fn candle_indicators(c: &mut Criterion) {
    let candles = synthetic_candles(N);
    let mut group = c.benchmark_group("candle_indicators");
    group.throughput(Throughput::Elements(N as u64));

    group.bench_function("atr_14", |b| {
        b.iter(|| {
            let mut atr = Atr::new(14).unwrap();
            black_box(atr.calculate(&candles).unwrap())
        })
    });
    group.bench_function("obv", |b| {
        b.iter(|| {
            let mut obv = Obv::new();
            black_box(obv.calculate(&candles).unwrap())
        })
    });
    group.bench_function("mfi_14", |b| {
        b.iter(|| {
            let mut mfi = Mfi::new(14).unwrap();
            black_box(mfi.calculate(&candles).unwrap())
        })
    });
    group.bench_function("adx_14", |b| {
        b.iter(|| {
            let mut adx = Adx::new(14).unwrap();
            black_box(adx.calculate(&candles).unwrap())
        })
    });
    group.bench_function("stoch_14_3", |b| {
        b.iter(|| {
            let mut s = StochasticOscillator::new(14, 3).unwrap();
            black_box(s.calculate(&candles).unwrap())
        })
    });
    group.finish();
}

criterion_group!(benches, close_indicators, candle_indicators);
criterion_main!(benches);
