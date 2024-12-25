"""
Test script for the Diagnosis Assistance Plugin

This script demonstrates how to use the diagnosis assistance plugin
with the LotaBots platform.
"""

import os
import sys
import json
from typing import Dict, Any

# Add the ai-services directory to the Python path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../ai-services")))

from ai_services.plugins.src.plugin_system import PluginManager

def load_config() -> Dict[str, Any]:
    """Load the plugin configuration."""
    return {
        "model_name": "distilbert-base-uncased-finetuned-sst-2-english",  # Example model
        "symptom_threshold": 0.5
    }

def main():
    """Run the diagnosis assistance plugin test."""
    # Initialize plugin manager
    plugin_dir = os.path.dirname(__file__)
    plugin_manager = PluginManager(plugin_dir)
    
    # Load and initialize the plugin
    config = load_config()
    plugin = plugin_manager.load_plugin("diagnosis_assistance", config)
    
    print("Diagnosis Assistance Plugin Test")
    print("Type 'quit' to exit\n")
    
    # Debug: Print the current PYTHONPATH
    print("PYTHONPATH:", sys.path)
    
    # Simple test loop
    while True:
        symptoms = input("Symptoms: ").strip()
        if symptoms.lower() == "quit":
            break
        
        # Process symptoms
        response = plugin.execute({"symptoms": symptoms})
        
        # Print response
        print("\nDiagnosis Suggested:" if response["suggest_diagnosis"] else "\nNo Diagnosis Suggested")
        print("Confidence:", response["confidence"])
        print()
    
    # Cleanup
    plugin_manager.unload_plugin("diagnosis_assistance")

if __name__ == "__main__":
    main() 