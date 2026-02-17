//! Phase 16 Week 2: Session Management
//!
//! User session lifecycle management with authentication, authorization,
//! and session state tracking.

use super::{SessionId, TenantId, UserId, SaasResult, SaasError};
use crate::containers::{ContainerManager, ContainerBackend};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono;

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is being created
    Creating,
    /// Session is active
    Active,
    /// Session is idle (no activity)
    Idle,
    /// Session is expiring soon
    Expiring,
    /// Session is being terminated
    Terminating,
    /// Session has ended
    Ended,
}

impl SessionStatus {
    /// Check if session is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Idle | Self::Expiring)
    }
    
    /// Check if session can be used
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Creating | Self::Active | Self::Idle)
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout in seconds
    pub timeout_seconds: u64,
    /// Idle timeout in seconds
    pub idle_timeout_seconds: u64,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Enable persistent session
    pub persistent: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Allowed sector types
    pub allowed_sector_types: Vec<String>,
    /// Custom environment variables
    pub env_vars: HashMap<String, String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 3600, // 1 hour
            idle_timeout_seconds: 300, // 5 minutes
            max_connections: 5,
            persistent: true,
            auto_save_interval: 60,
            allowed_sector_types: vec!["terminal".to_string()],
            env_vars: HashMap::new(),
        }
    }
}

/// Session information
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// Unique session ID
    pub id: SessionId,
    /// User ID
    pub user_id: UserId,
    /// Tenant ID
    pub tenant_id: TenantId,
    /// Session status
    pub status: SessionStatus,
    /// Session configuration
    pub config: SessionConfig,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// End timestamp (if ended)
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    /// IP address
    pub ip_address: String,
    /// User agent
    pub user_agent: String,
    /// Active sector IDs
    pub active_sectors: Vec<String>,
    /// Connection count
    pub connection_count: usize,
    /// Container manager for this session
    #[serde(skip)]
    pub container_manager: Option<ContainerManager>,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            user_id: self.user_id.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
            config: self.config.clone(),
            created_at: self.created_at,
            last_activity: self.last_activity,
            expires_at: self.expires_at,
            ended_at: self.ended_at,
            ip_address: self.ip_address.clone(),
            user_agent: self.user_agent.clone(),
            active_sectors: self.active_sectors.clone(),
            connection_count: self.connection_count,
            container_manager: None, // ContainerManager is not Clone
        }
    }
}

impl Session {
    /// Create a new session
    pub async fn new(
        id: SessionId,
        user_id: UserId,
        tenant_id: TenantId,
        config: SessionConfig,
        ip_address: String,
        user_agent: String,
    ) -> SaasResult<Self> {
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::seconds(config.timeout_seconds as i64);
        
        Ok(Self {
            id,
            user_id,
            tenant_id,
            status: SessionStatus::Creating,
            config,
            created_at: now,
            last_activity: now,
            expires_at,
            ended_at: None,
            ip_address,
            user_agent,
            active_sectors: Vec::new(),
            connection_count: 0,
            container_manager: None,
        })
    }
    
    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
    
    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> i64 {
        let now = chrono::Utc::now();
        if now > self.expires_at {
            0
        } else {
            (self.expires_at - now).num_seconds()
        }
    }
    
    /// Update last activity
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now();
        
