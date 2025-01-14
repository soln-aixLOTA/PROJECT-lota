# Google Cloud Monitoring Integration Guide

This guide explains how to integrate our custom security metrics with Google Cloud Monitoring.

## Prerequisites

1. Google Cloud SDK installed and configured
2. Access to Google Cloud Console with appropriate permissions
3. Application deployed with metrics enabled

## Setup Steps

### 1. Enable Required APIs

```bash
# Enable Cloud Monitoring API
gcloud services enable monitoring.googleapis.com

# Enable Cloud Logging API (for log-based metrics)
gcloud services enable logging.googleapis.com
```

### 2. Configure OpenTelemetry Exporter

The application is already configured to export metrics using OpenTelemetry. The metrics are exposed at the `/metrics` endpoint in Prometheus format.

### 3. Deploy OpenTelemetry Collector

Create a configuration file `otel-collector-config.yaml`:

```yaml
receivers:
  prometheus:
    config:
      scrape_configs:
        - job_name: 'lota-ai'
          scrape_interval: 10s
          static_configs:
            - targets: ['localhost:8080']
              labels:
                service: 'lota-ai'

exporters:
  googlecloud:
    project: "your-project-id"
    metric_prefix: "custom.googleapis.com/lota_ai"

service:
  pipelines:
    metrics:
      receivers: [prometheus]
      exporters: [googlecloud]
```

### 4. Create Alert Policies

You can create alert policies either through the Google Cloud Console or using `gcloud` commands:

```bash
# Create an alert policy for high error rate
gcloud alpha monitoring policies create \
  --display-name="High API Error Rate" \
  --condition-filter='metric.type="custom.googleapis.com/lota_ai/api_errors_total" AND resource.type="global"' \
  --condition-threshold-value=0.05 \
  --condition-threshold-duration=300s \
  --notification-channels="projects/$PROJECT_ID/notificationChannels/$CHANNEL_ID"

# Create an alert for authentication failures
gcloud alpha monitoring policies create \
  --display-name="High Auth Failure Rate" \
  --condition-filter='metric.type="custom.googleapis.com/lota_ai/api_auth_failures" AND resource.type="global"' \
  --condition-threshold-value=10 \
  --condition-threshold-duration=300s \
  --notification-channels="projects/$PROJECT_ID/notificationChannels/$CHANNEL_ID"
```

## Available Metrics

Our application exports the following metrics:

1. `api_requests_total` - Total number of API requests
   - Labels: method, path
   - Type: Counter

2. `api_errors_total` - Total number of API errors
   - Labels: error_type, status_code
   - Type: Counter

3. `api_auth_failures` - Authentication failures
   - Labels: reason
   - Type: Counter

4. `api_request_duration` - Request duration in seconds
   - Labels: path
   - Type: Histogram

5. `api_connections_active` - Number of active connections
   - Type: Counter

## Creating Dashboards

### 1. Basic Security Dashboard

```bash
# Create a dashboard using gcloud
cat << EOF > dashboard.json
{
  "displayName": "LOTA AI Security Dashboard",
  "gridLayout": {
    "columns": "2",
    "widgets": [
      {
        "title": "API Error Rate",
        "xyChart": {
          "dataSets": [{
            "timeSeriesQuery": {
              "timeSeriesFilter": {
                "filter": "metric.type=\"custom.googleapis.com/lota_ai/api_errors_total\"",
                "aggregation": {
                  "alignmentPeriod": "60s",
                  "perSeriesAligner": "ALIGN_RATE"
                }
              }
            }
          }]
        }
      },
      {
        "title": "Authentication Failures",
        "xyChart": {
          "dataSets": [{
            "timeSeriesQuery": {
              "timeSeriesFilter": {
                "filter": "metric.type=\"custom.googleapis.com/lota_ai/api_auth_failures\"",
                "aggregation": {
                  "alignmentPeriod": "60s",
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

gcloud monitoring dashboards create --config-from-file=dashboard.json
```

## Best Practices

