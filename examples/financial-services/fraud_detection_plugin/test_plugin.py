"""
Test script for the Fraud Detection Plugin

This script demonstrates how to use the fraud detection plugin
with the LotaBots platform.
"""

import os
import sys
import json
from typing import Dict, Any

# Debug: Print the current PYTHONPATH
print("PYTHONPATH:", sys.path)

# Add the project root to the Python path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../")))

# Add the ai-services directory to the Python path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../ai-services")))

from plugins.src.plugin_system import PluginManager

def load_config() -> Dict[str, Any]:
    """Load the plugin configuration."""
    return {
        "model_name": "distilbert-base-uncased-finetuned-sst-2-english",  # Example model
        "threshold": 0.5
    }

def main():
    """Run the fraud detection plugin test."""
    # Initialize plugin manager
    plugin_dir = os.path.dirname(__file__)
    plugin_manager = PluginManager(plugin_dir)
    
    # Discover plugins
    plugin_manager.discover_plugins()
    
    # Load and initialize the plugin
    config = load_config()
    plugin = plugin_manager.load_plugin("fraud_detection", config)
    
    print("Fraud Detection Plugin Test")
    print("Type 'quit' to exit\n")
    
    # Simple test loop
    while True:
        transaction = input("Transaction: ").strip()
        if transaction.lower() == "quit":
            break
        
        # Process transaction
        response = plugin.execute({"transaction": transaction})
        
        # Print response
        print("\nFraud Detected:" if response["is_fraud"] else "\nNo Fraud Detected")
        print("Confidence:", response["confidence"])
        print()
    
    # Cleanup
    plugin_manager.unload_plugin("fraud_detection")

if __name__ == "__main__":
    main() 