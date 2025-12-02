#!/usr/bin/env python3
"""Stream remaining PWC datasets directly to database without local caching."""

import os
import json
import psycopg2
from datasets import load_dataset

DATABASE_URL = os.getenv("DATABASE_URL") or os.getenv("POSTGRES_URI")
if not DATABASE_URL:
    raise ValueError("DATABASE_URL or POSTGRES_URI must be set")


def stream_evaluation_tables():
    """Stream evaluation-tables directly to database."""
    print("\n=== Streaming evaluation-tables ===")

    conn = psycopg2.connect(DATABASE_URL)
    cur = conn.cursor()

    ds = load_dataset('pwc-archive/evaluation-tables', streaming=True)

    inserted = 0
    errors = 0

    for i, row in enumerate(ds['train']):
        try:
            task_name = row.get('task', '')
            description = row.get('description', '')
            categories = row.get('categories', [])
            datasets_info = row.get('datasets', [])

            # Process each dataset in the evaluation table
            for ds_info in datasets_info:
                if isinstance(ds_info, dict):
                    dataset_name = ds_info.get('dataset', '')

                    if dataset_name and task_name:
                        # Get or create dataset
                        cur.execute("""
                            INSERT INTO datasets (name, description)
                            VALUES (%s, %s)
                            ON CONFLICT (name) DO UPDATE SET updated_at = NOW()
                            RETURNING id
                        """, (dataset_name, description[:500] if description else None))

                        result = cur.fetchone()
                        if result:
                            dataset_id = result[0]

                            # Create benchmark
                            benchmark_name = f"{task_name} on {dataset_name}"
                            cur.execute("""
                                INSERT INTO benchmarks (name, dataset_id, task, description)
                                VALUES (%s, %s, %s, %s)
                                ON CONFLICT (name, dataset_id) DO NOTHING
                            """, (benchmark_name, dataset_id, task_name, description[:500] if description else None))

                            inserted += 1

            if (i + 1) % 100 == 0:
                conn.commit()
                print(f"  Processed {i + 1} rows, inserted {inserted} benchmarks")

        except Exception as e:
            errors += 1
            if errors <= 5:
                print(f"  Error on row {i}: {e}")

    conn.commit()
    cur.close()
    conn.close()

    print(f"✓ Evaluation tables complete: {inserted} benchmarks inserted, {errors} errors")
    return inserted


def stream_files():
    """Stream files dataset to see what it contains."""
    print("\n=== Streaming files ===")

    ds = load_dataset('pwc-archive/files', streaming=True)

    count = 0
    for split_name in ds.keys():
        print(f"  Split: {split_name}")
        for i, row in enumerate(ds[split_name]):
            print(f"    Row {i}: {list(row.keys())}")
            count += 1
            if i >= 4:
                print(f"    ... (showing first 5)")
                break

    print(f"✓ Files dataset has multiple splits, {count}+ rows visible")
    return count


def main():
    print("Streaming remaining PWC datasets to database")
    print("=" * 60)

    # Stream evaluation tables
    eval_count = stream_evaluation_tables()

    # Stream files (just inspect for now)
    files_count = stream_files()

    print("\n" + "=" * 60)
    print("Summary:")
    print(f"  Evaluation tables: {eval_count} benchmarks")
    print(f"  Files: {files_count}+ (inspected)")
    print("=" * 60)


if __name__ == "__main__":
    main()
