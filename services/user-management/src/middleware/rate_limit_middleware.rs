use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use governor::{
    clock::DefaultClock,
    middleware::StateInformationMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::tenant::Tenant,
    services::tenant_service::TenantService,
};

pub struct RateLimitMiddleware {
    tenant_service: Arc<TenantService>,
    rate_limiters: Arc<dashmap::DashMap<Uuid, Arc<StateInformationMiddleware<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
}

impl RateLimitMiddleware {
    pub fn new(tenant_service: Arc<TenantService>) -> Self {
        Self {
            tenant_service,
            rate_limiters: Arc::new(dashmap::DashMap::new()),
        }
    }

    fn get_or_create_limiter(
        &self,
        tenant: &Tenant,
    ) -> Arc<StateInformationMiddleware<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>> {
        if let Some(limiter) = self.rate_limiters.get(&tenant.id) {
            return limiter.clone();
        }

        // Calculate requests per second based on daily limit
        let requests_per_second = (tenant.max_requests_per_day as f64 / 86400.0).ceil() as u32;
        let quota = Quota::per_second(nonzero!(requests_per_second.max(1)));
        let limiter = Arc::new(StateInformationMiddleware::new(
            RateLimiter::direct(quota)
        ));

        self.rate_limiters.insert(tenant.id, limiter.clone());
        limiter
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            tenant_service: self.tenant_service.clone(),
            rate_limiters: self.rate_limiters.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    tenant_service: Arc<TenantService>,
    rate_limiters: Arc<dashmap::DashMap<Uuid, Arc<StateInformationMiddleware<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let tenant_service = self.tenant_service.clone();
        let rate_limiters = self.rate_limiters.clone();

        Box::pin(async move {
            // Get tenant from request extensions (set by TenantMiddleware)
            let tenant = req
                .extensions()
                .get::<Tenant>()
                .ok_or_else(|| Error::from(ServiceError::InvalidRequest(
                    "Tenant not found in request extensions".to_string(),
                )))?;

            // Get or create rate limiter for tenant
            let limiter = if let Some(limiter) = rate_limiters.get(&tenant.id) {
                limiter.clone()
            } else {
                // Calculate requests per second based on daily limit
                let requests_per_second = (tenant.max_requests_per_day as f64 / 86400.0).ceil() as u32;
                let quota = Quota::per_second(nonzero!(requests_per_second.max(1)));
                let limiter = Arc::new(StateInformationMiddleware::new(
                    RateLimiter::direct(quota)
                ));
                rate_limiters.insert(tenant.id, limiter.clone());
                limiter
            };

            // Check rate limit
            match limiter.check() {
                Ok(_) => {
                    // Add rate limit headers to response
                    let res = service.call(req).await?;
                    let mut res = res.into_parts();

                    if let Some(state) = limiter.get_state() {
                        let remaining = state.remaining_burst_capacity();
                        let reset = state
                            .earliest_possible_time()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or(Duration::from_secs(0))
                            .as_secs();

                        res.1.headers_mut().insert(
                            "X-RateLimit-Remaining",
                            remaining.to_string().parse().unwrap(),
                        );
                        res.1.headers_mut().insert(
                            "X-RateLimit-Reset",
                            reset.to_string().parse().unwrap(),
                        );
                    }

                    Ok(ServiceResponse::new(res.0, res.1))
                }
                Err(_) => Err(Error::from(ServiceError::RateLimitExceeded)),
            }
        })
    }
} 