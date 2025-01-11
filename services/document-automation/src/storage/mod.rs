use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use tracing::info;

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn save_file(&self, content: Vec<u8>) -> Result<String, std::io::Error> {
        let file_id = Uuid::new_v4().to_string();
        let file_path = self.base_path.join(&file_id);
        
        info!("Creating directory: {:?}", self.base_path);
        fs::create_dir_all(&self.base_path).await?;
        
        info!("Saving file to: {:?}", file_path);
        fs::write(file_path, content).await?;
        
        Ok(file_id)
    }

    pub async fn update_file(&self, file_id: &str, content: Vec<u8>) -> Result<(), std::io::Error> {
        let file_path = self.base_path.join(file_id);
        info!("Attempting to update file: {:?}", file_path);
        
        // Check if file exists before updating
        let exists = fs::try_exists(&file_path).await?;
        info!("File exists check: {}", exists);
        
        if !exists {
            info!("File not found: {:?}", file_path);
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Document not found"
            ));
        }
        
        info!("Updating file: {:?}", file_path);
        fs::write(file_path, content).await?;
        info!("File updated successfully");
        Ok(())
    }

    pub async fn get_file(&self, file_id: &str) -> Result<Vec<u8>, std::io::Error> {
        let file_path = self.base_path.join(file_id);
        info!("Reading file: {:?}", file_path);
        fs::read(file_path).await
    }
} 