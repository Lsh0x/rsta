#!/usr/bin/env python3
"""Regenerate the golden indicator CSVs in tests/data/ using pandas-ta as the
reference implementation.

Two source datasets:

- ``tests/data/sample_ohlcv.csv``: 30 hand-authored synthetic candles where
  closes are ``10..=39``. Used by trivially hand-verifiable golden values
  that need no Python install (``golden_sma_5.csv`` is committed).
- ``tests/data/btc_usd_daily.csv``: real Kraken XBTUSD daily OHLCV (~4.5k
  bars, 2013-10-06 → 2026-04-21). Goldens generated against this dataset
  are prefixed ``golden_btc_*.csv`` and exercise the indicators on real
  price action with gaps, halts, and regime shifts.

Usage::

    pip install pandas pandas-ta
    python scripts/gen_golden.py

The script overwrites every ``golden_btc_*.csv`` it knows about. Commit the
resulting CSVs alongside the corresponding test additions.

Pin a specific ``pandas-ta`` version in the PR description if you upgrade —
some indicators have multiple implementation conventions and the reference
output may shift between releases.
"""
from __future__ import annotations

import sys
from pathlib import Path

try:
    import pandas as pd
    import pandas_ta as ta  # type: ignore
except ImportError:
    print("pandas / pandas-ta not installed. Run: pip install pandas pandas-ta")
    sys.exit(1)


ROOT = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "tests" / "data"
SAMPLE = DATA_DIR / "sample_ohlcv.csv"
BTC = DATA_DIR / "btc_usd_daily.csv"
KRAKEN_COLS = ["timestamp", "open", "high", "low", "close", "volume", "trade_count"]


def write_golden(name: str, series: pd.Series) -> None:
    """Write a (index, value) CSV, dropping NaN warmup rows."""
    df = series.dropna().reset_index().rename(columns={"index": "index"})
    df.columns = ["index", name.split("_")[0].lower()]
    out = DATA_DIR / f"golden_{name}.csv"
    df.to_csv(out, index=False)
    print(f"wrote {out} ({len(df)} rows)")


def gen_synthetic() -> None:
    df = pd.read_csv(SAMPLE)
    write_golden("sma_5", ta.sma(df["close"], length=5))
    write_golden("ema_5", ta.ema(df["close"], length=5))
    write_golden("rsi_14", ta.rsi(df["close"], length=14))


def recursive_ema(close: pd.Series, length: int) -> pd.Series:
    """Recursive EMA seeded with the first observation.

    rsta — like ta-rs and TradingView — uses ``adjust=False`` semantics
    (recursive EMA) rather than pandas' default ``adjust=True`` (unbiased
    weighted-mean formulation). They produce slightly different values that
    do *not* fully converge on long series, so we use ``adjust=False`` here
    to keep the golden CSVs aligned with rsta's convention.
    """
    return close.ewm(span=length, adjust=False).mean()


def recursive_macd(close: pd.Series, fast: int, slow: int, signal: int) -> dict:
    fast_ema = recursive_ema(close, fast)
    slow_ema = recursive_ema(close, slow)
    line = fast_ema - slow_ema
    sig = recursive_ema(line, signal)
    hist = line - sig
    return {"line": line, "signal": sig, "hist": hist}


def gen_btc() -> None:
    if not BTC.exists():
        print(f"skip BTC: {BTC} not found")
        return
    # Kraken raw OHLCV: no header.
    df = pd.read_csv(BTC, names=KRAKEN_COLS)
    write_golden("btc_sma_20", ta.sma(df["close"], length=20))

    # Recursive EMAs to match rsta's convention exactly.
    write_golden("btc_ema_20", recursive_ema(df["close"], 20))

    # RSI: pandas-ta's default already uses Wilder smoothing, which matches
    # rsta. ATR likewise.
    write_golden("btc_rsi_14", ta.rsi(df["close"], length=14))
    write_golden(
        "btc_atr_14",
        ta.atr(df["high"], df["low"], df["close"], length=14),
    )

    macd = recursive_macd(df["close"], fast=12, slow=26, signal=9)
    write_golden("btc_macd_12_26_9_line", macd["line"])
    write_golden("btc_macd_12_26_9_signal", macd["signal"])
    write_golden("btc_macd_12_26_9_hist", macd["hist"])


def main() -> None:
    gen_synthetic()
    gen_btc()


if __name__ == "__main__":
    main()
