# Scripts

This directory contains utility scripts for the CodeWithPapers project.

## download_pwc_data.py

Downloads Papers with Code Archive datasets from Hugging Face.

### Prerequisites

This project uses [uv](https://github.com/astral-sh/uv) for fast Python package management.

**Install uv (if not already installed):**
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Install project dependencies:**
```bash
# From the project root
uv sync
```

This will create a virtual environment and install all required packages.

### Usage

All commands use `uv run` to execute scripts in the managed environment:

**List available datasets:**
```bash
uv run python scripts/download_pwc_data.py --list
```

**Download all datasets:**
```bash
uv run python scripts/download_pwc_data.py --datasets all
```

**Download specific datasets:**
```bash
# Download just papers and links
uv run python scripts/download_pwc_data.py --datasets papers links

# Download methods only
uv run python scripts/download_pwc_data.py --datasets methods
```

**Save in multiple formats:**
```bash
# Save as parquet, JSON, and CSV
uv run python scripts/download_pwc_data.py --datasets papers --formats parquet json csv
```

**Use custom output directory:**
```bash
uv run python scripts/download_pwc_data.py --datasets all --output /path/to/data
```

### Available Datasets

| Dataset | Description | Size |
|---------|-------------|------|
| `papers` | Papers with abstracts, authors, and metadata | ~576k rows |
| `links` | Links between papers and GitHub repositories | ~300k rows |
| `datasets` | ML datasets with descriptions and metadata | ~15k rows |
| `methods` | ML methods and techniques | ~8.73k rows |
| `evaluation-tables` | Benchmark evaluation tables | ~2.25k rows |
| `files` | Additional archive files | ~62 rows |

### Output Structure

Downloaded data is saved in the following structure:

```
data/pwc-archive/
├── papers-with-abstracts/
│   └── train.parquet
├── links-between-paper-and-code/
│   └── train.parquet
├── datasets/
│   └── train.parquet
├── methods/
│   └── train.parquet
├── evaluation-tables/
│   └── train.parquet
└── files/
    └── train.parquet
```

### Examples

```bash
# Quick start - download everything
uv run python scripts/download_pwc_data.py --datasets all

# Download papers with multiple formats for analysis
uv run python scripts/download_pwc_data.py --datasets papers --formats parquet json csv

# Download just the core datasets for development
uv run python scripts/download_pwc_data.py --datasets papers links methods
```

### Notes

- Default output directory: `data/pwc-archive/`
- Default format: `parquet` (most efficient for large datasets)
- All datasets are licensed under CC-BY-SA-4.0
- The archive is a snapshot from July 28-29, 2025 and is no longer updated
