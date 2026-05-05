//! Streaming indicator usage: feed bars one at a time via `next()` rather
//! than batch-computing with `calculate()`. This is what you'd do when
//! plugging rsta into a live data feed.
//!
//! Run with:
//! ```text
//! cargo run --release --example realtime_streaming
//! ```

use rsta::indicators::momentum::Rsi;
use rsta::indicators::trend::{Ema, Sma};
use rsta::indicators::Indicator;
use rsta::signals::{Signal, SignalEvent, ThresholdAbove, ThresholdBelow};

fn main() {
    // Synthetic price stream: a random-walkish sequence anchored around 100.
    // Real systems would pipe in from a websocket or a tick queue; the
    // contract is the same — feed prices one at a time, react to emissions.
    let mut price = 100.0_f64;
    let prices: Vec<f64> = (0..200)
        .map(|i| {
            // Pseudo-random walk: deterministic, no rand dependency.
            let bump = ((i as f64 * 13.0).sin() + (i as f64 * 0.31).cos()) * 1.5;
            price = (price + bump).max(1.0);
            price
        })
        .collect();

    let mut sma = Sma::new(14).unwrap();
    let mut ema = Ema::new(14).unwrap();
    let mut rsi = Rsi::new(14).unwrap();
    let mut overbought = ThresholdAbove::new(70.0);
    let mut oversold = ThresholdBelow::new(30.0);

    println!(
        "{:>4}  {:>7}  {:>7}  {:>7}  {:>7}  signal",
        "bar", "price", "sma14", "ema14", "rsi14"
    );

    for (i, &p) in prices.iter().enumerate() {
        let sma_v = <Sma as Indicator<f64, f64>>::next(&mut sma, p).unwrap();
        let ema_v = <Ema as Indicator<f64, f64>>::next(&mut ema, p).unwrap();
        let rsi_v = rsi.next(p).unwrap();

        let signal = rsi_v.and_then(|r| {
            // Threshold signals only fire on a transition; they share the
            // RSI value as input.
            let up = overbought.next(r);
            let down = oversold.next(r);
            match (up, down) {
                (Some(SignalEvent::Long), _) => Some("RSI > 70 → overbought"),
                (_, Some(SignalEvent::Short)) => Some("RSI < 30 → oversold"),
                _ => None,
            }
        });

        // Print only bars where we have all three indicators — keeps the
        // output to the interesting region.
        if let (Some(s), Some(e), Some(r)) = (sma_v, ema_v, rsi_v) {
            print!("{:>4}  {:>7.2}  {:>7.2}  {:>7.2}  {:>7.2}  ", i, p, s, e, r);
            match signal {
                Some(msg) => println!("{msg}"),
                None => println!(),
            }
        }
    }
}
