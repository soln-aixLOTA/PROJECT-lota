import os
import pytest
from langchain_nvidia_ai_endpoints import (
    ChatNVIDIA,
    NVIDIAEmbeddings,
    NVIDIARerank
)

# Test configurations
EMBEDDING_URL = "http://nim-embedding-service:8001/v1"
RERANK_URL = "http://nim-rerank-service:8002/v1"
LLM_URL = "http://nim-llm-service:8000/v1"

def test_embedding_service():
    """Test the embedding service."""
    try:
        embedding_model = NVIDIAEmbeddings(
            model="nvidia/nv-embedqa-e5-v5",
            base_url=EMBEDDING_URL
        )
        
        # Test text embedding
        text = "This is a test sentence for embedding."
        embeddings = embedding_model.embed_query(text)
        
        assert len(embeddings) > 0, "Embedding vector should not be empty"
        assert isinstance(embeddings, list), "Embeddings should be a list"
        
    except Exception as e:
        pytest.fail(f"Embedding service test failed: {str(e)}")

def test_rerank_service():
    """Test the rerank service."""
    try:
        reranker = NVIDIARerank(
            model="nvidia/nv-rerankqa-mistral-4b-v3",
            base_url=RERANK_URL
        )
        
        # Test reranking
        query = "What is machine learning?"
        documents = [
            "Machine learning is a subset of AI.",
            "Machine learning uses data to learn patterns.",
            "Artificial intelligence is a broad field."
        ]
        
        results = reranker.rerank(query=query, documents=documents)
        
        assert len(results) == len(documents), "Should return same number of documents"
        assert all('score' in doc for doc in results), "Each result should have a score"
        
    except Exception as e:
        pytest.fail(f"Rerank service test failed: {str(e)}")

def test_llm_service():
    """Test the LLM service."""
    try:
        llm = ChatNVIDIA(
            base_url=LLM_URL,
            model="meta/llama-3.1-8b-instruct"
        )
        
        # Test chat completion
        prompt = "What is the capital of France?"
        response = llm.invoke(prompt)
        
        assert response is not None, "Response should not be None"
        assert isinstance(response, str), "Response should be a string"
        assert len(response) > 0, "Response should not be empty"
        
    except Exception as e:
        pytest.fail(f"LLM service test failed: {str(e)}")

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"]) 