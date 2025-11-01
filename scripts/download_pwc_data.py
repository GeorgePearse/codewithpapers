#!/usr/bin/env python3
"""
Download Papers with Code Archive datasets from Hugging Face.

This script downloads the PWC archive datasets and saves them in various formats
(parquet, JSON, CSV) for easy processing.
"""

import argparse
import sys
from pathlib import Path

try:
    from datasets import load_dataset
except ImportError:
    print("Error: Required packages not installed.")
    print("Please install them with: pip install datasets huggingface_hub")
    sys.exit(1)


# Dataset configurations
PWC_DATASETS = {
    "papers": {
        "id": "pwc-archive/papers-with-abstracts",
        "description": "576k papers with abstracts, authors, and metadata",
        "size": "~576k rows",
    },
    "links": {
        "id": "pwc-archive/links-between-paper-and-code",
        "description": "300k links between papers and GitHub repositories",
        "size": "~300k rows",
    },
    "datasets": {
        "id": "pwc-archive/datasets",
        "description": "15k ML datasets with descriptions and metadata",
        "size": "~15k rows",
    },
    "methods": {
        "id": "pwc-archive/methods",
        "description": "8.73k ML methods and techniques",
        "size": "~8.73k rows",
    },
    "evaluation-tables": {
        "id": "pwc-archive/evaluation-tables",
        "description": "2.25k benchmark evaluation tables",
        "size": "~2.25k rows",
    },
    "files": {
        "id": "pwc-archive/files",
        "description": "62 additional archive files",
        "size": "~62 rows",
    },
}


