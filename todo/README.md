# Roadmap

Tracking notes for planned features. Done items are promoted to released
code; only the genuinely outstanding ones remain. The authoritative list
of open work is GitHub issues — these files are kept as long-form design
notes for items still in incubation.

## Open

- [`002-chart-pattern-detection.md`](002-chart-pattern-detection.md)
  — head & shoulders, triangles, flags. Major effort (computer-vision
  territory), low priority.

## Tracked in GitHub issues

- Generic numeric type support (`T: Float`) — [#26](https://github.com/Lsh0x/rsta/issues/26).

## Already shipped

The following items were on the original 0.0.2 plan and are now in
released code:

- 001 Candlestick pattern recognition → `patterns::candlestick`
  module (11 patterns: Doji, Hammer, Inverted Hammer, Shooting Star,
  Hanging Man, Marubozu, Engulfing, Harami, Morning/Evening Star, Three
  White Soldiers, Three Black Crows) (post-0.1.0)
- 003 Ichimoku → `indicators::trend::Ichimoku` (0.1.0)
- 005 Parabolic SAR → `indicators::trend::Sar` (0.1.0)
- 006 Money Flow Index → `indicators::volume::Mfi` (0.1.0)
- 007 Commodity Channel Index → `indicators::momentum::Cci` (0.1.0)
- 008 Trading Signal Generation → `signals` module (0.1.0)
- 009 Crossover Detection → `signals::CrossUp` / `CrossDown` (0.1.0)
- 010 Divergence Detection → `signals::Divergence` (0.1.0)
- 011 Basic Backtesting → `backtest` module (0.1.0)
- 012 Position sizing → `backtest::sizing` (Kelly, fractional Kelly,
  fixed-fractional, risk-based, volatility-targeted) (post-0.1.0)
- 013 Performance Metrics → `backtest::Metrics` (Sharpe, max drawdown,
  win rate, profit factor, total return) (0.1.0)
