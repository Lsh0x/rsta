//! Shared helpers for golden-data based integration tests.
//!
//! Loads the bundled `tests/data/sample_ohlcv.csv` dataset, parses indicator
//! reference outputs (one column per indicator, indexed against the source
//! data), and compares with a configurable tolerance.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use rsta::indicators::Candle;

/// Default absolute tolerance for `f64` comparisons.
pub const DEFAULT_TOL: f64 = 1e-9;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

/// Load the bundled OHLCV sample as a `Vec<Candle>` (timestamps as row index).
pub fn load_sample() -> Vec<Candle> {
    let path = data_dir().join("sample_ohlcv.csv");
    load_candles(&path)
}

/// Load any OHLCV CSV with the expected `date,open,high,low,close,volume`
/// schema. Dates are not parsed (timestamps are set to the row index).
pub fn load_candles(path: &Path) -> Vec<Candle> {
    let file = File::open(path).unwrap_or_else(|e| panic!("open {path:?}: {e}"));
    let mut out = Vec::new();
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line.unwrap();
        if i == 0 {
            continue; // header
        }
        let mut cols = line.split(',');
        let _date = cols.next().expect("date");
        let open: f64 = cols.next().expect("open").parse().expect("open f64");
        let high: f64 = cols.next().expect("high").parse().expect("high f64");
        let low: f64 = cols.next().expect("low").parse().expect("low f64");
        let close: f64 = cols.next().expect("close").parse().expect("close f64");
        let volume: f64 = cols.next().expect("volume").parse().expect("volume f64");
        out.push(Candle {
            timestamp: (i as u64) - 1,
            open,
            high,
            low,
            close,
            volume,
        });
    }
    out
}

/// Load a golden CSV of the form `index,value` (header included).
pub fn load_golden(name: &str) -> Vec<(usize, f64)> {
    let path = data_dir().join(name);
    let file = File::open(&path).unwrap_or_else(|e| panic!("open {path:?}: {e}"));
    let mut out = Vec::new();
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line.unwrap();
        if i == 0 {
            continue;
        }
        let mut cols = line.split(',');
        let idx: usize = cols.next().expect("index").parse().expect("index usize");
        let val: f64 = cols.next().expect("value").parse().expect("value f64");
        out.push((idx, val));
    }
    out
}

/// Compare a computed series `produced` (warmup-padded at the head) against a
/// `(index, value)` golden series with absolute tolerance `tol`.
pub fn assert_matches_golden(n_input: usize, produced: &[f64], golden: &[(usize, f64)], tol: f64) {
    let warmup = n_input - produced.len();
    assert_eq!(
        produced.len(),
        golden.len(),
        "produced ({}) and golden ({}) length mismatch",
        produced.len(),
        golden.len(),
    );
    for ((idx, expected), got) in golden.iter().zip(produced.iter()) {
        let aligned_idx = warmup + (idx - golden[0].0);
        assert!(
            (got - expected).abs() <= tol,
            "mismatch at golden index {idx} (data row {aligned_idx}): got {got}, expected {expected}, tol {tol}",
        );
    }
}
