#!/usr/bin/env python3
"""Test connection to Neon Postgres database."""

import psycopg2
from psycopg2 import sql

# Connection string
DATABASE_URL = "postgresql://neondb_owner:npg_NwBESm09zFAW@ep-royal-rice-ad5hm3zh-pooler.c-2.us-east-1.aws.neon.tech/neondb?sslmode=require"

def test_connection():
    """Test database connection and list existing tables."""
    try:
        # Connect to the database
        conn = psycopg2.connect(DATABASE_URL)
        cur = conn.cursor()

        print("✓ Successfully connected to Neon Postgres database!")

        # Get database version
        cur.execute("SELECT version();")
        version = cur.fetchone()[0]
        print(f"\nPostgreSQL version:\n{version}\n")

        # List existing tables
        cur.execute("""
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            ORDER BY table_name;
        """)

        tables = cur.fetchall()
        if tables:
            print("Existing tables:")
            for table in tables:
                print(f"  - {table[0]}")
        else:
            print("No tables found in public schema.")

        cur.close()
        conn.close()

        return True

    except Exception as e:
        print(f"✗ Connection failed: {e}")
        return False

if __name__ == "__main__":
    test_connection()
