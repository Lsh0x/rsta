#!/usr/bin/env python3
"""Regenerate the golden indicator CSVs in tests/data/ from the bundled
sample_ohlcv.csv using pandas-ta as the reference implementation.

Usage:
    pip install pandas pandas-ta
    python scripts/gen_golden.py

Outputs (one column per indicator, indexed by row number into sample_ohlcv.csv):
    tests/data/golden_<indicator>_<params>.csv

The bundled `tests/data/golden_sma_5.csv` was hand-authored against the
synthetic sample dataset (close prices are exactly 10..=39, so SMA5 is
trivially derivable). When new indicators land, regenerate the corresponding
goldens with this script and commit them alongside the indicator change.

Pin `pandas-ta==0.3.14b0` (or whatever current stable is) and document any
upgrade in the PR description, since some indicators have multiple
implementation conventions and the reference may shift.
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


def write_golden(name: str, series: pd.Series) -> None:
    """Write a (index, value) CSV, dropping NaN warmup rows."""
    df = series.dropna().reset_index().rename(columns={"index": "index"})
    df.columns = ["index", name.split("_")[0].lower()]
    out = DATA_DIR / f"golden_{name}.csv"
    df.to_csv(out, index=False)
    print(f"wrote {out} ({len(df)} rows)")


def main() -> None:
    df = pd.read_csv(SAMPLE)
    write_golden("sma_5", ta.sma(df["close"], length=5))
    write_golden("ema_5", ta.ema(df["close"], length=5))
    write_golden("rsi_14", ta.rsi(df["close"], length=14))

    # MACD has three output series; persist each as its own golden CSV.
    macd = ta.macd(df["close"], fast=12, slow=26, signal=9)
    write_golden("macd_12_26_9_line", macd["MACD_12_26_9"])
    write_golden("macd_12_26_9_signal", macd["MACDs_12_26_9"])
    write_golden("macd_12_26_9_hist", macd["MACDh_12_26_9"])
    # Add additional indicators as they land in the crate.


if __name__ == "__main__":
    main()