1. **Metric Naming**
   - Use consistent naming conventions
   - Prefix custom metrics appropriately
   - Include relevant labels for better filtering

2. **Alert Thresholds**
   - Start with conservative thresholds
   - Adjust based on observed patterns
   - Consider time of day and usage patterns

3. **Dashboard Organization**
   - Group related metrics together
   - Include both overview and detailed views
   - Add documentation links

4. **Monitoring Coverage**
   - Monitor all critical endpoints
   - Track both successful and failed operations
   - Include infrastructure metrics

## Troubleshooting

### Common Issues

1. **Metrics Not Appearing**
   - Check OpenTelemetry Collector logs
   - Verify service account permissions
   - Ensure correct metric prefix

2. **Alert Delays**
   - Check alert policy conditions
   - Verify notification channel configuration
   - Check metric collection interval

3. **Missing Data**
   - Verify application is exporting metrics
   - Check network connectivity
   - Validate metric names and labels

## Additional Resources

- [Google Cloud Monitoring Documentation](https://cloud.google.com/monitoring/docs)
- [OpenTelemetry Collector Documentation](https://opentelemetry.io/docs/collector/)
- [Prometheus Query Language](https://prometheus.io/docs/prometheus/latest/querying/basics/)

## Log-Based Metrics

### Creating Log-Based Metrics

1. **Counter-Based Log Metrics**

```bash
# Create a metric for failed login attempts from logs
gcloud logging metrics create failed_login_attempts \
  --description="Count of failed login attempts" \
  --filter="resource.type=cloud_run_revision AND severity=WARNING AND jsonPayload.event=login_failed" \
  --metric-descriptor-type="counter"
```

2. **Distribution-Based Log Metrics**

```bash
# Create a metric for API response times
gcloud logging metrics create api_response_times \
  --description="Distribution of API response times" \
  --filter="resource.type=cloud_run_revision AND jsonPayload.duration_ms exists" \
  --metric-descriptor-type="distribution" \
  --value-extractor="EXTRACT(jsonPayload.duration_ms)"
```

### Common Log Filters

```yaml
# Authentication failures
resource.type="cloud_run_revision"
severity=WARNING
jsonPayload.event="auth_failure"

# Security-related errors
resource.type="cloud_run_revision"
severity>=ERROR
jsonPayload.category="security"

# Rate limiting events
resource.type="cloud_run_revision"
jsonPayload.event="rate_limit_exceeded"
```

## Notification Channels

### Email Notifications

```bash
# Create an email notification channel
gcloud alpha monitoring channels create \
  --display-name="Security Team Email" \
  --type="email" \
  --channel-labels="email_address=security@example.com"
```

### Slack Integration

1. Create a Slack app and get the webhook URL
2. Create the notification channel:

```bash
# Create a Slack notification channel
gcloud alpha monitoring channels create \
  --display-name="Security Alerts Slack" \
  --type="webhook_slack" \
  --channel-labels="url=https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
```

### PagerDuty Integration

1. Get your PagerDuty service key
2. Create the notification channel:

```bash
# Create a PagerDuty notification channel
gcloud alpha monitoring channels create \
  --display-name="Security Incidents PagerDuty" \
  --type="pagerduty" \
  --channel-labels="service_key=YOUR_PAGERDUTY_SERVICE_KEY"
```

### Channel Configuration Best Practices

1. **Severity-Based Routing**
   ```bash
   # Critical alerts to PagerDuty
   gcloud alpha monitoring policies create \
     --display-name="Critical Security Alerts" \
     --condition-filter='metric.type="custom.googleapis.com/lota_ai/api_auth_failures" AND metric.label.severity="critical"' \
     --notification-channels="projects/$PROJECT_ID/notificationChannels/$PAGERDUTY_CHANNEL_ID"

   # High severity to Slack
   gcloud alpha monitoring policies create \
     --display-name="High Severity Alerts" \
     --condition-filter='metric.type="custom.googleapis.com/lota_ai/api_errors_total" AND metric.label.severity="high"' \
     --notification-channels="projects/$PROJECT_ID/notificationChannels/$SLACK_CHANNEL_ID"
   ```

2. **Business Hours Configuration**
   ```bash
   # Create a notification channel for business hours
   gcloud alpha monitoring channels create \
     --display-name="Business Hours Email" \
     --type="email" \
     --channel-labels="email_address=ops@example.com" \
     --user-labels="hours=business"

   # Create a notification channel for after hours
   gcloud alpha monitoring channels create \
     --display-name="After Hours PagerDuty" \
     --type="pagerduty" \
     --channel-labels="service_key=YOUR_KEY" \
     --user-labels="hours=after"
   ```

## Dashboard Customization

### Advanced Chart Types

1. **Heatmap for Request Patterns**
```json
{
  "title": "API Request Patterns",
  "heatmapChart": {
    "dataSets": [{
      "timeSeriesQuery": {
        "timeSeriesFilter": {
          "filter": "metric.type=\"custom.googleapis.com/lota_ai/api_requests_total\"",
          "aggregation": {
            "alignmentPeriod": "60s",
            "crossSeriesReducer": "REDUCE_COUNT_TRUE",
            "groupByFields": ["metric.label.path", "metric.label.method"],
            "perSeriesAligner": "ALIGN_RATE"
          }
        }
      }
    }]
  }
}
```

2. **Stacked Bar Chart for Error Types**
```json
{
  "title": "Error Distribution by Type",
  "columnChart": {
    "dataSets": [{
      "timeSeriesQuery": {
        "timeSeriesFilter": {
          "filter": "metric.type=\"custom.googleapis.com/lota_ai/api_errors_total\"",
          "aggregation": {
            "alignmentPeriod": "300s",
            "crossSeriesReducer": "REDUCE_SUM",
            "groupByFields": ["metric.label.error_type"],
            "perSeriesAligner": "ALIGN_RATE"
          }
        }
      }
    }],
    "stacking": "STACKED"
  }
}
```

### Dashboard Organization

1. **Security Overview Dashboard**
```json
{
  "displayName": "Security Overview",
  "gridLayout": {
    "columns": "2",
    "widgets": [
      {
        "title": "Authentication Status",
        "scorecard": {
          "timeSeriesQuery": {
            "timeSeriesFilter": {
              "filter": "metric.type=\"custom.googleapis.com/lota_ai/api_auth_failures\"",
              "aggregation": {
                "alignmentPeriod": "300s",
                "perSeriesAligner": "ALIGN_RATE"
              }
            }
          },
          "thresholds": [
            { "value": 5, "color": "YELLOW" },
            { "value": 10, "color": "RED" }
          ]
        }
      },
      {
        "title": "Active Connections",
        "lineChart": {
          "dataSets": [{
            "timeSeriesQuery": {
              "timeSeriesFilter": {
                "filter": "metric.type=\"custom.googleapis.com/lota_ai/api_connections_active\"",
                "aggregation": {
                  "alignmentPeriod": "60s",
                  "perSeriesAligner": "ALIGN_MEAN"
                }
              }
            }
          }]
        }
      }
    ]
  }
}
```

2. **Security Alerts Dashboard**
```json
{
  "displayName": "Security Alerts",
  "gridLayout": {
    "columns": "2",
    "widgets": [
      {
        "title": "High Severity Incidents",
        "alertChart": {
          "alertPolicyFilter": "resource.type=\"global\" severity=\"critical\"",
          "aggregation": {
            "alignmentPeriod": "300s",
            "perSeriesAligner": "ALIGN_COUNT"
          }
        }
      }
    ]
  }
}
```

### Dashboard Best Practices

1. **Organization**
   - Group related metrics together
   - Use consistent naming conventions
   - Include both overview and detailed views
   - Add documentation links

2. **Visualization**
   - Choose appropriate chart types for different metrics
   - Use color coding consistently
   - Include thresholds and alerts
   - Add descriptive titles and legends

3. **Performance**
   - Optimize query periods
   - Use appropriate aggregation methods
   - Consider dashboard loading time
