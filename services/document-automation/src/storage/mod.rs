use std::fmt::Debug;
use std::path::Path;

use async_trait::async_trait;
use aws_sdk_s3::Client;
use tokio::fs;

use crate::core::error::{DocumentError, DocumentResult};

mod init;
pub use init::init_storage;

#[async_trait]
pub trait StorageBackend: Send + Sync + Debug {
    async fn upload_file(&self, path: &str, content: Vec<u8>) -> DocumentResult<()>;
    async fn download_file(&self, path: &str) -> DocumentResult<Vec<u8>>;
    async fn delete_file(&self, path: &str) -> DocumentResult<()>;
    fn clone_box(&self) -> Box<dyn StorageBackend>;
}

#[derive(Debug, Clone)]
pub struct LocalStorage {
    base_path: String,
}

impl LocalStorage {
    pub fn new(base_path: &str) -> DocumentResult<Self> {
        Ok(Self {
            base_path: base_path.to_string(),
        })
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    fn clone_box(&self) -> Box<dyn StorageBackend> {
        Box::new(self.clone())
    }

    async fn upload_file(&self, path: &str, content: Vec<u8>) -> DocumentResult<()> {
        let full_path = Path::new(&self.base_path).join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(full_path, content).await?;
        Ok(())
    }

    async fn download_file(&self, path: &str) -> DocumentResult<Vec<u8>> {
        let full_path = Path::new(&self.base_path).join(path);
        fs::read(full_path)
            .await
            .map_err(|e| DocumentError::Storage(e.to_string()))
    }

    async fn delete_file(&self, path: &str) -> DocumentResult<()> {
        let full_path = Path::new(&self.base_path).join(path);
        fs::remove_file(full_path)
            .await
            .map_err(|e| DocumentError::Storage(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub async fn new(bucket: String) -> DocumentResult<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let client = Client::new(&config);
        Ok(Self { client, bucket })
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    fn clone_box(&self) -> Box<dyn StorageBackend> {
        Box::new(self.clone())
    }

    async fn upload_file(&self, path: &str, content: Vec<u8>) -> DocumentResult<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(path)
            .body(content.into())
            .send()
            .await
            .map_err(|e| DocumentError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn download_file(&self, path: &str) -> DocumentResult<Vec<u8>> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| DocumentError::Storage(e.to_string()))?;

        let bytes = output
            .body
            .collect()
            .await
            .map_err(|e| DocumentError::Storage(e.to_string()))?
            .into_bytes();
        Ok(bytes.to_vec())
    }

    async fn delete_file(&self, path: &str) -> DocumentResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(path)
            .send()
            .await
            .map_err(|e| DocumentError::Storage(e.to_string()))?;
        Ok(())
    }
}
