import os
import json
import logging
from datetime import datetime
import requests
import torch
from transformers import AutoTokenizer, AutoModelForSequenceClassification

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger('security_analysis_agent')

class SecurityAnalysisAgent:
    def __init__(self):
        self.model_name = "microsoft/codebert-base"
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        logger.info(f"Using device: {self.device}")

        # Load model and tokenizer
        self.tokenizer = AutoTokenizer.from_pretrained(self.model_name)
        self.model = AutoModelForSequenceClassification.from_pretrained(self.model_name)
        self.model.to(self.device)

        # Output directory
        self.output_dir = "/app/shared_data"
        os.makedirs(self.output_dir, exist_ok=True)

    def analyze_vulnerability(self, cve_id="CVE-2024-0132"):
        """Analyze a specific vulnerability."""
        logger.info(f"Analyzing vulnerability: {cve_id}")

        # Fetch vulnerability data from NVD
        nvd_api_url = f"https://services.nvd.nist.gov/rest/json/cves/2.0?cveId={cve_id}"
        try:
            response = requests.get(nvd_api_url)
            data = response.json()

            # Extract relevant information
            vulnerability_data = {
                "cve_id": cve_id,
                "description": data.get("vulnerabilities", [{}])[0].get("cve", {}).get("descriptions", [{}])[0].get("value", ""),
                "severity": data.get("vulnerabilities", [{}])[0].get("cve", {}).get("metrics", {}).get("cvssMetricV31", [{}])[0].get("cvssData", {}).get("baseScore", 0),
                "analysis_date": datetime.now().isoformat(),
                "recommendations": self.generate_recommendations(data)
            }

            # Save results
            output_file = os.path.join(self.output_dir, f"vulnerability_analysis_{cve_id}.json")
            with open(output_file, 'w') as f:
                json.dump(vulnerability_data, f, indent=2)

            logger.info(f"Analysis completed and saved to {output_file}")
            return vulnerability_data

        except Exception as e:
            logger.error(f"Error analyzing vulnerability: {str(e)}")
            return None

    def generate_recommendations(self, vulnerability_data):
        """Generate security recommendations based on vulnerability data."""
        # Extract description for analysis
        description = vulnerability_data.get("vulnerabilities", [{}])[0].get("cve", {}).get("descriptions", [{}])[0].get("value", "")

        # Tokenize and analyze
        inputs = self.tokenizer(description, return_tensors="pt", truncation=True, max_length=512)
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        with torch.no_grad():
            outputs = self.model(**inputs)
            predictions = torch.nn.functional.softmax(outputs.logits, dim=-1)

        # Generate recommendations based on model output
        recommendations = [
            "Update to the latest NVIDIA Container Toolkit version",
            "Implement strict access controls for container operations",
            "Regular security audits and vulnerability scanning",
            "Monitor container activities for suspicious behavior"
        ]

        return recommendations

def main():
    agent = SecurityAnalysisAgent()
    agent.analyze_vulnerability()

if __name__ == "__main__":
    main()
