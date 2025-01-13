from sqlalchemy.orm import Session
import logging
from typing import Dict, List, Optional, Tuple
from datetime import datetime, timedelta
import numpy as np
import pandas as pd
import mlflow
from mlflow.tracking import MlflowClient
from . import models, schemas, metrics

# Initialize logging
logger = logging.getLogger(__name__)

class MLService:
    """Service for ML model predictions and recommendations"""

    def __init__(self):
        """Initialize ML service with model registry"""
        self.client = MlflowClient()
        self.model_registry = {}
        self._cache = {}

        # Load models
        self._load_models()

    def _load_models(self):
        """Load ML models from registry"""
        try:
            with metrics.MLMetrics(operation="load_models"):
                # Load sales prediction model
                self.model_registry['sales'] = mlflow.sklearn.load_model(
                    'models:/sales_prediction/Production'
                )
                metrics.MODEL_LOAD_COUNT.labels(model_name="sales").inc()

                # Load churn prediction model
                self.model_registry['churn'] = mlflow.sklearn.load_model(
                    'models:/churn_prediction/Production'
                )
                metrics.MODEL_LOAD_COUNT.labels(model_name="churn").inc()

                logger.info("ML models loaded successfully")

        except Exception as e:
            logger.error(f"Error loading ML models: {str(e)}")
            metrics.MODEL_LOAD_ERRORS.inc()
            raise

    def predict_sales(
        self,
        product_ids: Optional[List[int]] = None,
        customer_segments: Optional[List[str]] = None,
        prediction_days: int = 30,
        granularity: str = 'daily',
        db: Session = None,
        refresh_cache: bool = False
    ) -> schemas.SalesPrediction:
        """Generate sales predictions"""
        cache_key = f"sales_pred_{product_ids}_{customer_segments}_{prediction_days}_{granularity}"

        # Check cache
        if not refresh_cache:
            cached = self._cache.get(cache_key)
            if cached:
                metrics.track_cache_operation("memory", hit=True)
                return cached
            metrics.track_cache_operation("memory", hit=False)

        try:
            with metrics.MLMetrics(model_name="sales", operation="predict"):
                # Prepare features
                features = self._prepare_sales_features(
                    product_ids,
                    customer_segments,
                    prediction_days,
                    granularity,
                    db
                )

                # Generate predictions
                predictions = self.model_registry['sales'].predict(features)
                confidence_intervals = self._calculate_confidence_intervals(predictions)

                # Analyze contributing factors
                factors = self._analyze_contributing_factors(features, predictions)

                # Calculate metrics
                metrics_dict = {
                    'mape': self._calculate_mape(predictions),
                    'rmse': self._calculate_rmse(predictions),
                    'r2': self._calculate_r2(predictions)
                }

                # Track prediction metrics
                metrics.PREDICTION_ACCURACY.labels(
                    model_name="sales",
                    metric="mape"
                ).set(metrics_dict['mape'])

                # Generate timestamps
                timestamps = self._generate_prediction_timestamps(
                    prediction_days,
                    granularity
                )

                # Create response
                response = schemas.SalesPrediction(
                    predictions=predictions.tolist(),
                    timestamps=timestamps,
                    confidence_intervals=confidence_intervals,
                    contributing_factors=factors,
                    metrics=metrics_dict
                )

                # Cache response
                self._cache[cache_key] = response
                return response

        except Exception as e:
            logger.error(f"Error generating sales predictions: {str(e)}")
            metrics.PREDICTION_ERRORS.labels(model_name="sales").inc()
            raise

    def predict_churn(
        self,
        customer_id: int,
        db: Session,
        refresh_cache: bool = False
    ) -> schemas.ChurnPrediction:
        """Generate churn prediction for customer"""
        cache_key = f"churn_pred_{customer_id}"

        # Check cache
        if not refresh_cache:
            cached = self._cache.get(cache_key)
            if cached:
                metrics.track_cache_operation("memory", hit=True)
                return cached
            metrics.track_cache_operation("memory", hit=False)

        try:
            with metrics.MLMetrics(model_name="churn", operation="predict"):
                # Prepare features
                features = self._prepare_churn_features(customer_id, db)

                # Generate prediction
                churn_prob = float(self.model_registry['churn'].predict_proba(features)[0][1])

                # Analyze risk factors
                risk_factors = self._analyze_churn_risk_factors(features)

                # Generate recommendations
                recommendations = self._generate_churn_recommendations(risk_factors)

                # Track prediction
                metrics.CHURN_PROBABILITY.labels(
                    customer_id=str(customer_id)
                ).set(churn_prob)

                # Create response
                response = schemas.ChurnPrediction(
                    customer_id=customer_id,
                    churn_probability=churn_prob,
                    risk_factors=risk_factors,
                    recommendations=recommendations,
                    timestamp=datetime.utcnow()
                )

                # Cache response
                self._cache[cache_key] = response
                return response

        except Exception as e:
            logger.error(f"Error generating churn prediction: {str(e)}")
            metrics.PREDICTION_ERRORS.labels(model_name="churn").inc()
            raise

    def get_strategy_recommendations(
        self,
        context: Dict,
        objectives: List[str],
        db: Session,
        refresh_cache: bool = False
    ) -> schemas.StrategyRecommendation:
        """Generate strategic recommendations"""
        cache_key = f"strategy_rec_{hash(str(context))}_{hash(str(objectives))}"

        # Check cache
        if not refresh_cache:
            cached = self._cache.get(cache_key)
            if cached:
                metrics.track_cache_operation("memory", hit=True)
                return cached
            metrics.track_cache_operation("memory", hit=False)

        try:
            with metrics.MLMetrics(model_name="strategy", operation="recommend"):
                # Analyze business metrics
                business_metrics = self._analyze_business_metrics(context, db)

                # Generate recommendations
                recommendations = []
                for objective in objectives:
                    rec = self._generate_recommendation(objective, business_metrics)
                    recommendations.append(rec)

                # Create response
                response = schemas.StrategyRecommendation(
                    context=context,
                    objectives=objectives,
                    recommendations=recommendations,
                    business_metrics=business_metrics,
                    timestamp=datetime.utcnow()
                )

                # Cache response
                self._cache[cache_key] = response
                return response

        except Exception as e:
            logger.error(f"Error generating strategy recommendations: {str(e)}")
            metrics.RECOMMENDATION_ERRORS.inc()
            raise

    def _prepare_sales_features(
        self,
        product_ids: Optional[List[int]],
        customer_segments: Optional[List[str]],
        prediction_days: int,
        granularity: str,
        db: Session
    ) -> pd.DataFrame:
        """Prepare features for sales prediction"""
        try:
            with metrics.DatabaseMetrics(operation="select", table="orders"):
                # Implementation details...
                pass

        except Exception as e:
            logger.error(f"Error preparing sales features: {str(e)}")
            raise

    def _prepare_churn_features(
        self,
        customer_id: int,
        db: Session
    ) -> pd.DataFrame:
        """Prepare features for churn prediction"""
        try:
            with metrics.DatabaseMetrics(operation="select", table="customers"):
                # Implementation details...
                pass

        except Exception as e:
            logger.error(f"Error preparing churn features: {str(e)}")
            raise

    def _calculate_confidence_intervals(
        self,
        predictions: np.ndarray,
        confidence: float = 0.95
    ) -> List[Tuple[float, float]]:
        """Calculate confidence intervals for predictions"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error calculating confidence intervals: {str(e)}")
            raise

    def _analyze_contributing_factors(
        self,
        features: pd.DataFrame,
        predictions: np.ndarray
    ) -> List[Dict]:
        """Analyze factors contributing to predictions"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error analyzing contributing factors: {str(e)}")
            raise

    def _analyze_churn_risk_factors(
        self,
        features: pd.DataFrame
    ) -> List[Dict]:
        """Analyze risk factors for churn"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error analyzing churn risk factors: {str(e)}")
            raise

    def _generate_churn_recommendations(
        self,
        risk_factors: List[Dict]
    ) -> List[Dict]:
        """Generate recommendations for reducing churn risk"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error generating churn recommendations: {str(e)}")
            raise

    def _analyze_business_metrics(
        self,
        context: Dict,
        db: Session
    ) -> Dict:
        """Analyze business metrics for strategy recommendations"""
        try:
            with metrics.DatabaseMetrics(operation="select", table="multiple"):
                # Implementation details...
                pass

        except Exception as e:
            logger.error(f"Error analyzing business metrics: {str(e)}")
            raise

    def _generate_recommendation(
        self,
        objective: str,
        business_metrics: Dict
    ) -> Dict:
        """Generate recommendation for specific objective"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error generating recommendation: {str(e)}")
            raise

    def _calculate_mape(self, predictions: np.ndarray) -> float:
        """Calculate Mean Absolute Percentage Error"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error calculating MAPE: {str(e)}")
            raise

    def _calculate_rmse(self, predictions: np.ndarray) -> float:
        """Calculate Root Mean Square Error"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error calculating RMSE: {str(e)}")
            raise

    def _calculate_r2(self, predictions: np.ndarray) -> float:
        """Calculate R-squared score"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error calculating R2: {str(e)}")
            raise

    def _generate_prediction_timestamps(
        self,
        prediction_days: int,
        granularity: str
    ) -> List[str]:
        """Generate timestamps for predictions"""
        try:
            # Implementation details...
            pass

        except Exception as e:
            logger.error(f"Error generating prediction timestamps: {str(e)}")
            raise

# Create global ML service instance
ml_service = MLService()

# Module-level functions that use the global ML service instance
def predict_sales(
    product_ids: Optional[List[int]] = None,
    customer_segments: Optional[List[str]] = None,
    prediction_days: int = 30,
    granularity: str = 'daily',
    db: Session = None
) -> schemas.SalesPrediction:
    return ml_service.predict_sales(
        product_ids,
        customer_segments,
        prediction_days,
        granularity,
        db
    )

def predict_churn(
    customer_id: int,
    db: Session
) -> schemas.ChurnPrediction:
    return ml_service.predict_churn(customer_id, db)

def get_strategy_recommendations(
    context: Dict,
    objectives: List[str],
    db: Session
) -> schemas.StrategyRecommendation:
    return ml_service.get_strategy_recommendations(context, objectives, db)
