from pydantic import BaseModel, EmailStr, Field
from typing import List, Optional, Dict, Any
from datetime import datetime

# User schemas
class UserBase(BaseModel):
    username: str
    email: EmailStr

class UserCreate(UserBase):
    password: str

class User(UserBase):
    id: int
    is_active: bool
    is_superuser: bool
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

# Auth schemas
class Token(BaseModel):
    access_token: str
    token_type: str

class TokenData(BaseModel):
    user_id: Optional[str] = None

# Base schemas
class CustomerBase(BaseModel):
    name: str
    email: EmailStr
    segment: Optional[str] = None

class CustomerCreate(CustomerBase):
    external_id: str

class Customer(CustomerBase):
    id: int
    external_id: str
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

# Insight schemas
class KPIInsight(BaseModel):
    metric: str
    value: float
    trend: str
    analysis: str
    confidence: float
    timestamp: datetime

class TrendInsight(BaseModel):
    metric: str
    values: List[float]
    timestamps: List[datetime]
    trend_type: str
    analysis: str
    confidence: float

# Prediction schemas
class SalesPredictionRequest(BaseModel):
    product_ids: Optional[List[int]] = None
    customer_segments: Optional[List[str]] = None
    start_date: datetime
    end_date: datetime
    granularity: str = Field(..., description="daily, weekly, or monthly")

class SalesPrediction(BaseModel):
    predictions: List[float]
    timestamps: List[datetime]
    confidence_intervals: List[Dict[str, float]]
    factors: List[Dict[str, float]]
    accuracy_metrics: Dict[str, float]

class ChurnPredictionRequest(BaseModel):
    customer_ids: List[int]
    prediction_window: int = Field(..., description="Number of days to predict ahead")

class ChurnPrediction(BaseModel):
    predictions: Dict[int, float]  # customer_id -> churn probability
    risk_factors: Dict[int, List[Dict[str, Any]]]  # customer_id -> list of risk factors
    recommended_actions: Dict[int, List[str]]  # customer_id -> list of actions

# Recommendation schemas
class StrategyRequest(BaseModel):
    context: Dict[str, Any]
    objective: str
    constraints: Optional[Dict[str, Any]] = None
    timeframe: str

class StrategyRecommendation(BaseModel):
    recommendations: List[Dict[str, Any]]
    impact_analysis: Dict[str, float]
    implementation_steps: List[str]
    risks: List[Dict[str, Any]]
    confidence: float

# Query schemas
class QueryRequest(BaseModel):
    query: str
    context: Optional[Dict[str, Any]] = None
    filters: Optional[Dict[str, Any]] = None

class QueryResponse(BaseModel):
    answer: str
    data: Optional[Dict[str, Any]] = None
    visualizations: Optional[List[Dict[str, Any]]] = None
    confidence: float
    sources: List[str]

# Dashboard schemas
class Dashboard(BaseModel):
    id: int
    name: str
    category: str
    config: Dict[str, Any]
    is_active: bool
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

# ML Model schemas
class MLModelMetrics(BaseModel):
    accuracy: float
    precision: float
    recall: float
    f1_score: float
    training_date: datetime
    dataset_size: int

class MLModel(BaseModel):
    id: int
    name: str
    type: str
    version: str
    metrics: MLModelMetrics
    is_active: bool
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

# Campaign schemas
class CampaignBase(BaseModel):
    name: str
    type: str
    start_date: datetime
    end_date: datetime
    budget: float
    status: str

class CampaignCreate(CampaignBase):
    pass

class Campaign(CampaignBase):
    id: int
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

# Interaction schemas
class InteractionBase(BaseModel):
    customer_id: int
    channel: str
    type: str
    details: Dict[str, Any]

class InteractionCreate(InteractionBase):
    pass

class Interaction(InteractionBase):
    id: int
    created_at: datetime

    class Config:
        from_attributes = True

# Product schemas
class ProductBase(BaseModel):
    name: str
    category: str
    price: float
    cost: float
    inventory_level: int

class ProductCreate(ProductBase):
    pass

class Product(ProductBase):
    id: int
    created_at: datetime
    updated_at: datetime

    class Config:
        from_attributes = True

# Order schemas
class OrderItemBase(BaseModel):
    product_id: int
    quantity: int
    unit_price: float

class OrderItemCreate(OrderItemBase):
    pass

class OrderItem(OrderItemBase):
    id: int
    order_id: int
    created_at: datetime

    class Config:
        from_attributes = True

class OrderBase(BaseModel):
    customer_id: int
    order_date: datetime
    total_amount: float
    status: str

class OrderCreate(OrderBase):
    items: List[OrderItemCreate]

class Order(OrderBase):
    id: int
    created_at: datetime
    items: List[OrderItem]

    class Config:
        from_attributes = True
