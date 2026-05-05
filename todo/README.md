# Roadmap

Tracking notes for planned features. Done items have been promoted to
released code; only the genuinely outstanding ones remain. The
authoritative list of open work is GitHub issues — these files are kept
as long-form design notes for items still in incubation.

## Open

- [`001-candlestick-pattern-recognition.md`](001-candlestick-pattern-recognition.md)
  — doji, engulfing, hammer, shooting star, harami detection on 1–3 bars.
- [`002-chart-pattern-detection.md`](002-chart-pattern-detection.md)
  — head & shoulders, triangles, flags. Major effort, low priority.
- [`012-position-sizing.md`](012-position-sizing.md)
  — fixed-fractional, Kelly, etc. Lives most naturally in
  `src/backtest/`.

## Tracked in GitHub issues

- Generic numeric type support (`T: Float`) — [#26](https://github.com/Lsh0x/rsta/issues/26).

## Already shipped (removed from this folder)

The following items were on the original 0.0.2 plan and are now in
released code:

- 003 Ichimoku → `indicators::trend::Ichimoku` (0.1.0)
- 005 Parabolic SAR → `indicators::trend::Sar` (0.1.0)
- 006 Money Flow Index → `indicators::volume::Mfi` (0.1.0)
- 007 Commodity Channel Index → `indicators::momentum::Cci` (0.1.0)
- 008 Trading Signal Generation → `signals` module (0.1.0)
- 009 Crossover Detection → `signals::CrossUp` / `CrossDown` (0.1.0)
- 010 Divergence Detection → `signals::Divergence` (0.1.0)
- 011 Basic Backtesting → `backtest` module (0.1.0)
- 013 Performance Metrics → `backtest::Metrics` (Sharpe, max drawdown,
  win rate, profit factor, total return) (0.1.0)
