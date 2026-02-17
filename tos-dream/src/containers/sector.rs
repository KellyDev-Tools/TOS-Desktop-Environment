//! Phase 16: Sector Containerization
//!
//! Manages containerized sectors, providing isolation and resource management
//! for TOS workspaces.

use super::{
    ContainerManager, ContainerConfig, ContainerResult, ContainerError, ContainerId, VolumeMount, VolumeType, ResourceLimits, PortMapping, Protocol,
    NetworkConfig, NetworkDriver,
};
use crate::ContainerBackend;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Sector container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorContainerConfig {
    /// Sector ID
    pub sector_id: String,
    /// Sector name
    pub name: String,
    /// Base image to use
    pub base_image: String,
    /// TOS version
    pub tos_version: String,
    /// Hostname for the container
    pub hostname: String,
    /// Domain name
    pub domainname: String,
    /// User to run as
    pub user: String,
    /// Working directory
    pub working_dir: String,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Volume mounts
    pub volumes: Vec<SectorVolume>,
    /// Port mappings
    pub ports: Vec<PortMapping>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Auto-start on creation
    pub auto_start: bool,
    /// Restart policy
    pub restart_policy: SectorRestartPolicy,
    /// Health check configuration
    pub health_check: Option<SectorHealthCheck>,
    /// Security options
    pub security: SectorSecurityOptions,
    /// Network configuration
    pub network: SectorNetworkConfig,
    /// Labels
    pub labels: HashMap<String, String>,
}

impl Default for SectorContainerConfig {
    fn default() -> Self {
        Self {
            sector_id: String::new(),
            name: "sector".to_string(),
            base_image: "tos/sector-base:latest".to_string(),
            tos_version: "0.1.0".to_string(),
            hostname: "tos-sector".to_string(),
            domainname: "local".to_string(),
            user: "tos".to_string(),
            working_dir: "/home/tos".to_string(),
            env: HashMap::new(),
            volumes: Vec::new(),
            ports: vec![
                PortMapping {
                    host_port: 0, // Dynamic allocation
                    container_port: 8080,
                    protocol: Protocol::Tcp,
                    host_ip: None,
                },
            ],
            resource_limits: ResourceLimits {
                cpu_limit: Some(2.0),
                memory_limit: Some(4 * 1024 * 1024 * 1024), // 4GB
                ..Default::default()
            },
            auto_start: true,
            restart_policy: SectorRestartPolicy::UnlessStopped,
            health_check: Some(SectorHealthCheck::default()),
            security: SectorSecurityOptions::default(),
            network: SectorNetworkConfig::default(),
            labels: HashMap::new(),
        }
    }
}

/// Sector volume configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorVolume {
    /// Volume name or host path
    pub source: String,
    /// Container path
    pub target: String,
    /// Volume type
    pub volume_type: VolumeType,
    /// Read-only
    pub read_only: bool,
    /// Volume driver options
    pub driver_opts: HashMap<String, String>,
}

/// Sector restart policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectorRestartPolicy {
    No,
    OnFailure,
    Always,
    UnlessStopped,
}

impl SectorRestartPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::No => "no",
            Self::OnFailure => "on-failure",
            Self::Always => "always",
            Self::UnlessStopped => "unless-stopped",
        }
    }
}

/// Sector health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorHealthCheck {
    /// Test command
    pub test: Vec<String>,
    /// Interval in seconds
    pub interval: u64,
    /// Timeout in seconds
    pub timeout: u64,
    /// Start period in seconds
    pub start_period: u64,
    /// Retries
    pub retries: u32,
    /// Disable health check
    pub disable: bool,
}

impl Default for SectorHealthCheck {
    fn default() -> Self {
        Self {
            test: vec!["CMD".to_string(), "tos".to_string(), "health".to_string()],
            interval: 30,
            timeout: 10,
            start_period: 60,
            retries: 3,
            disable: false,
        }
    }
}

