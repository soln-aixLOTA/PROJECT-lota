from sqlalchemy.orm import Session
import logging
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
import json
from functools import lru_cache
import numpy as np
from sqlalchemy import func, text

from . import models, schemas, metrics

# Initialize logging
logger = logging.getLogger(__name__)

class AnalyticsService:
    """Service for analytics and insights generation"""

    def __init__(self):
        self._cache = {}

    def get_kpi_insights(
        self,
        timeframe: str,
        db: Session,
        refresh_cache: bool = False
    ) -> Dict[str, schemas.KPIInsight]:
        """Get KPI insights for specified timeframe"""
        cache_key = f"kpi_insights_{timeframe}"

        # Check cache if refresh not requested
        if not refresh_cache:
            cached = self._cache.get(cache_key)
            if cached:
                metrics.track_cache_operation("memory", hit=True)
                return cached
            metrics.track_cache_operation("memory", hit=False)

        try:
            with metrics.DatabaseMetrics(operation="select", table="multiple"):
                insights = {}

                # Revenue KPIs
                with metrics.INSIGHT_GENERATION_LATENCY.labels(insight_type="revenue").time():
                    revenue_data = self._calculate_revenue_kpi(timeframe, db)
                    insights["revenue"] = revenue_data
                    metrics.KPI_VALUES.labels(
                        kpi_name="revenue",
                        timeframe=timeframe
                    ).set(revenue_data.value)

                # Customer KPIs
                with metrics.INSIGHT_GENERATION_LATENCY.labels(insight_type="customer").time():
                    customer_data = self._calculate_customer_kpi(timeframe, db)
                    insights["customer"] = customer_data
                    metrics.KPI_VALUES.labels(
                        kpi_name="customer",
                        timeframe=timeframe
                    ).set(customer_data.value)

                # Product KPIs
                with metrics.INSIGHT_GENERATION_LATENCY.labels(insight_type="product").time():
                    product_data = self._calculate_product_kpi(timeframe, db)
                    insights["product"] = product_data
                    metrics.KPI_VALUES.labels(
                        kpi_name="product",
                        timeframe=timeframe
                    ).set(product_data.value)

                # Cache results
                self._cache[cache_key] = insights
                return insights

        except Exception as e:
            logger.error(f"Error getting KPI insights: {str(e)}")
            raise

    def get_trend_insights(
        self,
        metric: str,
        start_date: str,
        end_date: str,
        db: Session,
        refresh_cache: bool = False
    ) -> List[schemas.TrendInsight]:
        """Get trend insights for specified metric and date range"""
        cache_key = f"trend_insights_{metric}_{start_date}_{end_date}"

        # Check cache if refresh not requested
        if not refresh_cache:
            cached = self._cache.get(cache_key)
            if cached:
                metrics.track_cache_operation("memory", hit=True)
                return cached
            metrics.track_cache_operation("memory", hit=False)

        try:
            with metrics.DatabaseMetrics(operation="select", table="multiple"):
                with metrics.INSIGHT_GENERATION_LATENCY.labels(insight_type="trend").time():
                    # Get trend data
                    trend_data = self._get_trend_data(metric, start_date, end_date, db)

                    # Analyze trends
                    insights = self._analyze_trends(trend_data)

                    # Cache results
                    self._cache[cache_key] = insights
                    return insights

        except Exception as e:
            logger.error(f"Error getting trend insights: {str(e)}")
            raise

    def process_natural_language_query(
        self,
        query: str,
        db: Session,
        refresh_cache: bool = False
    ) -> schemas.QueryResponse:
        """Process natural language query and return insights"""
        cache_key = f"query_{hash(query)}"

        # Check cache if refresh not requested
        if not refresh_cache:
            cached = self._cache.get(cache_key)
            if cached:
                metrics.track_cache_operation("memory", hit=True)
                return cached
            metrics.track_cache_operation("memory", hit=False)

        try:
            with metrics.MLMetrics(model_name="query_processor", model_version="1.0"):
                # Parse query
                parsed_query = self._parse_query(query)

                # Execute query
                with metrics.DatabaseMetrics(operation="select", table="multiple"):
                    results = self._execute_query(parsed_query, db)

                # Format response
                response = self._format_query_response(results)

                # Cache results
                self._cache[cache_key] = response
                return response

        except Exception as e:
            logger.error(f"Error processing query: {str(e)}")
            raise

    def get_dashboards(
        self,
        category: Optional[str],
        db: Session
    ) -> List[schemas.Dashboard]:
        """Get available dashboards"""
        try:
            with metrics.DatabaseMetrics(operation="select", table="dashboards"):
                query = db.query(models.Dashboard)

                if category:
                    query = query.filter(models.Dashboard.category == category)

                return query.all()

        except Exception as e:
            logger.error(f"Error getting dashboards: {str(e)}")
            raise

    def _calculate_revenue_kpi(
        self,
        timeframe: str,
        db: Session
    ) -> schemas.KPIInsight:
        """Calculate revenue KPIs"""
        try:
            # Define time window
            end_date = datetime.utcnow()
            if timeframe == 'daily':
                start_date = end_date - timedelta(days=1)
                prev_start = start_date - timedelta(days=1)
            elif timeframe == 'weekly':
                start_date = end_date - timedelta(weeks=1)
                prev_start = start_date - timedelta(weeks=1)
            elif timeframe == 'monthly':
                start_date = end_date - timedelta(days=30)
                prev_start = start_date - timedelta(days=30)
            else:  # quarterly
                start_date = end_date - timedelta(days=90)
                prev_start = start_date - timedelta(days=90)

            # Query current period revenue
            current_revenue = db.query(
                func.sum(models.Order.total_amount).label('revenue')
            ).filter(
                models.Order.order_date.between(start_date, end_date)
            ).scalar() or 0.0

            # Query previous period revenue
            prev_revenue = db.query(
                func.sum(models.Order.total_amount).label('revenue')
            ).filter(
                models.Order.order_date.between(prev_start, start_date)
            ).scalar() or 0.0

            # Calculate trend
            if prev_revenue > 0:
                change_pct = ((current_revenue - prev_revenue) / prev_revenue) * 100
                trend = 'increasing' if change_pct > 0 else 'decreasing'
            else:
                change_pct = 0
                trend = 'stable'

            # Generate analysis
            analysis = (
                f"Revenue {trend} by {abs(change_pct):.1f}% compared to previous {timeframe} period. "
                f"Current period revenue: ${current_revenue:,.2f}"
            )

            return schemas.KPIInsight(
                metric="revenue",
                value=float(current_revenue),
                trend=trend,
                analysis=analysis,
                confidence=0.95,
                timestamp=end_date
            )

        except Exception as e:
            logger.error(f"Error calculating revenue KPI: {str(e)}")
            raise

    def _calculate_customer_kpi(
        self,
        timeframe: str,
        db: Session
    ) -> schemas.KPIInsight:
        """Calculate customer KPIs"""
        try:
            # Define time window
            end_date = datetime.utcnow()
            if timeframe == 'daily':
                start_date = end_date - timedelta(days=1)
                prev_start = start_date - timedelta(days=1)
            elif timeframe == 'weekly':
                start_date = end_date - timedelta(weeks=1)
                prev_start = start_date - timedelta(weeks=1)
            elif timeframe == 'monthly':
                start_date = end_date - timedelta(days=30)
                prev_start = start_date - timedelta(days=30)
            else:  # quarterly
                start_date = end_date - timedelta(days=90)
                prev_start = start_date - timedelta(days=90)

            # Query current period metrics
            current_metrics = db.query(
                func.count(distinct(models.Order.customer_id)).label('active_customers'),
                func.count(models.Order.id).label('total_orders'),
                func.avg(models.Order.total_amount).label('avg_order_value')
            ).filter(
                models.Order.order_date.between(start_date, end_date)
            ).first()

            # Query previous period metrics
            prev_metrics = db.query(
                func.count(distinct(models.Order.customer_id)).label('active_customers')
            ).filter(
                models.Order.order_date.between(prev_start, start_date)
            ).first()

            # Calculate customer activity trend
            current_customers = current_metrics.active_customers or 0
            prev_customers = prev_metrics.active_customers or 0

            if prev_customers > 0:
                change_pct = ((current_customers - prev_customers) / prev_customers) * 100
                trend = 'increasing' if change_pct > 0 else 'decreasing'
            else:
                change_pct = 0
                trend = 'stable'

            # Calculate average metrics
            avg_order_value = current_metrics.avg_order_value or 0
            orders_per_customer = (current_metrics.total_orders or 0) / max(current_customers, 1)

            # Generate analysis
            analysis = (
                f"Active customers {trend} by {abs(change_pct):.1f}% compared to previous {timeframe}. "
                f"Average order value: ${avg_order_value:.2f}, "
                f"Orders per customer: {orders_per_customer:.2f}"
            )

            return schemas.KPIInsight(
                metric="customer_engagement",
                value=float(current_customers),
                trend=trend,
                analysis=analysis,
                confidence=0.95,
                timestamp=end_date
            )

        except Exception as e:
            logger.error(f"Error calculating customer KPI: {str(e)}")
            raise

    def _calculate_product_kpi(
        self,
        timeframe: str,
        db: Session,
        top_product_limit: int = 3  # Add configurable limit
    ) -> schemas.KPIInsight:
        """Calculate product KPIs

        Args:
            timeframe: Time period for analysis ('daily', 'weekly', 'monthly', 'quarterly')
            db: Database session
            top_product_limit: Number of top performing products to include in analysis

        Returns:
            KPIInsight object containing product performance metrics and analysis
        """
        try:
            # Define time window
            end_date = datetime.utcnow()
            if timeframe == 'daily':
                start_date = end_date - timedelta(days=1)
                prev_start = start_date - timedelta(days=1)
            elif timeframe == 'weekly':
                start_date = end_date - timedelta(weeks=1)
                prev_start = start_date - timedelta(weeks=1)
            elif timeframe == 'monthly':
                start_date = end_date - timedelta(days=30)
                prev_start = start_date - timedelta(days=30)
            else:  # quarterly
                start_date = end_date - timedelta(days=90)
                prev_start = start_date - timedelta(days=90)

            # Query current period metrics
            current_metrics = db.query(
                func.count(distinct(models.OrderItem.product_id)).label('active_products'),
                func.sum(models.OrderItem.quantity).label('total_units'),
                func.avg(models.OrderItem.unit_price).label('avg_price'),
                func.sum(models.OrderItem.quantity * models.OrderItem.unit_price).label('total_revenue')
            ).join(
                models.Order
            ).filter(
                models.Order.order_date.between(start_date, end_date)
            ).first()

            # Query previous period metrics
            prev_metrics = db.query(
                func.count(distinct(models.OrderItem.product_id)).label('active_products'),
                func.sum(models.OrderItem.quantity).label('total_units'),
                func.avg(models.OrderItem.unit_price).label('avg_price'),
                func.sum(models.OrderItem.quantity * models.OrderItem.unit_price).label('total_revenue')
            ).join(
                models.Order
            ).filter(
                models.Order.order_date.between(prev_start, start_date)
            ).first()

            # Query top performing products
            top_products = db.query(
                models.Product.name,
                models.Product.category,
                func.sum(models.OrderItem.quantity).label('units_sold'),
                func.sum(models.OrderItem.quantity * models.OrderItem.unit_price).label('revenue')
            ).join(
                models.OrderItem
            ).join(
                models.Order
            ).filter(
                models.Order.order_date.between(start_date, end_date)
            ).group_by(
                models.Product.name,
                models.Product.category
            ).order_by(
                text('revenue DESC')
            ).limit(top_product_limit).all()

            # Calculate metrics and trends with safe handling of None values
            current_active = current_metrics.active_products or 0
            prev_active = prev_metrics.active_products or 0
            current_units = current_metrics.total_units or 0
            prev_units = prev_metrics.total_units or 0
            current_revenue = current_metrics.total_revenue or 0
            prev_revenue = prev_metrics.total_revenue or 0
            current_avg_price = current_metrics.avg_price or 0
            prev_avg_price = prev_metrics.avg_price or 0

            # Calculate trends with proper handling of zero division
            def calculate_trend(current: float, previous: float) -> tuple[float, str]:
                """Helper function to calculate trend and percentage change"""
                if previous > 0:
                    change_pct = ((current - previous) / previous) * 100
                    trend = 'increasing' if change_pct > 0 else 'decreasing'
                else:
                    change_pct = 0
                    trend = 'stable'
                return change_pct, trend

            # Calculate trends for all metrics
            active_change_pct, active_trend = calculate_trend(current_active, prev_active)
            units_change_pct, units_trend = calculate_trend(current_units, prev_units)
            revenue_change_pct, revenue_trend = calculate_trend(current_revenue, prev_revenue)
            avg_price_change_pct, avg_price_trend = calculate_trend(current_avg_price, prev_avg_price)

            # Generate analysis with comprehensive trend information
            top_products_text = ", ".join([
                f"{p.name} ({p.category}: {p.units_sold:,} units, ${p.revenue:,.2f} revenue)"
                for p in top_products
            ])

            analysis = (
                f"Product performance {units_trend} with {abs(units_change_pct):.1f}% change in units sold. "
                f"Active products {active_trend} with {active_change_pct:+.1f}% ({current_active:,} products), "
                f"Total revenue {revenue_trend} with {revenue_change_pct:+.1f}% (${current_revenue:,.2f}), "
                f"Average price {avg_price_trend} with {avg_price_change_pct:+.1f}% (${current_avg_price:.2f}). "
                f"Top {top_product_limit} performing products by revenue: {top_products_text}"
            )

            # Track additional metrics for monitoring
            metrics.KPI_VALUES.labels(
                kpi_name="product_active",
                timeframe=timeframe
            ).set(current_active)

            metrics.KPI_VALUES.labels(
                kpi_name="product_avg_price",
                timeframe=timeframe
            ).set(current_avg_price)

            return schemas.KPIInsight(
                metric="product_performance",
                value=float(current_units),
                trend=units_trend,
                analysis=analysis,
                confidence=0.95,
                timestamp=end_date
            )

        except Exception as e:
            logger.error(f"Error calculating product KPI: {str(e)}")
            raise

    def _get_trend_data(
        self,
        metric: str,
        start_date: str,
        end_date: str,
        db: Session
    ) -> Dict:
        """Get trend data from database"""
        try:
            # Convert dates
            start_dt = datetime.fromisoformat(start_date)
            end_dt = datetime.fromisoformat(end_date)

            if metric == 'revenue':
                # Query daily revenue
                query = db.query(
                    models.Order.order_date,
                    func.sum(models.Order.total_amount).label('value')
                ).filter(
                    models.Order.order_date.between(start_dt, end_dt)
                ).group_by(
                    models.Order.order_date
                )
            elif metric == 'orders':
                # Query daily order count
                query = db.query(
                    models.Order.order_date,
                    func.count(models.Order.id).label('value')
                ).filter(
                    models.Order.order_date.between(start_dt, end_dt)
                ).group_by(
                    models.Order.order_date
                )
            elif metric == 'customers':
                # Query daily active customers
                query = db.query(
                    models.Order.order_date,
                    func.count(distinct(models.Order.customer_id)).label('value')
                ).filter(
                    models.Order.order_date.between(start_dt, end_dt)
                ).group_by(
                    models.Order.order_date
                )
            else:
                raise ValueError(f"Unsupported metric: {metric}")

            # Execute query and convert to dict
            results = query.all()
            trend_data = {
                'dates': [r.order_date.isoformat() for r in results],
                'values': [float(r.value) for r in results]
            }

            return trend_data

        except Exception as e:
            logger.error(f"Error getting trend data: {str(e)}")
            raise

    def _analyze_trends(
        self,
        trend_data: Dict
    ) -> List[schemas.TrendInsight]:
        """Analyze trends in data"""
        try:
            insights = []

            # Convert data to numpy arrays for analysis
            dates = np.array(trend_data['dates'])
            values = np.array(trend_data['values'])

            # Calculate basic statistics
            mean_value = np.mean(values)
            std_value = np.std(values)
            min_value = np.min(values)
            max_value = np.max(values)

            # Calculate trend direction
            if len(values) >= 2:
                first_value = values[0]
                last_value = values[-1]
                total_change = ((last_value - first_value) / first_value) * 100

                if total_change > 5:
                    trend_type = 'increasing'
                elif total_change < -5:
                    trend_type = 'decreasing'
                else:
                    trend_type = 'stable'

                # Create main trend insight
                main_insight = schemas.TrendInsight(
                    metric="value",
                    trend_type=trend_type,
                    analysis=(
                        f"Overall trend is {trend_type} with {abs(total_change):.1f}% change. "
                        f"Average value: {mean_value:.2f}, "
                        f"Range: {min_value:.2f} to {max_value:.2f}"
                    ),
                    confidence=0.90
                )
                insights.append(main_insight)

                # Detect seasonality if enough data points
                if len(values) >= 14:
                    # Calculate weekly pattern
                    dates_dt = [datetime.fromisoformat(d) for d in dates]
                    day_of_week = [d.weekday() for d in dates_dt]
                    weekly_pattern = {}
                    for day in range(7):
                        day_values = values[np.array(day_of_week) == day]
                        if len(day_values) > 0:
                            weekly_pattern[day] = float(np.mean(day_values))

                    # Find peak and trough days
                    peak_day = max(weekly_pattern.items(), key=lambda x: x[1])[0]
                    trough_day = min(weekly_pattern.items(), key=lambda x: x[1])[0]

                    days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']
                    seasonality_insight = schemas.TrendInsight(
                        metric="seasonality",
                        trend_type="seasonal",
                        analysis=(
                            f"Weekly pattern detected. "
                            f"Peak day: {days[peak_day]}, "
                            f"Lowest day: {days[trough_day]}"
                        ),
                        confidence=0.85
                    )
                    insights.append(seasonality_insight)

                # Detect volatility
                volatility = std_value / mean_value if mean_value > 0 else 0
                volatility_level = (
                    'high' if volatility > 0.2
                    else 'moderate' if volatility > 0.1
                    else 'low'
                )
                volatility_insight = schemas.TrendInsight(
                    metric="volatility",
                    trend_type=volatility_level,
                    analysis=(
                        f"{volatility_level.capitalize()} volatility detected "
                        f"({volatility:.2f}). Consider this when making forecasts."
                    ),
                    confidence=0.80
                )
                insights.append(volatility_insight)

            return insights

        except Exception as e:
            logger.error(f"Error analyzing trends: {str(e)}")
            raise

    def _parse_query(
        self,
        query: str
    ) -> Dict:
        """Parse natural language query into structured format

        Args:
            query: Natural language query string

        Returns:
            Dict containing parsed query components:
            - intent: Query intent (kpi, trend, comparison)
            - metric: Target metric
            - timeframe: Time period
            - filters: Any filters to apply
            - grouping: How to group results
        """
        try:
            if not query:
                raise ValueError("Empty query string")

            # Normalize query
            query = query.lower().strip()

            # Initialize parsed components with defaults
            parsed = {
                "intent": "kpi",  # Default intent
                "metric": None,
                "timeframe": "monthly",  # Default timeframe
                "filters": {},
                "grouping": None
            }

            # Detect intent with expanded keywords
            intent_patterns = {
                "trend": ["trend", "change", "movement", "growth", "decline", "over time", "historical"],
                "comparison": ["compare", "difference", "versus", "vs", "against", "between"],
                "kpi": ["current", "latest", "now", "status", "performance", "metric"]
            }

            for intent, patterns in intent_patterns.items():
                if any(pattern in query for pattern in patterns):
                    parsed["intent"] = intent
                    break

            # Detect metric with expanded variations
            metric_patterns = {
                "revenue": ["revenue", "sales", "income", "earnings", "money"],
                "orders": ["order", "purchase", "sale", "transaction"],
                "customers": ["customer", "client", "buyer", "user", "account"],
                "products": ["product", "item", "sku", "merchandise"]
            }

            for metric, patterns in metric_patterns.items():
                if any(pattern in query for pattern in patterns):
                    parsed["metric"] = metric
                    break

            if not parsed["metric"]:
                raise ValueError("Could not determine metric from query")

            # Detect timeframe with expanded variations
            timeframe_patterns = {
                "daily": ["daily", "day", "24 hours", "today"],
                "weekly": ["weekly", "week", "7 days", "last week"],
                "monthly": ["monthly", "month", "30 days", "last month"],
                "quarterly": ["quarterly", "quarter", "90 days", "3 months"]
            }

            for timeframe, patterns in timeframe_patterns.items():
                if any(pattern in query for pattern in patterns):
                    parsed["timeframe"] = timeframe
                    break

            # Extract filters with better context handling
            def extract_value_after_keyword(text: str, keyword: str, context_words: int = 3) -> Optional[str]:
                """Extract value after keyword with context awareness"""
                words = text.split()
                if keyword in words:
                    idx = words.index(keyword)
                    if idx + 1 < len(words):
                        # Take up to n words after keyword, stopping at certain punctuation
                        value_words = []
                        for i in range(idx + 1, min(idx + 1 + context_words, len(words))):
                            if words[i] in ["and", "or", "with", "by", "in"]:
                                break
                            value_words.append(words[i])
                        return " ".join(value_words) if value_words else None
                return None

            # Extract category with context
            category_value = extract_value_after_keyword(query, "category")
            if category_value:
                parsed["filters"]["category"] = category_value

            # Extract segment with context
            segment_value = extract_value_after_keyword(query, "segment")
            if segment_value:
                parsed["filters"]["segment"] = segment_value

            # Detect grouping with expanded variations
            grouping_patterns = {
                "category": ["by category", "per category", "across categories", "for each category"],
                "segment": ["by segment", "per segment", "across segments", "for each segment"]
            }

            for grouping, patterns in grouping_patterns.items():
                if any(pattern in query for pattern in patterns):
                    parsed["grouping"] = grouping
                    break

            # If comparison intent but no grouping specified, try to infer from filters
            if parsed["intent"] == "comparison" and not parsed["grouping"]:
                if "category" in parsed["filters"]:
                    parsed["grouping"] = "category"
                elif "segment" in parsed["filters"]:
                    parsed["grouping"] = "segment"

            return parsed

        except ValueError as e:
            logger.error(f"Value error in query parsing: {str(e)}")
            raise
        except Exception as e:
            logger.error(f"Error parsing query: {str(e)}")
            raise

    def _execute_query(
        self,
        parsed_query: Dict,
        db: Session
    ) -> Dict:
        """Execute parsed query and return results

        Args:
            parsed_query: Parsed query components
            db: Database session

        Returns:
            Dict containing query results
        """
        try:
            if not parsed_query.get("metric"):
                raise ValueError("Query must specify a metric")

            # Calculate date ranges based on timeframe
            end_date = datetime.utcnow()
            if parsed_query["timeframe"] == "daily":
                start_date = end_date - timedelta(days=1)
            elif parsed_query["timeframe"] == "weekly":
                start_date = end_date - timedelta(weeks=1)
            elif parsed_query["timeframe"] == "monthly":
                start_date = end_date - timedelta(days=30)
            else:  # quarterly
                start_date = end_date - timedelta(days=90)

            results = {
                "data": [],
                "metadata": {
                    "query_type": parsed_query["intent"],
                    "metric": parsed_query["metric"],
                    "timeframe": parsed_query["timeframe"],
                    "start_date": start_date.isoformat(),
                    "end_date": end_date.isoformat()
                }
            }

            try:
                # Handle different query intents
                if parsed_query["intent"] == "kpi":
                    # Get KPI insights with error handling
                    try:
                        kpi_data = self.get_kpi_insights(
                            timeframe=parsed_query["timeframe"],
                            db=db,
                            refresh_cache=True
                        )

                        # Filter relevant KPI
                        metric_mapping = {
                            "revenue": "revenue_performance",
                            "orders": "order_volume",
                            "customers": "customer_engagement",
                            "products": "product_performance"
                        }

                        target_metric = metric_mapping.get(parsed_query["metric"])
                        if target_metric and target_metric in kpi_data:
                            results["data"] = [kpi_data[target_metric]]
                        else:
                            logger.warning(f"KPI data not found for metric: {parsed_query['metric']}")

                    except Exception as e:
                        logger.error(f"Error getting KPI insights: {str(e)}")
                        raise

                elif parsed_query["intent"] == "trend":
                    # Get trend insights with proper date handling
                    try:
                        trend_data = self.get_trend_insights(
                            metric=parsed_query["metric"],
                            start_date=start_date.isoformat(),
                            end_date=end_date.isoformat(),
                            db=db,
                            refresh_cache=True
                        )
                        results["data"] = trend_data

                    except Exception as e:
                        logger.error(f"Error getting trend insights: {str(e)}")
                        raise

                elif parsed_query["intent"] == "comparison":
                    # Handle comparison queries with optimized joins
                    if parsed_query["grouping"] == "category":
                        try:
                            # Compare by product category with date filtering
                            query = db.query(
                                models.Product.category,
                                func.count(distinct(models.OrderItem.id)).label('orders'),
                                func.sum(models.OrderItem.quantity).label('units'),
                                func.sum(models.OrderItem.quantity * models.OrderItem.unit_price).label('revenue')
                            ).join(
                                models.OrderItem, models.Product.id == models.OrderItem.product_id
                            ).join(
                                models.Order, models.OrderItem.order_id == models.Order.id
                            ).filter(
                                models.Order.order_date.between(start_date, end_date)
                            ).group_by(
                                models.Product.category
                            )

                            # Apply category filter if specified
                            if "category" in parsed_query["filters"]:
                                query = query.filter(models.Product.category == parsed_query["filters"]["category"])

                            results["data"] = [
                                {
                                    "category": row.category,
                                    "orders": row.orders,
                                    "units": row.units,
                                    "revenue": float(row.revenue) if row.revenue else 0.0
                                }
                                for row in query.all()
                            ]

                        except Exception as e:
                            logger.error(f"Error in category comparison query: {str(e)}")
                            raise

                    elif parsed_query["grouping"] == "segment":
                        try:
                            # Compare by customer segment with date filtering
                            query = db.query(
                                models.Customer.segment,
                                func.count(distinct(models.Order.id)).label('orders'),
                                func.count(distinct(models.Customer.id)).label('customers'),
                                func.sum(models.Order.total_amount).label('revenue')
                            ).join(
                                models.Order, models.Customer.id == models.Order.customer_id
                            ).filter(
                                models.Order.order_date.between(start_date, end_date)
                            ).group_by(
                                models.Customer.segment
                            )

                            # Apply segment filter if specified
                            if "segment" in parsed_query["filters"]:
                                query = query.filter(models.Customer.segment == parsed_query["filters"]["segment"])

                            results["data"] = [
                                {
                                    "segment": row.segment,
                                    "orders": row.orders,
                                    "customers": row.customers,
                                    "revenue": float(row.revenue) if row.revenue else 0.0
                                }
                                for row in query.all()
                            ]

                        except Exception as e:
                            logger.error(f"Error in segment comparison query: {str(e)}")
                            raise

                    else:
                        logger.warning(f"Unsupported grouping: {parsed_query.get('grouping')}")

                else:
                    logger.warning(f"Unsupported query intent: {parsed_query.get('intent')}")

                return results

            except Exception as e:
                logger.error(f"Error executing query: {str(e)}")
                raise

        except ValueError as e:
            logger.error(f"Value error in query execution: {str(e)}")
            raise
        except Exception as e:
            logger.error(f"Error in query execution: {str(e)}")
            raise

    def _format_query_response(
        self,
        results: Dict
    ) -> schemas.QueryResponse:
        """Format query results into response

        Args:
            results: Raw query results

        Returns:
            Formatted QueryResponse
        """
        try:
            if not results or not results.get("data"):
                raise ValueError("No results to format")

            # Initialize response with metadata
            response = {
                "answer": "",
                "data": results["data"],
                "visualizations": [],
                "confidence": 0.0,  # Will be calculated based on data quality
                "sources": ["orders", "customers", "products"],
                "metadata": results["metadata"]
            }

            def format_number(value: float) -> str:
                """Format number with appropriate units"""
                if value >= 1_000_000:
                    return f"${value/1_000_000:.1f}M"
                elif value >= 1_000:
                    return f"${value/1_000:.1f}K"
                return f"${value:.2f}"

            def calculate_confidence(data: List[Any]) -> float:
                """Calculate confidence score based on data quality"""
                if not data:
                    return 0.0

                confidence = 0.9  # Base confidence

                # Reduce confidence for small data sets
                if len(data) < 3:
                    confidence *= 0.8

                # Reduce confidence for null values
                null_values = sum(1 for item in data if any(v is None for v in item.__dict__.values()))
                if null_values:
                    confidence *= (1 - (null_values / len(data) * 0.5))

                return round(min(confidence, 1.0), 2)

            try:
                # Generate natural language answer based on query type
                if results["metadata"]["query_type"] == "kpi":
                    if results["data"]:
                        kpi = results["data"][0]
                        response["answer"] = (
                            f"For the {results['metadata']['timeframe']} period "
                            f"({results['metadata']['start_date'][:10]} to {results['metadata']['end_date'][:10]}), "
                            f"the {kpi.metric} {kpi.analysis}"
                        )
                        response["confidence"] = kpi.confidence

                        # Add appropriate visualization
                        response["visualizations"].append({
                            "type": "gauge_chart",
                            "metric": kpi.metric,
                            "value": kpi.value,
                            "trend": kpi.trend
                        })

                elif results["metadata"]["query_type"] == "trend":
                    if results["data"]:
                        trends = results["data"]
                        main_trend = trends[0]

                        # Create detailed trend description
                        trend_description = f"Analysis of {results['metadata']['metric']} trends shows: {main_trend.analysis}"
                        if len(trends) > 1:
                            trend_description += f" Additionally, {trends[1].analysis}"

                        response["answer"] = trend_description
                        response["confidence"] = calculate_confidence(trends)

                        # Add visualization suggestions
                        response["visualizations"].extend([
                            {
                                "type": "line_chart",
                                "metric": results["metadata"]["metric"],
                                "data": results["data"],
                                "time_range": {
                                    "start": results["metadata"]["start_date"],
                                    "end": results["metadata"]["end_date"]
                                }
                            },
                            {
                                "type": "sparkline",
                                "metric": results["metadata"]["metric"],
                                "data": results["data"][-7:]  # Last 7 data points
                            }
                        ])

                elif results["metadata"]["query_type"] == "comparison":
                    if results["data"]:
                        total_revenue = sum(item.get('revenue', 0) for item in results["data"])
                        total_orders = sum(item.get('orders', 0) for item in results["data"])

                        # Sort data by revenue for ranking
                        sorted_data = sorted(results["data"], key=lambda x: x.get('revenue', 0), reverse=True)

                        if "category" in results["data"][0]:
                            # Format category comparison response
                            top_category = sorted_data[0]["category"]
                            top_revenue = format_number(sorted_data[0]["revenue"])

                            response["answer"] = (
                                f"Comparison by product category shows: "
                                f"Total revenue: {format_number(total_revenue)}, "
                                f"Total orders: {total_orders:,}. "
                                f"Top performing category is {top_category} "
                                f"with {top_revenue} in revenue. "
                                f"Analysis covers {len(results['data'])} categories."
                            )

                        else:
                            # Format segment comparison response
                            top_segment = sorted_data[0]["segment"]
                            top_revenue = format_number(sorted_data[0]["revenue"])
                            total_customers = sum(item.get('customers', 0) for item in results["data"])

                            response["answer"] = (
                                f"Comparison by customer segment shows: "
                                f"Total revenue: {format_number(total_revenue)}, "
                                f"Total customers: {total_customers:,}, "
                                f"Total orders: {total_orders:,}. "
                                f"Top performing segment is {top_segment} "
                                f"with {top_revenue} in revenue. "
                                f"Analysis covers {len(results['data'])} segments."
                            )

                        response["confidence"] = calculate_confidence(results["data"])

                        # Add visualization suggestions
                        response["visualizations"].extend([
                            {
                                "type": "bar_chart",
                                "metric": results["metadata"]["metric"],
                                "data": results["data"],
                                "sort_by": "revenue",
                                "orientation": "vertical"
                            },
                            {
                                "type": "pie_chart",
                                "metric": "revenue_distribution",
                                "data": results["data"]
                            },
                            {
                                "type": "treemap",
                                "metric": results["metadata"]["metric"],
                                "data": results["data"]
                            }
                        ])

                return schemas.QueryResponse(**response)

            except Exception as e:
                logger.error(f"Error formatting specific query type: {str(e)}")
                raise

        except ValueError as e:
            logger.error(f"Value error in response formatting: {str(e)}")
            raise
        except Exception as e:
            logger.error(f"Error formatting query response: {str(e)}")
            raise
