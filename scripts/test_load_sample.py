#!/usr/bin/env python3
"""Test loading a small sample of data."""

import os
import psycopg2
import pandas as pd
import json
from pathlib import Path

DATABASE_URL = os.getenv('DATABASE_URL')
if not DATABASE_URL:
    raise ValueError("DATABASE_URL environment variable is not set. Please create a .env.local file with your database credentials.")
DATA_DIR = Path(__file__).parent.parent / "data" / "pwc-archive"

def test_load_sample():
    """Load a small sample and verify."""
    papers_file = DATA_DIR / "papers-with-abstracts" / "train.parquet"

    print("Reading sample data...")
    df = pd.read_parquet(papers_file)
    sample = df.head(10)  # Just 10 rows for testing

    print(f"Sample has {len(sample)} rows")
    print(f"Columns: {list(sample.columns)}")

    conn = psycopg2.connect(DATABASE_URL)
    cur = conn.cursor()

    inserted = 0
    for idx, row in sample.iterrows():
        # Skip papers without arxiv_id
        if pd.isna(row.get('arxiv_id')):
            print(f"  Skipping row {idx}: no arxiv_id")
            continue

        # Parse authors (it's a numpy array in parquet)
        authors = row.get('authors')
        if authors is not None and hasattr(authors, 'tolist'):
            # Convert numpy array to list
            authors = authors.tolist()
        elif isinstance(authors, str):
            try:
                authors = json.loads(authors)
            except:
                authors = None
        else:
            authors = None

        # Parse date
        published_date = row.get('date')
        if pd.notna(published_date):
            if isinstance(published_date, pd.Timestamp):
                published_date = published_date.date()

        try:
            cur.execute("""
                INSERT INTO papers (title, abstract, arxiv_id, arxiv_url, pdf_url, published_date, authors)
                VALUES (%s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (arxiv_id) DO NOTHING
                RETURNING id;
            """, (
                row.get('title'),
                row.get('abstract'),
                row.get('arxiv_id'),
                row.get('url_abs'),
                row.get('url_pdf'),
                published_date,
                json.dumps(authors) if authors else None
            ))

            result = cur.fetchone()
            if result:
                paper_id = result[0]
                print(f"  ✓ Inserted paper {idx}: {row.get('title')[:50]}... (ID: {paper_id})")
                inserted += 1
            else:
                print(f"  - Skipped paper {idx} (duplicate)")

        except Exception as e:
            print(f"  ✗ Error inserting paper {idx}: {e}")
            continue

    conn.commit()

    # Verify
    cur.execute("SELECT COUNT(*) FROM papers;")
    total = cur.fetchone()[0]
    print(f"\n✓ Test complete: {inserted} inserted, {total} total papers in database")

    cur.close()
    conn.close()

if __name__ == "__main__":
    test_load_sample()
