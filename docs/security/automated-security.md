# Automated Security Testing Guide

This guide outlines how to implement automated security testing in the CI/CD pipeline for the LOTA AI platform.

## GitHub Actions Integration

### 1. Security Scan Workflow

Create `.github/workflows/security-scan.yml`:

```yaml
name: Security Scan
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run Cargo Audit
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Run Clippy
        run: cargo clippy -- -D warnings

      - name: Run OWASP Dependency-Check
        uses: dependency-check/Dependency-Check_Action@main
        with:
          project: "LOTA AI"
          path: "."
          format: "HTML"
          args: >
            --failOnCVSS 7
            --enableRetired

      - name: Run Trivy Scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          ignore-unfixed: true
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Upload Trivy results
        uses: github/codeql-action/upload-sarif@v2
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'

      - name: Run Snyk
        uses: snyk/actions/python@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          args: --severity-threshold=high

      - name: Upload scan results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: security-scan-results
          path: |
            dependency-check-report.html
            trivy-results.sarif
```

### 2. Container Scanning Workflow

Create `.github/workflows/container-scan.yml`:

```yaml
name: Container Security Scan
on:
  push:
    paths:
      - 'Dockerfile'
      - '.dockerignore'
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sundays

jobs:
  container-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build container
        run: docker build -t lotaai:${{ github.sha }} .

      - name: Run Trivy container scan
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: 'lotaai:${{ github.sha }}'
          format: 'sarif'
          output: 'trivy-container.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Run Dockle
        uses: goodwithtech/dockle-action@v1
        with:
          image: lotaai:${{ github.sha }}
          format: sarif
          output: dockle-results.sarif

      - name: Upload scan results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: container-scan-results
          path: |
            trivy-container.sarif
            dockle-results.sarif
```

## Jenkins Pipeline Integration

Create `Jenkinsfile`:

```groovy
pipeline {
    agent any

    environment {
        CARGO_HOME = '/var/lib/jenkins/.cargo'
    }

    stages {
        stage('Security Scan') {
            parallel {
                stage('Dependency Check') {
                    steps {
                        sh 'cargo audit'
                        dependencyCheck(
                            additionalArguments: '--format HTML --failOnCVSS 7',
                            odcInstallation: 'OWASP Dependency-Check'
                        )
                    }
                }

                stage('SAST') {
                    steps {
                        sh 'cargo clippy -- -D warnings'
                        sh 'bandit -r . -f json -o bandit-report.json'
                    }
                }

                stage('Container Scan') {
                    steps {
                        sh '''
                            docker build -t lotaai:${BUILD_NUMBER} .
                            trivy image --format json --output trivy-results.json lotaai:${BUILD_NUMBER}
                        '''
                    }
                }
            }
        }

        stage('Security Gate') {
            steps {
                script {
                    def trivyResult = readJSON file: 'trivy-results.json'
                    def highVulns = trivyResult.vulnerabilities.findAll { it.severity == 'HIGH' }
                    if (highVulns.size() > 0) {
                        error "Found ${highVulns.size()} high severity vulnerabilities"
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: '''
                dependency-check-report.html,
                bandit-report.json,
                trivy-results.json
            ''', fingerprint: true

            publishHTML([
                allowMissing: false,
                alwaysLinkToLastBuild: true,
                keepAll: true,
                reportDir: '.',
                reportFiles: 'dependency-check-report.html',
                reportName: 'Dependency Check Report'
            ])
        }
    }
}
```

## Local Development Integration

### 1. Pre-commit Hooks

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/rusty-ferris-club/rust-pre-commit
    rev: v1.0.0
    hooks:
      - id: cargo-audit
      - id: clippy
        args: ["--", "-D", "warnings"]

  - repo: https://github.com/PyCQA/bandit
    rev: 1.7.5
    hooks:
      - id: bandit
        args: ["-c", "pyproject.toml"]

  - repo: https://github.com/aquasecurity/tfsec
    rev: v1.28.1
    hooks:
      - id: tfsec
```

### 2. VS Code Integration

Add to `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
  "security.workspace.trust.enabled": true,
  "security.workspace.trust.untrustedFiles": "newWindow"
}
```

## Automated Security Testing Tools

### 1. Static Analysis

```bash
# Install tools
cargo install cargo-audit cargo-deny
pip install bandit safety

# Run checks
cargo audit
cargo deny check advisories
bandit -r . -f json -o bandit-report.json
safety check
```

### 2. Container Security

```bash
# Install tools
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh
curl -L https://raw.githubusercontent.com/goodwithtech/dockle/master/install.sh | sh

# Run checks
trivy image lotaai:latest
dockle lotaai:latest
```

### 3. Infrastructure Security

```bash
# Install tools
curl -L "$(curl -s https://api.github.com/repos/terraform-linters/tflint/releases/latest | grep -o -E "https://.+?_linux_amd64.zip")" > tflint.zip
unzip tflint.zip && rm tflint.zip

# Run checks
tflint
terraform validate
```

## Reporting and Monitoring

### 1. Security Dashboard

Create a custom dashboard in Google Cloud Console:

```bash
# Create dashboard using gcloud
cat << EOF > security-dashboard.json
{
  "displayName": "Security Scan Results",
  "gridLayout": {
    "columns": "2",
    "widgets": [
      {
        "title": "High Severity Findings",
        "xyChart": {
          "dataSets": [{
            "timeSeriesQuery": {
              "timeSeriesFilter": {
                "filter": "metric.type=\"custom.googleapis.com/security/high_severity_findings\"",
                "aggregation": {
                  "alignmentPeriod": "86400s",
                  "perSeriesAligner": "ALIGN_SUM"
                }
              }
            }
          }]
        }
      }
    ]
  }
}
EOF

gcloud monitoring dashboards create --config-from-file=security-dashboard.json
```

### 2. Automated Reports

Create a script for generating weekly security reports:

```bash
#!/bin/bash
# security-report.sh

# Get date range
START_DATE=$(date -d "7 days ago" +%Y-%m-%d)
END_DATE=$(date +%Y-%m-%d)

# Generate report
echo "Security Scan Report: ${START_DATE} to ${END_DATE}" > report.md
echo "----------------------------------------" >> report.md

# Get vulnerability trends
gcloud logging read "resource.type=cloud_run_revision AND severity>=WARNING" \
  --project=$PROJECT_ID \
  --format="table(timestamp,severity,textPayload)" \
  --filter="timestamp >= \"${START_DATE}\"" >> report.md

# Get metrics
gcloud monitoring metrics list \
  --filter="metric.type=contains(\"security\")" \
  --format="table(metric.type,metric.labels)" >> report.md
```

## Best Practices

1. **Scan Frequency**
   - Run basic checks on every commit
   - Run full scans daily
   - Run deep scans weekly
   - Monitor continuously

2. **Failure Thresholds**
   - Block on critical vulnerabilities
   - Warn on high severity issues
   - Track medium and low severity
   - Monitor trends over time

3. **Integration Points**
   - Pre-commit hooks
   - CI/CD pipelines
   - Deployment gates
   - Runtime monitoring

4. **Response Actions**
   - Automated notifications
   - Ticket creation
   - Security team alerts
   - Deployment blocks
