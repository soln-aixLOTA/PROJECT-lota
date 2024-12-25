use serde::Deserialize;
use validator::{Validate, ValidationError};

pub trait Validator {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[derive(Debug, Deserialize, Validate)]
pub struct RequestValidator<T: Validate> {
    #[validate]
    pub payload: T,
}

impl<T: Validate> RequestValidator<T> {
    pub fn new(payload: T) -> Self {
        Self { payload }
    }
    
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.payload.validate()?;
        Ok(())
    }
}