/// Sector security options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorSecurityOptions {
    /// Read-only root filesystem
    pub read_only: bool,
    /// No new privileges
    pub no_new_privileges: bool,
    /// Drop all capabilities
    pub drop_all_capabilities: bool,
    /// Add specific capabilities
    pub add_capabilities: Vec<String>,
    /// Seccomp profile
    pub seccomp_profile: Option<String>,
    /// AppArmor profile
    pub apparmor_profile: Option<String>,
    /// SELinux options
    pub selinux_options: Vec<String>,
    /// Security options
    pub security_opts: Vec<String>,
}

impl Default for SectorSecurityOptions {
    fn default() -> Self {
        Self {
            read_only: true,
            no_new_privileges: true,
            drop_all_capabilities: true,
            add_capabilities: vec![
                "CHOWN".to_string(),
                "DAC_OVERRIDE".to_string(),
                "FSETID".to_string(),
                "FOWNER".to_string(),
                "MKNOD".to_string(),
                "NET_RAW".to_string(),
                "SETGID".to_string(),
                "SETUID".to_string(),
                "SETFCAP".to_string(),
                "SETPCAP".to_string(),
                "NET_BIND_SERVICE".to_string(),
                "SYS_CHROOT".to_string(),
                "KILL".to_string(),
                "AUDIT_WRITE".to_string(),
            ],
            seccomp_profile: None,
            apparmor_profile: Some("tos-sector".to_string()),
            selinux_options: Vec::new(),
            security_opts: vec!["no-new-privileges:true".to_string()],
        }
    }
}

/// Sector network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorNetworkConfig {
    /// Network mode (bridge, host, none, or custom network name)
    pub mode: String,
    /// Use custom network
    pub use_custom_network: bool,
    /// Custom network name
    pub custom_network_name: Option<String>,
    /// Static IP address
    pub static_ip: Option<String>,
    /// DNS servers
    pub dns: Vec<String>,
    /// DNS search domains
    pub dns_search: Vec<String>,
    /// Extra hosts
    pub extra_hosts: HashMap<String, String>,
    /// Exposed ports
    pub exposed_ports: Vec<u16>,
}

impl Default for SectorNetworkConfig {
    fn default() -> Self {
        Self {
            mode: "bridge".to_string(),
            use_custom_network: true,
            custom_network_name: None,
            static_ip: None,
            dns: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            dns_search: vec!["local".to_string()],
            extra_hosts: HashMap::new(),
            exposed_ports: vec![8080, 22],
        }
    }
}

/// Sector container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorContainer {
    /// Container ID
    pub container_id: ContainerId,
    /// Sector ID
    pub sector_id: String,
    /// Configuration
    pub config: SectorContainerConfig,
    /// Status
    pub status: SectorContainerStatus,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Started timestamp
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// IP address
    pub ip_address: Option<String>,
    /// Host port mappings
    pub host_ports: HashMap<u16, u16>,
    /// Volume paths
    pub volume_paths: HashMap<String, PathBuf>,
    /// Snapshot ID (if snapshotted)
    pub snapshot_id: Option<String>,
}

/// Sector container status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectorContainerStatus {
    Creating,
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Removing,
    Removed,
    Error,
}

impl SectorContainerStatus {
    /// Check if container is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
    
    /// Check if container can be started
    pub fn can_start(&self) -> bool {
        matches!(self, Self::Created | Self::Stopped | Self::Error)
    }
    
    /// Check if container can be stopped
    pub fn can_stop(&self) -> bool {
        matches!(self, Self::Running | Self::Paused)
    }
}

/// Sector container manager
#[derive(Debug)]
pub struct SectorContainerManager {
    container_manager: ContainerManager,
    sectors: std::sync::Mutex<HashMap<String, SectorContainer>>,
    data_root: PathBuf,
}

impl SectorContainerManager {
    /// Create a new sector container manager
    pub async fn new(backend: ContainerBackend) -> ContainerResult<Self> {
        let container_manager = ContainerManager::new(backend).await?;
        
        let data_root = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("tos/sector-containers");
        
        std::fs::create_dir_all(&data_root)?;
        
        Ok(Self {
            container_manager,
            sectors: std::sync::Mutex::new(HashMap::new()),
            data_root,
        })
    }
    
