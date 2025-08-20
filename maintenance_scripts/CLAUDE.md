# Papers with Code Archive Scraper - Development Notes

## Architecture Overview

The scraper is designed as a resilient, incremental web scraper that extracts structured data from archived Papers with Code pages on the Internet Archive.

### Core Components

1. **ArchiveScraper Class**: Main scraper class that handles all scraping logic
2. **Session Management**: HTTP session with retry logic and rate limiting
3. **Checkpoint System**: Tracks scraped URLs to enable resumption
4. **Data Extraction**: BeautifulSoup-based HTML parsing with fallback strategies
5. **Archive URL Validation**: Automatic snapshot discovery via CDX API

## Key Design Decisions

### Incremental Scraping

The scraper maintains a `scraped_urls.json` checkpoint file that tracks all previously scraped URLs. This enables:

- Resuming interrupted scraping sessions
- Avoiding duplicate work
- Incremental updates over time

### Archive.org Integration

The scraper specifically works with Archive.org snapshots because:

- Live Papers with Code may have rate limiting or anti-scraping measures
- Archive provides consistent, stable snapshots
- Historical data can be preserved

### Error Handling Strategy

1. **Automatic Snapshot Discovery**: If the configured archive URL fails, the scraper queries the CDX API to find valid snapshots
2. **Retry Logic**: Built-in retry mechanism with exponential backoff for transient failures
3. **Graceful Degradation**: Missing elements don't crash the scraper; it continues with partial data

## Code Structure

```
archive_scraper.py
├── Configuration Loading
│   └── _load_config()
├── Session Setup
│   └── _setup_session()
├── Archive Validation
│   ├── validate_archive_url()
│   └── find_valid_snapshot()
├── Page Fetching
│   └── _get_page()
├── Content Extraction
│   ├── scrape_main_page()
│   ├── scrape_task_page()
│   └── scrape_paper_page()
├── Data Processing
│   ├── _parse_table()
│   └── _normalize_url()
├── Persistence
│   ├── _save_json()
│   ├── _load_scraped_urls()
│   └── _save_scraped_urls()
└── Main Execution
    └── run()
```

## URL Normalization

The `_normalize_url()` method handles various URL formats:

1. Already normalized archive.org URLs
2. Archive-relative paths (`/web/TIMESTAMP/...`)
3. Direct paperswithcode.com URLs
4. Relative paths (`/task/...`, `/paper/...`)

## Data Extraction Patterns

### Main Page

- Navigation menu extraction with multiple fallback selectors
- Task discovery through link analysis
- Featured papers collection

### Task Pages

- SOTA table extraction with header mapping
- Paper list aggregation
- Dataset relationship discovery

### Paper Pages

- Metadata extraction (title, abstract, authors)
- Code repository link discovery
- Benchmark results parsing

## Performance Considerations

1. **Rate Limiting**: Default 1.5-second delay between requests
2. **Batch Processing**: Configurable limits for papers and tasks
3. **Memory Management**: Data saved incrementally to disk

## Extension Points

### Adding New Data Types

To scrape additional data types:

1. Create a new `scrape_*_page()` method
2. Add corresponding data directory in `__init__()`
3. Update `run()` method to include new scraping logic
4. Add URL pattern recognition in main page scraping

### Custom Extractors

For specific data formats, add methods following the pattern:

```python
def extract_custom_data(self, soup: BeautifulSoup) -> Dict:
    data = {}
    # Custom extraction logic
    return data
```

## Testing Strategy

### Manual Testing

```bash
# Test with minimal data
python archive_scraper.py --max-tasks 1 --max-papers 2

# Test checkpoint recovery
# Run, interrupt with Ctrl+C, then run again
python archive_scraper.py --max-tasks 5
```

### Validation

- Check JSON output for completeness
- Verify checkpoint file updates
- Monitor log output for errors

## Common Issues and Solutions

### Issue: SSL/TLS Error (525)

**Solution**: The scraper automatically finds a valid snapshot using the CDX API

### Issue: Missing Elements

**Solution**: The scraper uses multiple fallback selectors and continues with partial data

### Issue: Rate Limiting

**Solution**: Increase delay in config or reduce batch size

### Issue: Memory Usage

**Solution**: Process data in smaller batches, use checkpoint system

## Future Improvements

1. **Parallel Processing**: Add threading for faster scraping (with rate limit awareness)
2. **Data Validation**: Add schema validation for extracted data
3. **Export Formats**: Support for CSV, SQLite, or other formats
4. **Incremental Updates**: Smart detection of new content
5. **Relationship Mapping**: Build graph of paper-task-dataset relationships
6. **Search Functionality**: Add ability to search scraped data
7. **Deduplication**: Intelligent handling of duplicate papers across tasks

## Maintenance Notes

### Updating Archive URL

If the default archive snapshot becomes unavailable:

1. Find a new snapshot on https://web.archive.org/web/*/paperswithcode.com
2. Update `base_url` in `scraper_config.yaml`
3. Or let the automatic snapshot discovery handle it

### Monitoring Scraping Progress

```bash
# Watch the log file
tail -f scraper.log

# Check progress
ls -la scraped_data/tasks/ | wc -l  # Count scraped tasks
ls -la scraped_data/papers/ | wc -l # Count scraped papers
```

### Cleaning Up

To start fresh:

```bash
rm -rf scraped_data/
rm scraper.log
```

Or use the `--fresh-start` flag which ignores previous progress but keeps the data.

## Dependencies

- `requests`: HTTP client with session management
- `beautifulsoup4`: HTML parsing
- `pyyaml`: Configuration file parsing
- `urllib3`: Retry logic support

## Code Quality

The code follows these principles:

- Type hints for better IDE support
- Comprehensive logging for debugging
- Defensive programming with error handling
- Modular design for easy extension
- Clear separation of concerns
