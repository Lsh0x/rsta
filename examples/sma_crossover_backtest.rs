//! End-to-end demo: load the bundled BTC daily dataset, run an SMA(20)
//! over SMA(50) crossover strategy through the backtest engine, and print
//! the resulting performance metrics.
//!
//! Run with:
//! ```text
//! cargo run --release --example sma_crossover_backtest
//! ```
//!
//! The strategy enters long when the fast SMA crosses above the slow SMA
//! and exits when it crosses below. No shorts, no pyramiding — the simplest
//! crossover system there is. Useful as a baseline to validate that the
//! whole stack (indicators → signals → backtest → metrics) hangs together.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use rsta::backtest::{Action, BacktestConfig, Backtester, Context, Quantity, Strategy};
use rsta::indicators::trend::Sma;
use rsta::indicators::{Candle, Indicator};
use rsta::signals::{CrossDown, CrossUp, Signal, SignalEvent};

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/btc_usd_daily.csv")
}

/// Minimal Kraken-style CSV loader (no header, OHLCV + trade_count).
fn load_btc() -> Vec<Candle> {
    let path = data_path();
    let file = File::open(&path).unwrap_or_else(|e| panic!("open {}: {e}", path.display()));
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.expect("read line");
        if line.is_empty() {
            continue;
        }
        let mut cols = line.split(',');
        let ts: u64 = cols.next().unwrap().parse().unwrap();
        let o: f64 = cols.next().unwrap().parse().unwrap();
        let h: f64 = cols.next().unwrap().parse().unwrap();
        let l: f64 = cols.next().unwrap().parse().unwrap();
        let c: f64 = cols.next().unwrap().parse().unwrap();
        let v: f64 = cols.next().unwrap().parse().unwrap();
        out.push(Candle {
            timestamp: ts,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: v,
        });
    }
    out
}

/// SMA(fast) / SMA(slow) crossover. Long when fast crosses above slow, exit
/// when fast crosses below.
struct SmaCrossover {
    fast: Sma,
    slow: Sma,
    cross_up: CrossUp,
    cross_down: CrossDown,
}

impl SmaCrossover {
    fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast: Sma::new(fast_period).unwrap(),
            slow: Sma::new(slow_period).unwrap(),
            cross_up: CrossUp::new(),
            cross_down: CrossDown::new(),
        }
    }
}

impl Strategy for SmaCrossover {
    fn on_candle(&mut self, candle: &Candle, _ctx: &Context) -> Action {
        let fast = <Sma as Indicator<f64, f64>>::next(&mut self.fast, candle.close).unwrap();
        let slow = <Sma as Indicator<f64, f64>>::next(&mut self.slow, candle.close).unwrap();
        let (Some(f), Some(s)) = (fast, slow) else {
            // Still warming up — both signals still need a prior pair, so
            // feed them whatever we have to keep their internal state in
            // sync, but emit no action.
            return Action::Hold;
        };
        // Both indicators stable: feed each signal once per bar.
        let up = self.cross_up.next((f, s));
        let down = self.cross_down.next((f, s));
        match (up, down) {
            (Some(SignalEvent::Long), _) => Action::EnterLong(Quantity::AllCash),
            (_, Some(SignalEvent::Short)) => Action::Exit,
            _ => Action::Hold,
        }
    }
}

fn print_metrics(label: &str, m: &rsta::backtest::Metrics) {
    println!("=== {label} ===");
    println!("  final equity   : {:>14.2}", m.final_equity);
    println!("  total return   : {:>14.2}%", m.total_return * 100.0);
    println!("  max drawdown   : {:>14.2}%", m.max_drawdown * 100.0);
    println!("  sharpe         : {:>14.3}", m.sharpe);
    println!("  trades         : {:>14}", m.trade_count);
    println!("  win rate       : {:>14.2}%", m.win_rate * 100.0);
    println!("  profit factor  : {:>14.3}", m.profit_factor);
}

fn main() {
    let candles = load_btc();
    println!(
        "loaded {} BTC daily candles ({}..{})",
        candles.len(),
        candles.first().map(|c| c.timestamp).unwrap_or(0),
        candles.last().map(|c| c.timestamp).unwrap_or(0),
    );

    let config = BacktestConfig {
        initial_cash: 10_000.0,
        fee_rate: 0.001, // 10 bps per leg, realistic for retail crypto
        slippage: 0.0001,
        ..Default::default()
    };
    let bt = Backtester::new(config);

    // Reference: buy-and-hold for comparison.
    struct BuyAndHold {
        entered: bool,
    }
    impl Strategy for BuyAndHold {
        fn on_candle(&mut self, _c: &Candle, _ctx: &Context) -> Action {
            if !self.entered {
                self.entered = true;
                Action::EnterLong(Quantity::AllCash)
            } else {
                Action::Hold
            }
        }
    }
    let bh = bt.run(&candles, &mut BuyAndHold { entered: false });
    print_metrics("buy & hold", &bh.metrics);
    println!();

    // Strategy under test.
    let mut strat = SmaCrossover::new(20, 50);
    let result = bt.run(&candles, &mut strat);
    print_metrics("SMA(20) / SMA(50) crossover", &result.metrics);
}