    /// Create a containerized sector
    pub async fn create_sector(
        &self,
        sector_config: SectorContainerConfig,
    ) -> ContainerResult<SectorContainer> {
        tracing::info!("Creating sector container: {}", sector_config.name);
        
        // Check if sector already exists
        if self.sectors.lock().unwrap().contains_key(&sector_config.sector_id) {
            return Err(ContainerError::Runtime(
                format!("Sector {} already exists", sector_config.sector_id)
            ));
        }
        
        // Create sector-specific network if using custom network
        let network_name = if sector_config.network.use_custom_network {
            let net_name = format!("tos-sector-{}", sector_config.sector_id);
            let net_config = NetworkConfig {
                name: net_name.clone(),
                driver: NetworkDriver::Bridge,
                subnet: format!("172.{}.0.0/16", rand::random::<u8>() % 200 + 20),
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("tos.sector.id".to_string(), sector_config.sector_id.clone());
                    labels
                },
                ..Default::default()
            };
            self.container_manager.network_manager().create_network(net_config).await?;
            Some(net_name)
        } else {
            None
        };
        
        // Prepare volumes
        let sector_data_dir = self.data_root.join(&sector_config.sector_id);
        std::fs::create_dir_all(&sector_data_dir)?;
        
        let mut volumes = Vec::new();
        let mut volume_paths = HashMap::new();
        
        // Add sector data volume
        let data_volume = VolumeMount {
            source: sector_data_dir.join("data"),
            target: PathBuf::from("/home/tos/data"),
            read_only: false,
            volume_type: VolumeType::Bind,
        };
        std::fs::create_dir_all(&data_volume.source)?;
        volumes.push(data_volume.clone());
        volume_paths.insert("data".to_string(), data_volume.source.clone());
        
        // Add config volume
        let config_volume = VolumeMount {
            source: sector_data_dir.join("config"),
            target: PathBuf::from("/home/tos/.config/tos"),
            read_only: false,
            volume_type: VolumeType::Bind,
        };
        std::fs::create_dir_all(&config_volume.source)?;
        volumes.push(config_volume.clone());
        volume_paths.insert("config".to_string(), config_volume.source.clone());
        
        // Add user-specified volumes
        for sector_vol in &sector_config.volumes {
            let vol = VolumeMount {
                source: if sector_vol.source.starts_with('/') {
                    PathBuf::from(&sector_vol.source)
                } else {
                    sector_data_dir.join(&sector_vol.source)
                },
                target: PathBuf::from(&sector_vol.target),
                read_only: sector_vol.read_only,
                volume_type: sector_vol.volume_type,
            };
            std::fs::create_dir_all(&vol.source)?;
            volumes.push(vol);
        }
        
        // Build container config
        let container_config = ContainerConfig {
            name: format!("tos-sector-{}", sector_config.sector_id),
            image: sector_config.base_image.clone(),
            command: Some(vec!["tos".to_string(), "sector".to_string(), "start".to_string()]),
            env: sector_config.env.clone(),
            volumes,
            ports: sector_config.ports.clone(),
            network_mode: network_name.unwrap_or_else(|| "bridge".to_string()),
            resource_limits: sector_config.resource_limits.clone(),
            working_dir: Some(sector_config.working_dir.clone()),
            user: Some(sector_config.user.clone()),
            labels: {
                let mut labels = sector_config.labels.clone();
                labels.insert("tos.sector.id".to_string(), sector_config.sector_id.clone());
                labels.insert("tos.sector.name".to_string(), sector_config.name.clone());
                labels.insert("tos.version".to_string(), sector_config.tos_version.clone());
                labels
            },
            restart_policy: super::RestartPolicy::UnlessStopped,
            health_check: sector_config.health_check.as_ref().map(|h| super::HealthCheck {
                command: h.test.clone(),
                interval_seconds: h.interval,
                timeout_seconds: h.timeout,
                retries: h.retries,
                start_period_seconds: h.start_period,
            }),
            security_options: super::SecurityOptions {
                seccomp_profile: sector_config.security.seccomp_profile.clone(),
                apparmor_profile: sector_config.security.apparmor_profile.clone(),
                selinux_options: sector_config.security.selinux_options.clone(),
                no_new_privileges: sector_config.security.no_new_privileges,
                security_opts: sector_config.security.security_opts.clone(),
            },
            stdin: true,
            stdout: true,
            stderr: true,
            tty: true,
            auto_remove: false,
            privileged: false,
            cap_add: sector_config.security.add_capabilities.clone(),
            cap_drop: if sector_config.security.drop_all_capabilities {
                vec!["ALL".to_string()]
            } else {
                Vec::new()
            },
            read_only: sector_config.security.read_only,
            ipc_mode: None,
            pid_mode: None,
            uts_mode: Some(format!("tos-sector-{}", sector_config.sector_id)),
            cgroupns_mode: Some("private".to_string()),
        };
        
