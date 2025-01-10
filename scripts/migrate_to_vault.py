#!/usr/bin/env python3
import os
import yaml
import hvac
import logging
import asyncio
from typing import Dict, Any
import re

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class VaultMigrator:
    def __init__(self, vault_addr: str, vault_token: str):
        try:
            self.client = hvac.Client(
                url=vault_addr,
                token=vault_token
            )
            
            if not self.client.is_authenticated():
                raise Exception("Failed to authenticate with Vault")
        except Exception as e:
            logger.error(f"Failed to initialize Vault client: {e}")
            raise
            
    def _extract_secret_from_file(self, filepath: str, line_number: int, pattern_type: str) -> str:
        """Extract secret value from file based on line number and pattern type."""
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                lines = f.readlines()
                
            if line_number > len(lines):
                raise ValueError(f"Line number {line_number} exceeds file length")
                
            line = lines[line_number - 1]
            pattern = fr'(?i){pattern_type}[_-]?\w*\s*=\s*[\'"](.*?)[\'"]'
            match = re.search(pattern, line)
            
            if not match:
                raise ValueError(f"Could not find secret in line {line_number}")
                
            return match.group(1)
            
        except Exception as e:
            logger.error(f"Failed to extract secret from {filepath}: {e}")
            raise
            
    async def migrate_secrets(self, mapping_file: str):
        """Migrate secrets to Vault based on mapping file."""
        try:
            # Load secret mapping
            with open(mapping_file) as f:
                mapping = yaml.safe_load(f)
                
            # Track migration status
            status = {
                'total': len(mapping),
                'success': 0,
                'failed': 0,
                'skipped': 0
            }
            
            # Process each secret
            for filepath, config in mapping.items():
                try:
                    # Extract secret from file
                    secret_value = self._extract_secret_from_file(
                        filepath,
                        config['line'],
                        config['type']
                    )
                    
                    # Write to Vault
                    try:
                        self.client.secrets.kv.v2.create_or_update_secret(
                            path=config['vault_path'],
                            secret=dict(value=secret_value),
                            mount_point='lotabots'
                        )
                        
                        logger.info(f"Migrated secret from {filepath} to {config['vault_path']}")
                        status['success'] += 1
                    except Exception as e:
                        logger.error(f"Failed to write secret to Vault: {e}")
                        status['failed'] += 1
                        
                except Exception as e:
                    logger.error(f"Failed to migrate secret from {filepath}: {e}")
                    status['failed'] += 1
                    
            return status
            
        except Exception as e:
            logger.error(f"Migration failed: {e}")
            raise
            
    def verify_migration(self, mapping_file: str) -> Dict[str, Any]:
        """Verify that all secrets were successfully migrated to Vault."""
        try:
            with open(mapping_file) as f:
                mapping = yaml.safe_load(f)
                
            verification = {
                'total': len(mapping),
                'verified': 0,
                'failed': 0
            }
            
            for filepath, config in mapping.items():
                try:
                    # Read secret from Vault
                    secret = self.client.secrets.kv.v2.read_secret_version(
                        path=config['vault_path'],
                        mount_point='lotabots'
                    )
                    
                    if secret and 'data' in secret and 'data' in secret['data']:
                        verification['verified'] += 1
                    else:
                        verification['failed'] += 1
                        logger.error(f"Secret verification failed for {config['vault_path']}")
                        
                except Exception as e:
                    verification['failed'] += 1
                    logger.error(f"Failed to verify secret at {config['vault_path']}: {e}")
                    
            return verification
        except Exception as e:
            logger.error(f"Verification failed: {e}")
            raise
            
def main():
    try:
        import argparse
        parser = argparse.ArgumentParser(description='Migrate secrets to Vault')
        parser.add_argument('mapping_file', help='YAML file containing secret mapping')
        parser.add_argument('--vault-addr', default=os.getenv('VAULT_ADDR'),
                          help='Vault server address')
        parser.add_argument('--vault-token', default=os.getenv('VAULT_TOKEN'),
                          help='Vault token for authentication')
        args = parser.parse_args()
        
        if not args.vault_addr or not args.vault_token:
            parser.error("VAULT_ADDR and VAULT_TOKEN must be provided")
            
        try:
            migrator = VaultMigrator(args.vault_addr, args.vault_token)
            
            # Run migration
            logger.info("Starting secret migration...")
            status = asyncio.run(migrator.migrate_secrets(args.mapping_file))
            logger.info(f"Migration completed: {status}")
            
            # Verify migration
            logger.info("Verifying migration...")
            verification = migrator.verify_migration(args.mapping_file)
            logger.info(f"Verification completed: {verification}")
            
            if verification['failed'] > 0:
                logger.error("Some secrets failed verification!")
                exit(1)
                
        except Exception as e:
            logger.error(f"Migration failed: {e}")
            exit(1)
            
    except Exception as e:
        logger.error(f"Program failed: {e}")
        exit(1)
        
if __name__ == '__main__':
    main() 