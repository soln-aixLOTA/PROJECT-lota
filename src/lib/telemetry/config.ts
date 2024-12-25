import { diag, DiagConsoleLogger, DiagLogLevel } from '@opentelemetry/api';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';

// Enable OpenTelemetry debug logging
diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.INFO);

// Create a resource that identifies your application
export const resource = new Resource({
    [SemanticResourceAttributes.SERVICE_NAME]: 'genai-web-app',
    [SemanticResourceAttributes.SERVICE_VERSION]: '1.0.0',
    [SemanticResourceAttributes.DEPLOYMENT_ENVIRONMENT]: process.env.NODE_ENV || 'development',
});

// Configuration for metrics export (can be customized based on your needs)
export const metricExportConfig = {
    exportIntervalMillis: 10000, // Export metrics every 10 seconds
};

// Configuration for tracing (can be customized based on your needs)
export const tracingConfig = {
    sampler: {
        type: 'always_on' as const,
    },
}; 