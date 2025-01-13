from prometheus_client import Counter, Histogram, Gauge, Summary, CollectorRegistry
import time
import logging
from typing import Optional
from contextlib import contextmanager

# Initialize logging
logger = logging.getLogger(__name__)

# Create custom registry
REGISTRY = CollectorRegistry()

# Request metrics
REQUEST_COUNT = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status'],
    registry=REGISTRY
)

REQUEST_LATENCY = Histogram(
    'http_request_duration_seconds',
    'HTTP request duration in seconds',
    ['method', 'endpoint'],
    buckets=(0.1, 0.5, 1.0, 2.0, 5.0, 10.0),
    registry=REGISTRY
)

ACTIVE_REQUESTS = Gauge(
    'http_requests_active',
    'Number of active HTTP requests',
    ['method', 'endpoint'],
    registry=REGISTRY
)

# Database metrics
DB_POOL_CONNECTIONS = Gauge(
    'db_pool_connections',
    'Database connection pool status',
    ['state'],  # active, idle, total
    registry=REGISTRY
)

DB_QUERY_LATENCY = Histogram(
    'db_query_duration_seconds',
    'Database query duration in seconds',
    ['operation', 'table'],
    buckets=(0.01, 0.05, 0.1, 0.5, 1.0, 5.0),
    registry=REGISTRY
)

DB_ERRORS = Counter(
    'db_errors_total',
    'Total database errors',
    ['operation', 'table'],
    registry=REGISTRY
)

# Cache metrics
CACHE_OPERATIONS = Counter(
    'cache_operations_total',
    'Total cache operations',
    ['cache_type', 'operation', 'hit'],
    registry=REGISTRY
)

CACHE_SIZE = Gauge(
    'cache_size_bytes',
    'Cache size in bytes',
    ['cache_type'],
    registry=REGISTRY
)

# ML metrics
MODEL_LOAD_COUNT = Counter(
    'ml_model_loads_total',
    'Total number of model loads',
    ['model_name'],
    registry=REGISTRY
)

MODEL_LOAD_ERRORS = Counter(
    'ml_model_load_errors_total',
    'Total number of model load errors',
    registry=REGISTRY
)

PREDICTION_LATENCY = Histogram(
    'ml_prediction_duration_seconds',
    'ML prediction duration in seconds',
    ['model_name', 'operation'],
    buckets=(0.1, 0.5, 1.0, 2.0, 5.0, 10.0),
    registry=REGISTRY
)

PREDICTION_ERRORS = Counter(
    'ml_prediction_errors_total',
    'Total number of prediction errors',
    ['model_name'],
    registry=REGISTRY
)

PREDICTION_ACCURACY = Gauge(
    'ml_prediction_accuracy',
    'ML model prediction accuracy',
    ['model_name', 'metric'],
    registry=REGISTRY
)

CHURN_PROBABILITY = Gauge(
    'customer_churn_probability',
    'Customer churn probability',
    ['customer_id'],
    registry=REGISTRY
)

RECOMMENDATION_ERRORS = Counter(
    'recommendation_errors_total',
    'Total number of recommendation generation errors',
    registry=REGISTRY
)

# Business metrics
KPI_VALUES = Gauge(
    'kpi_value',
    'Current KPI values',
    ['kpi_name', 'timeframe'],
    registry=REGISTRY
)

INSIGHT_GENERATION_LATENCY = Histogram(
    'insight_generation_duration_seconds',
    'Insight generation duration in seconds',
    ['insight_type'],
    buckets=(0.1, 0.5, 1.0, 2.0, 5.0, 10.0),
    registry=REGISTRY
)

# System metrics
SYSTEM_MEMORY_USAGE = Gauge(
    'system_memory_bytes',
    'System memory usage in bytes',
    ['type'],  # used, free, cached
    registry=REGISTRY
)

SYSTEM_CPU_USAGE = Gauge(
    'system_cpu_percent',
    'System CPU usage percentage',
    ['type'],  # user, system, idle
    registry=REGISTRY
)

@contextmanager
def DatabaseMetrics(operation: str, table: str):
    """Context manager for tracking database operations"""
    start_time = time.time()
    try:
        yield
    except Exception as e:
        DB_ERRORS.labels(operation=operation, table=table).inc()
        raise
    finally:
        duration = time.time() - start_time
        DB_QUERY_LATENCY.labels(
            operation=operation,
            table=table
        ).observe(duration)

@contextmanager
def MLMetrics(model_name: Optional[str] = None, operation: Optional[str] = None):
    """Context manager for tracking ML operations"""
    start_time = time.time()
    try:
        yield
    except Exception as e:
        if model_name:
            PREDICTION_ERRORS.labels(model_name=model_name).inc()
        raise
    finally:
        if model_name and operation:
            duration = time.time() - start_time
            PREDICTION_LATENCY.labels(
                model_name=model_name,
                operation=operation
            ).observe(duration)

def track_cache_operation(cache_type: str, hit: bool):
    """Track cache operation"""
    CACHE_OPERATIONS.labels(
        cache_type=cache_type,
        operation='get',
        hit=str(hit).lower()
    ).inc()

def update_system_metrics():
    """Update system metrics"""
    try:
        import psutil

        # Memory metrics
        memory = psutil.virtual_memory()
        SYSTEM_MEMORY_USAGE.labels(type='used').set(memory.used)
        SYSTEM_MEMORY_USAGE.labels(type='free').set(memory.free)
        SYSTEM_MEMORY_USAGE.labels(type='cached').set(memory.cached)

        # CPU metrics
        cpu_times_percent = psutil.cpu_times_percent()
        SYSTEM_CPU_USAGE.labels(type='user').set(cpu_times_percent.user)
        SYSTEM_CPU_USAGE.labels(type='system').set(cpu_times_percent.system)
        SYSTEM_CPU_USAGE.labels(type='idle').set(cpu_times_percent.idle)

    except Exception as e:
        logger.error(f"Error updating system metrics: {str(e)}")

def update_db_pool_metrics(active: int, idle: int, total: int):
    """Update database connection pool metrics"""
    DB_POOL_CONNECTIONS.labels(state='active').set(active)
    DB_POOL_CONNECTIONS.labels(state='idle').set(idle)
    DB_POOL_CONNECTIONS.labels(state='total').set(total)

def update_cache_size(cache_type: str, size_bytes: int):
    """Update cache size metric"""
    CACHE_SIZE.labels(cache_type=cache_type).set(size_bytes)