        // Create container
        let container_info = self.container_manager.create_container(container_config).await?;
        
        // Start container if auto_start is enabled
        let mut status = SectorContainerStatus::Created;
        let mut started_at = None;
        
        if sector_config.auto_start {
            self.container_manager.start_container(&container_info.id).await?;
            status = SectorContainerStatus::Running;
            started_at = Some(chrono::Utc::now());
        }
        
        // Build host port mappings
        let mut host_ports = HashMap::new();
        for port in &sector_config.ports {
            if port.host_port == 0 {
                // Dynamic port allocation - in real impl, get from runtime
                host_ports.insert(port.container_port, 30000 + rand::random::<u16>() % 10000);
            } else {
                host_ports.insert(port.container_port, port.host_port);
            }
        }
        
        let sector = SectorContainer {
            container_id: container_info.id,
            sector_id: sector_config.sector_id.clone(),
            config: sector_config.clone(),
            status,
            created_at: chrono::Utc::now(),
            started_at,
            ip_address: None, // Would be populated from runtime
            host_ports,
            volume_paths,
            snapshot_id: None,
        };
        
        // Store sector
        self.sectors.lock().unwrap().insert(sector_config.sector_id.clone(), sector.clone());
        
        tracing::info!("Created sector container: {} ({})", sector.config.name, sector.container_id);
        
