# Papers with Code Archive Scraper

A comprehensive scraping solution for reconstructing Papers with Code from Web Archive snapshots.

## Overview

This scraper extracts paper information, benchmarks, code implementations, and relationships from archived Papers with Code pages. It's designed to be respectful to archive.org servers while efficiently gathering data.

## Setup

1. Install dependencies:

```bash
pip install -r requirements_scraper.txt
```

2. Configure scraper settings in `scraper_config.yaml` if needed (defaults are provided)

## Usage

### Basic Scraping

Run the main scraper to start collecting data:

```bash
python archive_scraper.py
```

This will:

- Scrape the main page for navigation and task categories
- Extract papers from task pages
- Collect detailed information for each paper
- Save raw data in `scraped_data/` directory

### Data Processing

Clean and normalize the scraped data:

```bash
python data_processor.py
```

This will:

- Clean text and fix encoding issues
- Extract keywords and relationships
- Build connection graphs between papers
- Generate statistics
- Save processed data in `processed_data/` directory

### Merge with Existing Data

Integrate scraped data with your existing benchmark_metrics.yaml:

```bash
python merge_data.py
```

This will:

- Load existing paper data
- Match and merge papers by title
- Add new papers not in existing dataset
- Update incomplete entries with new information
- Generate merge report showing changes

## File Structure

```
maintenance_scripts/
├── archive_scraper.py          # Main scraping logic
├── scraper_config.yaml         # Configuration settings
├── data_processor.py           # Data cleaning and processing
├── merge_data.py              # Merge with existing data
├── requirements_scraper.txt    # Python dependencies
│
├── scraped_data/              # Raw scraped data (created automatically)
│   ├── papers/                # Individual paper JSON files
│   ├── tasks/                 # Task pages data
│   ├── datasets/              # Dataset information
│   └── checkpoints/           # Scraping progress checkpoints
│
└── processed_data/            # Cleaned data (created automatically)
    ├── papers/                # Processed paper files
    ├── relationships/         # Paper relationship graphs
    └── statistics.json        # Summary statistics

```

## Configuration

Key settings in `scraper_config.yaml`:

- `delay`: Time between requests (default: 1.5 seconds)
- `max_papers_per_task`: Limit papers scraped per task
- `priority_tasks`: Tasks to scrape first
- `resume_from_checkpoint`: Continue from last checkpoint if interrupted

## Features

### Incremental Scraping

- Tracks scraped URLs to avoid duplicates
- Saves progress checkpoints
- Can resume from interruptions

### Data Quality

- Cleans Web Archive artifacts from text
- Normalizes author names
- Extracts GitHub repository information
- Identifies relationships between papers

### Respectful Scraping

- Rate limiting with configurable delays
- Retry logic with exponential backoff
- User-agent identification
- Respects archive.org server load

## Advanced Usage

### Scrape Specific Tasks

Modify the scraper to focus on specific research areas:

```python
scraper = ArchiveScraper()
task_url = "https://web.archive.org/web/20250708172035/https://paperswithcode.com/task/object-detection"
scraper.scrape_task_page(task_url)
```

### Custom Processing

Extend the data processor for custom fields:

```python
processor = DataProcessor()
processor.extract_custom_field = lambda paper: your_custom_logic(paper)
processor.run()
```

## Troubleshooting

### Common Issues

1. **429 Too Many Requests**: Increase delay in config
2. **Timeout Errors**: Increase timeout setting or check network
3. **Memory Issues**: Reduce max_papers_per_task
4. **Encoding Errors**: The processor handles most encoding issues automatically

### Logs

Check `scraper.log` for detailed execution information and errors.

## Notes

- The scraper is designed for archive.org specifically
- Be respectful of archive.org's servers - don't reduce delays below 1 second
- Large-scale scraping should be done incrementally over time
- Consider using archive.org's APIs if available for your use case

## Future Improvements

- Add async scraping for better performance
- Implement citation graph extraction
- Add paper PDF download capability
- Extract more detailed benchmark metrics
- Add command-line interface with arguments
