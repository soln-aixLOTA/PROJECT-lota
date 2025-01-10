#!/usr/bin/env python3
import glob
import re
import os
import yaml
import logging
from typing import Dict, List, Tuple

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

SECRET_PATTERNS = [
    r'(?i)api[_-]key\s*=\s*[\'"](.*?)[\'"]',
    r'(?i)password\s*=\s*[\'"](.*?)[\'"]',
    r'(?i)secret\s*=\s*[\'"](.*?)[\'"]',
    r'(?i)token\s*=\s*[\'"](.*?)[\'"]',
    r'(?i)credentials\s*=\s*[\'"](.*?)[\'"]',
    r'(?i)auth[_-]token\s*=\s*[\'"](.*?)[\'"]',
]

EXCLUDED_DIRS = [
    'node_modules',
    'target',
    'dist',
    'build',
    '__pycache__',
    '.git',
]

def should_scan_file(filepath: str) -> bool:
    """Determine if a file should be scanned for secrets."""
    # Skip excluded directories
    if any(excluded in filepath for excluded in EXCLUDED_DIRS):
        return False
        
    # Skip binary files and certain extensions
    excluded_extensions = {'.pyc', '.so', '.dll', '.exe', '.bin'}
    if os.path.splitext(filepath)[1] in excluded_extensions:
        return False
        
    return True

def categorize_secret(filepath: str, secret: str) -> Tuple[str, str]:
    """Categorize a secret and determine its Vault path."""
    if 'nvidia' in filepath.lower():
        return 'nvidia', f'nvidia/{os.path.basename(filepath)}'
    elif 'database' in filepath.lower() or 'db' in filepath.lower():
        return 'database', f'database/{os.path.basename(filepath)}'
    elif 'api-gateway' in filepath:
        return 'api-gateway', f'api-gateway/{os.path.basename(filepath)}'
    elif 'inference-service' in filepath:
        return 'inference-service', f'inference-service/{os.path.basename(filepath)}'
    elif 'user-auth' in filepath:
        return 'user-auth', f'user-auth/{os.path.basename(filepath)}'
    else:
        return 'shared', f'shared/{os.path.basename(filepath)}'

def scan_for_secrets(directory: str) -> Dict[str, List[Dict]]:
    """Scan directory for potential secrets."""
    secrets: Dict[str, List[Dict]] = {}
    
    for filepath in glob.glob(f"{directory}/**/*", recursive=True):
        if not should_scan_file(filepath):
            continue
            
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
                
            for pattern in SECRET_PATTERNS:
                matches = re.finditer(pattern, content)
                for match in matches:
                    secret_value = match.group(1)
                    category, vault_path = categorize_secret(filepath, secret_value)
                    
                    if category not in secrets:
                        secrets[category] = []
                        
                    secrets[category].append({
                        'file': os.path.relpath(filepath, directory),
                        'line': content.count('\n', 0, match.start()) + 1,
                        'vault_path': vault_path,
                        'pattern_type': pattern.split('[_-]')[0],
                    })
                    
        except Exception as e:
            logger.warning(f"Failed to process {filepath}: {e}")
            
    return secrets

def generate_mapping_file(secrets: Dict[str, List[Dict]], output_file: str):
    """Generate a YAML mapping file for secret migration."""
    mapping = {}
    
    for category, items in secrets.items():
        for item in items:
            mapping[item['file']] = {
                'vault_path': item['vault_path'],
                'category': category,
                'line': item['line'],
                'type': item['pattern_type'],
            }
            
    with open(output_file, 'w') as f:
        yaml.safe_dump(mapping, f, default_flow_style=False)
        
def main():
    import argparse
    parser = argparse.ArgumentParser(description='Scan codebase for secrets')
    parser.add_argument('directory', help='Directory to scan')
    parser.add_argument('--output', default='secret_mapping.yaml',
                      help='Output mapping file (default: secret_mapping.yaml)')
    args = parser.parse_args()
    
    logger.info(f"Scanning directory: {args.directory}")
    secrets = scan_for_secrets(args.directory)
    
    # Print summary
    total_secrets = sum(len(items) for items in secrets.values())
    logger.info(f"Found {total_secrets} potential secrets:")
    for category, items in secrets.items():
        logger.info(f"  {category}: {len(items)} secrets")
        
    # Generate mapping file
    generate_mapping_file(secrets, args.output)
    logger.info(f"Generated mapping file: {args.output}")
    
if __name__ == '__main__':
    main() 