        // Extend expiration if not expiring soon
        if self.status != SessionStatus::Expiring {
            self.expires_at = chrono::Utc::now() + 
                chrono::Duration::seconds(self.config.timeout_seconds as i64);
        }
    }
    
    /// Check if session is idle
    pub fn is_idle(&self) -> bool {
        let idle_duration = chrono::Utc::now() - self.last_activity;
        idle_duration.num_seconds() > self.config.idle_timeout_seconds as i64
    }
    
    /// Add active sector
    pub fn add_sector(&mut self, sector_id: String) -> SaasResult<()> {
        if !self.active_sectors.contains(&sector_id) {
            self.active_sectors.push(sector_id);
        }
        self.touch();
        Ok(())
    }
    
    /// Remove active sector
    pub fn remove_sector(&mut self, sector_id: &str) {
        self.active_sectors.retain(|id| id != sector_id);
        self.touch();
    }
    
    /// Increment connection count
    pub fn increment_connections(&mut self) -> SaasResult<()> {
        if self.connection_count >= self.config.max_connections {
            return Err(SaasError::Session(format!(
                "Maximum connections ({}) reached for session {}", 
                self.config.max_connections, self.id
            )));
        }
        self.connection_count += 1;
        self.touch();
        Ok(())
    }
    
    /// Decrement connection count
    pub fn decrement_connections(&mut self) {
        if self.connection_count > 0 {
            self.connection_count -= 1;
        }
        self.touch();
    }
    
    /// Initialize container manager for session
    pub async fn initialize_container_manager(
        &mut self,
        backend: ContainerBackend,
    ) -> SaasResult<()> {
        let manager = ContainerManager::new(backend).await
            .map_err(|e| SaasError::Session(format!("Failed to create container manager: {}", e)))?;
        
        self.container_manager = Some(manager);
        self.status = SessionStatus::Active;
        Ok(())
    }
    
    /// End the session
    pub fn end(&mut self, reason: &str) {
        self.status = SessionStatus::Ended;
        self.ended_at = Some(chrono::Utc::now());
        tracing::info!("Session {} ended: {}", self.id, reason);
    }
}

/// Session manager handles session lifecycle
#[derive(Debug)]
pub struct SessionManager {
    sessions: std::sync::Arc<std::sync::Mutex<HashMap<SessionId, Session>>>,
    timeout_seconds: u64,
    user_sessions: std::sync::Arc<std::sync::Mutex<HashMap<UserId, Vec<SessionId>>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            sessions: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
            timeout_seconds,
            user_sessions: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
    
    /// Initialize session manager
    pub async fn initialize(&mut self) -> SaasResult<()> {
        tracing::info!("Initializing session manager");
        // In real implementation, restore persistent sessions
        Ok(())
    }
    
    /// Create a new session
    pub async fn create_session(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        config: Option<SessionConfig>,
        ip_address: String,
        user_agent: String,
    ) -> SaasResult<Session> {
        let id = format!("session-{}", uuid::Uuid::new_v4());
        let mut config = config.unwrap_or_else(|| SessionConfig {
            timeout_seconds: self.timeout_seconds,
            ..Default::default()
        });
        
        // Ensure timeout matches manager setting
        config.timeout_seconds = self.timeout_seconds;
        
        let mut session = Session::new(
            id.clone(),
            user_id.clone(),
            tenant_id,
            config,
            ip_address,
            user_agent,
        ).await?;
        
        // Initialize container manager
        session.initialize_container_manager(ContainerBackend::Mock).await?;
        
        // Store session
        self.sessions.lock().unwrap().insert(id.clone(), session.clone());
        
        // Track user session
        self.user_sessions.lock().unwrap()
            .entry(user_id.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        tracing::info!("Created session: {} for user: {}", id, user_id);
        Ok(session)
    }
    
    /// Get session by ID
    pub fn get_session(&self, id: &SessionId) -> SaasResult<Session> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(id)
            .cloned()
            .ok_or_else(|| SaasError::NotFound(format!("Session {} not found", id)))?;
        
        // Check if expired
        if session.is_expired() && session.status.is_active() {
            drop(sessions);
            self.terminate_session(id, "Session expired")?;
            return Err(SaasError::Session("Session expired".to_string()));
        }
        
        Ok(session)
    }
    
    /// Update session
    pub fn update_session(&self, session: Session) -> SaasResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if !sessions.contains_key(&session.id) {
            return Err(SaasError::NotFound(format!("Session {} not found", session.id)));
        }
        
