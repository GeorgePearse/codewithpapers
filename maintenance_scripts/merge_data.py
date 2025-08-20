#!/usr/bin/env python3
"""
Merge scraped Papers with Code data with existing benchmark_metrics.yaml
"""

import json
import logging
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

import yaml

logger = logging.getLogger(__name__)


class DataMerger:
    """Merge scraped data with existing benchmark metrics"""
    
    def __init__(self, 
                 processed_dir: str = "processed_data",
                 existing_file: str = "../public/benchmark_metrics.yaml",
                 output_file: str = "../public/benchmark_metrics_merged.yaml"):
        self.processed_dir = Path(processed_dir)
        self.existing_file = Path(existing_file)
        self.output_file = Path(output_file)
        
        self.existing_data = self._load_existing_data()
        self.processed_papers = self._load_processed_papers()
        self.merge_report = {
            'timestamp': datetime.now().isoformat(),
            'new_papers': [],
            'updated_papers': [],
            'conflicts': [],
            'statistics': {}
        }
    
    def _load_existing_data(self) -> Dict:
        """Load existing benchmark_metrics.yaml"""
        if self.existing_file.exists():
            with open(self.existing_file, 'r', encoding='utf-8') as f:
                return yaml.safe_load(f) or {}
        return {'papers': [], 'metrics': {}}
    
    def _load_processed_papers(self) -> Dict:
        """Load processed papers index"""
        index_file = self.processed_dir / 'papers_index.json'
        if index_file.exists():
            with open(index_file, 'r') as f:
                return json.load(f)
        return {}
    
    def normalize_title(self, title: str) -> str:
        """Normalize paper title for matching"""
        if not title:
            return ""
        
        # Convert to lowercase and remove special characters
        title = title.lower().strip()
        title = ''.join(c for c in title if c.isalnum() or c.isspace())
        title = ' '.join(title.split())  # Normalize whitespace
        
        return title
    
    def find_existing_paper(self, paper: Dict) -> Optional[Dict]:
        """Find if paper already exists in current data"""
        new_title = self.normalize_title(paper.get('title', ''))
        
        for existing_paper in self.existing_data.get('papers', []):
            existing_title = self.normalize_title(existing_paper.get('title', ''))
            
            # Match by title (fuzzy matching)
            if new_title and existing_title:
                # Check for exact match or very similar titles
                if new_title == existing_title:
                    return existing_paper
                
                # Check if one title contains the other (for subtitle variations)
                if new_title in existing_title or existing_title in new_title:
                    return existing_paper
        
        return None
    
    def merge_paper_data(self, existing: Dict, new: Dict) -> Dict:
        """Merge new paper data with existing paper data"""
        merged = existing.copy()
        
        # Keep existing data but add new fields if missing
        if not merged.get('abstract') and new.get('abstract'):
            merged['abstract'] = new['abstract']
        
        # Merge authors (union of both lists)
        existing_authors = set(existing.get('authors', []))
        new_authors = set(new.get('authors', []))
        merged['authors'] = list(existing_authors | new_authors)
        
        # Add algorithm name if missing
        if not merged.get('algorithm') and new.get('title'):
            # Try to extract algorithm name from title
            merged['algorithm'] = self.extract_algorithm_name(new['title'])
        
        # Merge code links
        existing_links = existing.get('code_links', [])
        new_links = new.get('code_links', [])
        
        # Avoid duplicate links
        existing_urls = set()
        for link in existing_links:
            if isinstance(link, dict):
                existing_urls.add(link.get('url', ''))
            else:
                existing_urls.add(link)
        
        for link in new_links:
            url = link.get('url', '') if isinstance(link, dict) else link
            if url and url not in existing_urls:
                existing_links.append(link)
        
        if existing_links:
            merged['code_links'] = existing_links
        
        # Add year if missing
        if not merged.get('year') and new.get('year'):
            merged['year'] = new['year']
        
        # Add venue if missing
        if not merged.get('venue') and new.get('venue'):
            merged['venue'] = new['venue']
        
        # Merge keywords
        existing_keywords = set(existing.get('keywords', []))
        new_keywords = set(new.get('keywords', []))
        merged_keywords = list(existing_keywords | new_keywords)
        if merged_keywords:
            merged['keywords'] = merged_keywords
        
        # Add benchmarks if not present
        if not merged.get('benchmarks') and new.get('benchmarks'):
            merged['benchmarks'] = new['benchmarks']
        
        return merged
    
    def extract_algorithm_name(self, title: str) -> str:
        """Extract algorithm name from paper title"""
        # Common patterns in paper titles
        # "Algorithm: Description" or "Algorithm - Description"
        if ':' in title:
            return title.split(':')[0].strip()
        elif ' - ' in title:
            return title.split(' - ')[0].strip()
        else:
            # Take first few words as algorithm name
            words = title.split()
            if len(words) > 3:
                return ' '.join(words[:3])
            return title
    
    def convert_to_yaml_format(self, paper: Dict) -> Dict:
        """Convert processed paper to YAML format matching existing structure"""
        yaml_paper = {
            'title': paper.get('title', ''),
            'algorithm': paper.get('algorithm', '') or self.extract_algorithm_name(paper.get('title', ''))
        }
        
        # Add abstract if available
        if paper.get('abstract'):
            yaml_paper['abstract'] = paper['abstract']
        
        # Add authors if available
        if paper.get('authors'):
            yaml_paper['authors'] = paper['authors']
        
        # Add year if available
        if paper.get('year'):
            yaml_paper['year'] = paper['year']
        
        # Add venue if available  
        if paper.get('venue'):
            yaml_paper['venue'] = paper['venue']
        
        # Add code links
        if paper.get('code_links'):
            yaml_paper['implementations'] = []
            for link in paper['code_links']:
                if isinstance(link, dict):
                    impl = {
                        'url': link.get('url', ''),
                        'framework': 'PyTorch'  # Default, could be detected
                    }
                    if link.get('type') == 'github':
                        impl['repository'] = f"{link.get('owner')}/{link.get('repo')}"
                    yaml_paper['implementations'].append(impl)
        
        # Add keywords
        if paper.get('keywords'):
            yaml_paper['keywords'] = paper['keywords']
        
        # Add benchmarks
        if paper.get('benchmarks'):
            yaml_paper['benchmarks'] = paper['benchmarks']
        
        return yaml_paper
    
    def merge_metrics(self, new_papers: List[Dict]):
        """Merge metrics/benchmarks from new papers"""
        # This is a placeholder for merging the metrics section
        # The actual implementation would depend on the specific structure
        # of benchmarks in the scraped data
        
        if 'metrics' not in self.existing_data:
            self.existing_data['metrics'] = {}
        
        # Group papers by algorithm/model for metrics
        papers_by_algorithm = defaultdict(list)
        for paper in new_papers:
            algorithm = paper.get('algorithm', '')
            if algorithm:
                papers_by_algorithm[algorithm].append(paper)
        
        # Add to metrics if we have benchmark data
        # This would need to be expanded based on actual benchmark format
        pass
    
    def run(self):
        """Run the merge process"""
        logger.info("Starting data merge process...")
        
        merged_papers = self.existing_data.get('papers', []).copy()
        papers_index = {self.normalize_title(p.get('title', '')): i 
                       for i, p in enumerate(merged_papers)}
        
        new_count = 0
        updated_count = 0
        
        for paper_id, paper_data in self.processed_papers.items():
            # Check if paper already exists
            existing_paper = self.find_existing_paper(paper_data)
            
            if existing_paper:
                # Update existing paper
                existing_index = papers_index.get(
                    self.normalize_title(existing_paper.get('title', ''))
                )
                
                if existing_index is not None:
                    merged = self.merge_paper_data(existing_paper, paper_data)
                    merged_papers[existing_index] = merged
                    updated_count += 1
                    
                    self.merge_report['updated_papers'].append({
                        'title': paper_data.get('title', ''),
                        'changes': self._get_changes(existing_paper, merged)
                    })
            else:
                # Add new paper
                yaml_paper = self.convert_to_yaml_format(paper_data)
                merged_papers.append(yaml_paper)
                new_count += 1
                
                self.merge_report['new_papers'].append({
                    'title': paper_data.get('title', ''),
                    'algorithm': yaml_paper.get('algorithm', '')
                })
        
        # Update the data structure
        self.existing_data['papers'] = merged_papers
        
        # Merge metrics
        self.merge_metrics(merged_papers)
        
        # Update summary if it exists
        if 'summary' in self.existing_data:
            self.existing_data['summary']['last_updated'] = datetime.now().isoformat()
            self.existing_data['summary']['total_papers'] = len(merged_papers)
        
        # Save merged data
        with open(self.output_file, 'w', encoding='utf-8') as f:
            yaml.dump(self.existing_data, f, default_flow_style=False, 
                     allow_unicode=True, sort_keys=False)
        
        # Update merge report statistics
        self.merge_report['statistics'] = {
            'total_papers': len(merged_papers),
            'new_papers': new_count,
            'updated_papers': updated_count,
            'original_papers': len(self.existing_data.get('papers', [])) - updated_count
        }
        
        # Save merge report
        report_file = self.output_file.parent / 'merge_report.json'
        with open(report_file, 'w') as f:
            json.dump(self.merge_report, f, indent=2)
        
        logger.info(f"Merge completed: {new_count} new papers, {updated_count} updated papers")
        logger.info(f"Merged data saved to: {self.output_file}")
        logger.info(f"Merge report saved to: {report_file}")
        
        return self.merge_report
    
    def _get_changes(self, original: Dict, updated: Dict) -> List[str]:
        """Get list of changes between original and updated paper"""
        changes = []
        
        for key in updated:
            if key not in original:
                changes.append(f"Added {key}")
            elif original.get(key) != updated.get(key):
                if isinstance(updated[key], list):
                    orig_len = len(original.get(key, []))
                    new_len = len(updated[key])
                    if new_len > orig_len:
                        changes.append(f"Added {new_len - orig_len} {key}")
                else:
                    changes.append(f"Updated {key}")
        
        return changes


if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    
    merger = DataMerger()
    merger.run()