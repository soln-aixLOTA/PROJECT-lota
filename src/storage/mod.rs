use crate::core::error::DocumentResult;
use crate::core::StorageConfig;
use bytes::Bytes;
use std::path::PathBuf;

pub async fn init_storage(config: &StorageConfig) -> DocumentResult<()> {
    let storage_path = PathBuf::from(&config.base_path);
    if !storage_path.exists() {
        std::fs::create_dir_all(&storage_path)?;
    }
    Ok(())
}

pub async fn store_file(config: &StorageConfig, path: &str, data: Bytes) -> DocumentResult<()> {
    let file_path = PathBuf::from(&config.base_path).join(path);
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    tokio::fs::write(file_path, data).await?;
    Ok(())
}

pub async fn read_file(config: &StorageConfig, path: &str) -> DocumentResult<Bytes> {
    let file_path = PathBuf::from(&config.base_path).join(path);
    let data = tokio::fs::read(file_path).await?;
    Ok(Bytes::from(data))
}

pub async fn delete_file(config: &StorageConfig, path: &str) -> DocumentResult<()> {
    let file_path = PathBuf::from(&config.base_path).join(path);
    if file_path.exists() {
        tokio::fs::remove_file(file_path).await?;
    }
    Ok(())
}