        sessions.insert(session.id.clone(), session);
        Ok(())
    }
    
    /// Terminate a session
    pub fn terminate_session(&self, id: &SessionId, reason: &str) -> SaasResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(id)
            .ok_or_else(|| SaasError::NotFound(format!("Session {} not found", id)))?;
        
        session.status = SessionStatus::Terminating;
        
        // In real implementation:
        // 1. Close all connections
        // 2. Save session state
        // 3. Stop session containers
        
        session.end(reason);
        
        // Remove from user sessions
        let mut user_sessions = self.user_sessions.lock().unwrap();
        if let Some(user_session_ids) = user_sessions.get_mut(&session.user_id) {
            user_session_ids.retain(|sid| sid != id);
        }
        
        tracing::info!("Terminated session {}: {}", id, reason);
        Ok(())
    }
    
    /// List all sessions
    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.lock().unwrap().values().cloned().collect()
    }
    
    /// List active sessions
    pub fn list_active_sessions(&self) -> Vec<Session> {
        self.sessions.lock().unwrap().values()
            .filter(|s| s.status.is_active())
            .cloned()
            .collect()
    }
    
    /// List sessions for a user
    pub fn list_user_sessions(&self, user_id: &UserId) -> Vec<Session> {
        let user_sessions = self.user_sessions.lock().unwrap();
        let session_ids = user_sessions.get(user_id);
        
        if let Some(ids) = session_ids {
            let sessions = self.sessions.lock().unwrap();
            ids.iter()
                .filter_map(|id| sessions.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// List sessions for a tenant
    pub fn list_tenant_sessions(&self, tenant_id: &TenantId) -> Vec<Session> {
        self.sessions.lock().unwrap().values()
            .filter(|s| &s.tenant_id == tenant_id)
            .cloned()
            .collect()
    }
    
    /// Close all sessions (for shutdown)
    pub async fn close_all_sessions(&self) -> SaasResult<()> {
        let session_ids: Vec<_> = self.sessions.lock().unwrap().keys().cloned().collect();
        
        for id in session_ids {
            self.terminate_session(&id, "System shutdown")?;
        }
        
        Ok(())
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> SaasResult<usize> {
        let expired: Vec<_> = self.sessions.lock().unwrap().values()
            .filter(|s| s.is_expired() && s.status.is_active())
            .map(|s| s.id.clone())
            .collect();
        
        let count = expired.len();
        for id in expired {
            let _ = self.terminate_session(&id, "Expired");
        }
        
        tracing::info!("Cleaned up {} expired sessions", count);
        Ok(count)
    }
    
    /// Touch session (update last activity)
    pub fn touch_session(&self, id: &SessionId) -> SaasResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        let session = sessions.get_mut(id)
            .ok_or_else(|| SaasError::NotFound(format!("Session {} not found", id)))?;
        
        session.touch();
        Ok(())
    }
    
    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }
    
    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.lock().unwrap().values()
            .filter(|s| s.status.is_active())
            .count()
    }
}

/// Session builder for fluent API
#[derive(Debug)]
pub struct SessionBuilder {
    user_id: UserId,
    tenant_id: TenantId,
    config: SessionConfig,
    ip_address: String,
    user_agent: String,
}

