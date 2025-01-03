use crate::core::container::{
    ContainerConfig, ContainerManager, ContainerStatus, GPURequirements, HealthStatus, Mount,
    SecurityOptions,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn test_container_creation_with_security() -> Result<()> {
    let manager = ContainerManager::new();

    // Create a secure container configuration
    let config = ContainerConfig {
        image: "nvidia/triton-server:latest".to_string(),
        command: vec!["tritonserver".to_string()],
        env: HashMap::new(),
        mounts: vec![Mount {
            source: PathBuf::from("/tmp/models"),
            target: PathBuf::from("/models"),
            readonly: true,
            selinux_relabel: true,
        }],
        security: SecurityOptions {
            use_cdi: true,
            selinux_enabled: true,
            seccomp_profile: Some("/etc/seccomp/triton.json".to_string()),
            no_new_privileges: true,
            readonly_root_fs: true,
            drop_capabilities: vec!["NET_RAW".to_string(), "SYS_ADMIN".to_string()],
        },
        gpu_requirements: GPURequirements {
            count: 1,
            memory: 4 * 1024 * 1024 * 1024, // 4GB
            compute_capability: Some("8.0".to_string()),
            mig_profile: None,
        },
    };

    let container = manager.create_container(config).await?;
    assert_eq!(container.status, ContainerStatus::Created);
    assert!(container.config.security.use_cdi);
    assert!(container.config.security.selinux_enabled);
    assert!(container.config.security.no_new_privileges);
    assert!(container.config.security.readonly_root_fs);

    Ok(())
}

#[tokio::test]
async fn test_container_creation_with_insecure_config() -> Result<()> {
    let manager = ContainerManager::new();

    // Create an insecure container configuration
    let config = ContainerConfig {
        image: "nvidia/triton-server:latest".to_string(),
        command: vec!["tritonserver".to_string()],
        env: HashMap::new(),
        mounts: vec![Mount {
            source: PathBuf::from("/tmp/models"),
            target: PathBuf::from("/models"),
            readonly: false,        // Insecure: writable mount
            selinux_relabel: false, // Insecure: no SELinux relabeling
        }],
        security: SecurityOptions {
            use_cdi: false,            // Insecure: not using CDI
            selinux_enabled: false,    // Insecure: SELinux disabled
            seccomp_profile: None,     // Insecure: no seccomp profile
            no_new_privileges: false,  // Insecure: allows privilege escalation
            readonly_root_fs: false,   // Insecure: writable root filesystem
            drop_capabilities: vec![], // Insecure: no dropped capabilities
        },
        gpu_requirements: GPURequirements {
            count: 1,
            memory: 4 * 1024 * 1024 * 1024,
            compute_capability: Some("8.0".to_string()),
            mig_profile: None,
        },
    };

    // This should fail due to security requirements
    let result = manager.create_container(config).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("SELinux must be enabled")
            || err.contains("no_new_privileges must be set to true")
            || err.contains("A seccomp profile must be specified")
    );

    Ok(())
}

#[tokio::test]
async fn test_container_with_path_traversal() -> Result<()> {
    let manager = ContainerManager::new();

    // Create a container configuration with path traversal attempt
    let config = ContainerConfig {
        image: "nvidia/triton-server:latest".to_string(),
        command: vec!["tritonserver".to_string()],
        env: HashMap::new(),
        mounts: vec![Mount {
            source: PathBuf::from("/tmp/models/../../../etc/passwd"), // Path traversal attempt
            target: PathBuf::from("/models"),
            readonly: true,
            selinux_relabel: true,
        }],
        security: SecurityOptions {
            use_cdi: true,
            selinux_enabled: true,
            seccomp_profile: Some("/etc/seccomp/triton.json".to_string()),
            no_new_privileges: true,
            readonly_root_fs: true,
            drop_capabilities: vec![],
        },
        gpu_requirements: GPURequirements {
            count: 1,
            memory: 4 * 1024 * 1024 * 1024,
            compute_capability: Some("8.0".to_string()),
            mig_profile: None,
        },
    };

    // This should fail due to path traversal detection
    let result = manager.create_container(config).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Path traversal detected"));

    Ok(())
}

#[tokio::test]
async fn test_container_lifecycle_with_security() -> Result<()> {
    let manager = ContainerManager::new();

    // Create a secure container configuration
    let config = ContainerConfig {
        image: "nvidia/triton-server:latest".to_string(),
        command: vec!["tritonserver".to_string()],
        env: HashMap::new(),
        mounts: vec![Mount {
            source: PathBuf::from("/tmp/models"),
            target: PathBuf::from("/models"),
            readonly: true,
            selinux_relabel: true,
        }],
        security: SecurityOptions {
            use_cdi: true,
            selinux_enabled: true,
            seccomp_profile: Some("/etc/seccomp/triton.json".to_string()),
            no_new_privileges: true,
            readonly_root_fs: true,
            drop_capabilities: vec!["NET_RAW".to_string(), "SYS_ADMIN".to_string()],
        },
        gpu_requirements: GPURequirements {
            count: 1,
            memory: 4 * 1024 * 1024 * 1024,
            compute_capability: Some("8.0".to_string()),
            mig_profile: None,
        },
    };

    // Create container
    let container = manager.create_container(config).await?;
    assert_eq!(container.status, ContainerStatus::Created);

    // Start container
    manager.start_container(&container.id).await?;
    let container = manager.get_container(&container.id).await?;
    assert_eq!(container.status, ContainerStatus::Running);

    // Stop container
    manager.stop_container(&container.id).await?;
    let container = manager.get_container(&container.id).await?;
    assert_eq!(container.status, ContainerStatus::Stopped);

    // Remove container
    manager.remove_container(&container.id).await?;
    let result = manager.get_container(&container.id).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_container_with_invalid_capabilities() -> Result<()> {
    let manager = ContainerManager::new();

    // Create a container configuration with invalid capabilities
    let config = ContainerConfig {
        image: "nvidia/triton-server:latest".to_string(),
        command: vec!["tritonserver".to_string()],
        env: HashMap::new(),
        mounts: vec![Mount {
            source: PathBuf::from("/tmp/models"),
            target: PathBuf::from("/models"),
            readonly: true,
            selinux_relabel: true,
        }],
        security: SecurityOptions {
            use_cdi: true,
            selinux_enabled: true,
            seccomp_profile: Some("/etc/seccomp/triton.json".to_string()),
            no_new_privileges: true,
            readonly_root_fs: true,
            drop_capabilities: vec![
                "INVALID_CAP".to_string(), // Invalid capability
                "SYS_ADMIN".to_string(),
            ],
        },
        gpu_requirements: GPURequirements {
            count: 1,
            memory: 4 * 1024 * 1024 * 1024,
            compute_capability: Some("8.0".to_string()),
            mig_profile: None,
        },
    };

    // This should fail due to invalid capability
    let result = manager.create_container(config).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid capability specified"));

    Ok(())
}
