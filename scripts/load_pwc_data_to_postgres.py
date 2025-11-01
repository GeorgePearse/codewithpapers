#!/usr/bin/env python3
"""Load Papers with Code data from parquet files into Postgres database with checkpoint support."""

import os
import json
import psycopg2
import psycopg2.extras
import pandas as pd
from pathlib import Path
from datetime import datetime
from typing import Optional
import pickle
import argparse

# Connection string from environment
DATABASE_URL = os.getenv('DATABASE_URL')
if not DATABASE_URL:
    raise ValueError("DATABASE_URL environment variable is not set. Please create a .env.local file with your database credentials.")

# Data directory
DATA_DIR = Path(__file__).parent.parent / "data" / "pwc-archive"
CHECKPOINT_FILE = DATA_DIR / ".load_checkpoint.pkl"


def load_checkpoint():
    """Load checkpoint data if it exists."""
    if CHECKPOINT_FILE.exists():
        with open(CHECKPOINT_FILE, 'rb') as f:
            return pickle.load(f)
    return {
        'papers_offset': 0,
        'datasets_offset': 0,
        'links_offset': 0,
        'eval_offset': 0,
        'papers_complete': False,
        'datasets_complete': False,
        'links_complete': False,
        'eval_complete': False,
    }


def save_checkpoint(checkpoint):
    """Save checkpoint data."""
    with open(CHECKPOINT_FILE, 'wb') as f:
        pickle.dump(checkpoint, f)


def clear_checkpoint():
    """Remove checkpoint file."""
    if CHECKPOINT_FILE.exists():
        CHECKPOINT_FILE.unlink()


def connect_db():
    """Create database connection."""
    return psycopg2.connect(DATABASE_URL)


def load_papers(conn, batch_size=1000, checkpoint=None):
    """Load papers from parquet file into database."""
    papers_file = DATA_DIR / "papers-with-abstracts" / "train.parquet"

    if not papers_file.exists():
        print(f"⚠ Papers file not found: {papers_file}")
        print("  Run: uv run python scripts/download_pwc_data.py --datasets papers")
        return 0

    if checkpoint and checkpoint.get('papers_complete'):
        print("\n✓ Papers already loaded (skipping)")
        return 0

    print(f"\nLoading papers from {papers_file}...")
    df = pd.read_parquet(papers_file)

    start_offset = checkpoint.get('papers_offset', 0) if checkpoint else 0
    print(f"Total papers: {len(df)}, starting from offset: {start_offset}")

    cur = conn.cursor()
    inserted = 0
    skipped = 0

    # Process in batches
    for i in range(start_offset, len(df), batch_size):
        batch = df.iloc[i:i+batch_size]

        for _, row in batch.iterrows():
            try:
                # Skip papers without arxiv_id for now (they can't be linked properly)
                arxiv_id = row.get('arxiv_id')
                if pd.isna(arxiv_id):
                    skipped += 1
                    continue

                # Parse authors (it's a numpy array in parquet files)
                authors = row.get('authors')
                if authors is not None and hasattr(authors, 'tolist'):
                    # Convert numpy array to list
                    authors = authors.tolist()
                elif isinstance(authors, str):
                    try:
                        authors = json.loads(authors)
                    except:
                        authors = None
                elif isinstance(authors, list):
                    authors = authors
                else:
                    authors = None

                # Parse published date
                published_date = row.get('date')
                if pd.notna(published_date):
                    # Convert to string format if it's a timestamp
                    if isinstance(published_date, pd.Timestamp):
                        published_date = published_date.date()
                    elif isinstance(published_date, str):
                        # Try to parse the date string
                        try:
                            published_date = datetime.strptime(published_date, '%Y-%m-%d').date()
                        except:
                            published_date = None
                else:
                    published_date = None

                cur.execute("""
                    INSERT INTO papers (title, abstract, arxiv_id, arxiv_url, pdf_url, published_date, authors)
                    VALUES (%s, %s, %s, %s, %s, %s, %s)
                    ON CONFLICT (arxiv_id) DO NOTHING
                    RETURNING id;
                """, (
                    row.get('title'),
                    row.get('abstract'),
                    arxiv_id,
                    row.get('url_abs'),
                    row.get('url_pdf'),
                    published_date,
                    json.dumps(authors) if authors else None
                ))

                if cur.fetchone():
                    inserted += 1
                else:
                    skipped += 1

            except Exception as e:
                print(f"Error inserting paper: {e}")
                continue

        conn.commit()

        # Save checkpoint after each batch
        if checkpoint is not None:
            checkpoint['papers_offset'] = i + batch_size
            save_checkpoint(checkpoint)

        print(f"  Processed {min(i+batch_size, len(df))}/{len(df)} papers ({inserted} inserted, {skipped} skipped)")

    if checkpoint is not None:
        checkpoint['papers_complete'] = True
        save_checkpoint(checkpoint)

    cur.close()
    print(f"✓ Papers loaded: {inserted} inserted, {skipped} skipped (duplicates or missing arxiv_id)")
    return inserted


