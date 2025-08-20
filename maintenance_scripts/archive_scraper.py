#!/usr/bin/env python3
"""
Web Archive Scraper for Papers with Code
Scrapes the archived version of paperswithcode.com from archive.org
"""

import json
import logging
import os
import re
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set
from urllib.parse import urljoin, urlparse

import requests
import yaml
from bs4 import BeautifulSoup
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('scraper.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


class ArchiveScraper:
    """Scraper for archived Papers with Code website"""
    
    def __init__(self, config_path: str = "scraper_config.yaml"):
        """Initialize scraper with configuration"""
        self.config = self._load_config(config_path)
        self.base_url = self.config['base_url']
        self.session = self._setup_session()
        self.data_dir = Path(self.config['data_dir'])
        self.data_dir.mkdir(exist_ok=True)
        self.validate_archive_url()
        
        # Create subdirectories
        for subdir in ['papers', 'tasks', 'datasets', 'relationships', 'checkpoints']:
            (self.data_dir / subdir).mkdir(exist_ok=True)
        
        # Track scraped URLs to avoid duplicates
        self.scraped_urls = self._load_scraped_urls()
        
    def _load_config(self, config_path: str) -> Dict:
        """Load configuration from YAML file"""
        if os.path.exists(config_path):
            with open(config_path, 'r') as f:
                return yaml.safe_load(f)
        else:
            # Default configuration
            return {
                'base_url': 'https://web.archive.org/web/20240101123127/https://paperswithcode.com',
                'delay': 1.5,
                'max_retries': 3,
                'timeout': 30,
                'user_agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
                'data_dir': 'scraped_data'
            }
    
    def validate_archive_url(self):
        """Validate that the archive URL is accessible"""
        try:
            response = self.session.head(self.base_url, allow_redirects=True, timeout=10)
            if response.status_code >= 400:
                logger.warning(f"Archive URL returned status {response.status_code}. Trying to find valid snapshot...")
                self.base_url = self.find_valid_snapshot()
        except Exception as e:
            logger.warning(f"Could not validate archive URL: {e}. Trying to find valid snapshot...")
            self.base_url = self.find_valid_snapshot()
    
    def find_valid_snapshot(self) -> str:
        """Find a valid archive.org snapshot for paperswithcode.com"""
        try:
            # Query the CDX API for available snapshots
            cdx_url = "https://web.archive.org/cdx/search/cdx?url=paperswithcode.com&limit=10&output=json&fl=timestamp,statuscode&filter=statuscode:200&from=202301"
            response = self.session.get(cdx_url, timeout=10)
            data = response.json()
            
            if len(data) > 1:  # First row is headers
                # Get the most recent successful snapshot
                for row in reversed(data[1:]):
                    timestamp, status = row
                    if status == "200":
                        new_url = f"https://web.archive.org/web/{timestamp}/https://paperswithcode.com"
                        logger.info(f"Using valid snapshot: {new_url}")
                        return new_url
            
            # Fallback to a known working snapshot
            fallback_url = "https://web.archive.org/web/20240101123127/https://paperswithcode.com"
            logger.info(f"Using fallback snapshot: {fallback_url}")
            return fallback_url
            
        except Exception as e:
            logger.error(f"Error finding valid snapshot: {e}")
            # Return a known working snapshot as last resort
            return "https://web.archive.org/web/20240101123127/https://paperswithcode.com"
    
    def _setup_session(self) -> requests.Session:
        """Setup requests session with retry logic"""
        session = requests.Session()
        
        # Setup retry strategy
        retry = Retry(
            total=self.config['max_retries'],
            read=self.config['max_retries'],
            connect=self.config['max_retries'],
            backoff_factor=0.5,
            status_forcelist=(500, 502, 503, 504)
        )
        
        adapter = HTTPAdapter(max_retries=retry)
        session.mount('http://', adapter)
        session.mount('https://', adapter)
        
        # Set headers
        session.headers.update({
            'User-Agent': self.config['user_agent'],
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Accept-Encoding': 'gzip, deflate',
            'Connection': 'keep-alive',
        })
        
        return session
    
    def _load_scraped_urls(self) -> Set[str]:
        """Load set of already scraped URLs"""
        checkpoint_file = self.data_dir / 'checkpoints' / 'scraped_urls.json'
        if checkpoint_file.exists():
            with open(checkpoint_file, 'r') as f:
                return set(json.load(f))
        return set()
    
    def _save_scraped_urls(self):
        """Save set of scraped URLs"""
        checkpoint_file = self.data_dir / 'checkpoints' / 'scraped_urls.json'
        with open(checkpoint_file, 'w') as f:
            json.dump(list(self.scraped_urls), f, indent=2)
    
    def _get_page(self, url: str) -> Optional[BeautifulSoup]:
        """Fetch and parse a page"""
        if url in self.scraped_urls:
            logger.info(f"Skipping already scraped URL: {url}")
            return None
        
        try:
            logger.info(f"Fetching: {url}")
            response = self.session.get(url, timeout=self.config['timeout'], allow_redirects=True)
            
            # Check for common archive.org error patterns
            if response.status_code == 525:
                logger.error(f"SSL/TLS error (525) for {url}. The archived date might be invalid.")
                return None
            elif response.status_code == 404:
                logger.warning(f"Page not found (404) for {url}")
                return None
            
            response.raise_for_status()
            
            # Add delay to be respectful
            time.sleep(self.config['delay'])
            
            # Mark as scraped
            self.scraped_urls.add(url)
            
            # Parse HTML
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Remove Web Archive navigation bar and related elements
            for elem in soup.select('#wm-ipp-base, #wm-ipp, #donato, #wm-tb'):
                if elem:
                    elem.decompose()
            
            # Remove archive.org's injected toolbar
            archive_bar = soup.find('div', id='wm-ipp-print')
            if archive_bar:
                archive_bar.decompose()
            
            return soup
            
        except Exception as e:
            logger.error(f"Error fetching {url}: {e}")
            return None
    
    def scrape_main_page(self) -> Dict:
        """Scrape the main page for navigation and categories"""
        logger.info(f"Scraping main page from: {self.base_url}")
        
        # Check if main page data already exists
        main_page_file = self.data_dir / 'main_page.json'
        if main_page_file.exists():
            logger.info("Found existing main page data, checking if URL matches...")
            with open(main_page_file, 'r') as f:
                existing_data = json.load(f)
                # Check if the URL in the cached data matches current base_url
                if existing_data.get('url') == self.base_url:
                    logger.info("URLs match, loading existing data...")
                    return existing_data
                else:
                    logger.info(f"URL mismatch - cached: {existing_data.get('url')}, current: {self.base_url}")
                    logger.info("Re-scraping main page with new URL...")
        
        soup = self._get_page(self.base_url)
        if not soup:
            logger.error("Failed to scrape main page. Please check the archive URL.")
            return {}
        
        data = {
            'timestamp': datetime.now().isoformat(),
            'url': self.base_url,
            'navigation': {},
            'featured_papers': [],
            'trending_papers': [],
            'tasks': []
        }
        
        # Extract navigation menu - look for actual site navigation, not archive navigation
        # Try multiple possible selectors for navigation
        nav = soup.find('nav', class_=['navbar', 'navigation', 'main-nav']) or soup.find('div', class_='navigation')
        if not nav:
            # Try to find nav by looking for typical navigation links
            nav = soup.find(['nav', 'div'], attrs={'class': lambda x: x and ('nav' in x.lower() if isinstance(x, str) else False)})
        
        if nav:
            for link in nav.find_all('a'):
                href = link.get('href', '')
                text = link.get_text(strip=True)
                # Filter out archive.org links and empty links
                if href and text and not href.startswith('/web/2') and not href.startswith('#'):
                    # Only process paperswithcode links
                    if 'paperswithcode' in href or href.startswith('/'):
                        data['navigation'][text] = self._normalize_url(href)
        
        # Extract task categories from homepage
        # Look for task links anywhere on the page
        all_links = soup.find_all('a', href=True)
        for link in all_links:
            href = link.get('href', '')
            text = link.get_text(strip=True)
            
            # Collect task links
            if '/task/' in href and text:
                task_data = {
                    'name': text,
                    'url': self._normalize_url(href)
                }
                # Avoid duplicates
                if task_data not in data['tasks']:
                    data['tasks'].append(task_data)
            
            # Collect paper links from main page
            elif '/paper/' in href and text:
                paper_data = {
                    'title': text,
                    'url': self._normalize_url(href)
                }
                if paper_data not in data['featured_papers'] and len(data['featured_papers']) < 20:
                    data['featured_papers'].append(paper_data)
        
        # Save main page data
        self._save_json(data, self.data_dir / 'main_page.json')
        self._save_scraped_urls()
        
        return data
    
    def scrape_task_page(self, task_url: str) -> Dict:
        """Scrape a task page for papers and leaderboards"""
        soup = self._get_page(task_url)
        if not soup:
            return {}
        
        task_name = task_url.split('/task/')[-1].strip('/')
        logger.info(f"Scraping task: {task_name}")
        
        data = {
            'timestamp': datetime.now().isoformat(),
            'url': task_url,
            'name': task_name,
            'description': '',
            'papers': [],
            'datasets': [],
            'sota_table': []
        }
        
        # Extract task description
        desc = soup.find('div', class_='task-description')
        if desc:
            data['description'] = desc.get_text(strip=True)
        
        # Extract SOTA table
        sota_table = soup.find('table', class_=re.compile('sota|leaderboard'))
        if sota_table:
            headers = [th.get_text(strip=True) for th in sota_table.find_all('th')]
            
            for row in sota_table.find_all('tr')[1:]:  # Skip header row
                cells = row.find_all('td')
                if cells:
                    row_data = {}
                    for i, cell in enumerate(cells):
                        if i < len(headers):
                            # Check for paper links
                            link = cell.find('a')
                            if link and '/paper/' in link.get('href', ''):
                                row_data[headers[i]] = {
                                    'text': cell.get_text(strip=True),
                                    'paper_url': self._normalize_url(link.get('href'))
                                }
                            else:
                                row_data[headers[i]] = cell.get_text(strip=True)
                    
                    data['sota_table'].append(row_data)
        
        # Extract paper list
        paper_links = soup.find_all('a', href=re.compile('/paper/'))
        for link in paper_links[:50]:  # Limit to first 50 papers for initial testing
            paper_data = {
                'title': link.get_text(strip=True),
                'url': self._normalize_url(link.get('href'))
            }
            if paper_data not in data['papers']:
                data['papers'].append(paper_data)
        
        # Save task data
        filename = self.data_dir / 'tasks' / f"{task_name}.json"
        self._save_json(data, filename)
        self._save_scraped_urls()
        
        return data
    
    def scrape_paper_page(self, paper_url: str) -> Dict:
        """Scrape individual paper page"""
        soup = self._get_page(paper_url)
        if not soup:
            return {}
        
        paper_id = paper_url.split('/paper/')[-1].strip('/')
        logger.info(f"Scraping paper: {paper_id}")
        
        data = {
            'timestamp': datetime.now().isoformat(),
            'url': paper_url,
            'id': paper_id,
            'title': '',
            'abstract': '',
            'authors': [],
            'venue': '',
            'year': '',
            'code_links': [],
            'datasets': [],
            'tasks': [],
            'methods': [],
            'results': []
        }
        
        # Extract title
        title = soup.find('h1')
        if title:
            data['title'] = title.get_text(strip=True)
        
        # Extract abstract
        abstract = soup.find('div', class_='paper-abstract')
        if abstract:
            data['abstract'] = abstract.get_text(strip=True)
        
        # Extract authors
        authors = soup.find_all('a', href=re.compile('/author/'))
        data['authors'] = [a.get_text(strip=True) for a in authors]
        
        # Extract code links
        code_section = soup.find('div', class_=re.compile('code|implementation'))
        if code_section:
            for link in code_section.find_all('a', href=re.compile('github|gitlab|bitbucket')):
                data['code_links'].append({
                    'url': link.get('href'),
                    'text': link.get_text(strip=True)
                })
        
        # Extract results/benchmarks
        result_tables = soup.find_all('table', class_=re.compile('result|benchmark'))
        for table in result_tables:
            table_data = self._parse_table(table)
            if table_data:
                data['results'].append(table_data)
        
        # Save paper data
        filename = self.data_dir / 'papers' / f"{paper_id}.json"
        self._save_json(data, filename)
        self._save_scraped_urls()
        
        return data
    
    def _parse_table(self, table) -> List[Dict]:
        """Parse HTML table into structured data"""
        headers = [th.get_text(strip=True) for th in table.find_all('th')]
        rows = []
        
        for tr in table.find_all('tr')[1:]:
            cells = tr.find_all('td')
            if cells:
                row = {}
                for i, cell in enumerate(cells):
                    if i < len(headers):
                        row[headers[i]] = cell.get_text(strip=True)
                rows.append(row)
        
        return rows
    
    def _normalize_url(self, url: str) -> str:
        """Normalize URL to full archive.org URL"""
        # Extract timestamp from base URL
        timestamp = self.base_url.split('/web/')[1].split('/')[0]
        
        # Handle already normalized full archive.org URLs
        if url.startswith('https://web.archive.org'):
            return url
        
        # Handle archive.org relative paths (e.g., /web/TIMESTAMP/https://site.com/path)
        if url.startswith('/web/'):
            # This is already an archive path, just needs the domain prefix
            return f"https://web.archive.org{url}"
        
        # Handle direct paperswithcode.com URLs
        if url.startswith('http://paperswithcode.com') or url.startswith('https://paperswithcode.com'):
            return f"https://web.archive.org/web/{timestamp}/{url}"
        
        # Handle regular relative URLs (e.g., /task/something)
        if url.startswith('/'):
            return f"https://web.archive.org/web/{timestamp}/https://paperswithcode.com{url}"
        
        # Handle other patterns
        if 'paperswithcode.com' in url and not url.startswith('http'):
            return f"https://web.archive.org/web/{timestamp}/https://{url}"
        
        # Default fallback
        return urljoin(self.base_url, url)
    
    def _save_json(self, data: Dict, filepath: Path):
        """Save data as JSON"""
        filepath.parent.mkdir(exist_ok=True, parents=True)
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
        logger.info(f"Saved data to {filepath}")
    
    def run(self, max_papers: int = 10, max_tasks: int = 5, continue_from_last: bool = True):
        """Run the scraper
        
        Args:
            max_papers: Maximum number of papers to scrape per task
            max_tasks: Maximum number of tasks to scrape
            continue_from_last: Whether to continue from where we left off
        """
        logger.info("Starting Papers with Code archive scraper...")
        logger.info(f"Previously scraped URLs: {len(self.scraped_urls)}")
        
        # Scrape main page (will load existing if already scraped)
        main_data = self.scrape_main_page()
        
        if not main_data:
            logger.error("No main page data available. Cannot continue.")
            return
        
        # Scrape task pages
        tasks_to_scrape = main_data.get('tasks', [])[:max_tasks]
        logger.info(f"Found {len(tasks_to_scrape)} tasks to scrape")
        
        task_count = 0
        for task in tasks_to_scrape:
            task_count += 1
            logger.info(f"Processing task {task_count}/{len(tasks_to_scrape)}: {task['name']}")
            
            # Skip if already scraped and continue_from_last is True
            if continue_from_last and task['url'] in self.scraped_urls:
                logger.info(f"Task already scraped: {task['name']}")
                continue
            
            task_data = self.scrape_task_page(task['url'])
            
            # Scrape papers from this task
            papers_scraped = 0
            papers_in_task = task_data.get('papers', [])
            logger.info(f"Found {len(papers_in_task)} papers in task {task['name']}")
            
            for paper in papers_in_task:
                if papers_scraped >= max_papers:
                    logger.info(f"Reached max papers limit ({max_papers}) for task {task['name']}")
                    break
                
                # Skip if already scraped
                if paper['url'] in self.scraped_urls:
                    logger.info(f"Paper already scraped: {paper['title'][:50]}...")
                    continue
                    
                self.scrape_paper_page(paper['url'])
                papers_scraped += 1
                logger.info(f"Scraped {papers_scraped}/{min(max_papers, len(papers_in_task))} papers from {task['name']}")
        
        logger.info("Scraping completed!")
        logger.info(f"Total URLs scraped: {len(self.scraped_urls)}")


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Scrape Papers with Code archive")
    parser.add_argument('--max-papers', type=int, default=9999, help='Max papers per task (default: 10)')
    parser.add_argument('--max-tasks', type=int, default=9999, help='Max tasks to scrape (default: 5)')
    parser.add_argument('--fresh-start', action='store_true', help='Start fresh, ignore previous progress')
    args = parser.parse_args()
    
    scraper = ArchiveScraper()
    scraper.run(
        max_papers=args.max_papers,
        max_tasks=args.max_tasks,
        continue_from_last=not args.fresh_start
    )