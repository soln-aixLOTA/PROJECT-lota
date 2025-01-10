use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::core::error::{AppError, AppResult};

pub trait StorageProvider: Send + Sync {
    fn save_file(&self, file_name: &str, content: Vec<u8>) -> AppResult<String>;
    fn get_file(&self, file_path: &str) -> AppResult<Vec<u8>>;
    fn delete_file(&self, file_path: &str) -> AppResult<()>;
}

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    fn ensure_directory(&self) -> AppResult<()> {
        if !self.base_path.exists() {
            std::fs::create_dir_all(&self.base_path)
                .map_err(|e| AppError::Storage(format!("Failed to create directory: {}", e)))?;
        }
        Ok(())
    }
}

impl StorageProvider for LocalStorage {
    fn save_file(&self, file_name: &str, content: Vec<u8>) -> AppResult<String> {
        self.ensure_directory()?;
        let file_id = Uuid::new_v4();
        let file_path = self.base_path.join(file_id.to_string());
        
        std::fs::write(&file_path, content)
            .map_err(|e| AppError::Storage(format!("Failed to save file: {}", e)))?;

        Ok(file_id.to_string())
    }

    fn get_file(&self, file_path: &str) -> AppResult<Vec<u8>> {
        let path = self.base_path.join(file_path);
        std::fs::read(&path)
            .map_err(|e| AppError::Storage(format!("Failed to read file: {}", e)))
    }

    fn delete_file(&self, file_path: &str) -> AppResult<()> {
        let path = self.base_path.join(file_path);
        std::fs::remove_file(&path)
            .map_err(|e| AppError::Storage(format!("Failed to delete file: {}", e)))
    }
}
