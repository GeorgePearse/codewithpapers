#!/usr/bin/env python3
"""
Extract benchmark metrics from MMDetection and MMPose README files
and save them to a YAML file.
"""

import os
import re
import yaml
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
import logging

logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')


class MetricsExtractor:
    """Extract metrics from README files in MMDetection and MMPose repositories."""
    
    def __init__(self, base_path: str = "."):
        self.base_path = Path(base_path)
        self.mmdet_path = self.base_path / "mmdetection"
        self.mmpose_path = self.base_path / "mmpose"
        
    def find_readme_files(self, repo_path: Path) -> List[Path]:
        """Find all README.md files in the configs directory."""
        readme_files = []
        
        # Focus on configs directory where metrics are located
        configs_dir = repo_path / "configs"
        
        if configs_dir.exists():
            readme_files = list(configs_dir.rglob("README.md"))
                
        return readme_files
    
    def extract_tables(self, content: str) -> List[Dict[str, Any]]:
        """Extract all tables with their headers from markdown content."""
        lines = content.split('\n')
        tables = []
        current_table = {'headers': [], 'rows': []}
        in_table = False
        
        for i, line in enumerate(lines):
            # Check if line is a table row
            if '|' in line:
                # Check if this is a separator row (e.g., |---|---|)
                if re.match(r'^[\s\|:\-]+$', line):
                    # This means the previous line was headers
                    if i > 0 and '|' in lines[i-1]:
                        current_table['headers'] = self.parse_table_row_cells(lines[i-1])
                        in_table = True
                    continue
                    
                if in_table:
                    # Add data row
                    current_table['rows'].append(line)
                elif not current_table['headers']:
                    # Might be starting a new table
                    current_table = {'headers': [], 'rows': []}
            else:
                # End of table
                if in_table and current_table['rows']:
                    tables.append(current_table)
                    current_table = {'headers': [], 'rows': []}
                in_table = False
        
        # Don't forget the last table
        if in_table and current_table['rows']:
            tables.append(current_table)
                
        return tables
    
    def parse_table_row_cells(self, row: str) -> List[str]:
        """Parse a table row into individual cells."""
        # Split by | and clean up
        cells = row.split('|')
        # Remove empty cells at start and end
        if cells and not cells[0].strip():
            cells = cells[1:]
        if cells and not cells[-1].strip():
            cells = cells[:-1]
        # Strip whitespace from each cell
        return [cell.strip() for cell in cells]
    
    def parse_table_row_with_headers(self, row: str, headers: List[str]) -> Dict[str, Any]:
        """Parse a single table row using header information."""
        cells = self.parse_table_row_cells(row)
        
        if not cells:
            return {}
            
        result = {}
        
        # Map cells to headers
        for i, cell in enumerate(cells):
            if i >= len(headers):
                break
                
            header = headers[i].lower()
            # Clean up the cell value
            clean_cell = re.sub(r'\[([^\]]+)\]\([^\)]+\)', r'\1', cell)  # Remove markdown links
            clean_cell = re.sub(r'[*`]', '', clean_cell).strip()  # Remove formatting
            
            # Identify what this column contains based on header
            if any(term in header for term in ['model', 'method', 'backbone', 'arch', 'name']):
                if clean_cell and not re.match(r'^[\-\s]*$', clean_cell):
                    result['model'] = clean_cell
            elif any(term in header for term in ['config', 'cfg']):
                if 'config' in clean_cell.lower():
                    # Extract config file name
                    config_match = re.search(r'([\w\-]+\.py)', clean_cell)
                    if config_match:
                        result['config'] = config_match.group(1)
            elif 'lr' in header and 'schd' in header:
                result['lr_schedule'] = clean_cell
            elif 'style' in header:
                result['style'] = clean_cell
            elif 'fps' in header or 'time' in header:
                fps_match = re.search(r'([\d\.]+)', clean_cell)
                if fps_match:
                    result['fps'] = float(fps_match.group(1))
            else:
                # Try to identify metrics
                numbers = re.findall(r'(\d+\.?\d*)', clean_cell)
                if numbers:
                    # Use header to determine metric type
                    if 'box' in header and 'ap' in header:
                        result['box_AP'] = float(numbers[0])
                    elif 'mask' in header and 'ap' in header:
                        result['mask_AP'] = float(numbers[0])
                    elif 'map' in header:
                        result['mAP'] = float(numbers[0])
                    elif 'ap' in header:
                        result['AP'] = float(numbers[0])
                    elif 'pck' in header:
                        result['PCK'] = float(numbers[0])
                    elif 'nme' in header:
                        result['NME'] = float(numbers[0])
                    elif 'auc' in header:
                        result['AUC'] = float(numbers[0])
                    elif 'epe' in header:
                        result['EPE'] = float(numbers[0])
                    elif re.match(r'^[\d\.]+$', clean_cell):
                        # Pure number, store with header as key
                        clean_header = re.sub(r'[^\w]+', '_', header).strip('_')
                        if clean_header:
                            result[clean_header] = float(numbers[0])
                        
        return result
    
    def extract_metrics_from_readme(self, readme_path: Path) -> Dict[str, Any]:
        """Extract all metrics from a single README file."""
        try:
            with open(readme_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            logging.error(f"Error reading {readme_path}: {e}")
            return {}
            
        # Extract relative path for organization
        if self.mmdet_path in readme_path.parents:
            repo = "mmdetection"
            rel_path = readme_path.relative_to(self.mmdet_path)
        elif self.mmpose_path in readme_path.parents:
            repo = "mmpose"
            rel_path = readme_path.relative_to(self.mmpose_path)
        else:
            return {}
            
        # Extract all tables with headers
        tables = self.extract_tables(content)
        
        if not tables:
            return {}
            
        # Parse metrics from tables
        all_models = []
        for table in tables:
            # Check if this looks like a metrics table
            headers_str = ' '.join(table['headers']).lower()
            if any(metric in headers_str for metric in ['ap', 'map', 'pck', 'nme', 'auc', 'epe']):
                # Parse each row with headers
                for row in table['rows']:
                    metrics = self.parse_table_row_with_headers(row, table['headers'])
                    if metrics and ('model' in metrics or any(k for k in metrics if k.endswith('AP') or k in ['PCK', 'NME', 'AUC', 'EPE', 'mAP'])):
                        all_models.append(metrics)
                
        if not all_models:
            return {}
            
        # Extract config/algorithm name from path
        config_parts = rel_path.parts
        config_name = None
        if 'configs' in config_parts:
            idx = config_parts.index('configs')
            if idx + 1 < len(config_parts):
                config_name = config_parts[idx + 1]
                
        # Try to extract algorithm name from README title
        algorithm_name = None
        title_match = re.search(r'^#\s+(.+?)$', content, re.MULTILINE)
        if title_match:
            algorithm_name = title_match.group(1).strip()
                
        return {
            'repository': repo,
            'file': str(rel_path),
            'config': config_name,
            'algorithm': algorithm_name,
            'models': all_models
        }
    
    def extract_all_metrics(self) -> Dict[str, List[Dict]]:
        """Extract metrics from all README files in both repositories."""
        all_metrics = {
            'mmdetection': [],
            'mmpose': []
        }
        
        # Process MMDetection
        if self.mmdet_path.exists():
            logging.info("Processing MMDetection repository...")
            readme_files = self.find_readme_files(self.mmdet_path)
            logging.info(f"Found {len(readme_files)} README files in MMDetection")
            
            for readme in readme_files:
                metrics = self.extract_metrics_from_readme(readme)
                if metrics and metrics.get('models'):
                    all_metrics['mmdetection'].append(metrics)
                    
        # Process MMPose
        if self.mmpose_path.exists():
            logging.info("Processing MMPose repository...")
            readme_files = self.find_readme_files(self.mmpose_path)
            logging.info(f"Found {len(readme_files)} README files in MMPose")
            
            for readme in readme_files:
                metrics = self.extract_metrics_from_readme(readme)
                if metrics and metrics.get('models'):
                    all_metrics['mmpose'].append(metrics)
                    
        return all_metrics
    
    def save_to_yaml(self, metrics: Dict[str, List[Dict]], output_file: str = "benchmark_metrics.yaml"):
        """Save extracted metrics to a YAML file."""
        output_path = self.base_path / output_file
        
        # Add summary statistics
        summary = {
            'summary': {
                'mmdetection': {
                    'total_configs': len(metrics.get('mmdetection', [])),
                    'total_models': sum(len(cfg.get('models', [])) for cfg in metrics.get('mmdetection', []))
                },
                'mmpose': {
                    'total_configs': len(metrics.get('mmpose', [])),
                    'total_models': sum(len(cfg.get('models', [])) for cfg in metrics.get('mmpose', []))
                }
            },
            'metrics': metrics
        }
        
        with open(output_path, 'w') as f:
            yaml.dump(summary, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
            
        logging.info(f"Metrics saved to {output_path}")
        return output_path


def main():
    """Main function to run the metrics extraction."""
    extractor = MetricsExtractor()
    
    # Extract metrics
    logging.info("Starting metrics extraction...")
    metrics = extractor.extract_all_metrics()
    
    # Save to YAML
    output_file = extractor.save_to_yaml(metrics)
    
    # Print summary
    total_mmdet = sum(len(cfg.get('models', [])) for cfg in metrics.get('mmdetection', []))
    total_mmpose = sum(len(cfg.get('models', [])) for cfg in metrics.get('mmpose', []))
    
    print(f"\n=== Extraction Complete ===")
    print(f"MMDetection: {len(metrics.get('mmdetection', []))} configs, {total_mmdet} models")
    print(f"MMPose: {len(metrics.get('mmpose', []))} configs, {total_mmpose} models")
    print(f"Results saved to: {output_file}")


if __name__ == "__main__":
    main()