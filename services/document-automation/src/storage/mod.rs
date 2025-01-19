use std::path::{PathBuf, Component};
use tokio::fs;
use uuid::Uuid;
use tracing::info;
use crate::error::DocumentError;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn validate_file_id(&self, file_id: &str) -> Result<(), DocumentError> {
        // Check for empty file_id
        if file_id.is_empty() {
            return Err(DocumentError::InvalidFilename("File ID cannot be empty".to_string()));
        }

        // Check for valid UUID format if it looks like a UUID
        if file_id.len() == 36 {
            if Uuid::parse_str(file_id).is_err() {
                return Err(DocumentError::InvalidFilename(
                    "Invalid UUID format for file ID".to_string(),
                ));
            }
        }

        // Check for invalid characters and path traversal
        let path = PathBuf::from(file_id);
        for component in path.components() {
            match component {
                Component::Normal(c) => {
                    let c_str = c.to_string_lossy();
                    if c_str.contains(char::is_control) || c_str.contains(&['/', '\\'][..]) {
                        return Err(DocumentError::InvalidFilename(
                            "File ID contains invalid characters".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(DocumentError::InvalidPath(
                        "File ID contains invalid path components".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_path(&self, path: &PathBuf) -> Result<(), DocumentError> {
        // Ensure the path is within base_path
        if !path.starts_with(&self.base_path) {
            return Err(DocumentError::InvalidPath(
                "Path is outside the allowed storage area".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn store(&self, file_id: &str, content: &[u8]) -> Result<(), DocumentError> {
        self.validate_file_id(file_id)?;
        
        let file_path = self.base_path.join(file_id);
        self.validate_path(&file_path)?;
        
        info!("Creating directory: {:?}", self.base_path);
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| DocumentError::FileOperation {
                operation: "create_directory".to_string(),
                path: self.base_path.display().to_string(),
                error: e.to_string(),
            })?;
        
        info!("Saving file to: {:?}", file_path);
        fs::write(&file_path, content)
            .await
            .map_err(|e| DocumentError::FileOperation {
                operation: "write".to_string(),
                path: file_path.display().to_string(),
                error: e.to_string(),
            })?;
        
        Ok(())
    }

    pub async fn retrieve(&self, file_id: &str) -> Result<Vec<u8>, DocumentError> {
        self.validate_file_id(file_id)?;
        
        let file_path = self.base_path.join(file_id);
        self.validate_path(&file_path)?;
        
        info!("Retrieving file: {:?}", file_path);
        fs::read(&file_path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => {
                    DocumentError::NotFound(format!("Document {} not found", file_id))
                }
                _ => DocumentError::FileOperation {
                    operation: "read".to_string(),
                    path: file_path.display().to_string(),
                    error: e.to_string(),
                },
            })
    }

    pub async fn exists(&self, file_id: &str) -> Result<bool, DocumentError> {
        self.validate_file_id(file_id)?;
        
        let file_path = self.base_path.join(file_id);
        self.validate_path(&file_path)?;
        
        Ok(matches!(fs::try_exists(&file_path).await, Ok(true)))
    }

    pub async fn delete(&self, file_id: &str) -> Result<(), DocumentError> {
        self.validate_file_id(file_id)?;
        
        let file_path = self.base_path.join(file_id);
        self.validate_path(&file_path)?;
        
        if !self.exists(file_id).await? {
            return Err(DocumentError::NotFound(format!("Document {} not found", file_id)));
        }
        
        info!("Deleting file: {:?}", file_path);
        fs::remove_file(&file_path)
            .await
            .map_err(|e| DocumentError::FileOperation {
                operation: "delete".to_string(),
                path: file_path.display().to_string(),
                error: e.to_string(),
            })?;
        
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<String>, DocumentError> {
        info!("Listing files in: {:?}", self.base_path);
        
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| DocumentError::FileOperation {
                operation: "read_directory".to_string(),
                path: self.base_path.display().to_string(),
                error: e.to_string(),
            })?;
        
        let mut files = Vec::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DocumentError::FileOperation {
                operation: "read_directory_entry".to_string(),
                path: self.base_path.display().to_string(),
                error: e.to_string(),
            })?
        {
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        files.push(file_name.to_string());
                    }
                }
            }
        }
        
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::test;

    #[test]
    async fn test_store_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let storage = LocalStorage::new(temp_dir.path().to_path_buf());
        let content = b"test content";
        let file_id = "test-file";

        // Store
        storage.store(file_id, content).await.unwrap();

        // Verify exists
        assert!(storage.exists(file_id).await.unwrap());

        // Retrieve and verify content
        let retrieved = storage.retrieve(file_id).await.unwrap();
        assert_eq!(retrieved, content);

        // Delete
        storage.delete(file_id).await.unwrap();

        // Verify deleted
        assert!(!storage.exists(file_id).await.unwrap());
    }

    #[test]
    async fn test_list_files() {
        let temp_dir = tempdir().unwrap();
        let storage = LocalStorage::new(temp_dir.path().to_path_buf());
        
        // Store some files
        storage.store("file1", b"content1").await.unwrap();
        storage.store("file2", b"content2").await.unwrap();

        // List and verify
        let files = storage.list().await.unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1".to_string()));
        assert!(files.contains(&"file2".to_string()));
    }

    #[test]
    async fn test_not_found() {
        let temp_dir = tempdir().unwrap();
        let storage = LocalStorage::new(temp_dir.path().to_path_buf());
        
        // Try to retrieve non-existent file
        let result = storage.retrieve("nonexistent").await;
        assert!(matches!(result, Err(DocumentError::NotFound(_))));

        // Try to delete non-existent file
        let result = storage.delete("nonexistent").await;
        assert!(matches!(result, Err(DocumentError::NotFound(_))));
    }

    #[test]
    async fn test_invalid_file_id() {
        let temp_dir = tempdir().unwrap();
        let storage = LocalStorage::new(temp_dir.path().to_path_buf());

        // Test empty file_id
        let result = storage.store("", b"content").await;
        assert!(matches!(result, Err(DocumentError::InvalidFilename(_))));

        // Test path traversal
        let result = storage.store("../test", b"content").await;
        assert!(matches!(result, Err(DocumentError::InvalidPath(_))));

        // Test invalid characters
        let result = storage.store("test/file", b"content").await;
        assert!(matches!(result, Err(DocumentError::InvalidFilename(_))));
    }

    #[test]
    async fn test_invalid_uuid() {
        let temp_dir = tempdir().unwrap();
        let storage = LocalStorage::new(temp_dir.path().to_path_buf());

        // Test invalid UUID format
        let result = storage.store("12345678-1234-1234-1234-12345678901", b"content").await;
        assert!(matches!(result, Err(DocumentError::InvalidFilename(_))));
    }
} 