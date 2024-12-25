"""
Test script for the E-commerce Chatbot Plugin

This script demonstrates how to use the e-commerce chatbot plugin
with the LotaBots platform.
"""

import os
import sys
import json
from typing import Dict, Any

# Add the project root to the Python path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../")))

from ai_services.plugins.src.plugin_system import PluginManager

def load_config() -> Dict[str, Any]:
    """Load the plugin configuration."""
    return {
        "model_name": "gpt2",  # Using GPT-2 for this example
        "max_length": 128,
        "temperature": 0.7,
        "product_catalog_path": os.path.join(
            os.path.dirname(__file__),
            "sample_catalog.json"
        )
    }

def main():
    """Run the chatbot plugin test."""
    # Initialize plugin manager
    plugin_dir = os.path.dirname(__file__)
    plugin_manager = PluginManager(plugin_dir)
    
    # Load and initialize the plugin
    config = load_config()
    plugin = plugin_manager.load_plugin("ecommerce_chatbot", config)
    
    print("E-commerce Chatbot Plugin Test")
    print("Type 'quit' to exit\n")
    
    # Simple chat loop
    while True:
        user_input = input("User: ").strip()
        if user_input.lower() == "quit":
            break
        
        # Process user input
        response = plugin.execute({"message": user_input})
        
        # Print response
        print("\nAssistant:", response["response"])
        print("\nContext:", json.dumps(response["context"], indent=2))
        print()
    
    # Cleanup
    plugin_manager.unload_plugin("ecommerce_chatbot")

if __name__ == "__main__":
    main() 