def load_datasets(conn, batch_size=1000, checkpoint=None):
    """Load datasets from parquet file into database."""
    datasets_file = DATA_DIR / "datasets" / "train.parquet"

    if not datasets_file.exists():
        print(f"\n⚠ Datasets file not found: {datasets_file}")
        print("  Run: uv run python scripts/download_pwc_data.py --datasets datasets")
        return 0

    if checkpoint and checkpoint.get('datasets_complete'):
        print("\n✓ Datasets already loaded (skipping)")
        return 0

    print(f"\nLoading datasets from {datasets_file}...")
    df = pd.read_parquet(datasets_file)

    start_offset = checkpoint.get('datasets_offset', 0) if checkpoint else 0
    print(f"Total datasets: {len(df)}, starting from offset: {start_offset}")

    cur = conn.cursor()
    inserted = 0
    skipped = 0

    for i in range(start_offset, len(df), batch_size):
        batch = df.iloc[i:i+batch_size]

        for _, row in batch.iterrows():
            try:
                # Parse modalities and other arrays
                modalities = row.get('modalities')
                if modalities is not None and hasattr(modalities, 'tolist'):
                    modalities = modalities.tolist()
                elif isinstance(modalities, str):
                    modalities = [m.strip() for m in modalities.split(',')]
                elif not isinstance(modalities, list):
                    modalities = None

                # Extract paper URL if it's a dict
                paper_info = row.get('paper')
                paper_url = paper_info.get('url') if isinstance(paper_info, dict) else None

                cur.execute("""
                    INSERT INTO datasets (name, description, modalities, homepage_url, paper_url)
                    VALUES (%s, %s, %s, %s, %s)
                    ON CONFLICT (name) DO NOTHING
                    RETURNING id;
                """, (
                    row.get('name'),
                    row.get('description'),
                    modalities,
                    row.get('homepage'),
                    paper_url
                ))

                if cur.fetchone():
                    inserted += 1
                else:
                    skipped += 1

            except Exception as e:
                print(f"Error inserting dataset: {e}")
                continue

        conn.commit()

        # Save checkpoint after each batch
        if checkpoint is not None:
            checkpoint['datasets_offset'] = i + batch_size
            save_checkpoint(checkpoint)

        print(f"  Processed {min(i+batch_size, len(df))}/{len(df)} datasets ({inserted} inserted, {skipped} skipped)")

    if checkpoint is not None:
        checkpoint['datasets_complete'] = True
        save_checkpoint(checkpoint)

    cur.close()
    print(f"✓ Datasets loaded: {inserted} inserted, {skipped} skipped")
    return inserted


def load_code_links(conn, batch_size=1000, checkpoint=None):
    """Load code links from parquet file into database."""
    links_file = DATA_DIR / "links-between-paper-and-code" / "train.parquet"

    if not links_file.exists():
        print(f"\n⚠ Links file not found: {links_file}")
        print("  Run: uv run python scripts/download_pwc_data.py --datasets links")
        return 0

    if checkpoint and checkpoint.get('links_complete'):
        print("\n✓ Code links already loaded (skipping)")
        return 0

    print(f"\nLoading code links from {links_file}...")
    df = pd.read_parquet(links_file)

    start_offset = checkpoint.get('links_offset', 0) if checkpoint else 0
    print(f"Total code links: {len(df)}, starting from offset: {start_offset}")

    cur = conn.cursor()
    inserted = 0
    skipped = 0

    for i in range(start_offset, len(df), batch_size):
        batch = df.iloc[i:i+batch_size]

        for _, row in batch.iterrows():
            try:
                # Find paper by arxiv_id
                arxiv_id = row.get('paper_arxiv_id')
                github_url = row.get('repo_url')

                if not github_url or pd.isna(arxiv_id):
                    skipped += 1
                    continue

                # Get paper_id
                cur.execute("""
                    SELECT id FROM papers WHERE arxiv_id = %s LIMIT 1;
                """, (arxiv_id,))

                result = cur.fetchone()
                if not result:
                    skipped += 1
                    continue

                paper_id = result[0]

                # Extract framework and metadata
                framework = row.get('framework') if pd.notna(row.get('framework')) else None
                is_official = row.get('is_official', False)

                # Add unique constraint check to avoid duplicates
                cur.execute("""
                    INSERT INTO implementations (paper_id, github_url, framework, is_official)
                    VALUES (%s, %s, %s, %s)
                    ON CONFLICT DO NOTHING
                    RETURNING id;
                """, (paper_id, github_url, framework, is_official))

                if cur.fetchone():
                    inserted += 1
                else:
                    skipped += 1

            except Exception as e:
                print(f"Error inserting implementation: {e}")
                continue

        conn.commit()

        # Save checkpoint after each batch
        if checkpoint is not None:
            checkpoint['links_offset'] = i + batch_size
            save_checkpoint(checkpoint)

        print(f"  Processed {min(i+batch_size, len(df))}/{len(df)} links ({inserted} inserted, {skipped} skipped)")

    if checkpoint is not None:
        checkpoint['links_complete'] = True
        save_checkpoint(checkpoint)

    cur.close()
    print(f"✓ Code links loaded: {inserted} inserted, {skipped} skipped")
    return inserted


