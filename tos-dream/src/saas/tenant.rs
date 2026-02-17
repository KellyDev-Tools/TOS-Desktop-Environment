//! Phase 16 Week 2: Tenant Management
//!
//! Multi-tenant support with tenant isolation, resource quotas, and lifecycle management.

use super::{TenantId, UserId, SaasResult, SaasError, RateLimitConfig};
use crate::containers::{ContainerManager, ContainerBackend, ResourceLimits};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono;

/// Tenant status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    /// Tenant is being created
    Creating,
    /// Tenant is active and operational
    Active,
    /// Tenant is suspended (non-payment, violation, etc.)
    Suspended,
    /// Tenant is being deactivated
    Deactivating,
    /// Tenant is inactive
    Inactive,
    /// Tenant is being deleted
    Deleting,
    /// Tenant has been deleted
    Deleted,
}

impl TenantStatus {
    /// Check if tenant is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
    
    /// Check if tenant can be used
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Creating | Self::Active)
    }
}

/// Tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Maximum number of sectors per tenant
    pub max_sectors: usize,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Resource limits for tenant containers
    pub resource_limits: ResourceLimits,
    /// Rate limits for API requests
    pub rate_limits: RateLimitConfig,
    /// Allowed sector types
    pub allowed_sector_types: Vec<String>,
    /// Storage quota in bytes
    pub storage_quota: u64,
    /// Network isolation enabled
    pub network_isolation: bool,
    /// Custom labels
    pub labels: HashMap<String, String>,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            max_sectors: 10,
            max_sessions: 50,
            resource_limits: ResourceLimits::default(),
            rate_limits: RateLimitConfig::default(),
            allowed_sector_types: vec!["terminal".to_string(), "editor".to_string()],
            storage_quota: 10 * 1024 * 1024 * 1024, // 10 GB
            network_isolation: true,
            labels: HashMap::new(),
        }
    }
}

/// Tenant information
#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique tenant ID
    pub id: TenantId,
    /// Tenant name
    pub name: String,
    /// Tenant status
    pub status: TenantStatus,
    /// Tenant configuration
    pub config: TenantConfig,
    /// Owner user ID
    pub owner_id: UserId,
    /// List of member user IDs
    pub members: Vec<UserId>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Deletion timestamp (if deleted)
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Current storage usage in bytes
    pub storage_usage: u64,
    /// Current sector count
    pub sector_count: usize,
    /// Current session count
    pub session_count: usize,
    /// Container manager for this tenant
    #[serde(skip)]
    pub container_manager: Option<ContainerManager>,
}

impl Clone for Tenant {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            status: self.status,
            config: self.config.clone(),
            owner_id: self.owner_id.clone(),
            members: self.members.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            storage_usage: self.storage_usage,
            sector_count: self.sector_count,
            session_count: self.session_count,
            container_manager: None, // ContainerManager is not Clone
        }
    }
}

