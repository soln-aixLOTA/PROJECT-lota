use super::{LocalStorage, S3Storage, StorageBackend};
use crate::core::error::DocumentResult;

pub async fn init_storage(
    storage_type: &str,
    config: &str,
) -> DocumentResult<Box<dyn StorageBackend>> {
    match storage_type {
        "local" => {
            let storage = LocalStorage::new(config)?;
            Ok(Box::new(storage))
        }
        "s3" => {
            let storage = S3Storage::new(config.to_string()).await?;
            Ok(Box::new(storage))
        }
        _ => Err(crate::core::error::DocumentError::Internal(format!(
            "Unsupported storage type: {}",
            storage_type
        ))),
    }
}
