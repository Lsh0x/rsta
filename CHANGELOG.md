# Changelog

## [0.1.0](https://github.com/Lsh0x/rsta/releases/tag/v0.1.0)

[Full Changelog](https://github.com/Lsh0x/rsta/compare/v0.0.2...v0.1.0)

First usable release. The crate now spans the full "indicators → signals
→ backtest" loop and is verified against pandas-ta on real Kraken BTC/USD
daily data.

### Added

- **10 new indicators**: `Wma`, `Dema`, `Tema`, `Hma`, `Adx` (+`AdxResult`),
  `Sar`, `Ichimoku` (+`IchimokuResult`), `Cci`, `Donchian`
  (+`DonchianResult`), `Mfi`, `Vwap`. Pure-function pivot points
  (`pivot_classic`, `pivot_fibonacci`, `pivot_camarilla` returning
  `PivotResult`). Heikin-Ashi candle transform (`heikin_ashi`).
- **Signals layer** (`rsta::signals`): `Signal` trait, `SignalEvent`
  enum, `CrossUp`/`CrossDown`/`ThresholdAbove`/`ThresholdBelow`/
  `Breakout`/`Divergence` built-ins, `SignalExt::and`/`or`/`not`
  combinators.
- **Backtesting engine** (`rsta::backtest`): single-asset `Strategy`
  trait, `Action`/`Quantity` types, `Backtester` with configurable fees
  and slippage, `Metrics` (total return, max drawdown, annualised
  Sharpe, win rate, profit factor), full trade log and equity curve.
- **CSV pipeline** (`rsta::csv`, behind the `csv` feature): bidirectional
  loader/exporter via `CsvFormatter`.
- **`Indicator` trait extensions**: `name() -> &'static str` and
  `period() -> Option<usize>` with default impls — non-breaking.
- **Golden test infrastructure**: `tests/data/btc_usd_daily.csv` (12 years
  of real Kraken XBTUSD daily OHLCV) plus 7 pandas-ta-generated golden
  CSVs and a runnable `scripts/gen_golden.py`.
- Three runnable `examples/` (end-to-end backtest, real-time streaming,
  CSV enrichment) and two criterion `benches/` (indicator hot-paths,
  full backtest).

### Changed

- **Module split** (was already in 0.0.2): every indicator now lives in
  its own file under `src/indicators/<family>/<name>.rs`. Re-exports
  are unchanged.
- **`utils::calculate_ema` rewritten** to use recursive (`adjust=False`)
  seeding — emits one EMA per input bar starting from `data[0]`. This
  matches `Ema::next`, TradingView, and pandas `ewm(adjust=False)`. The
  previous `SMA(period)`-seeded variant produced different early-bar
  values from the streaming path; the two paths are now consistent.
  *Behavioural change for direct callers of `calculate_ema`.*
- **`Macd::calculate` rewritten** as a thin wrapper over `next()` so
  batch and streaming produce identical output bar-for-bar (one
  `MacdResult` per input bar, including the warmup-tainted prefix).
- `PriceDataAccessor<f64>` for `f64` made internally consistent
  (`get_high`/`get_low`/`get_open` now return `*data` rather than
  `*self`).

### Verification

- 194 unit + 9 golden + 43 doctests, all green with `--all-features`.
- `cargo clippy --all-features --all-targets -- -D warnings` clean.

### Deferred

- Generic numeric type support (`T: Float` for `f32`/`f64`) tracked in
  [#26](https://github.com/Lsh0x/rsta/issues/26).

## [0.0.2](https://github.com/Lsh0x/rsta/releases/tag/v0.0.2)

[Full Changelog](https://github.com/Lsh0x/rsta/compare/v0.0.1...v0.0.2)

- Module split: each indicator lives in its own file.
- `Macd` (+`MacdResult`) added.
- Short-name canonicalisation (`Sma`, `Ema`, `Atr`, …).

## [0.0.1](https://github.com/Lsh0x/rsta/releases/tag/v0.0.1)

- Initial release.
- Trend (SMA, EMA), momentum (RSI, Stochastic, Williams %R), volume
  (OBV, VROC, A/D Line), volatility (ATR, Bollinger, Keltner).
