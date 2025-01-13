from sqlalchemy import Boolean, Column, Integer, String, DateTime, Float, ForeignKey, JSON
from sqlalchemy.orm import relationship
from datetime import datetime

from .database import Base

class User(Base):
    """User model for authentication"""
    __tablename__ = "users"

    id = Column(Integer, primary_key=True, index=True)
    username = Column(String, unique=True, index=True)
    email = Column(String, unique=True, index=True)
    hashed_password = Column(String)
    is_active = Column(Boolean, default=True)
    is_superuser = Column(Boolean, default=False)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

class Customer(Base):
    """Customer model"""
    __tablename__ = "customers"

    id = Column(Integer, primary_key=True, index=True)
    external_id = Column(String, unique=True, index=True)
    name = Column(String)
    email = Column(String, unique=True, index=True)
    segment = Column(String)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    orders = relationship("Order", back_populates="customer")
    interactions = relationship("Interaction", back_populates="customer")

class Order(Base):
    """Order model"""
    __tablename__ = "orders"

    id = Column(Integer, primary_key=True, index=True)
    customer_id = Column(Integer, ForeignKey("customers.id"))
    order_date = Column(DateTime)
    total_amount = Column(Float)
    status = Column(String)
    created_at = Column(DateTime, default=datetime.utcnow)

    customer = relationship("Customer", back_populates="orders")
    items = relationship("OrderItem", back_populates="order")

class Product(Base):
    """Product model"""
    __tablename__ = "products"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    category = Column(String)
    price = Column(Float)
    cost = Column(Float)
    inventory_level = Column(Integer)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    order_items = relationship("OrderItem", back_populates="product")

class OrderItem(Base):
    """Order item model"""
    __tablename__ = "order_items"

    id = Column(Integer, primary_key=True, index=True)
    order_id = Column(Integer, ForeignKey("orders.id"))
    product_id = Column(Integer, ForeignKey("products.id"))
    quantity = Column(Integer)
    unit_price = Column(Float)
    created_at = Column(DateTime, default=datetime.utcnow)

    order = relationship("Order", back_populates="items")
    product = relationship("Product", back_populates="order_items")

class Interaction(Base):
    """Customer interaction model"""
    __tablename__ = "interactions"

    id = Column(Integer, primary_key=True, index=True)
    customer_id = Column(Integer, ForeignKey("customers.id"))
    channel = Column(String)
    type = Column(String)
    details = Column(JSON)
    created_at = Column(DateTime, default=datetime.utcnow)

    customer = relationship("Customer", back_populates="interactions")
    campaign = relationship("Campaign", back_populates="interactions")

class Campaign(Base):
    """Marketing campaign model"""
    __tablename__ = "campaigns"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    type = Column(String)
    start_date = Column(DateTime)
    end_date = Column(DateTime)
    budget = Column(Float)
    status = Column(String)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    interactions = relationship("Interaction", back_populates="campaign")

class MLModel(Base):
    """Machine learning model metadata"""
    __tablename__ = "ml_models"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String, unique=True, index=True)
    type = Column(String)
    version = Column(String)
    metrics = Column(JSON)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

class Dashboard(Base):
    """Dashboard configuration"""
    __tablename__ = "dashboards"

    id = Column(Integer, primary_key=True, index=True)
    name = Column(String)
    category = Column(String)
    config = Column(JSON)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=datetime.utcnow)
    updated_at = Column(DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

class Insight(Base):
    """Stored insights"""
    __tablename__ = "insights"

    id = Column(Integer, primary_key=True, index=True)
    type = Column(String)
    metric = Column(String)
    value = Column(Float)
    trend = Column(String)
    analysis = Column(String)
    confidence = Column(Float)
    created_at = Column(DateTime, default=datetime.utcnow)

class Recommendation(Base):
    """Business recommendations"""
    __tablename__ = "recommendations"

    id = Column(Integer, primary_key=True, index=True)
    type = Column(String)
    context = Column(JSON)
    recommendation = Column(String)
    impact_score = Column(Float)
    confidence = Column(Float)
    status = Column(String)
    created_at = Column(DateTime, default=datetime.utcnow)
    implemented_at = Column(DateTime)
