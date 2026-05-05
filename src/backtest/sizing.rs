//! Position sizing helpers.
//!
//! Pure functions that compute either a **fraction of equity** or a
//! **raw unit quantity** to feed into [`Quantity`](super::Quantity)
//! when a strategy emits an entry. Keeping them as plain functions
//! (rather than a `PositionSizer` trait) lets a `Strategy` mix and
//! match — Kelly to set the fraction, risk-based to size a stop.
//!
//! ## Conventions
//!
//! - "fraction" means a number in `[0.0, 1.0]` representing a share of
//!   total equity. Convert via `Quantity::PercentCash(f)`.
//! - "units" means an actual position size in the same units as the
//!   indicator's price input (typically shares or coins). Convert via
//!   `Quantity::Fixed(units)`.
//! - Every function clamps obviously bad inputs to safe values rather
//!   than panicking. Negative or zero stop distances, NaNs, and
//!   non-positive equities all return `0.0`.

/// Kelly criterion: optimal fraction of equity to risk on a trade given
/// historical win rate and the average win/loss ratio.
///
/// `f* = (b * p - q) / b` where:
/// - `p` = win probability
/// - `q = 1 - p` = loss probability
/// - `b` = win/loss payoff ratio (`avg_win / avg_loss`)
///
/// Returns the fraction in `[0.0, 1.0]`. Negative Kelly (the system has
/// negative expectancy) is clamped to `0.0` — never bet a system you
/// expect to lose. The fractional case ("half Kelly") is available via
/// [`fractional_kelly`].
///
/// # Example
/// ```
/// use rsta::backtest::sizing::kelly;
///
/// // 60% win rate, average winner is 2× the average loser.
/// // f* = (2 * 0.6 - 0.4) / 2 = 0.4
/// let f = kelly(0.6, 2.0, 1.0);
/// assert!((f - 0.4).abs() < 1e-9);
/// ```
pub fn kelly(win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
    if !win_rate.is_finite()
        || !avg_win.is_finite()
        || !avg_loss.is_finite()
        || avg_win <= 0.0
        || avg_loss <= 0.0
        || !(0.0..=1.0).contains(&win_rate)
    {
        return 0.0;
    }
    let b = avg_win / avg_loss;
    let p = win_rate;
    let q = 1.0 - p;
    let f = (b * p - q) / b;
    f.clamp(0.0, 1.0)
}

/// Fractional Kelly — multiplies [`kelly`] by `fraction`. The textbook
/// pick is `0.5` ("half Kelly"); typical practitioners go even lower
/// (`0.25`–`0.33`) to reduce drawdown variance at the cost of growth
/// rate.
///
/// `fraction` is clamped to `[0.0, 1.0]`.
pub fn fractional_kelly(win_rate: f64, avg_win: f64, avg_loss: f64, fraction: f64) -> f64 {
    let f = kelly(win_rate, avg_win, avg_loss);
    if !fraction.is_finite() {
        return 0.0;
    }
    f * fraction.clamp(0.0, 1.0)
}

/// Fixed-fractional sizing: always allocate a fixed share of current
/// equity to the trade. Equivalent to `Quantity::PercentCash(fraction)`,
/// but exposed here for symmetry with the other sizers.
///
/// Returns `0.0` if `fraction` is non-finite or non-positive.
pub fn fixed_fractional(fraction: f64) -> f64 {
    if !fraction.is_finite() || fraction <= 0.0 {
        return 0.0;
    }
    fraction.clamp(0.0, 1.0)
}

/// Risk-based sizing: choose `units` so that hitting the stop loses
/// exactly `risk_fraction × equity`.
///
/// `units = (equity × risk_fraction) / |entry − stop|`
///
/// Returns the raw position size in units (feed to
/// `Quantity::Fixed`). Returns `0.0` for any of:
///
/// - non-finite or non-positive equity / risk_fraction / prices
/// - `entry == stop` (zero stop distance — would imply infinite size)
///
/// # Example
/// ```
/// use rsta::backtest::sizing::risk_based;
///
/// // 10 000 USD account, willing to risk 1% per trade, entry 100,
/// // stop 98 → risk per unit is 2 → max units = 100 / 2 = 50.
/// let units = risk_based(10_000.0, 0.01, 100.0, 98.0);
/// assert!((units - 50.0).abs() < 1e-9);
/// ```
pub fn risk_based(equity: f64, risk_fraction: f64, entry: f64, stop: f64) -> f64 {
    if !equity.is_finite()
        || !risk_fraction.is_finite()
        || !entry.is_finite()
        || !stop.is_finite()
        || equity <= 0.0
        || risk_fraction <= 0.0
        || entry <= 0.0
        || stop <= 0.0
    {
        return 0.0;
    }
    let stop_distance = (entry - stop).abs();
    if stop_distance == 0.0 {
        return 0.0;
    }
    let dollar_risk = equity * risk_fraction.clamp(0.0, 1.0);
    dollar_risk / stop_distance
}

