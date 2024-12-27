#!/usr/bin/env python3

import pandas as pd
import numpy as np
from sklearn.ensemble import IsolationForest
from datetime import datetime, timedelta
import json
import re

def parse_log_line(line):
    """Parse a log line into structured data."""
    # Example log format: [2024-01-01 12:00:00] [INFO] message
    pattern = r'\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] \[(\w+)\] (.*)'
    match = re.match(pattern, line)
    if match:
        timestamp, level, message = match.groups()
        return {
            'timestamp': datetime.strptime(timestamp, '%Y-%m-%d %H:%M:%S'),
            'level': level,
            'message': message
        }
    return None

def collect_logs(log_paths):
    """Collect logs from multiple sources."""
    logs = []
    for path in log_paths:
        try:
            with open(path, 'r') as f:
                for line in f:
                    parsed = parse_log_line(line.strip())
                    if parsed:
                        logs.append(parsed)
        except Exception as e:
            print(f"Error reading log file {path}: {e}")
    return pd.DataFrame(logs)

def detect_anomalies(df):
    """Detect anomalies in log patterns."""
    # Convert timestamps to numeric features
    df['hour'] = df['timestamp'].dt.hour
    df['minute'] = df['timestamp'].dt.minute
    df['error_count'] = df['level'].apply(lambda x: 1 if x in ['ERROR', 'CRITICAL'] else 0)
    
    # Group by time windows
    windows = df.set_index('timestamp').resample('5T').agg({
        'error_count': 'sum',
        'level': 'count'
    }).fillna(0)
    
    # Train isolation forest
    clf = IsolationForest(contamination=0.1, random_state=42)
    anomalies = clf.fit_predict(windows)
    
    return windows[anomalies == -1]

def analyze_error_patterns(df):
    """Analyze patterns in error messages."""
    error_df = df[df['level'].isin(['ERROR', 'CRITICAL'])]
    patterns = error_df['message'].value_counts().head(10)
    return patterns

def generate_report(anomalies, patterns):
    """Generate analysis report."""
    report = []
    report.append("# Log Analysis Report")
    report.append(f"\nAnalysis Time: {datetime.now()}")
    
    report.append("\n## Anomalies Detected")
    for idx, row in anomalies.iterrows():
        report.append(f"- {idx}: {row['error_count']} errors in {row['level']} events")
    
    report.append("\n## Common Error Patterns")
    for pattern, count in patterns.items():
        report.append(f"- ({count} occurrences) {pattern}")
    
    with open('analysis_results.txt', 'w') as f:
        f.write('\n'.join(report))

def main():
    # Configure log paths
    log_paths = [
        '/var/log/api/application.log',
        '/var/log/api/error.log'
    ]
    
    # Collect and process logs
    print("Collecting logs...")
    df = collect_logs(log_paths)
    
    if df.empty:
        print("No logs found")
        return
    
    print("Detecting anomalies...")
    anomalies = detect_anomalies(df)
    
    print("Analyzing error patterns...")
    patterns = analyze_error_patterns(df)
    
    print("Generating report...")
    generate_report(anomalies, patterns)
    print("Analysis complete. Results written to analysis_results.txt")

if __name__ == '__main__':
    main() 