def load_evaluation_tables(conn, batch_size=1000, checkpoint=None):
    """Load evaluation results from parquet file into database."""
    eval_file = DATA_DIR / "evaluation-tables" / "train.parquet"

    if not eval_file.exists():
        print(f"\n⚠ Evaluation tables file not found: {eval_file}")
        print("  Run: uv run python scripts/download_pwc_data.py --datasets evaluation-tables")
        return 0

    if checkpoint and checkpoint.get('eval_complete'):
        print("\n✓ Evaluation tables already loaded (skipping)")
        return 0

    print(f"\nLoading evaluation results from {eval_file}...")
    df = pd.read_parquet(eval_file)

    start_offset = checkpoint.get('eval_offset', 0) if checkpoint else 0
    print(f"Total evaluation records: {len(df)}, starting from offset: {start_offset}")

    cur = conn.cursor()
    inserted_benchmarks = 0
    inserted_results = 0
    skipped = 0

    for i in range(start_offset, len(df), batch_size):
        batch = df.iloc[i:i+batch_size]

        for _, row in batch.iterrows():
            try:
                # Get or create benchmark
                dataset_name = row.get('dataset')
                task = row.get('task')

                if pd.isna(dataset_name) or pd.isna(task):
                    skipped += 1
                    continue

                benchmark_name = f"{dataset_name} - {task}"

                # Find dataset
                cur.execute("SELECT id FROM datasets WHERE name = %s LIMIT 1;", (dataset_name,))
                dataset_result = cur.fetchone()
                dataset_id = dataset_result[0] if dataset_result else None

                # Insert/get benchmark
                cur.execute("""
                    INSERT INTO benchmarks (name, dataset_id, task)
                    VALUES (%s, %s, %s)
                    ON CONFLICT (name, dataset_id) DO UPDATE SET task = EXCLUDED.task
                    RETURNING id;
                """, (benchmark_name, dataset_id, task))

                benchmark_id = cur.fetchone()[0]
                inserted_benchmarks += 1

                # Get paper if available
                paper_id = None
                if 'paper_arxiv_id' in row and pd.notna(row.get('paper_arxiv_id')):
                    cur.execute("SELECT id FROM papers WHERE arxiv_id = %s LIMIT 1;", (row.get('paper_arxiv_id'),))
                    paper_result = cur.fetchone()
                    paper_id = paper_result[0] if paper_result else None

                # Insert result if we have paper and metric
                metric_name = row.get('metric')
                metric_value = row.get('value')

                if paper_id and metric_name and pd.notna(metric_value):
                    try:
                        metric_value = float(metric_value)
                        cur.execute("""
                            INSERT INTO benchmark_results (paper_id, benchmark_id, metric_name, metric_value)
                            VALUES (%s, %s, %s, %s)
                            ON CONFLICT (paper_id, benchmark_id, metric_name) DO NOTHING
                            RETURNING id;
                        """, (paper_id, benchmark_id, metric_name, metric_value))

                        if cur.fetchone():
                            inserted_results += 1
                    except (ValueError, TypeError):
                        pass  # Skip non-numeric metrics

            except Exception as e:
                print(f"Error inserting evaluation data: {e}")
                continue

        conn.commit()

        # Save checkpoint after each batch
        if checkpoint is not None:
            checkpoint['eval_offset'] = i + batch_size
            save_checkpoint(checkpoint)

        print(f"  Processed {min(i+batch_size, len(df))}/{len(df)} records")

    if checkpoint is not None:
        checkpoint['eval_complete'] = True
        save_checkpoint(checkpoint)

    cur.close()
    print(f"✓ Evaluation data loaded: {inserted_benchmarks} benchmarks, {inserted_results} results")
    return inserted_results