/// Volatility-targeted sizing: scale the position so that the expected
/// per-bar P&L volatility is `target_volatility × equity`.
///
/// `units = (equity × target_volatility) / asset_volatility_per_unit`
///
/// `asset_volatility_per_unit` is typically derived from ATR or a
/// rolling stdev of returns; rsta doesn't compute it for you here so
/// the function stays pure and the caller picks the methodology.
///
/// Returns `0.0` on bad inputs (non-finite, non-positive volatility, …).
pub fn volatility_targeted(
    equity: f64,
    target_volatility: f64,
    asset_volatility_per_unit: f64,
) -> f64 {
    if !equity.is_finite()
        || !target_volatility.is_finite()
        || !asset_volatility_per_unit.is_finite()
        || equity <= 0.0
        || target_volatility <= 0.0
        || asset_volatility_per_unit <= 0.0
    {
        return 0.0;
    }
    (equity * target_volatility) / asset_volatility_per_unit
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    // ----- kelly -----

    #[test]
    fn kelly_textbook_example() {
        // 60% win rate, win:loss = 2 → f* = (1.2 - 0.4) / 2 = 0.4
        assert!(approx(kelly(0.6, 2.0, 1.0), 0.4));
    }

    #[test]
    fn kelly_50_50_breakeven() {
        // 50/50 with equal win/loss is zero-EV → Kelly = 0.
        assert!(approx(kelly(0.5, 1.0, 1.0), 0.0));
    }

    #[test]
    fn kelly_negative_edge_clamps_to_zero() {
        // Loser system: 40% win rate at 1:1 → would give -0.2.
        assert_eq!(kelly(0.4, 1.0, 1.0), 0.0);
    }

    #[test]
    fn kelly_clamps_to_one_max() {
        // Pathological: 100% win rate, infinite-ish edge.
        assert!(kelly(1.0, 5.0, 1.0) <= 1.0);
    }

    #[test]
    fn kelly_rejects_bad_inputs() {
        assert_eq!(kelly(f64::NAN, 1.0, 1.0), 0.0);
        assert_eq!(kelly(0.5, 0.0, 1.0), 0.0);
        assert_eq!(kelly(0.5, 1.0, 0.0), 0.0);
        assert_eq!(kelly(-0.1, 1.0, 1.0), 0.0);
        assert_eq!(kelly(1.5, 1.0, 1.0), 0.0);
    }

    // ----- fractional_kelly -----

    #[test]
    fn fractional_kelly_halves_full_kelly() {
        let full = kelly(0.6, 2.0, 1.0);
        let half = fractional_kelly(0.6, 2.0, 1.0, 0.5);
        assert!(approx(half, full * 0.5));
    }

    #[test]
    fn fractional_kelly_clamps_fraction() {
        let full = kelly(0.6, 2.0, 1.0);
        // fraction > 1 is clamped to 1 → identical to full Kelly.
        assert!(approx(fractional_kelly(0.6, 2.0, 1.0, 5.0), full));
        // negative fraction → 0.
        assert_eq!(fractional_kelly(0.6, 2.0, 1.0, -0.5), 0.0);
    }

    // ----- fixed_fractional -----

    #[test]
    fn fixed_fractional_passes_valid_input() {
        assert!(approx(fixed_fractional(0.05), 0.05));
        assert!(approx(fixed_fractional(0.5), 0.5));
    }

    #[test]
    fn fixed_fractional_clamps_and_rejects() {
        assert_eq!(fixed_fractional(0.0), 0.0);
        assert_eq!(fixed_fractional(-0.1), 0.0);
        assert_eq!(fixed_fractional(f64::NAN), 0.0);
        assert!(approx(fixed_fractional(2.0), 1.0));
    }

    // ----- risk_based -----

    #[test]
    fn risk_based_textbook_example() {
        // 10k account, 1% risk, entry 100, stop 98 → 50 units.
        assert!(approx(risk_based(10_000.0, 0.01, 100.0, 98.0), 50.0));
    }

    #[test]
    fn risk_based_handles_short_setup() {
        // For a short, stop sits above entry — abs distance still works.
        assert!(approx(risk_based(10_000.0, 0.01, 100.0, 102.0), 50.0));
    }

    #[test]
    fn risk_based_zero_stop_distance() {
        assert_eq!(risk_based(10_000.0, 0.01, 100.0, 100.0), 0.0);
    }

    #[test]
    fn risk_based_rejects_bad_inputs() {
        assert_eq!(risk_based(0.0, 0.01, 100.0, 98.0), 0.0);
        assert_eq!(risk_based(-1000.0, 0.01, 100.0, 98.0), 0.0);
        assert_eq!(risk_based(10_000.0, -0.01, 100.0, 98.0), 0.0);
        assert_eq!(risk_based(10_000.0, 0.01, 0.0, 98.0), 0.0);
        assert_eq!(risk_based(10_000.0, 0.01, f64::NAN, 98.0), 0.0);
    }

    #[test]
    fn risk_based_clamps_fraction() {
        // fraction >= 1 means risk full equity → units = equity / stop_distance.
        assert!(approx(risk_based(10_000.0, 5.0, 100.0, 98.0), 5_000.0));
    }

    // ----- volatility_targeted -----

    #[test]
    fn volatility_targeted_basic() {
        // 100k equity, target 1% vol per bar, asset moves $2 per unit per bar
        // → 100_000 * 0.01 / 2 = 500 units.
        assert!(approx(volatility_targeted(100_000.0, 0.01, 2.0), 500.0));
    }

    #[test]
    fn volatility_targeted_rejects_bad_inputs() {
        assert_eq!(volatility_targeted(0.0, 0.01, 2.0), 0.0);
        assert_eq!(volatility_targeted(100_000.0, 0.0, 2.0), 0.0);
        assert_eq!(volatility_targeted(100_000.0, 0.01, 0.0), 0.0);
        assert_eq!(volatility_targeted(100_000.0, f64::NAN, 2.0), 0.0);
    }
}
