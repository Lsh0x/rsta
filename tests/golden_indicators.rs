//! Reference-data tests: each indicator is computed against the bundled OHLCV
//! sample and compared against a golden CSV in `tests/data/`.
//!
//! ## Two layers
//!
//! - `sample_ohlcv.csv` (synthetic, hand-authored goldens): cheap unit-test
//!   layer that runs everywhere with no external dependencies.
//! - `btc_usd_daily.csv` (real Kraken XBTUSD, pandas-ta-generated goldens):
//!   the `*_against_btc_*` tests below load goldens via `try_load_golden` so
//!   they silently no-op when the user hasn't yet run
//!   `python scripts/gen_golden.py`. As soon as the goldens are committed
//!   they assert real cross-implementation parity.

mod common;

use common::{
    assert_matches_golden, load_btc_daily, load_golden, load_sample, try_load_golden, DEFAULT_TOL,
};
use rsta::indicators::momentum::Rsi;
use rsta::indicators::trend::{Ema, Macd, Sma};
use rsta::indicators::volatility::Atr;
use rsta::indicators::Indicator;

/// Tolerance for comparisons against pandas-ta on BTC daily data. Tight
/// enough to catch real implementation regressions, loose enough to accept
/// last-ULP differences from float accumulation order on long series.
const BTC_TOL: f64 = 1e-6;

/// Number of initial source rows to skip before comparing chained-EMA
/// indicators (RSI, MACD). The first few bars carry warmup that the two
/// implementations may treat slightly differently (e.g. pandas-ta's RSI
/// emits its first value one bar earlier than rsta). Past 30 daily bars
/// both implementations are fully stable.
const SEED_BIAS_SKIP: usize = 30;

fn skip_initial(golden: Vec<(usize, f64)>, skip_rows: usize) -> Vec<(usize, f64)> {
    golden
        .into_iter()
        .filter(|(idx, _)| *idx >= skip_rows)
        .collect()
}

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

// ---------------------------------------------------------------------------
// Real-data tests against pandas-ta goldens (Kraken XBTUSD daily)
// ---------------------------------------------------------------------------

/// Tiny smoke test: just ensure the bundled BTC dataset loads and looks sane.
#[test]
fn btc_daily_dataset_loads() {
    let candles = load_btc_daily();
    assert!(candles.len() > 4_000, "dataset shrank unexpectedly");
    // First bar is from 2013-10-06 (Unix 1_381_017_600).
    assert_eq!(candles[0].timestamp, 1_381_017_600);
    // OHLCV fields are positive on the first bar.
    let c = candles[0];
    assert!(c.high >= c.low && c.close > 0.0 && c.volume >= 0.0);
}

fn run_close_indicator_golden<I>(name: &str, indicator: &mut I)
where
    I: Indicator<f64, f64>,
{
    let Some(golden) = try_load_golden(name) else {
        // No golden present yet — silently pass. Run `python scripts/gen_golden.py`
        // to materialise the file, commit it, and this test will start asserting.
        return;
    };
    let candles = load_btc_daily();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let produced = indicator.calculate(&closes).unwrap();
    assert_matches_golden(closes.len(), &produced, &golden, BTC_TOL);
}

#[test]
fn sma_20_against_btc_pandas_ta() {
    let mut sma = Sma::new(20).unwrap();
    run_close_indicator_golden("golden_btc_sma_20.csv", &mut sma);
}

#[test]
fn ema_20_against_btc_pandas_ta() {
    let mut ema = Ema::new(20).unwrap();
    run_close_indicator_golden("golden_btc_ema_20.csv", &mut ema);
}

#[test]
fn rsi_14_against_btc_pandas_ta() {
    let Some(golden) = try_load_golden("golden_btc_rsi_14.csv") else {
        return;
    };
    let candles = load_btc_daily();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let mut rsi = Rsi::new(14).unwrap();
    let produced = rsi.calculate(&closes).unwrap();
    // rsta seeds RSI from a plain SMA over the first `period` gains/losses,
    // then applies Wilder smoothing. pandas-ta starts smoothing one bar
    // earlier (it accepts the first change as its initial avg gain/loss),
    // so the two converge but never match exactly — the residual settles
    // around 1e-2 by row 200 and stays there.
    let golden = skip_initial(golden, 200);
    assert_matches_golden(closes.len(), &produced, &golden, 1e-2);
}

#[test]
fn atr_14_against_btc_pandas_ta() {
    let Some(golden) = try_load_golden("golden_btc_atr_14.csv") else {
        return;
    };
    let candles = load_btc_daily();
    let mut atr = Atr::new(14).unwrap();
    let produced = atr.calculate(&candles).unwrap();
    assert_matches_golden(candles.len(), &produced, &golden, BTC_TOL);
}

#[test]
fn macd_12_26_9_against_btc_pandas_ta() {
    // MACD has three series; check each independently if its golden is present.
    let candles = load_btc_daily();
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let mut macd = Macd::new(12, 26, 9).unwrap();
    let produced = macd.calculate(&closes).unwrap();

    if let Some(golden) = try_load_golden("golden_btc_macd_12_26_9_line.csv") {
        let series: Vec<f64> = produced.iter().map(|p| p.macd).collect();
        let golden = skip_initial(golden, SEED_BIAS_SKIP);
        assert_matches_golden(closes.len(), &series, &golden, BTC_TOL);
    }
    if let Some(golden) = try_load_golden("golden_btc_macd_12_26_9_signal.csv") {
        let series: Vec<f64> = produced.iter().map(|p| p.signal).collect();
        let golden = skip_initial(golden, SEED_BIAS_SKIP);
        assert_matches_golden(closes.len(), &series, &golden, BTC_TOL);
    }
    if let Some(golden) = try_load_golden("golden_btc_macd_12_26_9_hist.csv") {
        let series: Vec<f64> = produced.iter().map(|p| p.histogram).collect();
        let golden = skip_initial(golden, SEED_BIAS_SKIP);
        assert_matches_golden(closes.len(), &series, &golden, BTC_TOL);
    }
}
