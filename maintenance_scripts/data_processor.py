#!/usr/bin/env python3
"""
Data processor for cleaning and transforming scraped Papers with Code data
"""

import json
import logging
import re
from collections import defaultdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set

import yaml

logger = logging.getLogger(__name__)


class DataProcessor:
    """Process and clean scraped data from Papers with Code archive"""
    
    def __init__(self, data_dir: str = "scraped_data", output_dir: str = "processed_data"):
        self.data_dir = Path(data_dir)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        
        # Create output subdirectories
        for subdir in ['papers', 'tasks', 'datasets', 'relationships']:
            (self.output_dir / subdir).mkdir(exist_ok=True)
        
        self.papers_index = {}
        self.relationships = defaultdict(list)
        
    def clean_text(self, text: str) -> str:
        """Clean and normalize text"""
        if not text:
            return ""
        
        # Remove extra whitespace
        text = ' '.join(text.split())
        
        # Remove Web Archive artifacts
        text = re.sub(r'\[.*?wayback.*?\]', '', text, flags=re.IGNORECASE)
        
        # Fix common encoding issues
        text = text.replace('â€™', "'").replace('â€"', '-').replace('â€œ', '"').replace('â€', '"')
        
        return text.strip()
    
    def extract_year(self, text: str) -> Optional[int]:
        """Extract year from text"""
        if not text:
            return None
        
        # Look for 4-digit year
        match = re.search(r'\b(19|20)\d{2}\b', text)
        if match:
            return int(match.group())
        
        return None
    
    def normalize_author_name(self, name: str) -> str:
        """Normalize author name format"""
        name = self.clean_text(name)
        
        # Remove numbers or special characters often added to names
        name = re.sub(r'\s*\d+$', '', name)
        name = re.sub(r'\s*\*$', '', name)
        
        return name
    
    def extract_github_info(self, url: str) -> Dict:
        """Extract GitHub repository information from URL"""
        if not url or 'github.com' not in url:
            return {}
        
        # Extract owner and repo from GitHub URL
        match = re.search(r'github\.com/([^/]+)/([^/\s]+)', url)
        if match:
            return {
                'owner': match.group(1),
                'repo': match.group(2).rstrip('.git'),
                'url': f"https://github.com/{match.group(1)}/{match.group(2).rstrip('.git')}"
            }
        
        return {}
    
    def process_paper(self, paper_file: Path) -> Dict:
        """Process individual paper data"""
        with open(paper_file, 'r', encoding='utf-8') as f:
            paper_data = json.load(f)
        
        # Clean basic fields
        processed = {
            'id': paper_data.get('id', ''),
            'title': self.clean_text(paper_data.get('title', '')),
            'abstract': self.clean_text(paper_data.get('abstract', '')),
            'authors': [self.normalize_author_name(author) for author in paper_data.get('authors', [])],
            'venue': self.clean_text(paper_data.get('venue', '')),
            'year': self.extract_year(paper_data.get('year', '')) or self.extract_year(paper_data.get('venue', '')),
            'url': paper_data.get('url', ''),
            'tasks': paper_data.get('tasks', []),
            'datasets': paper_data.get('datasets', []),
            'methods': paper_data.get('methods', [])
        }
        
        # Process code links
        code_links = []
        for link in paper_data.get('code_links', []):
            if isinstance(link, dict):
                url = link.get('url', '')
            else:
                url = link
            
            github_info = self.extract_github_info(url)
            if github_info:
                code_links.append({
                    'type': 'github',
                    **github_info
                })
            elif url:
                code_links.append({
                    'type': 'other',
                    'url': url
                })
        
        processed['code_links'] = code_links
        
        # Process results/benchmarks
        processed['benchmarks'] = []
        for result in paper_data.get('results', []):
            if isinstance(result, list):
                # It's a table of results
                processed['benchmarks'].extend(result)
            elif isinstance(result, dict):
                processed['benchmarks'].append(result)
        
        # Generate keywords from title and abstract
        processed['keywords'] = self.extract_keywords(
            processed['title'] + ' ' + processed['abstract']
        )
        
        return processed
    
    def extract_keywords(self, text: str) -> List[str]:
        """Extract keywords from text"""
        # Common ML/DL keywords to look for
        keyword_patterns = [
            r'\b(neural network|deep learning|machine learning|reinforcement learning)\b',
            r'\b(CNN|RNN|LSTM|GRU|transformer|attention)\b',
            r'\b(GAN|VAE|autoencoder)\b',
            r'\b(classification|detection|segmentation|generation)\b',
            r'\b(supervised|unsupervised|self-supervised|semi-supervised)\b',
            r'\b(transfer learning|fine-tuning|pre-training)\b',
            r'\b(optimization|regularization|normalization)\b'
        ]
        
        keywords = set()
        text_lower = text.lower()
        
        for pattern in keyword_patterns:
            matches = re.findall(pattern, text_lower, re.IGNORECASE)
            keywords.update(matches)
        
        return list(keywords)
    
    def build_relationships(self, papers: List[Dict]):
        """Build relationship graph between papers"""
        relationships = {
            'citation_graph': {},
            'author_connections': defaultdict(list),
            'task_connections': defaultdict(list),
            'dataset_connections': defaultdict(list)
        }
        
        # Build author connections
        for paper in papers:
            paper_id = paper['id']
            
            # Group papers by authors
            for author in paper.get('authors', []):
                if author:
                    relationships['author_connections'][author].append(paper_id)
            
            # Group papers by tasks
            for task in paper.get('tasks', []):
                relationships['task_connections'][task].append(paper_id)
            
            # Group papers by datasets
            for dataset in paper.get('datasets', []):
                relationships['dataset_connections'][dataset].append(paper_id)
        
        # Convert to list format for JSON serialization
        relationships['author_connections'] = dict(relationships['author_connections'])
        relationships['task_connections'] = dict(relationships['task_connections'])
        relationships['dataset_connections'] = dict(relationships['dataset_connections'])
        
        return relationships
    
    def process_all_papers(self) -> Dict:
        """Process all scraped papers"""
        papers_dir = self.data_dir / 'papers'
        if not papers_dir.exists():
            logger.warning(f"Papers directory not found: {papers_dir}")
            return {}
        
        processed_papers = []
        
        for paper_file in papers_dir.glob('*.json'):
            try:
                processed = self.process_paper(paper_file)
                processed_papers.append(processed)
                self.papers_index[processed['id']] = processed
                
                # Save individual processed paper
                output_file = self.output_dir / 'papers' / paper_file.name
                with open(output_file, 'w', encoding='utf-8') as f:
                    json.dump(processed, f, indent=2, ensure_ascii=False)
                
            except Exception as e:
                logger.error(f"Error processing {paper_file}: {e}")
        
        # Build relationships
        relationships = self.build_relationships(processed_papers)
        
        # Save relationships
        with open(self.output_dir / 'relationships' / 'graph.json', 'w') as f:
            json.dump(relationships, f, indent=2)
        
        # Save papers index
        with open(self.output_dir / 'papers_index.json', 'w') as f:
            json.dump(self.papers_index, f, indent=2)
        
        logger.info(f"Processed {len(processed_papers)} papers")
        
        return {
            'papers': processed_papers,
            'relationships': relationships
        }
    
    def generate_summary_stats(self, papers: List[Dict]) -> Dict:
        """Generate summary statistics of the processed data"""
        stats = {
            'total_papers': len(papers),
            'papers_with_code': sum(1 for p in papers if p.get('code_links')),
            'papers_with_abstract': sum(1 for p in papers if p.get('abstract')),
            'unique_authors': len(set(author for p in papers for author in p.get('authors', []))),
            'unique_venues': len(set(p.get('venue', '') for p in papers if p.get('venue'))),
            'year_distribution': defaultdict(int),
            'top_keywords': defaultdict(int),
            'code_availability': {
                'github': 0,
                'other': 0,
                'none': 0
            }
        }
        
        for paper in papers:
            # Year distribution
            year = paper.get('year')
            if year:
                stats['year_distribution'][year] += 1
            
            # Keyword frequency
            for keyword in paper.get('keywords', []):
                stats['top_keywords'][keyword] += 1
            
            # Code availability
            if paper.get('code_links'):
                has_github = any(link.get('type') == 'github' for link in paper['code_links'])
                if has_github:
                    stats['code_availability']['github'] += 1
                else:
                    stats['code_availability']['other'] += 1
            else:
                stats['code_availability']['none'] += 1
        
        # Convert defaultdicts to regular dicts and sort
        stats['year_distribution'] = dict(sorted(stats['year_distribution'].items()))
        stats['top_keywords'] = dict(sorted(
            stats['top_keywords'].items(),
            key=lambda x: x[1],
            reverse=True
        )[:20])  # Top 20 keywords
        
        return stats
    
    def run(self):
        """Run the data processor"""
        logger.info("Starting data processing...")
        
        # Process all papers
        result = self.process_all_papers()
        
        if result:
            # Generate statistics
            stats = self.generate_summary_stats(result['papers'])
            
            # Save statistics
            with open(self.output_dir / 'statistics.json', 'w') as f:
                json.dump(stats, f, indent=2)
            
            logger.info("Data processing completed!")
            logger.info(f"Statistics: {json.dumps(stats, indent=2)}")
        else:
            logger.warning("No data to process")


if __name__ == "__main__":
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    
    processor = DataProcessor()
    processor.run()