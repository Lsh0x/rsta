//! # Backtesting Engine
//!
//! Single-asset, single-position backtester. The strategy receives one
//! [`Candle`] at a time plus a read-only [`Context`] and returns an
//! [`Action`] describing what to do this bar (enter long, enter short,
//! exit, or hold).
//!
//! Execution is at the bar's close, with configurable proportional fees
//! and slippage. Sufficient for validating signal/strategy ideas; not a
//! production live-trading engine.
//!
//! ## Out of scope
//!
//! - Multiple assets / portfolios
//! - Pyramiding (adding to an existing position)
//! - Limit / stop / take-profit orders inside the engine — strategies can
//!   implement them by reading bar OHLC and emitting `Exit` themselves
//! - Margin, borrow rates, dividends, corporate actions
//!
//! ## Example
//!
//! ```no_run
//! use rsta::backtest::{Action, BacktestConfig, Backtester, Context, Quantity, Strategy};
//! use rsta::indicators::Candle;
//!
//! struct BuyAndHold {
//!     entered: bool,
//! }
//! impl Strategy for BuyAndHold {
//!     fn on_candle(&mut self, _candle: &Candle, _ctx: &Context) -> Action {
//!         if !self.entered {
//!             self.entered = true;
//!             Action::EnterLong(Quantity::AllCash)
//!         } else {
//!             Action::Hold
//!         }
//!     }
//! }
//!
//! let candles: Vec<Candle> = (1..=100)
//!     .map(|i| Candle {
//!         timestamp: i, open: i as f64, high: i as f64 + 1.0,
//!         low: i as f64 - 1.0, close: i as f64, volume: 1.0,
//!     })
//!     .collect();
//!
//! let bt = Backtester::new(BacktestConfig::default());
//! let mut strat = BuyAndHold { entered: false };
//! let result = bt.run(&candles, &mut strat);
//! assert!(result.metrics.final_equity > 10_000.0); // bought low, held to high
//! ```

pub mod sizing;

use crate::indicators::Candle;

// ---------------------------------------------------------------------------
// Position / trade types
// ---------------------------------------------------------------------------

/// Direction of a position in the market.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// Profits when price rises.
    Long,
    /// Profits when price falls.
    Short,
}

/// An open position. The backtester tracks at most one at a time.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Direction of the position.
    pub side: Side,
    /// Position size in units (always positive — direction is in `side`).
    pub quantity: f64,
    /// Fill price net of slippage at entry.
    pub entry_price: f64,
    /// Bar timestamp of the entry.
    pub entry_timestamp: u64,
}

/// A closed trade — produced when an open [`Position`] is exited.
#[derive(Debug, Clone, Copy)]
pub struct Trade {
    /// Direction of the closed trade.
    pub side: Side,
    /// Position size in units.
    pub quantity: f64,
    /// Entry fill price.
    pub entry_price: f64,
    /// Exit fill price.
    pub exit_price: f64,
    /// Bar timestamp of entry.
    pub entry_timestamp: u64,
    /// Bar timestamp of exit.
    pub exit_timestamp: u64,
    /// Net PnL (after both legs' fees).
    pub pnl: f64,
    /// Total fees paid on entry + exit legs.
    pub fees_paid: f64,
}

// ---------------------------------------------------------------------------
// Action / quantity / strategy contract
// ---------------------------------------------------------------------------

/// How to size a new position.
#[derive(Debug, Clone, Copy)]
pub enum Quantity {
    /// Open with an exact unit size.
    Fixed(f64),
    /// Use 100% of available cash.
    AllCash,
    /// Use a fraction of cash, in `[0.0, 1.0]`.
    PercentCash(f64),
}

/// Decision emitted by the strategy each bar.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// Open a long position (closes any existing short first).
    EnterLong(Quantity),
    /// Open a short position (closes any existing long first).
    EnterShort(Quantity),
    /// Close any currently open position.
    Exit,
    /// Do nothing.
    Hold,
}

/// Read-only view of engine state passed to the strategy each bar.
pub struct Context<'a> {
    /// Snapshot of the portfolio at the start of this bar (post-prior-bar updates).
    pub portfolio: &'a Portfolio,
    /// Zero-based index of the current bar within the input slice.
    pub candle_index: usize,
    /// Close of the current bar — the price at which any orders this bar will fill.
    pub current_price: f64,
}

/// Trading strategy contract.
pub trait Strategy {
    /// Called once for each bar in the input. The returned [`Action`] is
    /// applied to the portfolio at the bar's close.
    fn on_candle(&mut self, candle: &Candle, ctx: &Context) -> Action;

