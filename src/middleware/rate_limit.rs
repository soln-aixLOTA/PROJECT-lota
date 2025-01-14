use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorTooManyRequests,
    Error,
};
use futures_util::future::LocalBoxFuture;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::keyed::DashMapStateStore,
    Quota, RateLimiter as Governor,
};
use nonzero_ext::nonzero;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct RateLimiter {
    limiter: Arc<Governor<String, DashMapStateStore<String>, DefaultClock, NoOpMiddleware>>,
}

impl RateLimiter {
    pub fn new(_requests_per_second: f64) -> Self {
        let quota = Quota::per_second(nonzero!(100u32));
        let state = DashMapStateStore::new();
        let limiter = Arc::new(Governor::new(quota, state, &DefaultClock::default()));
        Self { limiter }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let limiter = self.limiter.clone();
        Box::pin(async move { Ok(RateLimiterMiddleware { service, limiter }) })
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    limiter: Arc<Governor<String, DashMapStateStore<String>, DefaultClock, NoOpMiddleware>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let fut = self.service.call(req);
        let limiter = self.limiter.clone();

        Box::pin(async move {
            match limiter.check_key(&ip) {
                Ok(_) => fut.await,
                Err(_) => Err(ErrorTooManyRequests::<&'static str>("Too many requests")),
            }
        })
    }
}
