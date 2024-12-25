"""
Fraud Detection Plugin for Financial Services

This plugin analyzes transaction data to detect potential fraud using a pre-trained model.
"""

from typing import Any, Dict
import torch
from transformers import AutoModelForSequenceClassification, AutoTokenizer
import numpy as np

from ai_services.plugins.src.plugin_system import PluginInterface

class Plugin(PluginInterface):
    """Fraud detection plugin implementation."""
    
    def initialize(self, config: Dict[str, Any]) -> None:
        """Initialize the fraud detection plugin."""
        self.config = config
        
        # Load the model and tokenizer
        self.model = AutoModelForSequenceClassification.from_pretrained(
            config["model_name"],
            device_map="auto"  # Automatically use available GPUs
        )
        self.tokenizer = AutoTokenizer.from_pretrained(config["model_name"])
        
        # Set threshold for fraud detection
        self.threshold = config.get("threshold", 0.5)
    
    def execute(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze transaction data and detect potential fraud."""
        transaction = input_data.get("transaction", "")
        if not transaction:
            return {"error": "No transaction data provided"}
        
        # Tokenize transaction data
        inputs = self.tokenizer(transaction, return_tensors="pt")
        inputs = {k: v.to(self.model.device) for k, v in inputs.items()}
        
        # Run model inference
        with torch.no_grad():
            outputs = self.model(**inputs)
            scores = torch.softmax(outputs.logits, dim=1).cpu().numpy()[0]
        
        # Determine if transaction is fraudulent
        is_fraud = scores[1] > self.threshold
        
        return {
            "is_fraud": is_fraud,
            "confidence": float(scores[1]),
            "scores": scores.tolist()
        }
    
    def cleanup(self) -> None:
        """Clean up resources used by the plugin."""
        # Free GPU memory
        if hasattr(self, "model"):
            del self.model
        if hasattr(self, "tokenizer"):
            del self.tokenizer
        torch.cuda.empty_cache() 