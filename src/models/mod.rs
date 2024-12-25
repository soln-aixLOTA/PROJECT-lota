pub mod user;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct HealthcareData {
    #[validate(length(min = 1))]
    pub patient_id: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub medical_record: String,

    pub timestamp: chrono::DateTime<chrono::Utc>,
}