def download_dataset(
    dataset_id: str,
    output_dir: Path,
    formats: list[str] | None = None,
    split: str | None = None,
) -> bool:
    """
    Download a dataset from Hugging Face and save in specified formats.

    Args:
        dataset_id: HuggingFace dataset ID (e.g., 'pwc-archive/papers-with-abstracts')
        output_dir: Directory to save the downloaded data
        formats: List of formats to save ('parquet', 'json', 'csv')
        split: Specific split to download (e.g., 'train'), or None for all splits

    Returns:
        True if successful, False otherwise
    """
    if formats is None:
        formats = ["parquet"]

    try:
        print(f"\n{'='*60}")
        print(f"Downloading: {dataset_id}")
        print(f"{'='*60}")

        # Create output directory
        dataset_name = dataset_id.split("/")[-1]
        dataset_dir = output_dir / dataset_name
        dataset_dir.mkdir(parents=True, exist_ok=True)

        # Load dataset
        print("Loading dataset from Hugging Face...")
        dataset = load_dataset(dataset_id, split=split)

        # Handle dataset dict vs single dataset
        if hasattr(dataset, "keys"):
            # Multiple splits
            print(f"Found splits: {list(dataset.keys())}")
            datasets_to_save = dataset
        else:
            # Single split
            datasets_to_save = {"train": dataset}

        # Save in requested formats
        for split_name, split_data in datasets_to_save.items():
            print(f"\nProcessing split '{split_name}' ({len(split_data)} rows)...")

            for format_type in formats:
                output_file = dataset_dir / f"{split_name}.{format_type}"
                print(f"  Saving as {format_type}: {output_file.name}")

                if format_type == "parquet":
                    split_data.to_parquet(str(output_file))
                elif format_type == "json":
                    split_data.to_json(str(output_file), orient="records", lines=True)
                elif format_type == "csv":
                    split_data.to_csv(str(output_file), index=False)
                else:
                    print(f"  Warning: Unknown format '{format_type}', skipping")

        print(f"\n✓ Successfully downloaded {dataset_id}")
        print(f"  Saved to: {dataset_dir}")
        return True

    except Exception as e:
        print(f"\n✗ Error downloading {dataset_id}: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="Download Papers with Code Archive datasets from Hugging Face",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Available datasets:
  papers              - Papers with abstracts (576k rows)
  links               - Links between papers and code (300k rows)
  datasets            - ML datasets metadata (15k rows)
  methods             - ML methods and techniques (8.73k rows)
  evaluation-tables   - Benchmark evaluation tables (2.25k rows)
  files               - Additional archive files (62 rows)
  all                 - Download all datasets

Examples:
  # Download all datasets
  python download_pwc_data.py --datasets all

  # Download specific datasets
  python download_pwc_data.py --datasets papers links methods

  # Save in multiple formats
  python download_pwc_data.py --datasets papers --formats parquet json csv

  # Use custom output directory
  python download_pwc_data.py --datasets all --output /path/to/data
        """
    )

    parser.add_argument(
        "--datasets",
        nargs="+",
        default=["all"],
        choices=list(PWC_DATASETS.keys()) + ["all"],
        help="Datasets to download (default: all)",
    )

    parser.add_argument(
        "--output",
        type=str,
        default="data/pwc-archive",
        help="Output directory for downloaded data (default: data/pwc-archive)",
    )

    parser.add_argument(
        "--formats",
        nargs="+",
        default=["parquet"],
        choices=["parquet", "json", "csv"],
        help="Output formats (default: parquet)",
    )

    parser.add_argument(
        "--split",
        type=str,
        default=None,
        help="Specific split to download (e.g., 'train'), or None for all splits",
    )

    parser.add_argument(
        "--list",
        action="store_true",
        help="List available datasets and exit",
    )

    parser.add_argument(
        "--load-to-db",
        action="store_true",
        help="Automatically load downloaded data into Postgres database",
    )

    args = parser.parse_args()

    # List datasets if requested
    if args.list:
        print("\nAvailable Papers with Code Archive datasets:\n")
        for name, info in PWC_DATASETS.items():
            print(f"  {name:20} - {info['description']}")
            print(f"  {'':20}   Size: {info['size']}")
            print(f"  {'':20}   ID: {info['id']}\n")
        return

    # Determine which datasets to download
    if "all" in args.datasets:
        datasets_to_download = list(PWC_DATASETS.keys())
    else:
        datasets_to_download = args.datasets

    # Create output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("\nPapers with Code Archive Downloader")
    print(f"{'='*60}")
    print(f"Output directory: {output_dir.absolute()}")
    print(f"Formats: {', '.join(args.formats)}")
    print(f"Datasets to download: {', '.join(datasets_to_download)}")

    # Download datasets
    results = {}
    for dataset_name in datasets_to_download:
        dataset_info = PWC_DATASETS[dataset_name]
        success = download_dataset(
            dataset_id=dataset_info["id"],
            output_dir=output_dir,
            formats=args.formats,
            split=args.split,
        )
        results[dataset_name] = success

    # Print summary
    print(f"\n{'='*60}")
    print("Download Summary")
    print(f"{'='*60}")

    successful = sum(1 for s in results.values() if s)
    failed = sum(1 for s in results.values() if not s)

    for name, success in results.items():
        status = "✓" if success else "✗"
        print(f"  {status} {name:20} - {PWC_DATASETS[name]['description']}")

    print(f"\nTotal: {successful} successful, {failed} failed")
    print(f"Data saved to: {output_dir.absolute()}")

    # Load to database if requested
    if args.load_to_db and successful > 0:
        print(f"\n{'='*60}")
        print("Loading data into Postgres database...")
        print(f"{'='*60}")

        try:
            import subprocess
            result = subprocess.run(
                ["python", "scripts/load_pwc_data_to_postgres.py"],
                cwd=output_dir.parent.parent,
                capture_output=False
            )
            if result.returncode == 0:
                print("\n✓ Data successfully loaded into database")
            else:
                print("\n⚠ Error loading data into database")
        except Exception as e:
            print(f"\n⚠ Could not load data into database: {e}")
            print("  You can manually run: uv run python scripts/load_pwc_data_to_postgres.py")


if __name__ == "__main__":
    main()
