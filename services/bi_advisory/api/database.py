from sqlalchemy import create_engine
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.orm import sessionmaker
from sqlalchemy.engine import Engine
from sqlalchemy import event
import time
import logging
import contextlib
from typing import Generator

from . import config, metrics

# Initialize logging
logger = logging.getLogger(__name__)

# Get settings
settings = config.get_settings()

# Create SQLAlchemy engine
engine = create_engine(
    settings.DATABASE_URL,
    pool_size=settings.DB_POOL_SIZE,
    max_overflow=settings.DB_MAX_OVERFLOW,
    pool_timeout=settings.DB_POOL_TIMEOUT,
    pool_pre_ping=True
)

# Create session factory
SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)

# Create base class for declarative models
Base = declarative_base()

# Track database connection pool metrics
@event.listens_for(engine, "checkin")
def on_connection_checkin(dbapi_connection, connection_record):
    """Track connection pool metrics on connection check-in"""
    metrics.DB_CONNECTION_POOL.labels(status="idle").inc()
    metrics.DB_CONNECTION_POOL.labels(status="used").dec()

@event.listens_for(engine, "checkout")
def on_connection_checkout(dbapi_connection, connection_record, connection_proxy):
    """Track connection pool metrics on connection check-out"""
    metrics.DB_CONNECTION_POOL.labels(status="idle").dec()
    metrics.DB_CONNECTION_POOL.labels(status="used").inc()

@event.listens_for(engine, "connect")
def on_connection_connect(dbapi_connection, connection_record):
    """Track connection pool metrics on new connection"""
    metrics.DB_CONNECTION_POOL.labels(status="total").inc()

@event.listens_for(engine, "close")
def on_connection_close(dbapi_connection, connection_record):
    """Track connection pool metrics on connection close"""
    metrics.DB_CONNECTION_POOL.labels(status="total").dec()

# Track query execution time
@event.listens_for(Engine, "before_cursor_execute")
def before_cursor_execute(conn, cursor, statement, parameters, context, executemany):
    """Store start time for query execution"""
    conn.info.setdefault('query_start_time', []).append(time.time())

@event.listens_for(Engine, "after_cursor_execute")
def after_cursor_execute(conn, cursor, statement, parameters, context, executemany):
    """Calculate query execution time and update metrics"""
    total_time = time.time() - conn.info['query_start_time'].pop()

    # Extract operation type and table name from statement
    operation = statement.split()[0].lower()
    table = None

    if "FROM" in statement.upper():
        table = statement.upper().split("FROM")[1].strip().split()[0]
    elif "INTO" in statement.upper():
        table = statement.upper().split("INTO")[1].strip().split()[0]
    elif "UPDATE" in statement.upper():
        table = statement.upper().split("UPDATE")[1].strip().split()[0]
    elif "DELETE" in statement.upper():
        table = statement.upper().split("FROM")[1].strip().split()[0]

    if table:
        table = table.strip('"').strip('`').lower()
        metrics.DB_QUERY_LATENCY.labels(
            operation=operation,
            table=table
        ).observe(total_time)

@contextlib.contextmanager
def get_db() -> Generator:
    """Get database session with metrics tracking"""
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

def init_db() -> None:
    """Initialize database"""
    try:
        # Create all tables
        Base.metadata.create_all(bind=engine)
        logger.info("Database tables created successfully")

        # Initialize connection pool metrics
        metrics.DB_CONNECTION_POOL.labels(status="total").set(0)
        metrics.DB_CONNECTION_POOL.labels(status="idle").set(settings.DB_POOL_SIZE)
        metrics.DB_CONNECTION_POOL.labels(status="used").set(0)
        metrics.DB_CONNECTION_POOL.labels(status="overflow").set(0)

    except Exception as e:
        logger.error(f"Error initializing database: {str(e)}")
        raise

def check_db_connection() -> bool:
    """
    Check if database connection is working.
    Returns True if connection is successful, False otherwise.
    """
    try:
        db = SessionLocal()
        db.execute("SELECT 1")
        db.close()
        logger.info("Successfully checked database connection")
        return True
    except Exception as e:
        logger.error(f"Error checking database connection: {str(e)}")
        return False