        Ok(sector)
    }
    
    /// Start a sector container
    pub async fn start_sector(&self, sector_id: &str) -> ContainerResult<()> {
        tracing::info!("Starting sector: {}", sector_id);
        
        let mut sectors = self.sectors.lock().unwrap();
        let sector = sectors.get_mut(sector_id)
            .ok_or_else(|| ContainerError::Runtime(format!("Sector {} not found", sector_id)))?;
        
        if !sector.status.can_start() {
            return Err(ContainerError::Runtime(
                format!("Sector {} cannot be started (status: {:?})", sector_id, sector.status)
            ));
        }
        
        self.container_manager.start_container(&sector.container_id).await?;
        sector.status = SectorContainerStatus::Running;
        sector.started_at = Some(chrono::Utc::now());
        
        tracing::info!("Started sector: {}", sector_id);
        Ok(())
    }
    
    /// Stop a sector container
    pub async fn stop_sector(&self, sector_id: &str, timeout: u64) -> ContainerResult<()> {
        tracing::info!("Stopping sector: {}", sector_id);
        
        let mut sectors = self.sectors.lock().unwrap();
        let sector = sectors.get_mut(sector_id)
            .ok_or_else(|| ContainerError::Runtime(format!("Sector {} not found", sector_id)))?;
        
        if !sector.status.can_stop() {
            return Err(ContainerError::Runtime(
                format!("Sector {} is not running", sector_id)
            ));
        }
        
        sector.status = SectorContainerStatus::Stopping;
        
        self.container_manager.stop_container(&sector.container_id, timeout).await?;
        sector.status = SectorContainerStatus::Stopped;
        
        tracing::info!("Stopped sector: {}", sector_id);
        Ok(())
    }
    
    /// Remove a sector container
    pub async fn remove_sector(&self, sector_id: &str, force: bool) -> ContainerResult<()> {
        tracing::info!("Removing sector: {}", sector_id);
        
        let mut sectors = self.sectors.lock().unwrap();
        let sector = sectors.get(sector_id)
            .ok_or_else(|| ContainerError::Runtime(format!("Sector {} not found", sector_id)))?;
        
        // Remove container
        self.container_manager.remove_container(&sector.container_id, force).await?;
        
        // Remove from tracking
        sectors.remove(sector_id);
        
        // Clean up data directory
        let sector_data_dir = self.data_root.join(sector_id);
        if sector_data_dir.exists() {
            std::fs::remove_dir_all(&sector_data_dir)?;
        }
        
        tracing::info!("Removed sector: {}", sector_id);
        Ok(())
    }
    
    /// Get sector container
    pub async fn get_sector(&self, sector_id: &str) -> ContainerResult<SectorContainer> {
        self.sectors.lock().unwrap()
            .get(sector_id)
            .cloned()
            .ok_or_else(|| ContainerError::Runtime(format!("Sector {} not found", sector_id)))
    }
    
    /// List all sector containers
    pub async fn list_sectors(&self) -> ContainerResult<Vec<SectorContainer>> {
        let sectors = self.sectors.lock().unwrap();
        Ok(sectors.values().cloned().collect())
    }
    
    /// Get sector logs
    pub async fn get_sector_logs(
        &self,
        sector_id: &str,
        tail: Option<usize>,
    ) -> ContainerResult<String> {
        let sector = self.get_sector(sector_id).await?;
        self.container_manager.get_logs(&sector.container_id, tail, None).await
    }
    
    /// Execute command in sector
    pub async fn exec_in_sector(
        &self,
        sector_id: &str,
        command: &[String],
    ) -> ContainerResult<(String, i32)> {
        let sector = self.get_sector(sector_id).await?;
        self.container_manager.exec_command(&sector.container_id, command, false).await
    }
    
    /// Create snapshot of sector
    pub async fn snapshot_sector(&self, sector_id: &str, name: &str) -> ContainerResult<String> {
        tracing::info!("Creating snapshot of sector {}: {}", sector_id, name);
        
        let _sector = self.get_sector(sector_id).await?;
        
        // Create snapshot image
        let _snapshot_tag = format!("tos/sector-{}-snapshot:{}", sector_id, name);
        
        // In real implementation, commit container to image
        // For now, just record the snapshot ID
        
        let snapshot_id = format!("snapshot-{}", uuid::Uuid::new_v4());
        
        tracing::info!("Created snapshot: {}", snapshot_id);
        Ok(snapshot_id)
    }
    
    /// Restore sector from snapshot
    pub async fn restore_sector(&self, sector_id: &str, snapshot_id: &str) -> ContainerResult<()> {
        tracing::info!("Restoring sector {} from snapshot {}", sector_id, snapshot_id);
        
        // In real implementation, create new container from snapshot image
        Ok(())
    }
    
    /// Get sector stats
    pub async fn get_sector_stats(&self, sector_id: &str) -> ContainerResult<super::ContainerMetrics> {
        let sector = self.get_sector(sector_id).await?;
        self.container_manager.get_stats(&sector.container_id).await
    }
    
    /// Pause sector
    pub async fn pause_sector(&self, sector_id: &str) -> ContainerResult<()> {
        let mut sectors = self.sectors.lock().unwrap();
        let sector = sectors.get_mut(sector_id)
            .ok_or_else(|| ContainerError::Runtime(format!("Sector {} not found", sector_id)))?;
        
        self.container_manager.pause_container(&sector.container_id).await?;
        sector.status = SectorContainerStatus::Paused;
        
        Ok(())
    }
    
    /// Unpause sector
    pub async fn unpause_sector(&self, sector_id: &str) -> ContainerResult<()> {
        let mut sectors = self.sectors.lock().unwrap();
        let sector = sectors.get_mut(sector_id)
            .ok_or_else(|| ContainerError::Runtime(format!("Sector {} not found", sector_id)))?;
        
        self.container_manager.unpause_container(&sector.container_id).await?;
        sector.status = SectorContainerStatus::Running;
        
        Ok(())
    }
    
    /// Access to underlying container manager
    pub fn container_manager(&self) -> &ContainerManager {
        &self.container_manager
    }
}

