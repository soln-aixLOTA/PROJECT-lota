"""
E-commerce Chatbot Plugin for LotaBots

This plugin provides customer service chatbot functionality for e-commerce applications.
It uses a pre-trained language model and product catalog data to generate responses.
"""

import json
from typing import Any, Dict, List
import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, AutoModel
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity

from ai_services.plugins.src.plugin_system import PluginInterface

class ProductCatalog:
    """Manages product catalog data for the chatbot."""
    
    def __init__(self, catalog_path: str):
        with open(catalog_path, 'r') as f:
            self.catalog = json.load(f)
        
        # Load BERT model for embeddings
        self.embedding_model = AutoModel.from_pretrained('sentence-transformers/all-MiniLM-L6-v2')
        self.embedding_tokenizer = AutoTokenizer.from_pretrained('sentence-transformers/all-MiniLM-L6-v2')
        
        # Move model to GPU if available
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        self.embedding_model = self.embedding_model.to(self.device)
        
        self.product_embeddings = {}
        self._compute_embeddings()
    
    def _get_embedding(self, text: str) -> np.ndarray:
        """Compute embedding for a text string."""
        # Tokenize and move to device
        inputs = self.embedding_tokenizer(
            text,
            padding=True,
            truncation=True,
            max_length=128,
            return_tensors='pt'
        ).to(self.device)
        
        # Generate embeddings
        with torch.no_grad():
            outputs = self.embedding_model(**inputs)
            embeddings = outputs.last_hidden_state.mean(dim=1).cpu().numpy()
        
        return embeddings[0]
    
    def _compute_embeddings(self) -> None:
        """Compute embeddings for product descriptions."""
        for product in self.catalog["products"]:
            # Combine product information for embedding
            product_text = f"{product['name']} {product['description']} {product['category']}"
            
            # Add specifications to the text
            specs = product['specifications']
            specs_text = ' '.join(f"{k}: {v}" for k, v in specs.items())
            product_text += ' ' + specs_text
            
            # Compute and store embedding
            self.product_embeddings[product['id']] = self._get_embedding(product_text)
    
    def search_products(self, query: str, limit: int = 5) -> List[Dict[str, Any]]:
        """Search for products matching the query using semantic search."""
        # Get query embedding
        query_embedding = self._get_embedding(query)
        
        # Calculate similarities
        similarities = []
        for product_id, embedding in self.product_embeddings.items():
            similarity = cosine_similarity(
                query_embedding.reshape(1, -1),
                embedding.reshape(1, -1)
            )[0][0]
            similarities.append((product_id, similarity))
        
        # Sort by similarity and get top matches
        similarities.sort(key=lambda x: x[1], reverse=True)
        top_matches = similarities[:limit]
        
        # Get product details for top matches
        results = []
        for product_id, similarity in top_matches:
            for product in self.catalog["products"]:
                if product["id"] == product_id:
                    product_info = product.copy()
                    product_info["search_similarity"] = float(similarity)
                    results.append(product_info)
                    break
        
        return results
    
    def get_product_info(self, product_id: str) -> Dict[str, Any]:
        """Get detailed information about a product."""
        for product in self.catalog["products"]:
            if product["id"] == product_id:
                return product
        return {}

class Plugin(PluginInterface):
    """E-commerce chatbot plugin implementation."""
    
    def initialize(self, config: Dict[str, Any]) -> None:
        """Initialize the chatbot plugin."""
        self.config = config
        
        # Load the language model and tokenizer
        self.model = AutoModelForCausalLM.from_pretrained(
            config["model_name"],
            device_map="auto"  # Automatically use available GPUs
        )
        self.tokenizer = AutoTokenizer.from_pretrained(config["model_name"])
        
        # Load the product catalog
        self.product_catalog = ProductCatalog(config["product_catalog_path"])
        
        # Initialize conversation history
        self.conversation_history = []
        
        # Set generation parameters
        self.max_length = config.get("max_length", 128)
        self.temperature = config.get("temperature", 0.7)
    
    def _format_conversation(self) -> str:
        """Format conversation history for the model."""
        formatted = ""
        for turn in self.conversation_history[-5:]:  # Keep last 5 turns for context
            formatted += f"User: {turn['user']}\nAssistant: {turn['assistant']}\n"
        return formatted.strip()
    
    def _generate_response(self, prompt: str) -> str:
        """Generate a response using the language model."""
        inputs = self.tokenizer(prompt, return_tensors="pt")
        inputs = {k: v.to(self.model.device) for k, v in inputs.items()}
        
        with torch.no_grad():
            outputs = self.model.generate(
                **inputs,
                max_length=self.max_length,
                temperature=self.temperature,
                pad_token_id=self.tokenizer.eos_token_id
            )
        
        response = self.tokenizer.decode(outputs[0], skip_special_tokens=True)
        return response.split("Assistant: ")[-1].strip()
    
    def _enhance_response_with_products(self, user_message: str, response: str) -> str:
        """Enhance the response with product information when relevant."""
        products = self.product_catalog.search_products(user_message)
        if products:
            product_info = "\n\nHere are some relevant products:\n"
            for i, product in enumerate(products, 1):
                product_info += f"{i}. {product['name']} - ${product['price']}\n"
            response += product_info
        return response
    
    def execute(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Process a user message and generate a response."""
        user_message = input_data.get("message", "")
        if not user_message:
            return {"error": "No message provided"}
        
        # Format conversation with history
        self.conversation_history.append({"user": user_message, "assistant": ""})
        prompt = self._format_conversation()
        
        # Generate base response
        response = self._generate_response(prompt)
        
        # Enhance response with product information
        enhanced_response = self._enhance_response_with_products(user_message, response)
        
        # Update conversation history
        self.conversation_history[-1]["assistant"] = enhanced_response
        
        return {
            "response": enhanced_response,
            "confidence": 0.95,  # TODO: Implement proper confidence scoring
            "context": {
                "conversation_turns": len(self.conversation_history),
                "products_mentioned": bool(self.product_catalog.search_products(user_message))
            }
        }
    
    def cleanup(self) -> None:
        """Clean up resources used by the plugin."""
        # Free GPU memory
        if hasattr(self, "model"):
            del self.model
        if hasattr(self, "tokenizer"):
            del self.tokenizer
        torch.cuda.empty_cache()
        
        # Clear conversation history
        self.conversation_history.clear() 