impl SessionBuilder {
    /// Create a new session builder
    pub fn new(
        user_id: impl Into<UserId>,
        tenant_id: impl Into<TenantId>,
        ip_address: impl Into<String>,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            tenant_id: tenant_id.into(),
            config: SessionConfig::default(),
            ip_address: ip_address.into(),
            user_agent: user_agent.into(),
        }
    }
    
    /// Set timeout
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }
    
    /// Set idle timeout
    pub fn idle_timeout_seconds(mut self, seconds: u64) -> Self {
        self.config.idle_timeout_seconds = seconds;
        self
    }
    
    /// Set max connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }
    
    /// Set persistent
    pub fn persistent(mut self, persistent: bool) -> Self {
        self.config.persistent = persistent;
        self
    }
    
    /// Add allowed sector type
    pub fn allow_sector_type(mut self, sector_type: impl Into<String>) -> Self {
        self.config.allowed_sector_types.push(sector_type.into());
        self
    }
    
    /// Add environment variable
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env_vars.insert(key.into(), value.into());
        self
    }
    
    /// Build the session
    pub async fn build(self, manager: &SessionManager) -> SaasResult<Session> {
        manager.create_session(
            self.user_id,
            self.tenant_id,
            Some(self.config),
            self.ip_address,
            self.user_agent,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_status() {
        assert!(SessionStatus::Active.is_active());
        assert!(SessionStatus::Idle.is_active());
        assert!(SessionStatus::Active.is_usable());
        assert!(!SessionStatus::Ended.is_active());
    }
    
    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.timeout_seconds, 3600);
        assert_eq!(config.idle_timeout_seconds, 300);
        assert_eq!(config.max_connections, 5);
        assert!(config.persistent);
    }
    
    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new(3600);
        let session = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        ).await.unwrap();
        
        assert_eq!(session.user_id, "user-123");
        assert_eq!(session.tenant_id, "tenant-456");
        assert!(session.status.is_active());
        assert!(session.container_manager.is_some());
        // Allow for slight timing differences
        assert!(session.remaining_seconds() >= 3598 && session.remaining_seconds() <= 3600);
    }
    
    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = SessionManager::new(3600);
        let session = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        ).await.unwrap();
        
        let id = session.id.clone();
        
        // Touch session
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.touch_session(&id).unwrap();
        
        let session = manager.get_session(&id).unwrap();
        assert!(session.last_activity > session.created_at);
        
        // Terminate
        manager.terminate_session(&id, "Test termination").unwrap();
        let session = manager.get_session(&id).unwrap();
        assert_eq!(session.status, SessionStatus::Ended);
        assert!(session.ended_at.is_some());
    }
    
    #[tokio::test]
    async fn test_session_connections() {
        let manager = SessionManager::new(3600);
        let mut session = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        ).await.unwrap();
        
        // Increment connections
        for i in 0..5 {
            assert!(session.increment_connections().is_ok());
            assert_eq!(session.connection_count, i + 1);
        }
        
        // Max connections reached
        assert!(session.increment_connections().is_err());
        
        // Decrement
        session.decrement_connections();
        assert_eq!(session.connection_count, 4);
    }
    
    #[tokio::test]
    async fn test_session_sectors() {
        let manager = SessionManager::new(3600);
        let mut session = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        ).await.unwrap();
        
        // Add sectors
        session.add_sector("sector-1".to_string()).unwrap();
        session.add_sector("sector-2".to_string()).unwrap();
        assert_eq!(session.active_sectors.len(), 2);
        
        // Remove sector
        session.remove_sector("sector-1");
        assert_eq!(session.active_sectors.len(), 1);
        assert!(!session.active_sectors.contains(&"sector-1".to_string()));
    }
    
    #[tokio::test]
    async fn test_session_builder() {
        let manager = SessionManager::new(7200);
        let session = SessionBuilder::new("user-123", "tenant-456", "192.168.1.1", "TestAgent")
            .timeout_seconds(7200)
            .idle_timeout_seconds(600)
            .max_connections(10)
            .persistent(false)
            .allow_sector_type("browser")
            .env_var("KEY", "value")
            .build(&manager).await.unwrap();
        
        assert_eq!(session.config.timeout_seconds, 7200);
        assert_eq!(session.config.idle_timeout_seconds, 600);
        assert_eq!(session.config.max_connections, 10);
        assert!(!session.config.persistent);
        assert!(session.config.allowed_sector_types.contains(&"browser".to_string()));
        assert_eq!(session.config.env_vars.get("KEY"), Some(&"value".to_string()));
    }
    
    #[tokio::test]
    async fn test_list_user_sessions() {
        let manager = SessionManager::new(3600);
        
        // Create sessions for user-123
        let session1 = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        ).await.unwrap();
        
        let session2 = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.2".to_string(),
            "Chrome/90.0".to_string(),
        ).await.unwrap();
        
        // Create session for different user
        let _session3 = manager.create_session(
            "user-999".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.3".to_string(),
            "Safari/14.0".to_string(),
        ).await.unwrap();
        
        let user_sessions = manager.list_user_sessions(&"user-123".to_string());
        assert_eq!(user_sessions.len(), 2);
        assert!(user_sessions.iter().any(|s| s.id == session1.id));
        assert!(user_sessions.iter().any(|s| s.id == session2.id));
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let manager = SessionManager::new(0); // Immediate expiration
        
        let session = manager.create_session(
            "user-123".to_string(),
            "tenant-456".to_string(),
            None,
            "192.168.1.1".to_string(),
            "Mozilla/5.0".to_string(),
        ).await.unwrap();
        
        // Session should be expired immediately
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let cleaned = manager.cleanup_expired_sessions().unwrap();
        assert_eq!(cleaned, 1);
        
        let session = manager.get_session(&session.id).unwrap();
        assert_eq!(session.status, SessionStatus::Ended);
    }
}