/// Builder for sector container configuration
#[derive(Debug)]
pub struct SectorContainerBuilder {
    config: SectorContainerConfig,
}

impl SectorContainerBuilder {
    /// Create a new sector container builder
    pub fn new(sector_id: impl Into<String>) -> Self {
        Self {
            config: SectorContainerConfig {
                sector_id: sector_id.into(),
                ..Default::default()
            },
        }
    }
    
    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }
    
    /// Set base image
    pub fn base_image(mut self, image: impl Into<String>) -> Self {
        self.config.base_image = image.into();
        self
    }
    
    /// Set TOS version
    pub fn tos_version(mut self, version: impl Into<String>) -> Self {
        self.config.tos_version = version.into();
        self
    }
    
    /// Add environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.env.insert(key.into(), value.into());
        self
    }
    
    /// Add volume
    pub fn volume(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.config.volumes.push(SectorVolume {
            source: source.into(),
            target: target.into(),
            volume_type: VolumeType::Bind,
            read_only: false,
            driver_opts: HashMap::new(),
        });
        self
    }
    
    /// Add port mapping
    pub fn port(mut self, host_port: u16, container_port: u16) -> Self {
        self.config.ports.push(PortMapping {
            host_port,
            container_port,
            protocol: Protocol::Tcp,
            host_ip: None,
        });
        self
    }
    
    /// Set CPU limit
    pub fn cpu_limit(mut self, limit: f64) -> Self {
        self.config.resource_limits.cpu_limit = Some(limit);
        self
    }
    
    /// Set memory limit (in bytes)
    pub fn memory_limit(mut self, limit: u64) -> Self {
        self.config.resource_limits.memory_limit = Some(limit);
        self
    }
    
    /// Disable auto-start
    pub fn no_auto_start(mut self) -> Self {
        self.config.auto_start = false;
        self
    }
    
    /// Set restart policy
    pub fn restart_policy(mut self, policy: SectorRestartPolicy) -> Self {
        self.config.restart_policy = policy;
        self
    }
    
    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.labels.insert(key.into(), value.into());
        self
    }
    
    /// Build configuration
    pub fn build(self) -> SectorContainerConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sector_container_builder() {
        let config = SectorContainerBuilder::new("test-sector")
            .name("Test Sector")
            .base_image("tos/sector:v1.0")
            .tos_version("1.0.0")
            .env("DEBUG", "true")
            .volume("/host/data", "/container/data")
            .port(8080, 80)
            .cpu_limit(2.0)
            .memory_limit(2 * 1024 * 1024 * 1024)
            .label("env", "test")
            .build();
        
        assert_eq!(config.sector_id, "test-sector");
        assert_eq!(config.name, "Test Sector");
        assert_eq!(config.base_image, "tos/sector:v1.0");
        assert_eq!(config.env.get("DEBUG"), Some(&"true".to_string()));
        assert_eq!(config.resource_limits.cpu_limit, Some(2.0));
    }
    
    #[test]
    fn test_sector_container_status() {
        assert!(SectorContainerStatus::Running.is_active());
        assert!(SectorContainerStatus::Paused.is_active());
        assert!(!SectorContainerStatus::Stopped.is_active());
        
        assert!(SectorContainerStatus::Created.can_start());
        assert!(SectorContainerStatus::Stopped.can_start());
        assert!(!SectorContainerStatus::Running.can_start());
        
        assert!(SectorContainerStatus::Running.can_stop());
        assert!(!SectorContainerStatus::Stopped.can_stop());
    }
    
    #[test]
    fn test_sector_security_options_default() {
        let opts = SectorSecurityOptions::default();
        assert!(opts.read_only);
        assert!(opts.no_new_privileges);
        assert!(opts.drop_all_capabilities);
        assert!(!opts.add_capabilities.is_empty());
    }
}
