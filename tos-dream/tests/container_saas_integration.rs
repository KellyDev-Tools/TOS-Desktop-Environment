//! Phase 16: Container Strategy & Cloud Resource Architecture Integration Tests

use tos_core::TosState;
use tos_core::containers::{
    ContainerManager, ContainerBackend, ContainerConfig, 
    ContainerStatus, SecurityPolicy
};
use tos_core::saas::{
    CloudResourceManager, CloudConfig,
    PersistenceManager, SecretsManager
};
use tempfile::tempdir;

#[tokio::test]
async fn test_tos_state_saas_integration() {
    let state = TosState::new();
    
    // Verify cloud manager is initialized
    assert!(state.cloud_manager.is_some());

    let cloud = state.cloud_manager.as_ref().unwrap();
    
    // Create a tenant
    let tenant = cloud.tenant_manager().create_tenant(
        "Integration Tenant".to_string(), 
        "owner-1".to_string(), 
        None
    ).await.unwrap();
    
    assert_eq!(tenant.name, "Integration Tenant");
    assert!(tenant.status.is_active());
}

#[tokio::test]
async fn test_container_manager_lifecycle() {
    // We use Mock backend for integration tests to avoid Docker dependency
    let manager = ContainerManager::new(ContainerBackend::Mock).await.unwrap();
    
    let config = ContainerConfig {
        name: "test-container".to_string(),
        image: "alpine:latest".to_string(),
        ..Default::default()
    };

    let info = manager.create_container(config).await.unwrap();
    assert_eq!(info.status, ContainerStatus::Created);

    manager.start_container(&info.id).await.unwrap();
    let info = manager.get_container(&info.id).await.unwrap();
    assert_eq!(info.status, ContainerStatus::Running);

    manager.stop_container(&info.id, 10).await.unwrap();
    let info = manager.get_container(&info.id).await.unwrap();
    assert_eq!(info.status, ContainerStatus::Exited);
}

#[tokio::test]
async fn test_saas_billing_and_persistence_integration() {
    let cloud_config = CloudConfig::default();
    let cloud = CloudResourceManager::new(cloud_config);
    
    let tenant = cloud.tenant_manager().create_tenant(
        "Usage Tenant".to_string(),
        "user-123".to_string(),
        None
    ).await.unwrap();

    // Test Billing
    let session_id = "sess-abc".to_string();
    cloud.billing().start_session(tenant.id.clone(), session_id.clone());
    cloud.billing().end_session(&session_id).unwrap();
    
    let _stats = cloud.billing().get_tenant_stats(&tenant.id);

    // Test Persistence
    let temp = tempdir().unwrap();
    let persistence = PersistenceManager::new(temp.path());
    persistence.initialize().await.unwrap();
    
    let storage_path = persistence.create_tenant_storage(&tenant.id).unwrap();
    assert!(storage_path.exists());
}

#[tokio::test]
async fn test_container_security_policy() {
    let policy = SecurityPolicy::default(); // Restricted by default
    
    let mut config = ContainerConfig::default();
    config.privileged = true; // Should be rejected by restricted policy
    
    let result = policy.validate(&config);
    assert!(result.is_err());
    
    config.privileged = false;
    config.read_only = true;
    let result = policy.validate(&config);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_saas_secrets_integration() {
    let secrets = SecretsManager::new();
    let tenant_id = "tenant-1".to_string();
    
    secrets.set_secret(tenant_id.clone(), "api_key".to_string(), "sk-12345".to_string()).unwrap();
    let key = secrets.get_secret(&tenant_id, "api_key").unwrap();
    
    assert_eq!(key, "sk-12345");
}
