<<<<<<< HEAD
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
=======
use async_trait::async_trait;
use aws_sdk_s3::{Client as S3Client, Config as S3Config};
use aws_types::region::Region;
use bytes::Bytes;
use std::path::Path;
use tokio::fs;

use crate::core::error::{DocumentError, DocumentResult};

#[async_trait]
pub trait StorageBackend: Send + Sync + 'static {
    async fn upload_file(&self, path: &str, content: Bytes) -> DocumentResult<()>;
    async fn download_file(&self, path: &str) -> DocumentResult<Bytes>;
    async fn delete_file(&self, path: &str) -> DocumentResult<()>;
}

pub struct LocalStorage {
    base_path: String,
}

impl LocalStorage {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    fn get_full_path(&self, path: &str) -> String {
        format!("{}/{}", self.base_path, path)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn upload_file(&self, path: &str, content: Bytes) -> DocumentResult<()> {
        let full_path = self.get_full_path(path);
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| DocumentError::StorageError(e.to_string()))?;
        }
        fs::write(&full_path, content)
            .await
            .map_err(|e| DocumentError::StorageError(e.to_string()))
    }

    async fn download_file(&self, path: &str) -> DocumentResult<Bytes> {
        let full_path = self.get_full_path(path);
        fs::read(&full_path)
            .await
            .map(Bytes::from)
            .map_err(|e| DocumentError::StorageError(e.to_string()))
    }

    async fn delete_file(&self, path: &str) -> DocumentResult<()> {
        let full_path = self.get_full_path(path);
        fs::remove_file(&full_path)
            .await
            .map_err(|e| DocumentError::StorageError(e.to_string()))
    }
}

pub struct S3Storage {
    client: S3Client,
    bucket: String,
}

impl S3Storage {
    pub async fn new(region: String, bucket: String) -> DocumentResult<Self> {
        let config = S3Config::builder().region(Region::new(region)).build();
        let client = S3Client::from_conf(config);
        Ok(Self { client, bucket })
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn upload_file(&self, path: &str, content: Bytes) -> DocumentResult<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(content.into())
            .send()
            .await
            .map_err(|e| DocumentError::StorageError(e.to_string()))?;
        Ok(())
    }

    async fn download_file(&self, path: &str) -> DocumentResult<Bytes> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| DocumentError::StorageError(e.to_string()))?;

        let bytes = output
            .body
            .collect()
            .await
            .map_err(|e| DocumentError::StorageError(e.to_string()))?
            .into_bytes();
        Ok(bytes)
    }

    async fn delete_file(&self, path: &str) -> DocumentResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| DocumentError::StorageError(e.to_string()))?;
        Ok(())
    }
}

pub mod init;
pub use init::init_storage;
>>>>>>> 921251a (fetch)
