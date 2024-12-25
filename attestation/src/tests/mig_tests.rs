#[cfg(test)]
mod tests {
    use crate::core::mig_manager::{MigManager, MigProfile};
    use anyhow::Result;

    #[tokio::test]
    async fn test_mig_manager_initialization() -> Result<()> {
        let manager = MigManager::new()?;
        manager.initialize().await?;

        let configs = manager.list_configurations().await;
        for (gpu_id, config) in configs {
            println!("GPU {}: {:?}", gpu_id, config);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_enable_disable_mig() -> Result<()> {
        let manager = MigManager::new()?;
        manager.initialize().await?;

        let configs = manager.list_configurations().await;
        let gpu_id = configs.keys().next().unwrap().clone();

        // Try to enable MIG
        let result = manager.enable_mig(&gpu_id).await;
        if let Err(e) = result {
            println!("Error: {}", e);
            // If we get a permission error, that's expected when not running as root
            assert!(e.to_string().contains("permission"));
            return Ok(());
        }

        // If enable succeeded, try to disable
        let result = manager.disable_mig(&gpu_id).await;
        if let Err(e) = result {
            println!("Error: {}", e);
            assert!(e.to_string().contains("permission"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_instance_lifecycle() -> Result<()> {
        let manager = MigManager::new()?;
        manager.initialize().await?;

        let configs = manager.list_configurations().await;
        let gpu_id = configs.keys().next().unwrap().clone();

        // Try to enable MIG
        let result = manager.enable_mig(&gpu_id).await;
        if let Err(e) = result {
            println!("Error: {}", e);
            // If we get a permission error, that's expected when not running as root
            assert!(e.to_string().contains("permission"));
            return Ok(());
        }

        // Create a test profile
        let profile = MigProfile {
            gpu_instance_size: 1,
            compute_instance_size: 1,
            memory_size: 5 * 1024 * 1024 * 1024, // 5GB
            max_instances: 7,
        };

        // Create instance
        let instance = manager.create_instance(&gpu_id, profile).await?;
        assert!(!instance.allocated);

        // Allocate instance
        manager
            .allocate_instance(&gpu_id, &instance.id, "test-tenant")
            .await?;
        let config = manager.get_configuration(&gpu_id).await?;
        let instance = config
            .instances
            .iter()
            .find(|i| i.id == instance.id)
            .unwrap();
        assert!(instance.allocated);
        assert_eq!(instance.tenant_id.as_ref().unwrap(), "test-tenant");

        // Release instance
        manager.release_instance(&gpu_id, &instance.id).await?;
        let config = manager.get_configuration(&gpu_id).await?;
        let instance = config
            .instances
            .iter()
            .find(|i| i.id == instance.id)
            .unwrap();
        assert!(!instance.allocated);
        assert!(instance.tenant_id.is_none());

        // Destroy instance
        manager.destroy_instance(&gpu_id, &instance.id).await?;
        let config = manager.get_configuration(&gpu_id).await?;
        assert!(!config.instances.iter().any(|i| i.id == instance.id));

        // Disable MIG
        let result = manager.disable_mig(&gpu_id).await;
        if let Err(e) = result {
            println!("Error: {}", e);
            assert!(e.to_string().contains("permission"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling() -> Result<()> {
        let manager = MigManager::new()?;
        manager.initialize().await?;

        // Test with invalid GPU ID
        let result = manager.enable_mig("invalid-gpu").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("GPU not found"));

        let configs = manager.list_configurations().await;
        let gpu_id = configs.keys().next().unwrap().clone();

        // Try to create instance without enabling MIG
        let profile = MigProfile {
            gpu_instance_size: 1,
            compute_instance_size: 1,
            memory_size: 5 * 1024 * 1024 * 1024,
            max_instances: 7,
        };

        // Try to enable MIG
        let result = manager.enable_mig(&gpu_id).await;
        if let Err(e) = result {
            println!("Error: {}", e);
            // If we get a permission error, that's expected when not running as root
            assert!(e.to_string().contains("permission"));
            return Ok(());
        }

        // Try to allocate non-existent instance
        let result = manager
            .allocate_instance(&gpu_id, "non-existent", "test-tenant")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Instance not found"));

        // Create and allocate an instance
        let instance = manager.create_instance(&gpu_id, profile).await?;
        manager
            .allocate_instance(&gpu_id, &instance.id, "test-tenant")
            .await?;

        // Try to disable MIG with active instances
        let result = manager.disable_mig(&gpu_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot disable MIG while instances are in use"));

        // Try to destroy allocated instance
        let result = manager.destroy_instance(&gpu_id, &instance.id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot destroy allocated instance"));

        // Try to allocate already allocated instance
        let result = manager
            .allocate_instance(&gpu_id, &instance.id, "another-tenant")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Instance already allocated"));

        Ok(())
    }
}
