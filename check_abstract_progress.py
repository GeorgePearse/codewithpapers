#!/usr/bin/env python3
"""Check progress of adding abstracts to benchmark_metrics.yaml"""

import yaml
import re

def check_abstract_progress():
    # Load the benchmark metrics file
    with open('benchmark_metrics.yaml', 'r') as f:
        data = yaml.safe_load(f)
    
    # Get unique algorithms from metrics
    algorithms_in_metrics = set()
    for entry in data.get('metrics', {}).get('mmdetection', []):
        algo = entry.get('algorithm', '')
        if algo and algo != 'Backbones Trained by Self-Supervise Algorithms':
            algorithms_in_metrics.add(algo)
    
    # Get algorithms that have abstracts in papers section
    algorithms_with_abstracts = set()
    for paper in data.get('papers', []):
        algo = paper.get('algorithm', '')
        if algo and 'abstract' in paper:
            algorithms_with_abstracts.add(algo)
    
    # Calculate progress
    missing_abstracts = algorithms_in_metrics - algorithms_with_abstracts
    
    print(f"Total unique algorithms in metrics: {len(algorithms_in_metrics)}")
    print(f"Algorithms with abstracts: {len(algorithms_with_abstracts)}")
    print(f"Missing abstracts: {len(missing_abstracts)}")
    print(f"Progress: {len(algorithms_with_abstracts)}/{len(algorithms_in_metrics)} ({100*len(algorithms_with_abstracts)/len(algorithms_in_metrics):.1f}%)")
    
    print("\nAlgorithms with abstracts:")
    for algo in sorted(algorithms_with_abstracts):
        print(f"  ✓ {algo}")
    
    print("\nAlgorithms missing abstracts:")
    for algo in sorted(missing_abstracts):
        print(f"  ✗ {algo}")
    
    return missing_abstracts

if __name__ == "__main__":
    missing = check_abstract_progress()