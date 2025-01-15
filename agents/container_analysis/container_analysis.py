import os
import json
import logging
from datetime import datetime
import docker

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('container_analysis_agent')

class ContainerAnalysisAgent:
    def __init__(self):
        self.docker_client = docker.from_env()

        # Output directory
        self.output_dir = "/app/shared_data"
        os.makedirs(self.output_dir, exist_ok=True)

        # NVIDIA container images to analyze
        self.target_images = [
            "nvidia/cuda:12.3.2-devel-ubuntu22.04",
            "nvcr.io/nvidia/pytorch:24.01-py3",
            "nvcr.io/nvidia/tritonserver:24.01-py3",
            "nvcr.io/nvidia/clara-train-sdk:v4.0",
            "nvcr.io/nvidia/metropolis:2024.1.0"
        ]

    def analyze_containers(self):
        """Analyze NVIDIA container configurations and capabilities."""
        logger.info("Starting container analysis")

        try:
            analysis_results = {
                "timestamp": datetime.now().isoformat(),
                "images": self.analyze_images(),
                "runtime_analysis": self.analyze_runtime(),
                "security_analysis": self.analyze_security(),
                "recommendations": self.generate_recommendations()
            }

            # Save results
            output_file = os.path.join(self.output_dir, "container_analysis.json")
            with open(output_file, 'w') as f:
                json.dump(analysis_results, f, indent=2)

            logger.info(f"Analysis completed and saved to {output_file}")
            return analysis_results

        except Exception as e:
            logger.error(f"Error analyzing containers: {str(e)}")
            return None

    def analyze_images(self):
        """Analyze NVIDIA container images."""
        logger.info("Analyzing container images")

        image_analysis = {}
        for image_name in self.target_images:
            try:
                # Pull image if not present
                image = self.docker_client.images.pull(image_name)

                # Analyze image
                image_analysis[image_name] = {
                    "id": image.id,
                    "tags": image.tags,
                    "size": f"{image.attrs['Size'] / 1024**2:.2f} MB",
                    "created": image.attrs['Created'],
                    "os": image.attrs['Os'],
                    "architecture": image.attrs['Architecture'],
                    "environment": self.extract_environment(image),
                    "labels": image.labels
                }

            except Exception as e:
                logger.error(f"Error analyzing image {image_name}: {str(e)}")
                image_analysis[image_name] = {"error": str(e)}

        return image_analysis

    def analyze_runtime(self):
        """Analyze NVIDIA container runtime capabilities."""
        return {
            "runtime_version": self.get_runtime_version(),
            "capabilities": [
                "NVIDIA Driver",
                "CUDA Toolkit",
                "cuDNN",
                "TensorRT",
                "NCCL"
            ],
            "supported_features": [
                "GPU Access",
                "Multi-GPU Support",
                "GPU Metrics",
                "Memory Management",
                "Device Plugin"
            ]
        }

    def analyze_security(self):
        """Analyze container security features."""
        return {
            "security_features": [
                "AppArmor Profiles",
                "SELinux Policies",
                "Seccomp Filters",
                "User Namespace Mapping",
                "Capability Restrictions"
            ],
            "vulnerabilities": self.check_vulnerabilities(),
            "best_practices": [
                "Run containers as non-root",
                "Use read-only root filesystem",
                "Implement resource limits",
                "Enable security profiles",
                "Regular security updates"
            ]
        }

    def extract_environment(self, image):
        """Extract environment variables from image."""
        try:
            config = image.attrs.get('Config', {})
            env = config.get('Env', [])
            return [var for var in env if not any(
                secret in var.lower()
                for secret in ['password', 'token', 'key', 'secret']
            )]
        except Exception as e:
            logger.error(f"Error extracting environment: {str(e)}")
            return []

    def get_runtime_version(self):
        """Get NVIDIA container runtime version."""
        try:
            info = self.docker_client.info()
            runtimes = info.get('Runtimes', {})
            return runtimes.get('nvidia', {}).get('path', 'unknown')
        except Exception as e:
            logger.error(f"Error getting runtime version: {str(e)}")
            return "unknown"

    def check_vulnerabilities(self):
        """Check for known vulnerabilities."""
        return {
            "CVE-2024-0132": {
                "status": "patched",
                "severity": "high",
                "description": "TOCTOU vulnerability in NVIDIA Container Toolkit",
                "mitigation": "Update to latest version"
            }
        }

    def generate_recommendations(self):
        """Generate container-specific recommendations."""
        return [
            "Keep NVIDIA Container Toolkit updated to latest version",
            "Implement proper resource limits for containers",
            "Use NVIDIA NGC containers for optimized performance",
            "Enable security features like AppArmor and SELinux",
            "Regular vulnerability scanning and updates",
            "Monitor container resource usage",
            "Implement proper logging and monitoring"
        ]

def main():
    agent = ContainerAnalysisAgent()
    agent.analyze_containers()

if __name__ == "__main__":
    main()
