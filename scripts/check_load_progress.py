#!/usr/bin/env python3
"""Check the progress of data loading."""

import os
import pickle
import psycopg2
from pathlib import Path

from dotenv import load_dotenv
load_dotenv()

DATABASE_URL = os.getenv('DATABASE_URL') or os.getenv('POSTGRES_URI')
if not DATABASE_URL:
    raise ValueError("DATABASE_URL or POSTGRES_URI environment variable is not set.")
CHECKPOINT_FILE = Path(__file__).parent.parent / "data" / "pwc-archive" / ".load_checkpoint.pkl"

# Total expected counts
TOTAL_PAPERS = 576261
TOTAL_DATASETS = 15008
TOTAL_LINKS = 300000  # Approximate
TOTAL_EVAL = 2250000  # Approximate

def main():
    print("=" * 60)
    print("Data Loading Progress")
    print("=" * 60)

    # Check checkpoint
    if CHECKPOINT_FILE.exists():
        with open(CHECKPOINT_FILE, 'rb') as f:
            checkpoint = pickle.load(f)

        papers_offset = checkpoint.get('papers_offset', 0)
        datasets_offset = checkpoint.get('datasets_offset', 0)
        links_offset = checkpoint.get('links_offset', 0)
        eval_offset = checkpoint.get('eval_offset', 0)

        papers_complete = checkpoint.get('papers_complete', False)
        datasets_complete = checkpoint.get('datasets_complete', False)
        links_complete = checkpoint.get('links_complete', False)
        eval_complete = checkpoint.get('eval_complete', False)

        print("\nCheckpoint Status:")
        print(f"  Papers: {papers_offset:,}/{TOTAL_PAPERS:,} ({papers_offset/TOTAL_PAPERS*100:.1f}%) {'✓ Complete' if papers_complete else 'In Progress'}")
        print(f"  Datasets: {datasets_offset:,}/{TOTAL_DATASETS:,} ({datasets_offset/TOTAL_DATASETS*100:.1f}% if TOTAL_DATASETS > 0 else 0) {'✓ Complete' if datasets_complete else 'In Progress'}")
        print(f"  Links: {links_offset:,} {'✓ Complete' if links_complete else 'In Progress'}")
        print(f"  Evaluation: {eval_offset:,} {'✓ Complete' if eval_complete else 'In Progress'}")

    else:
        print("\n⚠ No checkpoint found - loading may not have started")

    # Check database
    try:
        conn = psycopg2.connect(DATABASE_URL)
        cur = conn.cursor()

        print("\nDatabase Row Counts:")

        cur.execute("SELECT COUNT(*) FROM papers;")
        papers_count = cur.fetchone()[0]
        print(f"  Papers: {papers_count:,}")

        cur.execute("SELECT COUNT(*) FROM datasets;")
        datasets_count = cur.fetchone()[0]
        print(f"  Datasets: {datasets_count:,}")

        cur.execute("SELECT COUNT(*) FROM implementations;")
        impl_count = cur.fetchone()[0]
        print(f"  Implementations: {impl_count:,}")

        cur.execute("SELECT COUNT(*) FROM benchmarks;")
        bench_count = cur.fetchone()[0]
        print(f"  Benchmarks: {bench_count:,}")

        cur.execute("SELECT COUNT(*) FROM benchmark_results;")
        results_count = cur.fetchone()[0]
        print(f"  Benchmark Results: {results_count:,}")

        # Get latest paper
        cur.execute("SELECT title, published_date FROM papers ORDER BY created_at DESC LIMIT 1;")
        latest = cur.fetchone()
        if latest:
            print(f"\nLatest Paper: {latest[0][:60]}... ({latest[1]})")

        cur.close()
        conn.close()

    except Exception as e:
        print(f"\n✗ Error connecting to database: {e}")

    print("=" * 60)

if __name__ == "__main__":
    main()
