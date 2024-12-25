"""
Diagnosis Assistance Plugin for Healthcare

This plugin analyzes patient symptoms to suggest possible diagnoses using a pre-trained model.
"""

from typing import Any, Dict
import torch
from transformers import AutoModelForSequenceClassification, AutoTokenizer
import numpy as np

from ai_services.plugins.src.plugin_system import PluginInterface

class Plugin(PluginInterface):
    """Diagnosis assistance plugin implementation."""
    
    def initialize(self, config: Dict[str, Any]) -> None:
        """Initialize the diagnosis assistance plugin."""
        self.config = config
        
        # Load the model and tokenizer
        self.model = AutoModelForSequenceClassification.from_pretrained(
            config["model_name"],
            device_map="auto"  # Automatically use available GPUs
        )
        self.tokenizer = AutoTokenizer.from_pretrained(config["model_name"])
        
        # Set threshold for diagnosis suggestion
        self.symptom_threshold = config.get("symptom_threshold", 0.5)
    
    def execute(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Analyze patient symptoms and suggest possible diagnoses."""
        symptoms = input_data.get("symptoms", "")
        if not symptoms:
            return {"error": "No symptoms provided"}
        
        # Tokenize symptoms
        inputs = self.tokenizer(symptoms, return_tensors="pt")
        inputs = {k: v.to(self.model.device) for k, v in inputs.items()}
        
        # Run model inference
        with torch.no_grad():
            outputs = self.model(**inputs)
            scores = torch.softmax(outputs.logits, dim=1).cpu().numpy()[0]
        
        # Determine if symptoms suggest a diagnosis
        suggest_diagnosis = scores[1] > self.symptom_threshold
        
        return {
            "suggest_diagnosis": suggest_diagnosis,
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