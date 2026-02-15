//! Phase 16: Container Strategy & SaaS Architecture
//!
//! Provides containerization support for TOS sectors, modules, and
//! multi-tenant SaaS deployments. Supports Docker, Podman, and
//! Kubernetes backends.

pub mod runtime;
pub mod image;
pub mod network;
pub mod sector;
pub mod security;
pub mod metrics;

pub use runtime::{ContainerRuntime, ContainerBackend, RestartPolicy, SecurityOptions, MockRuntime};
pub use image::{ImageManager, ImageBuilder, ImageInfo};
pub use network::{ContainerNetwork, NetworkConfig, PortMapping, Protocol, NetworkDriver};
pub use sector::{SectorContainer, SectorContainerManager};
pub use security::{SecurityPolicy, SeccompProfile, Capability};
pub use metrics::{ContainerMetrics, ResourceUsage};

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Unique identifier for containers
pub type ContainerId = String;

/// Container status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Creating,
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
    Unknown,
}

impl ContainerStatus {
    /// Check if container is active (running or paused)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
    
    /// Check if container has exited
    pub fn has_exited(&self) -> bool {
        matches!(self, Self::Exited | Self::Dead)
    }
}

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: ContainerId,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub labels: HashMap<String, String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMount>,
    pub resource_limits: ResourceLimits,
}

/// Volume mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub source: PathBuf,
    pub target: PathBuf,
    pub read_only: bool,
    pub volume_type: VolumeType,
}

/// Volume type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeType {
    Bind,
    Volume,
    Tmpfs,
}

/// Resource limits for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit (0.0 - 1.0 = 1 core, 2.0 = 2 cores, etc.)
    pub cpu_limit: Option<f64>,
    /// Memory limit in bytes
    pub memory_limit: Option<u64>,
    /// GPU device access
    pub gpu_devices: Vec<String>,
    /// I/O weight (10-1000)
    pub io_weight: Option<u16>,
    /// PIDs limit
    pub pids_limit: Option<i64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_limit: None,
            memory_limit: None,
            gpu_devices: Vec::new(),
            io_weight: None,
            pids_limit: None,
        }
    }
}

/// Environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub secret: bool, // If true, value is a secret reference
}

/// Container health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub command: Vec<String>,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retries: u32,
    pub start_period_seconds: u64,
}

/// Container configuration
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Container name (must be unique)
    pub name: String,
    /// Image to use
    pub image: String,
    /// Command to run (optional, uses image default if not set)
    pub command: Option<Vec<String>>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Network mode (bridge, host, none, or custom network name)
    pub network_mode: String,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Working directory
    pub working_dir: Option<String>,
    /// User to run as (uid:gid or username)
    pub user: Option<String>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Restart policy
    pub restart_policy: crate::containers::runtime::RestartPolicy,
    /// Health check configuration
    pub health_check: Option<HealthCheck>,
    /// Security options
    pub security_options: crate::containers::runtime::SecurityOptions,
    /// Stdin attached
    pub stdin: bool,
    /// Stdout attached
    pub stdout: bool,
    /// Stderr attached
    pub stderr: bool,
    /// TTY enabled
    pub tty: bool,
    /// Auto-remove on exit
    pub auto_remove: bool,
    /// Privileged mode
    pub privileged: bool,
    /// Capabilities to add
    pub cap_add: Vec<String>,
    /// Capabilities to drop
    pub cap_drop: Vec<String>,
    /// Read-only root filesystem
    pub read_only: bool,
    /// IPC namespace mode
    pub ipc_mode: Option<String>,
    /// PID namespace mode
    pub pid_mode: Option<String>,
    /// UTS namespace mode
    pub uts_mode: Option<String>,
    /// cgroup namespace mode
    pub cgroupns_mode: Option<String>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            image: String::new(),
            command: None,
            env: HashMap::new(),
            volumes: Vec::new(),
            ports: Vec::new(),
            network_mode: "bridge".to_string(),
            resource_limits: ResourceLimits::default(),
            working_dir: None,
            user: None,
            labels: HashMap::new(),
            restart_policy: crate::containers::runtime::RestartPolicy::No,
            health_check: None,
            security_options: crate::containers::runtime::SecurityOptions::default(),
            stdin: false,
            stdout: true,
            stderr: true,
            tty: false,
            auto_remove: false,
            privileged: false,
            cap_add: Vec::new(),
            cap_drop: vec!["ALL".to_string()],
            read_only: true,
            ipc_mode: None,
            pid_mode: None,
            uts_mode: None,
            cgroupns_mode: None,
        }
    }
}

/// Container errors
#[derive(Debug)]
pub enum ContainerError {
    Runtime(String),
    ImageNotFound(String),
    ContainerNotFound(ContainerId),
    ContainerExists(String),
    Network(String),
    Security(String),
    ResourceLimit(String),
    Io(std::io::Error),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            Self::ImageNotFound(img) => write!(f, "Image not found: {}", img),
            Self::ContainerNotFound(id) => write!(f, "Container not found: {}", id),
            Self::ContainerExists(name) => write!(f, "Container already exists: {}", name),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Security(msg) => write!(f, "Security policy violation: {}", msg),
            Self::ResourceLimit(msg) => write!(f, "Resource limit exceeded: {}", msg),
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ContainerError {}

impl From<std::io::Error> for ContainerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for ContainerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e)
    }
}

