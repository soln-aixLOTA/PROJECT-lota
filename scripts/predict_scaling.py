#!/usr/bin/env python3

import pandas as pd
import numpy as np
from prophet import Prophet
from datetime import datetime, timedelta
import json
import yaml

def load_historical_data():
    """Load historical traffic and resource usage data."""
    # In a real implementation, this would load data from your monitoring system
    # For demonstration, we'll generate sample data
    dates = pd.date_range(start='2024-01-01', end='2024-01-31', freq='H')
    data = {
        'ds': dates,
        'y': np.random.normal(100, 20, len(dates))  # Sample traffic data
    }
    return pd.DataFrame(data)

def train_model(df):
    """Train Prophet model for traffic prediction."""
    model = Prophet(
        yearly_seasonality=True,
        weekly_seasonality=True,
        daily_seasonality=True,
        changepoint_prior_scale=0.05
    )
    model.fit(df)
    return model

def generate_predictions(model, periods=24):
    """Generate predictions for the next periods."""
    future = model.make_future_dataframe(periods=periods, freq='H')
    forecast = model.predict(future)
    return forecast.tail(periods)

def calculate_resource_requirements(traffic):
    """Calculate required resources based on traffic predictions."""
    # Simple resource calculation - in reality, this would be more complex
    cpu_per_request = 0.001  # CPU cores per request
    memory_per_request = 10  # MB per request
    
    return {
        'cpu': traffic * cpu_per_request,
        'memory': traffic * memory_per_request,
    }

def generate_scaling_config(predictions):
    """Generate Kubernetes HPA configuration based on predictions."""
    max_predicted_traffic = predictions['yhat'].max()
    resources = calculate_resource_requirements(max_predicted_traffic)
    
    config = {
        'apiVersion': 'autoscaling/v2',
        'kind': 'HorizontalPodAutoscaler',
        'metadata': {
            'name': 'api-gateway',
            'namespace': 'default'
        },
        'spec': {
            'scaleTargetRef': {
                'apiVersion': 'apps/v1',
                'kind': 'Deployment',
                'name': 'api-gateway'
            },
            'minReplicas': 2,
            'maxReplicas': max(5, int(max_predicted_traffic / 1000)),
            'metrics': [
                {
                    'type': 'Resource',
                    'resource': {
                        'name': 'cpu',
                        'target': {
                            'type': 'Utilization',
                            'averageUtilization': 70
                        }
                    }
                }
            ],
            'behavior': {
                'scaleUp': {
                    'stabilizationWindowSeconds': 60,
                    'policies': [
                        {
                            'type': 'Pods',
                            'value': 4,
                            'periodSeconds': 60
                        }
                    ]
                },
                'scaleDown': {
                    'stabilizationWindowSeconds': 300,
                    'policies': [
                        {
                            'type': 'Pods',
                            'value': 1,
                            'periodSeconds': 60
                        }
                    ]
                }
            }
        }
    }
    
    return config

def save_configurations(config):
    """Save the generated configurations."""
    # Save HPA config
    with open('scaling-config.yml', 'w') as f:
        yaml.dump(config, f, default_flow_style=False)
    
    # Save predictions and recommendations
    summary = {
        'timestamp': datetime.now().isoformat(),
        'recommendations': {
            'min_replicas': config['spec']['minReplicas'],
            'max_replicas': config['spec']['maxReplicas'],
            'target_cpu_utilization': config['spec']['metrics'][0]['resource']['target']['averageUtilization']
        }
    }
    
    with open('scaling-recommendations.json', 'w') as f:
        json.dump(summary, f, indent=2)

def main():
    print("Loading historical data...")
    df = load_historical_data()
    
    print("Training prediction model...")
    model = train_model(df)
    
    print("Generating predictions...")
    predictions = generate_predictions(model)
    
    print("Generating scaling configuration...")
    config = generate_scaling_config(predictions)
    
    print("Saving configurations...")
    save_configurations(config)
    
    print("Analysis complete. Configuration files generated:")
    print("- scaling-config.yml")
    print("- scaling-recommendations.json")

if __name__ == '__main__':
    main() 