impl Tenant {
    /// Create a new tenant
    pub async fn new(
        id: TenantId,
        name: String,
        owner_id: UserId,
        config: TenantConfig,
    ) -> SaasResult<Self> {
        let now = chrono::Utc::now();
        
        Ok(Self {
            id,
            name,
            status: TenantStatus::Creating,
            config,
            owner_id,
            members: Vec::new(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            storage_usage: 0,
            sector_count: 0,
            session_count: 0,
            container_manager: None,
        })
    }
    
    /// Check if user is owner
    pub fn is_owner(&self, user_id: &UserId) -> bool {
        &self.owner_id == user_id
    }
    
    /// Check if user is member (including owner)
    pub fn is_member(&self, user_id: &UserId) -> bool {
        self.is_owner(user_id) || self.members.contains(user_id)
    }
    
    /// Add member to tenant
    pub fn add_member(&mut self, user_id: UserId) -> SaasResult<()> {
        if self.members.contains(&user_id) {
            return Err(SaasError::Tenant(format!(
                "User {} is already a member of tenant {}", user_id, self.id
            )));
        }
        
        self.members.push(user_id);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Remove member from tenant
    pub fn remove_member(&mut self, user_id: &UserId) -> SaasResult<()> {
        if !self.members.contains(user_id) {
            return Err(SaasError::Tenant(format!(
                "User {} is not a member of tenant {}", user_id, self.id
            )));
        }
        
        self.members.retain(|id| id != user_id);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Update storage usage
    pub fn update_storage_usage(&mut self, usage: u64) {
        self.storage_usage = usage;
        self.updated_at = chrono::Utc::now();
    }
    
    /// Check if storage quota is exceeded
    pub fn is_storage_quota_exceeded(&self) -> bool {
        self.storage_usage > self.config.storage_quota
    }
    
    /// Get remaining storage
    pub fn remaining_storage(&self) -> u64 {
        if self.storage_usage >= self.config.storage_quota {
            0
        } else {
            self.config.storage_quota - self.storage_usage
        }
    }
    
    /// Increment sector count
    pub fn increment_sectors(&mut self) -> SaasResult<()> {
        if self.sector_count >= self.config.max_sectors {
            return Err(SaasError::Tenant(format!(
                "Maximum sectors ({}) reached for tenant {}", 
                self.config.max_sectors, self.id
            )));
        }
        self.sector_count += 1;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Decrement sector count
    pub fn decrement_sectors(&mut self) {
        if self.sector_count > 0 {
            self.sector_count -= 1;
        }
        self.updated_at = chrono::Utc::now();
    }
    
    /// Increment session count
    pub fn increment_sessions(&mut self) -> SaasResult<()> {
        if self.session_count >= self.config.max_sessions {
            return Err(SaasError::Tenant(format!(
                "Maximum sessions ({}) reached for tenant {}", 
                self.config.max_sessions, self.id
            )));
        }
        self.session_count += 1;
        self.updated_at = chrono::Utc::now();
        Ok(())
    }
    
    /// Decrement session count
    pub fn decrement_sessions(&mut self) {
        if self.session_count > 0 {
            self.session_count -= 1;
        }
        self.updated_at = chrono::Utc::now();
    }
    
    /// Initialize container manager for tenant
    pub async fn initialize_container_manager(
        &mut self,
        backend: ContainerBackend,
    ) -> SaasResult<()> {
        let manager = ContainerManager::new(backend).await
            .map_err(|e| SaasError::Tenant(format!("Failed to create container manager: {}", e)))?;
        
        self.container_manager = Some(manager);
        Ok(())
    }
}

/// Tenant manager handles tenant lifecycle
#[derive(Debug)]
pub struct TenantManager {
    tenants: std::sync::Arc<std::sync::Mutex<HashMap<TenantId, Tenant>>>,
    default_config: TenantConfig,
}

impl TenantManager {
    /// Create a new tenant manager
    pub fn new(default_config: TenantConfig) -> Self {
        Self {
            tenants: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            default_config,
        }
    }
    
    /// Initialize tenant manager
    pub async fn initialize(&mut self) -> SaasResult<()> {
        tracing::info!("Initializing tenant manager");
        // In real implementation, load tenants from persistence
        Ok(())
    }
    
    /// Create a new tenant
    pub async fn create_tenant(
        &self,
        name: String,
        owner_id: UserId,
        config: Option<TenantConfig>,
    ) -> SaasResult<Tenant> {
        let id = format!("tenant-{}", uuid::Uuid::new_v4());
        let config = config.unwrap_or_else(|| self.default_config.clone());
        
        let mut tenant = Tenant::new(id.clone(), name, owner_id, config).await?;
        
        // Initialize container manager
        tenant.initialize_container_manager(ContainerBackend::Mock).await?;
        
        // Set active status
        tenant.status = TenantStatus::Active;
        
        // Store tenant
        self.tenants.lock().unwrap().insert(id.clone(), tenant.clone());
        
        tracing::info!("Created tenant: {} ({})", tenant.name, id);
        Ok(tenant)
    }
    
    /// Get tenant by ID
    pub fn get_tenant(&self, id: &TenantId) -> SaasResult<Tenant> {
        self.tenants.lock().unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| SaasError::NotFound(format!("Tenant {} not found", id)))
    }
    
    /// Update tenant
    pub fn update_tenant(&self, tenant: Tenant) -> SaasResult<()> {
        let mut tenants = self.tenants.lock().unwrap();
        if !tenants.contains_key(&tenant.id) {
            return Err(SaasError::NotFound(format!("Tenant {} not found", tenant.id)));
        }
        
        tenants.insert(tenant.id.clone(), tenant);
        Ok(())
    }
    
    /// Delete tenant
    pub async fn delete_tenant(&self, id: &TenantId) -> SaasResult<()> {
        let mut tenants = self.tenants.lock().unwrap();
        let tenant = tenants.get_mut(id)
            .ok_or_else(|| SaasError::NotFound(format!("Tenant {} not found", id)))?;
        
        tenant.status = TenantStatus::Deleting;
        
        // In real implementation:
        // 1. Stop all tenant containers
        // 2. Backup tenant data
        // 3. Delete tenant resources
        
        tenant.status = TenantStatus::Deleted;
        tenant.deleted_at = Some(chrono::Utc::now());
        
        tracing::info!("Deleted tenant: {}", id);
        Ok(())
    }
    
    /// List all tenants
    pub fn list_tenants(&self) -> Vec<Tenant> {
        self.tenants.lock().unwrap().values().cloned().collect()
    }
    
    /// List active tenants
    pub fn list_active_tenants(&self) -> Vec<Tenant> {
        self.tenants.lock().unwrap().values()
            .filter(|t| t.status.is_active())
            .cloned()
            .collect()
    }
    
    /// Suspend tenant
    pub fn suspend_tenant(&self, id: &TenantId, reason: &str) -> SaasResult<()> {
        let mut tenants = self.tenants.lock().unwrap();
        let tenant = tenants.get_mut(id)
            .ok_or_else(|| SaasError::NotFound(format!("Tenant {} not found", id)))?;
        
        tenant.status = TenantStatus::Suspended;
        tenant.updated_at = chrono::Utc::now();
        
        tracing::warn!("Suspended tenant {}: {}", id, reason);
        Ok(())
    }
    
    /// Reactivate tenant
    pub fn reactivate_tenant(&self, id: &TenantId) -> SaasResult<()> {
        let mut tenants = self.tenants.lock().unwrap();
        let tenant = tenants.get_mut(id)
            .ok_or_else(|| SaasError::NotFound(format!("Tenant {} not found", id)))?;
        
        if tenant.status != TenantStatus::Suspended {
            return Err(SaasError::Tenant(format!(
                "Tenant {} is not suspended (status: {:?})", 
                id, tenant.status
            )));
        }
        
        tenant.status = TenantStatus::Active;
        tenant.updated_at = chrono::Utc::now();
        
        tracing::info!("Reactivated tenant: {}", id);
        Ok(())
    }
    
    /// Deactivate all tenants (for shutdown)
    pub async fn deactivate_all(&self) -> SaasResult<()> {
        let mut tenants = self.tenants.lock().unwrap();
        for tenant in tenants.values_mut() {
            if tenant.status.is_active() {
                tenant.status = TenantStatus::Deactivating;
                // In real implementation, stop all tenant resources
                tenant.status = TenantStatus::Inactive;
            }
        }
        Ok(())
    }
    
    /// Get default configuration
    pub fn default_config(&self) -> &TenantConfig {
        &self.default_config
    }
    
    /// Update default configuration
    pub fn set_default_config(&mut self, config: TenantConfig) {
        self.default_config = config;
    }
}

/// Tenant builder for fluent API
#[derive(Debug)]
pub struct TenantBuilder {
    name: String,
    owner_id: UserId,
    config: TenantConfig,
}

impl TenantBuilder {
    /// Create a new tenant builder
    pub fn new(name: impl Into<String>, owner_id: impl Into<UserId>) -> Self {
        Self {
            name: name.into(),
            owner_id: owner_id.into(),
            config: TenantConfig::default(),
        }
    }
    
    /// Set maximum sectors
    pub fn max_sectors(mut self, max: usize) -> Self {
        self.config.max_sectors = max;
        self
    }
    
    /// Set maximum sessions
    pub fn max_sessions(mut self, max: usize) -> Self {
        self.config.max_sessions = max;
        self
    }
    
    /// Set storage quota (in GB)
    pub fn storage_quota_gb(mut self, gb: u64) -> Self {
        self.config.storage_quota = gb * 1024 * 1024 * 1024;
        self
    }
    
    /// Enable/disable network isolation
    pub fn network_isolation(mut self, enabled: bool) -> Self {
        self.config.network_isolation = enabled;
        self
    }
    
    /// Add allowed sector type
    pub fn allow_sector_type(mut self, sector_type: impl Into<String>) -> Self {
        self.config.allowed_sector_types.push(sector_type.into());
        self
    }
    
    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.labels.insert(key.into(), value.into());
        self
    }
    
    /// Build the tenant
    pub async fn build(self, manager: &TenantManager) -> SaasResult<Tenant> {
        manager.create_tenant(self.name, self.owner_id, Some(self.config)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_status() {
        assert!(TenantStatus::Active.is_active());
        assert!(TenantStatus::Active.is_usable());
        assert!(!TenantStatus::Suspended.is_active());
        assert!(!TenantStatus::Deleted.is_usable());
    }
    
    #[test]
    fn test_tenant_config_default() {
        let config = TenantConfig::default();
        assert_eq!(config.max_sectors, 10);
        assert_eq!(config.max_sessions, 50);
        assert_eq!(config.storage_quota, 10 * 1024 * 1024 * 1024);
        assert!(config.network_isolation);
    }
    
    #[tokio::test]
    async fn test_tenant_creation() {
        let manager = TenantManager::new(TenantConfig::default());
        let tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            "user-123".to_string(),
            None,
        ).await.unwrap();
        
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.owner_id, "user-123");
        assert!(tenant.status.is_active());
        assert!(tenant.container_manager.is_some());
    }
    
    #[tokio::test]
    async fn test_tenant_members() {
        let manager = TenantManager::new(TenantConfig::default());
        let mut tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            "owner-123".to_string(),
            None,
        ).await.unwrap();
        
        // Test owner is member
        assert!(tenant.is_member(&"owner-123".to_string()));
        assert!(tenant.is_owner(&"owner-123".to_string()));
        
        // Add member
        tenant.add_member("member-456".to_string()).unwrap();
        assert!(tenant.is_member(&"member-456".to_string()));
        assert!(!tenant.is_owner(&"member-456".to_string()));
        
        // Remove member
        tenant.remove_member(&"member-456".to_string()).unwrap();
        assert!(!tenant.is_member(&"member-456".to_string()));
    }
    
    #[tokio::test]
    async fn test_tenant_storage_quota() {
        let manager = TenantManager::new(TenantConfig::default());
        let mut tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            "user-123".to_string(),
            None,
        ).await.unwrap();
        
        // Default quota is 10 GB
        assert_eq!(tenant.remaining_storage(), 10 * 1024 * 1024 * 1024);
        
        // Update usage
        tenant.update_storage_usage(5 * 1024 * 1024 * 1024); // 5 GB
        assert_eq!(tenant.remaining_storage(), 5 * 1024 * 1024 * 1024);
        assert!(!tenant.is_storage_quota_exceeded());
        
        // Exceed quota
        tenant.update_storage_usage(15 * 1024 * 1024 * 1024); // 15 GB
        assert!(tenant.is_storage_quota_exceeded());
        assert_eq!(tenant.remaining_storage(), 0);
    }
    