/// Result type for container operations
pub type ContainerResult<T> = Result<T, ContainerError>;

/// Container manager coordinates all container operations
#[derive(Debug)]
pub struct ContainerManager {
    runtime: Box<dyn ContainerRuntime>,
    image_manager: ImageManager,
    network_manager: ContainerNetwork,
    security_policy: SecurityPolicy,
}

impl ContainerManager {
    /// Create a new container manager with the specified backend
    pub async fn new(backend: ContainerBackend) -> ContainerResult<Self> {
        let runtime = runtime::create_runtime(backend).await?;
        let image_manager = ImageManager::new(runtime.clone_box());
        let network_manager = ContainerNetwork::new(runtime.clone_box());
        let security_policy = SecurityPolicy::default();
        
        Ok(Self {
            runtime,
            image_manager,
            network_manager,
            security_policy,
        })
    }
    
    /// Create and start a container
    pub async fn create_container(&self, config: ContainerConfig) -> ContainerResult<ContainerInfo> {
        // Validate security policy
        self.security_policy.validate(&config)?;
        
        // Pull image if needed
        if !self.image_manager.image_exists(&config.image).await? {
            self.image_manager.pull_image(&config.image).await?;
        }
        
        // Create container
        let info = self.runtime.create_container(config).await?;
        
        tracing::info!("Created container: {} ({})", info.name, info.id);
        
        Ok(info)
    }
    
    /// Start a container
    pub async fn start_container(&self, id: &ContainerId) -> ContainerResult<()> {
        self.runtime.start_container(id).await?;
        tracing::info!("Started container: {}", id);
        Ok(())
    }
    
    /// Stop a container
    pub async fn stop_container(&self, id: &ContainerId, timeout: u64) -> ContainerResult<()> {
        self.runtime.stop_container(id, timeout).await?;
        tracing::info!("Stopped container: {}", id);
        Ok(())
    }
    
    /// Remove a container
    pub async fn remove_container(&self, id: &ContainerId, force: bool) -> ContainerResult<()> {
        self.runtime.remove_container(id, force).await?;
        tracing::info!("Removed container: {}", id);
        Ok(())
    }
    
    /// Get container info
    pub async fn get_container(&self, id: &ContainerId) -> ContainerResult<ContainerInfo> {
        self.runtime.get_container(id).await
    }
    
    /// List all containers
    pub async fn list_containers(&self, all: bool) -> ContainerResult<Vec<ContainerInfo>> {
        self.runtime.list_containers(all).await
    }
    
    /// Execute command in container
    pub async fn exec_command(
        &self,
        id: &ContainerId,
        command: &[String],
        tty: bool,
    ) -> ContainerResult<(String, i32)> {
        self.runtime.exec_command(id, command, tty).await
    }
    
    /// Get container logs
    pub async fn get_logs(
        &self,
        id: &ContainerId,
        tail: Option<usize>,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ContainerResult<String> {
        self.runtime.get_logs(id, tail, since).await
    }
    
    /// Get container stats
    pub async fn get_stats(&self, id: &ContainerId) -> ContainerResult<ContainerMetrics> {
        self.runtime.get_stats(id).await
    }
    
    /// Pause a container
    pub async fn pause_container(&self, id: &ContainerId) -> ContainerResult<()> {
        self.runtime.pause_container(id).await?;
        tracing::info!("Paused container: {}", id);
        Ok(())
    }
    
    /// Unpause a container
    pub async fn unpause_container(&self, id: &ContainerId) -> ContainerResult<()> {
        self.runtime.unpause_container(id).await?;
        tracing::info!("Unpaused container: {}", id);
        Ok(())
    }
    
    /// Access to image manager
    pub fn image_manager(&self) -> &ImageManager {
        &self.image_manager
    }
    
    /// Access to network manager
    pub fn network_manager(&self) -> &ContainerNetwork {
        &self.network_manager
    }
    
    /// Update security policy
    pub fn set_security_policy(&mut self, policy: SecurityPolicy) {
        self.security_policy = policy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_container_status() {
        assert!(ContainerStatus::Running.is_active());
        assert!(ContainerStatus::Paused.is_active());
        assert!(!ContainerStatus::Exited.is_active());
        assert!(ContainerStatus::Exited.has_exited());
    }
    
    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.cpu_limit.is_none());
        assert!(limits.memory_limit.is_none());
        assert!(limits.gpu_devices.is_empty());
    }
    
    #[test]
    fn test_volume_mount() {
        let mount = VolumeMount {
            source: PathBuf::from("/host/data"),
            target: PathBuf::from("/container/data"),
            read_only: true,
            volume_type: VolumeType::Bind,
        };
        assert!(mount.read_only);
        assert_eq!(mount.volume_type, VolumeType::Bind);
    }
}
