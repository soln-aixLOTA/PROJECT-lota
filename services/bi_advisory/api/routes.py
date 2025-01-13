from fastapi import APIRouter, Depends, HTTPException, Query, status
from fastapi.security import OAuth2PasswordRequestForm
from sqlalchemy.orm import Session
from typing import List, Optional
from datetime import datetime, timedelta

from . import schemas, database, analytics_service, auth
from .database import get_db
from .config import get_settings

# Initialize router
router = APIRouter()

# Initialize analytics service
analytics = analytics_service.AnalyticsService()

# Initialize settings
settings = get_settings()

# Authentication routes
@router.post("/api/v1/auth/token", response_model=schemas.Token, tags=["auth"])
async def login_for_access_token(
    form_data: OAuth2PasswordRequestForm = Depends(),
    db: Session = Depends(get_db)
):
    """
    OAuth2 compatible token login, get an access token for future requests
    """
    user = await auth.authenticate_user(db, form_data.username, form_data.password)
    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Incorrect username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )

    access_token_expires = timedelta(minutes=settings.ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = auth.create_access_token(
        data={"sub": str(user.id)},
        expires_delta=access_token_expires
    )

    return {
        "access_token": access_token,
        "token_type": "bearer"
    }

@router.post("/api/v1/auth/register", response_model=schemas.User, tags=["auth"])
async def register_user(
    user: schemas.UserCreate,
    db: Session = Depends(get_db)
):
    """
    Register a new user
    """
    db_user = await auth.get_user_by_email(db, email=user.email)
    if db_user:
        raise HTTPException(
            status_code=400,
            detail="Email already registered"
        )
    return auth.create_user(db=db, user=user)

# Analytics routes
@router.get("/api/v1/analytics/kpi/{timeframe}", response_model=dict[str, schemas.KPIInsight], tags=["analytics"])
async def get_kpi_insights(
    timeframe: str,
    refresh_cache: bool = False,
    db: Session = Depends(get_db),
    current_user: schemas.User = Depends(auth.get_current_active_user)
):
    """
    Get KPI insights for the specified timeframe.
    Timeframe can be: daily, weekly, monthly, quarterly
    """
    try:
        if timeframe not in ['daily', 'weekly', 'monthly', 'quarterly']:
            raise HTTPException(
                status_code=400,
                detail="Invalid timeframe. Must be one of: daily, weekly, monthly, quarterly"
            )

        return analytics.get_kpi_insights(timeframe, db, refresh_cache)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/api/v1/analytics/trends", response_model=List[schemas.TrendInsight], tags=["analytics"])
async def get_trend_insights(
    metric: str = Query(..., description="Metric to analyze (revenue, orders, customers)"),
    start_date: str = Query(..., description="Start date in YYYY-MM-DD format"),
    end_date: str = Query(..., description="End date in YYYY-MM-DD format"),
    refresh_cache: bool = False,
    db: Session = Depends(get_db),
    current_user: schemas.User = Depends(auth.get_current_active_user)
):
    """
    Get trend insights for specified metric and date range
    """
    try:
        # Validate metric
        if metric not in ['revenue', 'orders', 'customers']:
            raise HTTPException(
                status_code=400,
                detail="Invalid metric. Must be one of: revenue, orders, customers"
            )

        # Validate dates
        try:
            datetime.strptime(start_date, '%Y-%m-%d')
            datetime.strptime(end_date, '%Y-%m-%d')
        except ValueError:
            raise HTTPException(
                status_code=400,
                detail="Invalid date format. Use YYYY-MM-DD"
            )

        return analytics.get_trend_insights(metric, start_date, end_date, db, refresh_cache)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/api/v1/analytics/query", response_model=schemas.QueryResponse, tags=["analytics"])
async def process_query(
    request: schemas.QueryRequest,
    refresh_cache: bool = False,
    db: Session = Depends(get_db),
    current_user: schemas.User = Depends(auth.get_current_active_user)
):
    """
    Process natural language query and return insights
    """
    try:
        return analytics.process_natural_language_query(request.query, db, refresh_cache)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/api/v1/analytics/dashboards", response_model=List[schemas.Dashboard], tags=["analytics"])
async def get_dashboards(
    category: Optional[str] = None,
    db: Session = Depends(get_db),
    current_user: schemas.User = Depends(auth.get_current_active_user)
):
    """
    Get available dashboards, optionally filtered by category
    """
    try:
        return analytics.get_dashboards(category, db)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/health", tags=["health"])
async def health_check():
    """
    Health check endpoint
    """
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow()
    }
