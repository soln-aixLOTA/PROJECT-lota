# Security Tool Configurations

## Bandit Configuration

Bandit is configured through `pyproject.toml`. Key settings include:

```toml
[tool.bandit]
exclude_dirs = ["tests", "venv", ".venv"]  # Directories to exclude
skips = ["B101", "B104"]  # Tests to skip
```

### Common Test IDs
- B101: Use of assert
- B102: exec used
- B103: Set bad file permissions
- B104: Hardcoded bind all interfaces
- B105: Password string in source code
- B108: Hardcoded temp file paths
- B301: pickle usage
- B506: yaml load

### Running Bandit
```bash
# Run with pyproject.toml config
bandit -r . -c pyproject.toml

# Generate HTML report
bandit -r . -f html -o bandit-report.html
```

## Snyk Configuration

### Setup
1. Install Snyk CLI:
   ```bash
   npm install -g snyk
   ```

2. Authenticate:
   ```bash
   snyk auth
   ```

3. Configure project:
   ```bash
   # Create .snyk file
   snyk wizard
   ```

### GitHub Actions Integration
```yaml
- name: Run Snyk scan
  uses: snyk/actions/python@0.4.0
  env:
    SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
  with:
    command: monitor
```

### Local Usage
```bash
# Test dependencies
snyk test

# Monitor project
snyk monitor

# Test specific requirements file
snyk test --file=requirements.txt
```

## OWASP Dependency-Check

### Configuration Options
```yaml
- name: Run OWASP Dependency-Check
  uses: dependency-check/Dependency-Check_Action@v3.0.0
  with:
    project: "LOTA AI"
    path: "."
    format: "HTML"
    args: >
      --failOnCVSS 7
      --enableRetired
```

### Common Arguments
- `--failOnCVSS`: Set CVSS score threshold
- `--enableRetired`: Include retired vulnerabilities
- `--suppression`: Specify suppression file
- `--exclude`: Exclude paths from scan

### Local Usage
```bash
# Install
pip install dependency-check

# Run scan
dependency-check --scan /path/to/project --format HTML
```

## Interpreting Security Scan Results

### Bandit Results
- **Severity Levels**: LOW, MEDIUM, HIGH
- **Confidence Levels**: LOW, MEDIUM, HIGH
- Focus on HIGH severity + HIGH confidence issues first

### Snyk Results
- **Priority Scores**: 0-1000 (higher = more critical)
- **Upgrade Path**: Follow recommended version upgrades
- **Patches**: Apply available patches when possible

### OWASP Dependency-Check Results
- **CVSS Scores**: 0.0-10.0
  - 9.0-10.0: Critical
  - 7.0-8.9: High
  - 4.0-6.9: Medium
  - 0.1-3.9: Low
- **CPE Matches**: Check for false positives

## Security Monitoring Setup

### Google Cloud Monitoring

1. Enable monitoring:
```bash
gcloud services enable monitoring.googleapis.com
```

2. Configure metrics:
```yaml
monitoring:
  metrics:
    - name: "security/failed_login_attempts"
      type: "counter"
      threshold: 10
      period: "5m"
    - name: "security/api_requests"
      type: "rate"
      threshold: 1000
      period: "1m"
```

3. Set up alerts:
```yaml
alerts:
  - name: "High Failed Login Rate"
    condition: "failed_login_attempts > 10 per 5m"
    notification_channels:
      - email: "security@example.com"
      - slack: "#security-alerts"
```

### Key Metrics to Monitor
1. Authentication Events
   - Failed login attempts
   - Password reset requests
   - Token revocations

2. API Security
   - Request rates
   - Error rates
   - Response times
   - Authentication failures

3. Infrastructure
   - Resource utilization
   - Network traffic patterns
   - Database connections
   - File system changes

4. Dependencies
   - Package version changes
   - New vulnerability alerts
   - Dependency health checks

### Alert Thresholds
- Critical: Immediate response required (< 15 minutes)
- High: Response required within 1 hour
- Medium: Response required within 24 hours
- Low: Response required within 1 week

## Additional Resources
- [Bandit Documentation](https://bandit.readthedocs.io/)
- [Snyk Documentation](https://docs.snyk.io/)
- [OWASP Dependency-Check](https://owasp.org/www-project-dependency-check/)
- [Google Cloud Monitoring](https://cloud.google.com/monitoring)
