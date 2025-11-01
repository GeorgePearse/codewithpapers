#!/usr/bin/env python3
"""Create database tables in Neon Postgres."""

import os
import psycopg2
from pathlib import Path

# Connection string from environment
DATABASE_URL = os.getenv('DATABASE_URL')
if not DATABASE_URL:
    raise ValueError("DATABASE_URL environment variable is not set. Please create a .env.local file with your database credentials.")

def create_tables():
    """Execute migration SQL to create tables."""
    try:
        # Read migration SQL
        migration_file = Path(__file__).parent.parent / "supabase" / "migrations" / "001_initial_schema.sql"

        print(f"Reading migration file: {migration_file}")
        with open(migration_file, 'r') as f:
            migration_sql = f.read()

        # Connect to database
        print("\nConnecting to database...")
        conn = psycopg2.connect(DATABASE_URL)
        conn.autocommit = True  # Enable autocommit for CREATE EXTENSION and other commands
        cur = conn.cursor()

        print("Executing migration SQL...")
        cur.execute(migration_sql)

        print("✓ Tables created successfully!\n")

        # List created tables
        cur.execute("""
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name IN ('papers', 'datasets', 'benchmarks', 'implementations', 'benchmark_results', 'paper_datasets')
            ORDER BY table_name;
        """)

        tables = cur.fetchall()
        if tables:
            print("Created tables:")
            for table in tables:
                # Get row count
                cur.execute(f"SELECT COUNT(*) FROM {table[0]};")
                count = cur.fetchone()[0]
                print(f"  - {table[0]} ({count} rows)")

        cur.close()
        conn.close()

        return True

    except Exception as e:
        print(f"✗ Error creating tables: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    create_tables()