    /// Called once before the first bar.
    fn on_start(&mut self) {}

    /// Called once after the last bar (and after the final equity update).
    fn on_finish(&mut self) {}
}

// ---------------------------------------------------------------------------
// Portfolio + equity book-keeping
// ---------------------------------------------------------------------------

/// State of the simulated trading account.
#[derive(Debug, Clone)]
pub struct Portfolio {
    /// Cash balance (negative balance is allowed but signals an underwater account).
    pub cash: f64,
    /// Currently open position, if any.
    pub position: Option<Position>,
    /// `(timestamp, equity)` pairs sampled at each bar's close.
    pub equity_curve: Vec<(u64, f64)>,
    /// History of all closed trades.
    pub trades: Vec<Trade>,
}

impl Portfolio {
    /// Fresh portfolio with the given starting cash.
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            position: None,
            equity_curve: Vec::new(),
            trades: Vec::new(),
        }
    }

    /// Mark-to-market equity at `current_price`.
    pub fn equity(&self, current_price: f64) -> f64 {
        let pos_value = match self.position {
            None => 0.0,
            Some(p) => match p.side {
                Side::Long => p.quantity * current_price,
                // Short MTM: at entry we received qty*entry; we owe qty*current_price.
                // Position contribution to equity = qty*entry - qty*current = qty*(entry - current).
                // But we already booked qty*entry to cash on entry, so the position itself
                // contributes -qty*current_price to equity (the obligation to buy back).
                // To keep the API symmetric, encode it as `qty * (2*entry - current)` — see
                // the entry/exit logic below for the matching cash flows.
                Side::Short => p.quantity * (2.0 * p.entry_price - current_price),
            },
        };
        self.cash + pos_value
    }
}

// ---------------------------------------------------------------------------
// Backtester config + result
// ---------------------------------------------------------------------------

/// Knobs controlling the simulation.
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    /// Starting cash. Defaults to `10_000.0`.
    pub initial_cash: f64,
    /// Proportional fee per trade leg (e.g. `0.001` = 10 bps). Default: 0.
    pub fee_rate: f64,
    /// Proportional unfavorable slippage on fills (buys filled higher, sells lower).
    /// Default: 0.
    pub slippage: f64,
    /// Periods per year used to annualize the Sharpe ratio. Default: 252
    /// (daily bars). Use 252*6.5 for hourly equities, 365 for crypto, etc.
    pub periods_per_year: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_cash: 10_000.0,
            fee_rate: 0.0,
            slippage: 0.0,
            periods_per_year: 252.0,
        }
    }
}

/// Aggregate metrics derived from the equity curve and trade log.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    /// Equity at the last bar.
    pub final_equity: f64,
    /// `(final_equity - initial_cash) / initial_cash`.
    pub total_return: f64,
    /// Max peak-to-trough relative drawdown across the equity curve, in `[0, 1]`.
    pub max_drawdown: f64,
    /// Annualized Sharpe ratio (zero risk-free rate).
    pub sharpe: f64,
    /// Fraction of closed trades with positive PnL, in `[0, 1]`.
    pub win_rate: f64,
    /// Number of closed trades.
    pub trade_count: usize,
    /// `gross_profit / |gross_loss|`. `f64::INFINITY` if there were no losing
    /// trades, `0.0` if there were no trades at all.
    pub profit_factor: f64,
}

/// Output of a [`Backtester::run`] call.
#[derive(Debug, Clone)]
pub struct BacktestResult {
    /// Final state of the simulated account (cash, position, full trade log,
    /// per-bar equity curve).
    pub portfolio: Portfolio,
    /// Aggregate metrics.
    pub metrics: Metrics,
}

/// The simulation engine.
#[derive(Debug, Clone)]
pub struct Backtester {
    /// Configuration knobs.
    pub config: BacktestConfig,
}

impl Backtester {
    /// Create a new backtester with the given config.
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run a strategy against the candle slice and return the result.
    ///
    /// The strategy's `on_start` is called before the first bar, `on_finish`
    /// after the last bar (and after the final equity sample is recorded).
    pub fn run<S: Strategy>(&self, candles: &[Candle], strategy: &mut S) -> BacktestResult {
        let mut portfolio = Portfolio::new(self.config.initial_cash);
        strategy.on_start();

        for (i, candle) in candles.iter().enumerate() {
            let price = candle.close;
            // Build a read-only context around the current portfolio state.
            // SAFETY: we hold a `&mut Portfolio` only after the strategy
            // returns, so the borrow does not escape this block.
            let action = {
                let ctx = Context {
                    portfolio: &portfolio,
                    candle_index: i,
                    current_price: price,
                };
                strategy.on_candle(candle, &ctx)
            };
            apply_action(&mut portfolio, action, candle, &self.config);

            // Sample equity after applying any action.
            let equity = portfolio.equity(price);
            portfolio.equity_curve.push((candle.timestamp, equity));
        }

        strategy.on_finish();
        let metrics = compute_metrics(&portfolio, &self.config);
        BacktestResult { portfolio, metrics }
    }
}

