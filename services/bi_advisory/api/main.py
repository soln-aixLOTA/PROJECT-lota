from fastapi import FastAPI, Request, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse, Response
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.openapi.utils import get_openapi
import logging
import time
import psutil
from typing import Callable
from datetime import datetime
from prometheus_client import generate_latest, CONTENT_TYPE_LATEST

from . import routes, database, models, config, metrics
from .database import engine

# Initialize logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Create database tables
models.Base.metadata.create_all(bind=engine)

# Initialize settings
settings = config.get_settings()

# Initialize FastAPI app
app = FastAPI(
    title=settings.PROJECT_NAME,
    description="""
    Business Intelligence and AI Advisory System API.

    Key Features:
    * 📊 KPI Analysis and Insights
    * 📈 Trend Analysis and Forecasting
    * 🤖 Natural Language Query Processing
    * 📱 Interactive Dashboards
    * 🔮 ML-Powered Predictions

    For authentication, use the /auth/token endpoint to obtain a JWT token.
    Include the token in the Authorization header as: Bearer <token>
    """,
    version=settings.VERSION,
    docs_url="/api/docs",
    redoc_url="/api/redoc",
    openapi_url="/api/openapi.json",
    openapi_tags=[
        {
            "name": "analytics",
            "description": "Analytics and insights operations",
        },
        {
            "name": "auth",
            "description": "Authentication operations",
        },
        {
            "name": "health",
            "description": "API health check operations",
        },
        {
            "name": "metrics",
            "description": "Prometheus metrics endpoints",
        }
    ]
)

# Configure CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.BACKEND_CORS_ORIGINS,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Add Gzip compression
app.add_middleware(GZipMiddleware, minimum_size=1000)

# Add metrics middleware
app.add_middleware(metrics.MetricsMiddleware)

# Custom middleware for request logging and timing
@app.middleware("http")
async def log_requests(request: Request, call_next: Callable):
    start_time = time.time()
    response = await call_next(request)
    duration = time.time() - start_time

    logger.info(
        f"Path: {request.url.path} "
        f"Method: {request.method} "
        f"Duration: {duration:.2f}s "
        f"Status: {response.status_code}"
    )

    return response

# Exception handler for custom error responses
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    error_id = str(int(time.time()))
    logger.error(f"Error ID: {error_id} - Global exception handler caught: {str(exc)}")

    if isinstance(exc, HTTPException):
        return JSONResponse(
            status_code=exc.status_code,
            content={
                "error_id": error_id,
                "detail": exc.detail,
                "timestamp": datetime.utcnow().isoformat(),
                "path": str(request.url)
            }
        )

    return JSONResponse(
        status_code=500,
        content={
            "error_id": error_id,
            "detail": "Internal server error",
            "timestamp": datetime.utcnow().isoformat(),
            "path": str(request.url)
        }
    )

# Include routers
app.include_router(routes.router)

# Metrics endpoint
@app.get("/metrics", tags=["metrics"])
async def get_metrics():
    """
    Get Prometheus metrics
    """
    # Update system metrics
    memory = psutil.virtual_memory()
    cpu = psutil.cpu_times_percent()

    metrics.update_system_metrics(
        memory_usage={
            "total": memory.total,
            "available": memory.available,
            "used": memory.used,
            "cached": memory.cached if hasattr(memory, 'cached') else 0
        },
        cpu_usage={
            "user": cpu.user,
            "system": cpu.system,
            "idle": cpu.idle
        }
    )

    return Response(
        content=generate_latest(metrics.REGISTRY),
        media_type=CONTENT_TYPE_LATEST
    )

# Root endpoint
@app.get("/", tags=["health"])
async def root():
    """
    Root endpoint providing basic API information and health status.

    Returns:
        dict: Basic information about the API including name, version, and operational status
    """
    return {
        "name": settings.PROJECT_NAME,
        "version": settings.VERSION,
        "status": "operational",
        "timestamp": datetime.utcnow()
    }

# Startup event
@app.on_event("startup")
async def startup_event():
    """Initialize required resources on application startup"""
    logger.info(f"Starting up {settings.PROJECT_NAME}")
    # Initialize any required resources here

# Shutdown event
@app.on_event("shutdown")
async def shutdown_event():
    """Clean up resources on application shutdown"""
    logger.info(f"Shutting down {settings.PROJECT_NAME}")
    # Clean up any resources here

def custom_openapi():
    """Generate custom OpenAPI schema with additional information"""
    if app.openapi_schema:
        return app.openapi_schema

    openapi_schema = get_openapi(
        title=settings.PROJECT_NAME,
        version=settings.VERSION,
        description=app.description,
        routes=app.routes,
    )

    # Add API logo
    openapi_schema["info"]["x-logo"] = {
        "url": "https://fastapi.tiangolo.com/img/logo-margin/logo-teal.png"
    }

    # Add security schemes
    openapi_schema["components"]["securitySchemes"] = {
        "bearerAuth": {
            "type": "http",
            "scheme": "bearer",
            "bearerFormat": "JWT",
        }
    }

    # Apply security globally
    openapi_schema["security"] = [{"bearerAuth": []}]

    app.openapi_schema = openapi_schema
    return app.openapi_schema

app.openapi = custom_openapi
