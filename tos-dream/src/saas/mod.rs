//! Phase 16 Week 2: Cloud Resource Integration
//!
//! Infrastructure for leveraging external Enterprise and Cloud resources
//! (AWS, Kubernetes) within TOS, providing isolated environments for 
//! remote sectors and offloaded computation.

pub mod tenant;
pub mod session;
pub mod gateway;
pub mod billing;
pub mod persistence;
pub mod secrets;
pub mod kubernetes;
pub mod cloud;
pub mod telemetry;
pub mod logging;

pub use tenant::{TenantManager, Tenant, TenantConfig, TenantStatus};
pub use session::{SessionManager, Session, SessionConfig, SessionStatus};
pub use gateway::{ApiGateway, GatewayConfig, Route, RateLimit};
pub use billing::BillingManager;
pub use persistence::PersistenceManager;
pub use secrets::SecretsManager;
pub use kubernetes::K8sManager;
pub use cloud::{AwsManager, AwsConfig};
pub use telemetry::TracingManager;
pub use logging::LoggingManager;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Unique identifier for Cloud resources
pub type TenantId = String;
pub type SessionId = String;
pub type UserId = String;

/// Cloud Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// Default tenant configuration
    pub default_tenant_config: TenantConfig,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Maximum sessions per tenant
    pub max_sessions_per_tenant: usize,
    /// API gateway configuration
    pub gateway_config: GatewayConfig,
    /// Enable multi-tenancy for shared resources
    pub multi_tenancy_enabled: bool,
    /// Default rate limits for external APIs
    pub default_rate_limits: RateLimitConfig,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            default_tenant_config: TenantConfig::default(),
            session_timeout: 3600, // 1 hour
            max_sessions_per_tenant: 100,
            gateway_config: GatewayConfig::default(),
            multi_tenancy_enabled: true,
            default_rate_limits: RateLimitConfig::default(),
        }
    }
}

/// Rate limit configuration for cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Requests per hour
    pub requests_per_hour: u32,
    /// Burst capacity
    pub burst_capacity: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_capacity: 10,
        }
    }
}

/// CloudResourceManager coordinates access to external infrastructure
#[derive(Debug)]
pub struct CloudResourceManager {
    config: CloudConfig,
    tenant_manager: TenantManager,
    session_manager: SessionManager,
    gateway: ApiGateway,
    billing: BillingManager,
    persistence: PersistenceManager,
    secrets: SecretsManager,
    k8s: Option<K8sManager>,
    aws: Option<AwsManager>,
    tracing_manager: TracingManager,
    logging_manager: LoggingManager,
}

impl CloudResourceManager {
    /// Create a new cloud resource manager
    pub fn new(config: CloudConfig) -> Self {
        let tenant_manager = TenantManager::new(config.default_tenant_config.clone());
        let session_manager = SessionManager::new(config.session_timeout);
        let gateway = ApiGateway::new(config.gateway_config.clone());
        let billing = BillingManager::new();
        let persistence = PersistenceManager::new("data/cloud");
        let secrets = SecretsManager::new();
        let tracing_manager = TracingManager::new("tos-cloud", "http://localhost:4317");
        let logging_manager = LoggingManager::new();
        
        Self {
            config,
            tenant_manager,
            session_manager,
            gateway,
            billing,
            persistence,
            secrets,
            k8s: None,
            aws: None,
            tracing_manager,
            logging_manager,
        }
    }
    
    /// Initialize the Cloud infrastructure
    pub async fn initialize(&mut self) -> SaasResult<()> {
        tracing::info!("Initializing Cloud Resource infrastructure");
        
        // Initialize tenant manager
        self.tenant_manager.initialize().await?;
        
        // Initialize session manager
        self.session_manager.initialize().await?;
        
        // Initialize API gateway
        self.gateway.initialize().await?;
        
        // Initialize persistence
        self.persistence.initialize().await?;
        
        // Initialize observability
        self.tracing_manager.initialize()?;
        self.logging_manager.initialize()?;
        
        tracing::info!("Cloud Resource infrastructure initialized");
        Ok(())
    }
    
    /// Shutdown the Cloud infrastructure
    pub async fn shutdown(&mut self) -> SaasResult<()> {
        tracing::info!("Shutting down Cloud Resource infrastructure");
        
        // Close all sessions
        self.session_manager.close_all_sessions().await?;
        
        // Deactivate all tenants
        self.tenant_manager.deactivate_all().await?;
        
        // Shutdown gateway
        self.gateway.shutdown().await?;
        
        tracing::info!("Cloud Resource infrastructure shutdown complete");
        Ok(())
    }
    
    /// Access tenant manager
    pub fn tenant_manager(&self) -> &TenantManager {
        &self.tenant_manager
    }
    
    /// Access tenant manager mutably
    pub fn tenant_manager_mut(&mut self) -> &mut TenantManager {
        &mut self.tenant_manager
    }
    
    /// Access session manager
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }
    
    /// Access session manager mutably
    pub fn session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }
    
    /// Access API gateway
    pub fn gateway(&self) -> &ApiGateway {
        &self.gateway
    }
    
    /// Access gateway mutably
    pub fn gateway_mut(&mut self) -> &mut ApiGateway {
        &mut self.gateway
    }
    
    /// Get configuration
    pub fn config(&self) -> &CloudConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: CloudConfig) {
        self.config = config;
    }

    /// Access billing manager
    pub fn billing(&self) -> &BillingManager {
        &self.billing
    }

    /// Access persistence manager
    pub fn persistence(&self) -> &PersistenceManager {
        &self.persistence
    }

    /// Access secrets manager
    pub fn secrets(&self) -> &SecretsManager {
        &self.secrets
    }

    /// Access K8s manager
    pub fn k8s(&self) -> Option<&K8sManager> {
        self.k8s.as_ref()
    }

    /// Access Cloud manager
    pub fn aws(&self) -> Option<&AwsManager> {
        self.aws.as_ref()
    }

    /// Access tracing manager
    pub fn tracing(&self) -> &TracingManager {
        &self.tracing_manager
    }

    /// Access logging manager
    pub fn logging(&self) -> &LoggingManager {
        &self.logging_manager
    }
}

/// SaaS errors
#[derive(Debug)]
pub enum SaasError {
    Tenant(String),
    Session(String),
    Gateway(String),
    Authentication(String),
    Authorization(String),
    RateLimitExceeded,
    NotFound(String),
    Io(std::io::Error),
}

impl std::fmt::Display for SaasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tenant(msg) => write!(f, "Tenant error: {}", msg),
            Self::Session(msg) => write!(f, "Session error: {}", msg),
            Self::Gateway(msg) => write!(f, "Gateway error: {}", msg),
            Self::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            Self::Authorization(msg) => write!(f, "Authorization error: {}", msg),
            Self::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            Self::NotFound(msg) => write!(f, "Resource not found: {}", msg),
            Self::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for SaasError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SaasError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Result type for SaaS operations
pub type SaasResult<T> = Result<T, SaasError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cloud_config_default() {
        let config = CloudConfig::default();
        assert_eq!(config.session_timeout, 3600);
        assert_eq!(config.max_sessions_per_tenant, 100);
        assert!(config.multi_tenancy_enabled);
    }
    
    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert_eq!(config.requests_per_hour, 1000);
        assert_eq!(config.burst_capacity, 10);
    }
    
    #[tokio::test]
    async fn test_cloud_manager_creation() {
        let config = CloudConfig::default();
        let manager = CloudResourceManager::new(config);
        
        assert!(manager.tenant_manager().list_tenants().is_empty());
        assert!(manager.session_manager().list_sessions().is_empty());
    }
}
