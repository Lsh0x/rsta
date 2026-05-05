//! Load OHLCV from a CSV, attach a handful of indicators, export an
//! augmented CSV with one column per indicator.
//!
//! Run with the `csv` feature enabled:
//! ```text
//! cargo run --release --features csv --example csv_to_indicators -- input.csv output.csv
//! ```
//!
//! The input CSV must follow the default header schema
//! (`Date,Open,High,Low,Close,Volume`) — see the `rsta::csv::CsvConfig`
//! documentation if your data uses different column names or order.

use std::env;
use std::process::ExitCode;

use rsta::csv::CsvFormatter;
use rsta::indicators::momentum::Rsi;
use rsta::indicators::trend::{Ema, Sma};
use rsta::indicators::volatility::Atr;
use rsta::indicators::volume::Obv;

fn usage() -> ExitCode {
    eprintln!("usage: csv_to_indicators <input.csv> <output.csv>");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let (input, output) = match args.as_slice() {
        [_, input, output] => (input.clone(), output.clone()),
        _ => return usage(),
    };

    let mut formatter = CsvFormatter::new();
    if let Err(e) = formatter.load_from_file(&input) {
        eprintln!("failed to load {input}: {e}");
        return ExitCode::from(1);
    }
    println!("loaded {} rows from {input}", formatter.data().len());

    // Period choice depends on the input length — a 30-row CSV cannot
    // produce SMA(50). The values below are deliberately small so this
    // example works on the bundled `tests/data/sample_ohlcv.csv` (30 rows);
    // bump them to 20/50/200 on real, multi-year datasets.
    formatter
        .add_close_indicator("SMA5", Box::new(Sma::new(5).unwrap()))
        .add_close_indicator("SMA10", Box::new(Sma::new(10).unwrap()))
        .add_close_indicator("EMA5", Box::new(Ema::new(5).unwrap()))
        .add_close_indicator("RSI7", Box::new(Rsi::new(7).unwrap()))
        .add_candle_indicator("ATR7", Box::new(Atr::new(7).unwrap()))
        .add_candle_indicator("OBV", Box::new(Obv::new()));

    if let Err(e) = formatter.calculate_and_export(&output) {
        eprintln!("failed to compute/export: {e}");
        return ExitCode::from(1);
    }
    println!("wrote enriched CSV to {output}");
    ExitCode::SUCCESS
}
