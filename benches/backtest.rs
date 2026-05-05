//! End-to-end backtest microbenchmark: SMA-crossover strategy on the bundled
//! BTC daily dataset, plus a synthetic 1M-bar run that stresses the engine.
//!
//! Run with:
//! ```text
//! cargo bench --bench backtest
//! ```

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rsta::backtest::{Action, BacktestConfig, Backtester, Context, Quantity, Strategy};
use rsta::indicators::trend::Sma;
use rsta::indicators::{Candle, Indicator};
use rsta::signals::{CrossDown, CrossUp, Signal, SignalEvent};

fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/btc_usd_daily.csv")
}

fn load_btc() -> Vec<Candle> {
    let file = File::open(data_path()).expect("open btc daily");
    let mut out = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
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

fn synthetic_candles(n: usize) -> Vec<Candle> {
    (0..n)
        .map(|i| {
            let t = i as f64;
            let close = 100.0 + (t * 0.01).sin() * 20.0 + t * 0.001;
            Candle {
                timestamp: i as u64,
                open: close - 0.25,
                high: close + 0.5,
                low: close - 0.5,
                close,
                volume: 1_000.0,
            }
        })
        .collect()
}

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
        let f = <Sma as Indicator<f64, f64>>::next(&mut self.fast, candle.close).unwrap();
        let s = <Sma as Indicator<f64, f64>>::next(&mut self.slow, candle.close).unwrap();
        let (Some(f), Some(s)) = (f, s) else {
            return Action::Hold;
        };
        let up = self.cross_up.next((f, s));
        let down = self.cross_down.next((f, s));
        match (up, down) {
            (Some(SignalEvent::Long), _) => Action::EnterLong(Quantity::AllCash),
            (_, Some(SignalEvent::Short)) => Action::Exit,
            _ => Action::Hold,
        }
    }
}

fn bench_btc_daily(c: &mut Criterion) {
    let candles = load_btc();
    let bt = Backtester::new(BacktestConfig::default());
    let mut group = c.benchmark_group("backtest_btc_daily");
    group.throughput(Throughput::Elements(candles.len() as u64));
    group.bench_function("sma_20_50_crossover", |b| {
        b.iter(|| {
            let mut strat = SmaCrossover::new(20, 50);
            black_box(bt.run(&candles, &mut strat));
        })
    });
    group.finish();
}

fn bench_synthetic_1m(c: &mut Criterion) {
    let candles = synthetic_candles(1_000_000);
    let bt = Backtester::new(BacktestConfig::default());
    let mut group = c.benchmark_group("backtest_synthetic_1m");
    group.sample_size(10); // bigger inputs → fewer samples
    group.throughput(Throughput::Elements(candles.len() as u64));
    group.bench_function("sma_20_50_crossover", |b| {
        b.iter(|| {
            let mut strat = SmaCrossover::new(20, 50);
            black_box(bt.run(&candles, &mut strat));
        })
    });
    group.finish();
}

criterion_group!(benches, bench_btc_daily, bench_synthetic_1m);
criterion_main!(benches);
