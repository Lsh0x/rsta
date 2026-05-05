//! Reference-data tests: each indicator is computed against the bundled OHLCV
//! sample and compared against a golden CSV in `tests/data/`.

mod common;

use common::{assert_matches_golden, load_golden, load_sample, DEFAULT_TOL};
use rsta::indicators::trend::SimpleMovingAverage;
use rsta::indicators::Indicator;

#[test]
fn sma_5_matches_golden() {
    let candles = load_sample();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut sma = SimpleMovingAverage::new(5).unwrap();
    let produced = sma.calculate(&closes).unwrap();

    let golden = load_golden("golden_sma_5.csv");
    assert_matches_golden(closes.len(), &produced, &golden, DEFAULT_TOL);
}

#[test]
fn sma_5_candle_helper_matches_f64_path() {
    let candles = load_sample();

    let mut sma_a = SimpleMovingAverage::new(5).unwrap();
    let from_candles = sma_a.calculate_candles(&candles).unwrap();

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let mut sma_b = SimpleMovingAverage::new(5).unwrap();
    let from_closes = sma_b.calculate(&closes).unwrap();

    assert_eq!(from_candles, from_closes);
}

#[test]
fn indicator_trait_metadata() {
    let sma = SimpleMovingAverage::new(20).unwrap();
    let s: &dyn Indicator<f64, f64> = &sma;
    assert_eq!(s.name(), "SMA");
    assert_eq!(s.period(), Some(20));
}