    #[tokio::test]
    async fn test_tenant_builder() {
        let manager = TenantManager::new(TenantConfig::default());
        let tenant = TenantBuilder::new("Builder Test", "user-123")
            .max_sectors(20)
            .max_sessions(100)
            .storage_quota_gb(50)
            .network_isolation(false)
            .allow_sector_type("browser")
            .label("env", "test")
            .build(&manager).await.unwrap();
        
        assert_eq!(tenant.config.max_sectors, 20);
        assert_eq!(tenant.config.max_sessions, 100);
        assert_eq!(tenant.config.storage_quota, 50 * 1024 * 1024 * 1024);
        assert!(!tenant.config.network_isolation);
        assert!(tenant.config.allowed_sector_types.contains(&"browser".to_string()));
    }
    
    #[tokio::test]
    async fn test_tenant_suspend_reactivate() {
        let manager = TenantManager::new(TenantConfig::default());
        let tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            "user-123".to_string(),
            None,
        ).await.unwrap();
        
        let id = tenant.id.clone();
        
        // Suspend
        manager.suspend_tenant(&id, "Payment overdue").unwrap();
        let tenant = manager.get_tenant(&id).unwrap();
        assert_eq!(tenant.status, TenantStatus::Suspended);
        
        // Reactivate
        manager.reactivate_tenant(&id).unwrap();
        let tenant = manager.get_tenant(&id).unwrap();
        assert_eq!(tenant.status, TenantStatus::Active);
    }
}