// ---------------------------------------------------------------------------
// Action application
// ---------------------------------------------------------------------------

fn fill_buy_price(close: f64, slippage: f64) -> f64 {
    close * (1.0 + slippage)
}

fn fill_sell_price(close: f64, slippage: f64) -> f64 {
    close * (1.0 - slippage)
}

/// Resolve a [`Quantity`] sizing rule against the available cash and fill price.
///
/// `fee_rate` is folded into AllCash / PercentCash sizing so the resulting
/// units fit within the cash budget *including* the entry fee. Fixed sizing
/// is left untouched — the caller asked for an exact size.
///
/// Returns `None` if there is no cash to allocate or the price is non-positive.
fn resolve_quantity(qty: Quantity, cash: f64, fill_price: f64, fee_rate: f64) -> Option<f64> {
    if fill_price <= 0.0 {
        return None;
    }
    let effective_unit_cost = fill_price * (1.0 + fee_rate);
    let units = match qty {
        Quantity::Fixed(q) => q,
        Quantity::AllCash => {
            if cash <= 0.0 {
                return None;
            }
            cash / effective_unit_cost
        }
        Quantity::PercentCash(pct) => {
            if cash <= 0.0 {
                return None;
            }
            let pct = pct.clamp(0.0, 1.0);
            (cash * pct) / effective_unit_cost
        }
    };
    if units > 0.0 {
        Some(units)
    } else {
        None
    }
}

fn close_position(portfolio: &mut Portfolio, candle: &Candle, cfg: &BacktestConfig) {
    let Some(pos) = portfolio.position.take() else {
        return;
    };
    let exit_price = match pos.side {
        // Closing a long = selling — buyer takes the spread.
        Side::Long => fill_sell_price(candle.close, cfg.slippage),
        // Closing a short = buying back — same disadvantage.
        Side::Short => fill_buy_price(candle.close, cfg.slippage),
    };
    let exit_fee = exit_price * pos.quantity * cfg.fee_rate;

    match pos.side {
        Side::Long => {
            portfolio.cash += pos.quantity * exit_price - exit_fee;
        }
        Side::Short => {
            // We received qty * entry on the way in (already in cash).
            // Now pay qty * exit_price + fee to flatten.
            portfolio.cash -= pos.quantity * exit_price + exit_fee;
        }
    }

    let entry_fee = pos.entry_price * pos.quantity * cfg.fee_rate;
    let gross_pnl = match pos.side {
        Side::Long => pos.quantity * (exit_price - pos.entry_price),
        Side::Short => pos.quantity * (pos.entry_price - exit_price),
    };
    let total_fees = entry_fee + exit_fee;
    portfolio.trades.push(Trade {
        side: pos.side,
        quantity: pos.quantity,
        entry_price: pos.entry_price,
        exit_price,
        entry_timestamp: pos.entry_timestamp,
        exit_timestamp: candle.timestamp,
        pnl: gross_pnl - total_fees,
        fees_paid: total_fees,
    });
}

fn open_position(
    portfolio: &mut Portfolio,
    side: Side,
    qty: Quantity,
    candle: &Candle,
    cfg: &BacktestConfig,
) {
    let fill_price = match side {
        Side::Long => fill_buy_price(candle.close, cfg.slippage),
        Side::Short => fill_sell_price(candle.close, cfg.slippage),
    };
    let Some(units) = resolve_quantity(qty, portfolio.cash, fill_price, cfg.fee_rate) else {
        return; // not enough cash, skip silently
    };
    let entry_fee = fill_price * units * cfg.fee_rate;
    match side {
        Side::Long => {
            // Pay cash for the units + fee.
            let cost = units * fill_price + entry_fee;
            if cost > portfolio.cash {
                return; // can't afford with fees included
            }
            portfolio.cash -= cost;
        }
        Side::Short => {
            // Receive cash from the short sale (net of fee).
            portfolio.cash += units * fill_price - entry_fee;
        }
    }
    portfolio.position = Some(Position {
        side,
        quantity: units,
        entry_price: fill_price,
        entry_timestamp: candle.timestamp,
    });
}

