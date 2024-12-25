import { ConsoleMetricExporter, MeterProvider, PeriodicExportingMetricReader } from '@opentelemetry/sdk-metrics';
import { BatchSpanProcessor, ConsoleSpanExporter } from '@opentelemetry/sdk-trace-base';
import { WebTracerProvider } from '@opentelemetry/sdk-trace-web';
import { metricExportConfig, resource, tracingConfig } from './config';

let isInitialized = false;

export function initTelemetry() {
    if (isInitialized) {
        return;
    }

    // Initialize tracer
    const tracerProvider = new WebTracerProvider({
        resource: resource,
        sampler: tracingConfig.sampler,
    });

    // Use console exporter for development
    // In production, you would use a proper exporter (e.g., Jaeger, Zipkin)
    tracerProvider.addSpanProcessor(
        new BatchSpanProcessor(new ConsoleSpanExporter())
    );

    // Register the tracer
    tracerProvider.register();

    // Initialize metrics
    const metricReader = new PeriodicExportingMetricReader({
        exporter: new ConsoleMetricExporter(),
        exportIntervalMillis: metricExportConfig.exportIntervalMillis,
    });

    const meterProvider = new MeterProvider({
        resource: resource,
    });

    meterProvider.addMetricReader(metricReader);

    // Register the meter provider
    meterProvider.register();

    isInitialized = true;
}

// Initialize telemetry if we're in a browser environment
if (typeof window !== 'undefined') {
    initTelemetry();
} 