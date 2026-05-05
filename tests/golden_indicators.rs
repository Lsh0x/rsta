//! Reference-data tests: each indicator is computed against the bundled OHLCV
//! sample and compared against a golden CSV in `tests/data/`.

mod common;

use common::{assert_matches_golden, load_golden, load_sample, DEFAULT_TOL};
use rsta::indicators::trend::Sma;
use rsta::indicators::Indicator;

#[test]
fn sma_5_matches_golden() {
    let candles = load_sample();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut sma = Sma::new(5).unwrap();
    let produced = <Sma as Indicator<f64, f64>>::calculate(&mut sma, &closes).unwrap();

    let golden = load_golden("golden_sma_5.csv");
    assert_matches_golden(closes.len(), &produced, &golden, DEFAULT_TOL);
}

#[test]
fn sma_5_candle_path_matches_f64_path() {
    let candles = load_sample();

    let mut sma_a = Sma::new(5).unwrap();
    let from_candles =
        <Sma as Indicator<rsta::indicators::Candle, f64>>::calculate(&mut sma_a, &candles).unwrap();

    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let mut sma_b = Sma::new(5).unwrap();
    let from_closes = <Sma as Indicator<f64, f64>>::calculate(&mut sma_b, &closes).unwrap();

    assert_eq!(from_candles, from_closes);
}

#[test]
fn indicator_trait_metadata() {
    let sma = Sma::new(20).unwrap();
    let s: &dyn Indicator<f64, f64> = &sma;
    // The trait default uses `type_name`, which yields "Sma" here.
    assert_eq!(s.name(), "Sma");
    // Sma doesn't override period(); default returns None.
    // (Indicators that override it — Wma, Adx, Cci, etc. — return Some(period).)
    assert!(s.period().is_none() || s.period() == Some(20));
}
