# LotaBots AI Analysis System

A comprehensive system for analyzing NVIDIA container technologies, security features, and performance metrics using AI-powered agents.

## System Architecture

The system consists of several specialized AI agents:

1. **Security Analysis Agent**
   - Analyzes security vulnerabilities
   - Uses NVD API for CVE information
   - Generates security recommendations

2. **AI Security Agent**
   - Analyzes AI-specific security concerns
   - Evaluates NIM Blueprint implementation
   - Monitors AI model security features

3. **Performance Benchmark Agent**
   - Runs GPU performance benchmarks
   - Measures matrix operations, model inference, and memory bandwidth
   - Integrates with Prometheus for metrics

4. **Container Analysis Agent**
   - Analyzes NVIDIA container configurations
   - Checks runtime capabilities and security features
   - Monitors container health and resource usage

## Prerequisites

- Docker with NVIDIA Container Toolkit
- NVIDIA GPU with latest drivers
- Python 3.9+
- Docker Compose

## Setup

1. **Install NVIDIA Container Toolkit:**
   ```bash
   distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
   curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
   curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | sudo tee /etc/apt/sources.list.d/nvidia-docker.list
   sudo apt-get update
   sudo apt-get install -y nvidia-docker2
   ```

2. **Clone the Repository:**
   ```bash
   git clone <repository-url>
   cd lotabots
   ```

3. **Start the Services:**
   ```bash
   docker-compose up -d
   ```

## Monitoring

- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000
- Jaeger UI: http://localhost:16686
- Kibana: http://localhost:5601

## Agent Outputs

All agents write their analysis results to the shared volume at `/app/shared_data`:

- `security_analysis_*.json`: Security vulnerability reports
- `ai_security_analysis.json`: AI security analysis results
- `benchmark_results.json`: Performance benchmark results
- `container_analysis.json`: Container analysis reports

## Infrastructure Services

- **PostgreSQL**: Main database (port 5432)
- **Redis**: Caching and message queue (port 6379)
- **Elasticsearch**: Log aggregation (port 9200)
- **Prometheus**: Metrics collection (port 9090)
- **Grafana**: Metrics visualization (port 3000)
- **Jaeger**: Distributed tracing (port 16686)

## Development

1. **Install Python Dependencies:**
   ```bash
   pip install -r agents/requirements.txt
   ```

2. **Run Individual Agents:**
   ```bash
   python agents/security/security_analysis.py
   python agents/ai_security/ai_security_analysis.py
   python agents/benchmark/benchmark.py
   python agents/container_analysis/container_analysis.py
   ```

## Security Notes

- All agents run with proper NVIDIA GPU access
- Container security features are enabled
- Regular vulnerability scanning is implemented
- Sensitive data is properly handled

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
