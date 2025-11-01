# Database Setup Guide

This guide explains how to download Papers with Code data and load it into your Postgres database.

## Data Sources

- **Hugging Face Archive**: https://huggingface.co/datasets/pwc-archive (primary source for parquet files)
- **Archive.org**: https://web.archive.org/web/20241101000000*/paperswithcode.com (most recent archive snapshots)

The goal is to rebuild Papers with Code using these archived datasets.

## Prerequisites

The database connection is already configured to:
```
postgresql://neondb_owner:npg_NwBESm09zFAW@ep-royal-rice-ad5hm3zh-pooler.c-2.us-east-1.aws.neon.tech/neondb?sslmode=require
```

## Quick Start

### 1. Test Database Connection

```bash
uv run python scripts/test_postgres_connection.py
```

This will verify you can connect to the Neon Postgres database.

### 2. Create Database Tables

The tables have already been created, but if you need to recreate them:

```bash
uv run python scripts/create_database_tables.py
```

This creates the following tables:
- `papers` - Paper metadata (title, abstract, arxiv_id, etc.)
- `datasets` - ML dataset information
- `benchmarks` - Benchmark definitions
- `implementations` - Code implementations (GitHub repos)
- `benchmark_results` - Performance metrics
- `paper_datasets` - Many-to-many relationship table

### 3. Download and Load Data

**Note**: Loading the full dataset takes 10-30 minutes and processes ~576k papers. The script saves checkpoints every 1,000 rows, so you can safely interrupt and resume.

#### Option A: Download and Load in One Step (Recommended)

```bash
# Download all datasets and automatically load into database
uv run python scripts/download_pwc_data.py --datasets all --load-to-db

# Or download specific datasets
uv run python scripts/download_pwc_data.py --datasets papers links datasets --load-to-db
```

#### Option B: Download First, Then Load Separately

```bash
# 1. Download the data
uv run python scripts/download_pwc_data.py --datasets all

# 2. Load into database
uv run python scripts/load_pwc_data_to_postgres.py
```

## Available Datasets

- **papers** - 576k papers with abstracts, authors, and metadata
- **links** - 300k links between papers and GitHub repositories
- **datasets** - 15k ML datasets with descriptions
- **methods** - 8.73k ML methods and techniques
- **evaluation-tables** - 2.25k benchmark evaluation tables
- **all** - All of the above

## Data Loading Details

The loading script processes data in batches of 1000 rows and:

- **Papers**: Inserts papers with arxiv_id, skips duplicates
- **Datasets**: Inserts dataset metadata
- **Code Links**: Links papers to their GitHub implementations
- **Evaluation Tables**: Creates benchmarks and links results to papers

## Checking Progress

The script saves checkpoints every 1,000 rows. To check current progress:

```bash
uv run python scripts/check_load_progress.py
```

This shows:
- Checkpoint status (rows processed per dataset)
- Database row counts
- Latest paper inserted
- Percentage complete

## Resuming After Interruption

The loading script automatically resumes from the last checkpoint:

```bash
# Simply run the script again - it will continue where it left off
uv run python scripts/load_pwc_data_to_postgres.py

# Or explicitly use --continue flag
uv run python scripts/load_pwc_data_to_postgres.py --continue

# To start fresh (ignore checkpoint)
uv run python scripts/load_pwc_data_to_postgres.py --fresh

# To clear checkpoint and exit
uv run python scripts/load_pwc_data_to_postgres.py --clear-checkpoint
```

## Testing

To test with a small sample (10 rows):

```bash
uv run python scripts/test_load_sample.py
```

## Database Schema

### Papers Table
```sql
- id (UUID, primary key)
- title (TEXT)
- abstract (TEXT)
- arxiv_id (VARCHAR(20), unique)
- arxiv_url (TEXT)
- pdf_url (TEXT)
- published_date (DATE)
- authors (JSONB)
```

### Datasets Table
```sql
- id (UUID, primary key)
- name (TEXT, unique)
- description (TEXT)
- modalities (TEXT[])
- homepage_url (TEXT)
- paper_url (TEXT)
```

### Implementations Table
```sql
- id (UUID, primary key)
- paper_id (UUID, foreign key)
- github_url (TEXT)
- framework (VARCHAR(50))
- stars (INTEGER)
- is_official (BOOLEAN)
```

## Troubleshooting

### Connection Issues
If you can't connect, verify the connection string is correct:
```bash
psql 'postgresql://neondb_owner:npg_NwBESm09zFAW@ep-royal-rice-ad5hm3zh-pooler.c-2.us-east-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require'
```

### Missing Dependencies
If you get import errors, sync dependencies:
```bash
uv sync
```

### Data Already Exists
The scripts use `ON CONFLICT DO NOTHING` so they're safe to run multiple times. Duplicates will be skipped.

## Expected Data Volumes

After loading all datasets, you should have approximately:
- **Papers**: ~509k (only papers with arxiv_id are loaded)
- **Datasets**: ~15k
- **Implementations**: varies based on links
- **Benchmarks**: ~2k+
- **Results**: varies based on evaluation tables

## Notes

- Only papers with an `arxiv_id` are loaded to ensure proper linking
- Loading the full dataset can take 10-30 minutes depending on your connection
- The database uses UUID primary keys and automatic timestamp tracking
- All text fields support full Unicode content
