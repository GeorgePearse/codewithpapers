#!/usr/bin/env python3
"""Inspect parquet file structure."""

import pandas as pd
from pathlib import Path

DATA_DIR = Path(__file__).parent.parent / "data" / "pwc-archive"

def inspect_file(filepath):
    """Inspect a parquet file."""
    print(f"\n{'='*60}")
    print(f"File: {filepath.name}")
    print(f"{'='*60}")

    df = pd.read_parquet(filepath)

    print(f"Rows: {len(df)}")
    print(f"\nColumns ({len(df.columns)}):")
    for col in df.columns:
        dtype = df[col].dtype
        non_null = df[col].notna().sum()
        print(f"  {col:30} {str(dtype):15} {non_null}/{len(df)} non-null")

    print(f"\nFirst 2 rows:")
    print(df.head(2).to_string())

# Inspect each dataset
files = [
    DATA_DIR / "papers-with-abstracts" / "train.parquet",
    DATA_DIR / "datasets" / "train.parquet",
    DATA_DIR / "links-between-paper-and-code" / "train.parquet",
]

for f in files:
    if f.exists():
        inspect_file(f)
    else:
        print(f"\n✗ File not found: {f}")
