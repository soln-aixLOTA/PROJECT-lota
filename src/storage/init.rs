use std::sync::Arc;

use crate::core::error::{DocumentError, DocumentResult};
use crate::core::StorageConfig;
use crate::storage::{LocalStorage, S3Storage, StorageBackend};

pub async fn init_storage(config: &StorageConfig) -> DocumentResult<Arc<dyn StorageBackend>> {
    match config.storage_type.as_str() {
        "local" => {
            let base_path = config
                .config
                .get("base_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    DocumentError::StorageError("Missing base_path for local storage".to_string())
                })?;
            Ok(Arc::new(LocalStorage::new(base_path.to_string())))
        }
        "s3" => {
            let region = config
                .config
                .get("region")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    DocumentError::StorageError("Missing region for S3 storage".to_string())
                })?;
            let bucket = config
                .config
                .get("bucket")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    DocumentError::StorageError("Missing bucket for S3 storage".to_string())
                })?;
            let storage = S3Storage::new(region.to_string(), bucket.to_string()).await?;
            Ok(Arc::new(storage))
        }
        _ => Err(DocumentError::StorageError(format!(
            "Unsupported storage type: {}",
            config.storage_type
        ))),
    }
}
