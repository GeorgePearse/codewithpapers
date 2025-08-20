#!/usr/bin/env python3
"""Extract all abstracts from mmdetection README files and add them to benchmark_metrics.yaml"""

import os
import re
import yaml
from pathlib import Path

def extract_abstract_from_readme(readme_path):
    """Extract the abstract section from a README file"""
    try:
        with open(readme_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Try to find the algorithm name from the title
        title_match = re.search(r'^# (.+)$', content, re.MULTILINE)
        algorithm = title_match.group(1) if title_match else None
        
        # Try to find the paper title (usually in quotes after >)
        paper_match = re.search(r'> \[(.+?)\]', content)
        paper_title = paper_match.group(1) if paper_match else None
        
        # Find the abstract section
        abstract_match = re.search(r'## Abstract\s*\n\s*\n(.+?)(?=\n##|\n<div|\Z)', content, re.DOTALL)
        abstract = abstract_match.group(1).strip() if abstract_match else None
        
        if algorithm and paper_title and abstract:
            # Clean up the abstract (remove extra whitespace, newlines)
            abstract = ' '.join(abstract.split())
            return {
                'algorithm': algorithm,
                'title': paper_title,
                'abstract': abstract
            }
    except Exception as e:
        print(f"Error processing {readme_path}: {e}")
    
    return None

def main():
    # Path to mmdetection configs
    configs_dir = Path('references/mmdetection/configs')
    
    # Load current benchmark metrics
    with open('benchmark_metrics.yaml', 'r') as f:
        data = yaml.safe_load(f)
    
    # Get existing algorithms with abstracts
    existing_algorithms = {paper['algorithm'] for paper in data.get('papers', [])}
    
    # Extract abstracts from all README files
    new_papers = []
    readme_files = list(configs_dir.glob('*/README.md'))
    
    print(f"Found {len(readme_files)} README files to process")
    
    for readme_path in readme_files:
        paper_info = extract_abstract_from_readme(readme_path)
        if paper_info and paper_info['algorithm'] not in existing_algorithms:
            new_papers.append(paper_info)
            print(f"✓ Extracted abstract for {paper_info['algorithm']}")
        elif paper_info and paper_info['algorithm'] in existing_algorithms:
            print(f"⚠ Already have abstract for {paper_info['algorithm']}")
    
    # Add new papers to the data
    if new_papers:
        data['papers'].extend(new_papers)
        
        # Save the updated file
        with open('benchmark_metrics.yaml', 'w') as f:
            yaml.dump(data, f, default_flow_style=False, sort_keys=False, allow_unicode=True, width=200)
        
        print(f"\nAdded {len(new_papers)} new abstracts to benchmark_metrics.yaml")
    else:
        print("\nNo new abstracts to add")
    
    # Report final stats
    total_algorithms = len(set(paper['algorithm'] for paper in data.get('papers', [])))
    print(f"\nTotal algorithms with abstracts: {total_algorithms}")

if __name__ == "__main__":
    main()