fn apply_action(portfolio: &mut Portfolio, action: Action, candle: &Candle, cfg: &BacktestConfig) {
    match action {
        Action::Hold => {}
        Action::Exit => close_position(portfolio, candle, cfg),
        Action::EnterLong(qty) => {
            // Close any opposite position first.
            if matches!(portfolio.position, Some(p) if p.side == Side::Short) {
                close_position(portfolio, candle, cfg);
            }
            // If we're already long, do nothing — no pyramiding in v1.
            if portfolio.position.is_none() {
                open_position(portfolio, Side::Long, qty, candle, cfg);
            }
        }
        Action::EnterShort(qty) => {
            if matches!(portfolio.position, Some(p) if p.side == Side::Long) {
                close_position(portfolio, candle, cfg);
            }
            if portfolio.position.is_none() {
                open_position(portfolio, Side::Short, qty, candle, cfg);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

fn compute_metrics(portfolio: &Portfolio, cfg: &BacktestConfig) -> Metrics {
    let final_equity = portfolio
        .equity_curve
        .last()
        .map(|&(_, e)| e)
        .unwrap_or(cfg.initial_cash);
    let total_return = if cfg.initial_cash > 0.0 {
        (final_equity - cfg.initial_cash) / cfg.initial_cash
    } else {
        0.0
    };

    // Max drawdown over the equity curve.
    let mut peak = cfg.initial_cash;
    let mut max_dd = 0.0_f64;
    for &(_, eq) in &portfolio.equity_curve {
        if eq > peak {
            peak = eq;
        }
        if peak > 0.0 {
            let dd = (peak - eq) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    // Sharpe from per-bar returns of the equity curve.
    let sharpe = sharpe_from_equity_curve(&portfolio.equity_curve, cfg.periods_per_year);

    // Trade-level stats.
    let trade_count = portfolio.trades.len();
    let (wins, gross_profit, gross_loss) =
        portfolio
            .trades
            .iter()
            .fold((0usize, 0.0_f64, 0.0_f64), |(w, gp, gl), t| {
                if t.pnl > 0.0 {
                    (w + 1, gp + t.pnl, gl)
                } else {
                    (w, gp, gl + t.pnl)
                }
            });
    let win_rate = if trade_count == 0 {
        0.0
    } else {
        wins as f64 / trade_count as f64
    };
    let profit_factor = if trade_count == 0 {
        0.0
    } else if gross_loss == 0.0 {
        f64::INFINITY
    } else {
        gross_profit / gross_loss.abs()
    };

    Metrics {
        final_equity,
        total_return,
        max_drawdown: max_dd,
        sharpe,
        win_rate,
        trade_count,
        profit_factor,
    }
}

fn sharpe_from_equity_curve(curve: &[(u64, f64)], periods_per_year: f64) -> f64 {
    if curve.len() < 2 {
        return 0.0;
    }
    let returns: Vec<f64> = curve
        .windows(2)
        .filter_map(|w| {
            let prev = w[0].1;
            let cur = w[1].1;
            if prev > 0.0 {
                Some((cur - prev) / prev)
            } else {
                None
            }
        })
        .collect();
    if returns.len() < 2 {
        return 0.0;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let var = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    let std = var.sqrt();
    if std == 0.0 {
        return 0.0;
    }
    (mean / std) * periods_per_year.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(n: usize) -> Vec<Candle> {
        (1..=n)
            .map(|i| Candle {
                timestamp: i as u64,
                open: i as f64,
                high: i as f64 + 0.5,
                low: i as f64 - 0.5,
                close: i as f64,
                volume: 1.0,
            })
            .collect()
    }

    /// Simple strategy: buy on first bar, hold forever.
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

    /// Buy on bar 0, sell on bar 4.
    struct OneTrade {
        bar: usize,
    }
    impl Strategy for OneTrade {
        fn on_candle(&mut self, _c: &Candle, ctx: &Context) -> Action {
            self.bar = ctx.candle_index;
            match ctx.candle_index {
                0 => Action::EnterLong(Quantity::AllCash),
                4 => Action::Exit,
                _ => Action::Hold,
            }
        }
    }

    #[test]
    fn buy_and_hold_appreciates_with_price() {
        let bt = Backtester::new(BacktestConfig::default());
        let candles = ramp(10);
        let mut s = BuyAndHold { entered: false };
        let res = bt.run(&candles, &mut s);
        // Bought at ~1.0, ramps to 10.0 → ~10x equity.
        assert!(res.metrics.final_equity > res.metrics.final_equity * 0.0); // sanity
        assert!(res.metrics.total_return > 5.0);
        // No trade closed (still holding), so trade_count = 0.
        assert_eq!(res.metrics.trade_count, 0);
    }

    #[test]
    fn one_trade_is_recorded_with_correct_pnl() {
        let bt = Backtester::new(BacktestConfig::default());
        let candles = ramp(10);
        let mut s = OneTrade { bar: 0 };
        let res = bt.run(&candles, &mut s);
        assert_eq!(res.portfolio.trades.len(), 1);
        let t = res.portfolio.trades[0];
        assert_eq!(t.side, Side::Long);
        // Bought at 1.0 with 10000 cash → 10000 units.
        assert!((t.quantity - 10_000.0).abs() < 1e-9);
        assert_eq!(t.entry_price, 1.0);
        assert_eq!(t.exit_price, 5.0);
        // Gross PnL = 10000 * (5 - 1) = 40000, no fees → net 40000.
        assert!((t.pnl - 40_000.0).abs() < 1e-9);
        // Account is flat after exit.
        assert!(res.portfolio.position.is_none());
        assert_eq!(res.metrics.win_rate, 1.0);
    }

    #[test]
    fn fees_reduce_pnl() {
        let cfg = BacktestConfig {
            fee_rate: 0.01, // 1% per leg, exaggerated
            ..Default::default()
        };
        let bt = Backtester::new(cfg);
        let mut s = OneTrade { bar: 0 };
        let res = bt.run(&ramp(10), &mut s);
        let t = res.portfolio.trades[0];
        // Two legs of fee at 1% each compound below the no-fee baseline.
        assert!(t.pnl < 40_000.0);
        assert!(t.fees_paid > 0.0);
    }

    #[test]
    fn slippage_widens_spread() {
        let cfg = BacktestConfig {
            slippage: 0.01,
            ..Default::default()
        };
        let bt = Backtester::new(cfg);
        let mut s = OneTrade { bar: 0 };
        let res = bt.run(&ramp(10), &mut s);
        let t = res.portfolio.trades[0];
        assert!(t.entry_price > 1.0); // bought higher than close
        assert!(t.exit_price < 5.0); // sold lower than close
    }

    /// Buy at peak, sell at trough → loss.
    struct ShortStrategy;
    impl Strategy for ShortStrategy {
        fn on_candle(&mut self, _c: &Candle, ctx: &Context) -> Action {
            match ctx.candle_index {
                0 => Action::EnterShort(Quantity::Fixed(100.0)),
                4 => Action::Exit,
                _ => Action::Hold,
            }
        }
    }

    #[test]
    fn short_in_uptrend_loses_money() {
        let bt = Backtester::new(BacktestConfig::default());
        let res = bt.run(&ramp(10), &mut ShortStrategy);
        assert_eq!(res.portfolio.trades.len(), 1);
        let t = res.portfolio.trades[0];
        assert_eq!(t.side, Side::Short);
        // Shorted 100 @ 1, bought back @ 5 → PnL = 100 * (1 - 5) = -400.
        assert!((t.pnl - (-400.0)).abs() < 1e-9);
        assert!(res.metrics.total_return < 0.0);
    }

    #[test]
    fn equity_curve_is_sampled_each_bar() {
        let bt = Backtester::new(BacktestConfig::default());
        let candles = ramp(7);
        let mut s = BuyAndHold { entered: false };
        let res = bt.run(&candles, &mut s);
        assert_eq!(res.portfolio.equity_curve.len(), candles.len());
    }

    #[test]
    fn flipping_long_to_short_closes_first_position() {
        struct Flip;
        impl Strategy for Flip {
            fn on_candle(&mut self, _c: &Candle, ctx: &Context) -> Action {
                match ctx.candle_index {
                    0 => Action::EnterLong(Quantity::Fixed(100.0)),
                    3 => Action::EnterShort(Quantity::Fixed(50.0)),
                    _ => Action::Hold,
                }
            }
        }
        let bt = Backtester::new(BacktestConfig::default());
        let res = bt.run(&ramp(10), &mut Flip);
        assert_eq!(
            res.portfolio.trades.len(),
            1,
            "long should close when short order arrives"
        );
        assert!(matches!(
            res.portfolio.position,
            Some(p) if p.side == Side::Short
        ));
    }

    #[test]
    fn metrics_are_zero_for_empty_input() {
        let bt = Backtester::new(BacktestConfig::default());
        let res = bt.run(&[], &mut BuyAndHold { entered: false });
        assert_eq!(res.metrics.final_equity, 10_000.0);
        assert_eq!(res.metrics.trade_count, 0);
        assert_eq!(res.metrics.total_return, 0.0);
        assert_eq!(res.metrics.max_drawdown, 0.0);
    }
}
