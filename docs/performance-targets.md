# Performance Targets

## Current Targets
- **API Gateway Throughput:** 5,000 requests/second at p95 latency < 50ms  
- **Training Service Time:** Training a medium-size model within 10 minutes on a single GPU  
- **Inference Service Latency:** < 20ms p95 for text-based requests

## Updates Based on Recent Tests
- The system currently meets these targets for moderate workloads.  
- For higher concurrency scenarios (10,000+ requests/second), there is a noticeable increase in p95 latency (up to 80ms).  
- **Recommendation:** Investigate caching strategies or additional clustering to handle peak loads.

... 