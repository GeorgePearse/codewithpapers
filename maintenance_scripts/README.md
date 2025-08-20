# Papers with Code Archive Scraper

A Python scraper for extracting research papers, benchmarks, and datasets from archived versions of Papers with Code on the Internet Archive.

## Features

- Scrapes papers, tasks, datasets, and benchmarks from Papers with Code archives
- Supports incremental scraping with checkpoint recovery
- Handles Archive.org rate limiting and errors gracefully
- Extracts structured data including paper abstracts, authors, code links, and benchmark results
- Saves data in JSON format for easy processing

## Installation

```bash
# Install required dependencies
pip install requests beautifulsoup4 pyyaml
```

## Usage

### Basic Usage

```bash
# Run with default settings (5 tasks, 10 papers per task)
python archive_scraper.py

# Scrape more papers per task
python archive_scraper.py --max-papers 50

# Scrape more tasks
python archive_scraper.py --max-tasks 20

# Scrape everything (be careful - this will take a long time!)
python archive_scraper.py --max-tasks 100 --max-papers 100

# Start fresh (ignore previous progress)
python archive_scraper.py --fresh-start
```

### Command Line Options

- `--max-papers`: Maximum number of papers to scrape per task (default: 10)
- `--max-tasks`: Maximum number of tasks to scrape (default: 5)
- `--fresh-start`: Start from the beginning, ignoring previous progress

### Continuing Interrupted Scraping

The scraper automatically saves progress and can resume from where it left off:

```bash
# First run - scrapes 5 tasks
python archive_scraper.py --max-tasks 5

# Continue with more tasks
python archive_scraper.py --max-tasks 10
```

## Output Structure

The scraper creates a `scraped_data/` directory with the following structure:

```
scraped_data/
├── main_page.json           # Main page data with navigation and featured papers
├── tasks/                   # Task pages with SOTA tables
│   ├── quantization.json
│   ├── text-generation.json
│   └── ...
├── papers/                  # Individual paper data
│   ├── paper-id-1.json
│   ├── paper-id-2.json
│   └── ...
├── datasets/               # Dataset information (if available)
├── relationships/          # Paper-task-dataset relationships
└── checkpoints/            # Scraping progress
    └── scraped_urls.json  # List of already scraped URLs
```

## Data Format

### Paper JSON Format

```json
{
  "timestamp": "2025-08-20T10:00:00",
  "url": "archive.org URL",
  "id": "paper-id",
  "title": "Paper Title",
  "abstract": "Paper abstract...",
  "authors": ["Author 1", "Author 2"],
  "venue": "Conference/Journal",
  "year": "2024",
  "code_links": [
    {
      "url": "https://github.com/...",
      "text": "Official Implementation"
    }
  ],
  "datasets": ["dataset1", "dataset2"],
  "tasks": ["task1", "task2"],
  "results": [...]
}
```

### Task JSON Format

```json
{
  "timestamp": "2025-08-20T10:00:00",
  "url": "archive.org URL",
  "name": "task-name",
  "description": "Task description...",
  "papers": [
    {
      "title": "Paper Title",
      "url": "paper URL"
    }
  ],
  "sota_table": [
    {
      "Method": "Method Name",
      "Score": "95.2",
      "Paper": {
        "text": "Paper Title",
        "paper_url": "URL"
      }
    }
  ]
}
```

## Configuration

The scraper can be configured via `scraper_config.yaml`:

```yaml
base_url: 'https://web.archive.org/web/20240101123127/https://paperswithcode.com'
delay: 1.5 # Delay between requests (seconds)
max_retries: 3 # Maximum retry attempts
timeout: 30 # Request timeout (seconds)
user_agent: 'Mozilla/5.0...' # User agent string
data_dir: 'scraped_data' # Output directory
```

## Troubleshooting

### Archive URL Issues

If you encounter SSL/TLS errors (status 525), the scraper will automatically try to find a valid archive snapshot. You can also manually specify a different archive date in the config.

### Rate Limiting

The scraper includes a default 1.5-second delay between requests. If you encounter rate limiting, increase the delay in `scraper_config.yaml`.

### Memory Issues

For large-scale scraping, consider:

- Running in smaller batches
- Using the checkpoint system to save progress
- Processing data incrementally rather than loading everything at once

## Development

See `CLAUDE.md` for development notes and architecture details.

## License

This tool is for research and educational purposes. Please respect the Papers with Code terms of service and the Internet Archive's usage guidelines.
