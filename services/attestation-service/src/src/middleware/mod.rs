use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub mod auth;
pub mod validation;

pub use auth::AttestationAuthMiddleware;
pub use validation::ValidationMiddleware;