def main():
    """Main loading function."""
    parser = argparse.ArgumentParser(
        description="Load Papers with Code data into Postgres database",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Start or continue loading data
  python load_pwc_data_to_postgres.py

  # Start fresh (ignore any existing checkpoint)
  python load_pwc_data_to_postgres.py --fresh

  # Explicitly continue from checkpoint
  python load_pwc_data_to_postgres.py --continue

  # Clear checkpoint and exit
  python load_pwc_data_to_postgres.py --clear-checkpoint
        """
    )

    parser.add_argument(
        "--fresh",
        action="store_true",
        help="Start fresh, ignoring any existing checkpoint"
    )

    parser.add_argument(
        "--continue",
        dest="continue_load",
        action="store_true",
        help="Continue from last checkpoint (default behavior)"
    )

    parser.add_argument(
        "--clear-checkpoint",
        action="store_true",
        help="Clear checkpoint file and exit"
    )

    args = parser.parse_args()

    print("=" * 60)
    print("Papers with Code Data Loader (with Resume Support)")
    print("=" * 60)

    # Handle clear checkpoint
    if args.clear_checkpoint:
        if CHECKPOINT_FILE.exists():
            clear_checkpoint()
            print("\n✓ Checkpoint cleared")
        else:
            print("\n⚠ No checkpoint found")
        return

    if not DATA_DIR.exists():
        print(f"\n✗ Data directory not found: {DATA_DIR}")
        print("  Run: uv run python scripts/download_pwc_data.py")
        return

    # Load checkpoint
    if args.fresh:
        print("\n🆕 Starting fresh (ignoring checkpoint)")
        clear_checkpoint()
        checkpoint = load_checkpoint()
    else:
        checkpoint = load_checkpoint()

        if any([checkpoint.get('papers_complete'), checkpoint.get('datasets_complete'),
                checkpoint.get('links_complete'), checkpoint.get('eval_complete')]):
            print("\n📍 Found existing checkpoint - resuming from last position")

            papers_status = '✓ Complete' if checkpoint.get('papers_complete') else f'Resume at {checkpoint.get("papers_offset", 0)}'
            datasets_status = '✓ Complete' if checkpoint.get('datasets_complete') else f'Resume at {checkpoint.get("datasets_offset", 0)}'
            links_status = '✓ Complete' if checkpoint.get('links_complete') else f'Resume at {checkpoint.get("links_offset", 0)}'
            eval_status = '✓ Complete' if checkpoint.get('eval_complete') else f'Resume at {checkpoint.get("eval_offset", 0)}'

            print(f"  Papers: {papers_status}")
            print(f"  Datasets: {datasets_status}")
            print(f"  Links: {links_status}")
            print(f"  Evaluation: {eval_status}")
        else:
            print("\n🆕 Starting fresh load (no checkpoint found)")

    try:
        conn = connect_db()
        print("✓ Connected to database")

        # Load data in order (respecting foreign key constraints)
        papers_count = load_papers(conn, checkpoint=checkpoint)
        datasets_count = load_datasets(conn, checkpoint=checkpoint)
        links_count = load_code_links(conn, checkpoint=checkpoint)
        eval_count = load_evaluation_tables(conn, checkpoint=checkpoint)

        conn.close()

        print("\n" + "=" * 60)
        print("Summary:")
        print(f"  Papers: {papers_count}")
        print(f"  Datasets: {datasets_count}")
        print(f"  Code links: {links_count}")
        print(f"  Evaluation results: {eval_count}")
        print("=" * 60)

        # Clear checkpoint on successful completion
        print("\n✓ All data loaded successfully!")
        print("  Clearing checkpoint file...")
        clear_checkpoint()

    except KeyboardInterrupt:
        print("\n\n⚠ Interrupted by user")
        print("  Progress has been saved. Run the script again to resume.")
    except Exception as e:
        print(f"\n✗ Error: {e}")
        print("  Progress has been saved. Run the script again to resume.")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
