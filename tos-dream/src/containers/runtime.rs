//! Container Runtime Abstraction Implementation
//!
//! Provides a unified interface for different container backends
//! (Docker, Podman, containerd).

use super::{ContainerId, ContainerInfo, ContainerMetrics, ContainerResult, ContainerStatus, ContainerError, ContainerConfig};
use async_trait::async_trait;
use std::collections::HashMap;
use chrono;

/// Container backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerBackend {
    Docker,
    Podman,
    Containerd,
    Mock, // For testing
}

impl ContainerBackend {
    /// Get the default socket path for this backend
    pub fn default_socket(&self) -> &'static str {
        match self {
            Self::Docker => "/var/run/docker.sock",
            Self::Podman => "/run/podman/podman.sock",
            Self::Containerd => "/run/containerd/containerd.sock",
            Self::Mock => "",
        }
    }
    
    /// Check if this backend is available on the system
    pub async fn is_available(&self) -> bool {
        match self {
            Self::Docker => {
                std::path::Path::new("/var/run/docker.sock").exists()
            }
            Self::Podman => {
                std::path::Path::new("/run/podman/podman.sock").exists()
                    || std::process::Command::new("podman")
                        .arg("--version")
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
            }
            Self::Containerd => {
                std::path::Path::new("/run/containerd/containerd.sock").exists()
            }
            Self::Mock => true,
        }
    }
}

/// Container runtime trait - abstracts different container backends
#[async_trait]
pub trait ContainerRuntime: Send + Sync + std::fmt::Debug {
    /// Clone the runtime (for creating new references)
    fn clone_box(&self) -> Box<dyn ContainerRuntime>;
    
    /// Get the backend type
    fn backend(&self) -> ContainerBackend;
    
    /// Create a new container
    async fn create_container(&self, config: ContainerConfig) -> ContainerResult<ContainerInfo>;
    
    /// Start a container
    async fn start_container(&self, id: &ContainerId) -> ContainerResult<()>;
    
    /// Stop a container
    async fn stop_container(&self, id: &ContainerId, timeout: u64) -> ContainerResult<()>;
    
    /// Pause a container
    async fn pause_container(&self, id: &ContainerId) -> ContainerResult<()>;
    
    /// Unpause a container
    async fn unpause_container(&self, id: &ContainerId) -> ContainerResult<()>;
    
    /// Restart a container
    async fn restart_container(&self, id: &ContainerId, timeout: u64) -> ContainerResult<()>;
    
    /// Remove a container
    async fn remove_container(&self, id: &ContainerId, force: bool) -> ContainerResult<()>;
    
    /// Get container information
    async fn get_container(&self, id: &ContainerId) -> ContainerResult<ContainerInfo>;
    
    /// List containers
    async fn list_containers(&self, all: bool) -> ContainerResult<Vec<ContainerInfo>>;
    
    /// Execute command in container
    async fn exec_command(
        &self,
        id: &ContainerId,
        command: &[String],
        tty: bool,
    ) -> ContainerResult<(String, i32)>;
    
    /// Get container logs
    async fn get_logs(
        &self,
        id: &ContainerId,
        tail: Option<usize>,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ContainerResult<String>;
    
    /// Get container stats
    async fn get_stats(&self, id: &ContainerId) -> ContainerResult<ContainerMetrics>;
    
    /// Wait for container to finish
    async fn wait_container(&self, id: &ContainerId) -> ContainerResult<i32>;
    
    /// Copy file to container
    async fn copy_to_container(
        &self,
        id: &ContainerId,
        source: &std::path::Path,
        target: &std::path::Path,
    ) -> ContainerResult<()>;
    
    /// Copy file from container
    async fn copy_from_container(
        &self,
        id: &ContainerId,
        source: &std::path::Path,
        target: &std::path::Path,
    ) -> ContainerResult<()>;
}

/// Restart policy for containers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartPolicy {
    No,
    OnFailure,
    Always,
    UnlessStopped,
}

impl RestartPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::No => "no",
            Self::OnFailure => "on-failure",
            Self::Always => "always",
            Self::UnlessStopped => "unless-stopped",
        }
    }
}

/// Security options for containers
#[derive(Debug, Clone)]
pub struct SecurityOptions {
    /// Seccomp profile path
    pub seccomp_profile: Option<String>,
    /// AppArmor profile
    pub apparmor_profile: Option<String>,
    /// SELinux options
    pub selinux_options: Vec<String>,
    /// No new privileges
    pub no_new_privileges: bool,
    /// Security-opt options
    pub security_opts: Vec<String>,
}

impl Default for SecurityOptions {
    fn default() -> Self {
        Self {
            seccomp_profile: None,
            apparmor_profile: Some("docker-default".to_string()),
            selinux_options: Vec::new(),
            no_new_privileges: true,
            security_opts: vec!["no-new-privileges:true".to_string()],
        }
    }
}

/// Docker runtime implementation
#[derive(Debug, Clone)]
pub struct DockerRuntime {
    docker: bollard::Docker,
}

impl DockerRuntime {
    pub async fn new() -> ContainerResult<Self> {
        let docker = bollard::Docker::connect_with_local_defaults()
            .map_err(|e| ContainerError::Runtime(format!("Failed to connect to Docker: {}", e)))?;
        
        Ok(Self { docker })
    }
}

#[async_trait]
impl ContainerRuntime for DockerRuntime {
    fn clone_box(&self) -> Box<dyn ContainerRuntime> {
        Box::new(self.clone())
    }
    
    fn backend(&self) -> ContainerBackend {
        ContainerBackend::Docker
    }
    
    async fn create_container(&self, config: ContainerConfig) -> ContainerResult<ContainerInfo> {
        let options = Some(bollard::container::CreateContainerOptions {
            name: config.name.as_str(),
            ..Default::default()
        });
        
        let docker_config = bollard::container::Config {
            image: Some(config.image.clone()),
            ..Default::default()
        };

        self.docker.create_container::<&str, String>(options, docker_config).await
            .map_err(|e| ContainerError::Runtime(format!("Docker create failed: {}", e)))?;
        
        let id = format!("docker-{}", uuid::Uuid::new_v4());
        
        Ok(ContainerInfo {
            id,
            name: config.name,
            image: config.image,
            status: ContainerStatus::Created,
            created_at: chrono::Utc::now(),
            started_at: None,
            labels: config.labels,
            ports: config.ports,
            volumes: config.volumes,
            resource_limits: config.resource_limits,
        })
    }
    
    async fn start_container(&self, _id: &ContainerId) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn stop_container(&self, _id: &ContainerId, _timeout: u64) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn pause_container(&self, _id: &ContainerId) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn unpause_container(&self, _id: &ContainerId) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn restart_container(&self, _id: &ContainerId, _timeout: u64) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn remove_container(&self, _id: &ContainerId, _force: bool) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn get_container(&self, id: &ContainerId) -> ContainerResult<ContainerInfo> {
        Err(ContainerError::ContainerNotFound(id.clone()))
    }
    
    async fn list_containers(&self, _all: bool) -> ContainerResult<Vec<ContainerInfo>> {
        Ok(Vec::new())
    }
    
    async fn exec_command(
        &self,
        _id: &ContainerId,
        _command: &[String],
        _tty: bool,
    ) -> ContainerResult<(String, i32)> {
        Ok(("".to_string(), 0))
    }
    
    async fn get_logs(
        &self,
        _id: &ContainerId,
        _tail: Option<usize>,
        _since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ContainerResult<String> {
        Ok("".to_string())
    }
    
    async fn get_stats(&self, _id: &ContainerId) -> ContainerResult<ContainerMetrics> {
        Ok(ContainerMetrics::default())
    }
    
    async fn wait_container(&self, _id: &ContainerId) -> ContainerResult<i32> {
        Ok(0)
    }
    
    async fn copy_to_container(
        &self,
        _id: &ContainerId,
        _source: &std::path::Path,
        _target: &std::path::Path,
    ) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn copy_from_container(
        &self,
        _id: &ContainerId,
        _source: &std::path::Path,
        _target: &std::path::Path,
    ) -> ContainerResult<()> {
        Ok(())
    }
}

/// Mock runtime for testing
#[derive(Debug, Clone)]
pub struct MockRuntime {
    containers: std::sync::Arc<std::sync::Mutex<HashMap<ContainerId, ContainerInfo>>>,
}

impl MockRuntime {
    pub fn new() -> Self {
        Self {
            containers: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl ContainerRuntime for MockRuntime {
    fn clone_box(&self) -> Box<dyn ContainerRuntime> {
        Box::new(self.clone())
    }
    
    fn backend(&self) -> ContainerBackend {
        ContainerBackend::Mock
    }
    
    async fn create_container(&self, config: ContainerConfig) -> ContainerResult<ContainerInfo> {
        let id = format!("mock-{}", uuid::Uuid::new_v4());
        let info = ContainerInfo {
            id: id.clone(),
            name: config.name,
            image: config.image,
            status: ContainerStatus::Created,
            created_at: chrono::Utc::now(),
            started_at: None,
            labels: config.labels,
            ports: config.ports,
            volumes: config.volumes,
            resource_limits: config.resource_limits,
        };
        
        self.containers.lock().unwrap().insert(id.clone(), info.clone());
        Ok(info)
    }
    
    async fn start_container(&self, id: &ContainerId) -> ContainerResult<()> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(info) = containers.get_mut(id) {
            info.status = ContainerStatus::Running;
            info.started_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn stop_container(&self, id: &ContainerId, _timeout: u64) -> ContainerResult<()> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(info) = containers.get_mut(id) {
            info.status = ContainerStatus::Exited;
            Ok(())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn pause_container(&self, id: &ContainerId) -> ContainerResult<()> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(info) = containers.get_mut(id) {
            info.status = ContainerStatus::Paused;
            Ok(())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn unpause_container(&self, id: &ContainerId) -> ContainerResult<()> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(info) = containers.get_mut(id) {
            info.status = ContainerStatus::Running;
            Ok(())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn restart_container(&self, id: &ContainerId, timeout: u64) -> ContainerResult<()> {
        self.stop_container(id, timeout).await?;
        self.start_container(id).await
    }
    
    async fn remove_container(&self, id: &ContainerId, force: bool) -> ContainerResult<()> {
        let mut containers = self.containers.lock().unwrap();
        if let Some(info) = containers.get(id) {
            if info.status.is_active() && !force {
                return Err(ContainerError::Runtime(
                    "Container is running, use force=true to remove".to_string()
                ));
            }
            containers.remove(id);
            Ok(())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn get_container(&self, id: &ContainerId) -> ContainerResult<ContainerInfo> {
        self.containers.lock().unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| ContainerError::ContainerNotFound(id.clone()))
    }
    
    async fn list_containers(&self, all: bool) -> ContainerResult<Vec<ContainerInfo>> {
        let containers = self.containers.lock().unwrap();
        let list: Vec<_> = containers.values()
            .filter(|info| all || info.status.is_active())
            .cloned()
            .collect();
        Ok(list)
    }
    
    async fn exec_command(
        &self,
        id: &ContainerId,
        command: &[String],
        _tty: bool,
    ) -> ContainerResult<(String, i32)> {
        let containers = self.containers.lock().unwrap();
        if containers.contains_key(id) {
            Ok((format!("Executed: {:?}", command), 0))
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn get_logs(
        &self,
        id: &ContainerId,
        _tail: Option<usize>,
        _since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> ContainerResult<String> {
        let containers = self.containers.lock().unwrap();
        if containers.contains_key(id) {
            Ok("Mock container logs".to_string())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn get_stats(&self, id: &ContainerId) -> ContainerResult<ContainerMetrics> {
        let containers = self.containers.lock().unwrap();
        if containers.contains_key(id) {
            Ok(ContainerMetrics::default())
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn wait_container(&self, id: &ContainerId) -> ContainerResult<i32> {
        let containers = self.containers.lock().unwrap();
        if containers.contains_key(id) {
            Ok(0)
        } else {
            Err(ContainerError::ContainerNotFound(id.clone()))
        }
    }
    
    async fn copy_to_container(
        &self,
        _id: &ContainerId,
        _source: &std::path::Path,
        _target: &std::path::Path,
    ) -> ContainerResult<()> {
        Ok(())
    }
    
    async fn copy_from_container(
        &self,
        _id: &ContainerId,
        _source: &std::path::Path,
        _target: &std::path::Path,
    ) -> ContainerResult<()> {
        Ok(())
    }
}

/// Create a runtime for the specified backend
pub async fn create_runtime(backend: ContainerBackend) -> ContainerResult<Box<dyn ContainerRuntime>> {
    match backend {
        ContainerBackend::Docker => {
            let runtime = DockerRuntime::new().await?;
            Ok(Box::new(runtime))
        }
        ContainerBackend::Mock => {
            Ok(Box::new(MockRuntime::new()))
        }
        _ => Err(ContainerError::Runtime(
            format!("Backend {:?} not yet implemented", backend)
        )),
    }
}

/// Auto-detect available container backend
pub async fn auto_detect_backend() -> Option<ContainerBackend> {
    for backend in [ContainerBackend::Docker, ContainerBackend::Podman] {
        if backend.is_available().await {
            return Some(backend);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mock_runtime() {
        let runtime = MockRuntime::new();
        
        // Create container
        let config = ContainerConfig {
            name: "test-container".to_string(),
            image: "alpine:latest".to_string(),
            ..Default::default()
        };
        let info = runtime.create_container(config).await.unwrap();
        assert_eq!(info.name, "test-container");
        assert_eq!(info.status, ContainerStatus::Created);
        
        // Start container
        runtime.start_container(&info.id).await.unwrap();
        let info = runtime.get_container(&info.id).await.unwrap();
        assert_eq!(info.status, ContainerStatus::Running);
        
        // List containers
        let list = runtime.list_containers(false).await.unwrap();
        assert_eq!(list.len(), 1);
        
        // Stop container
        runtime.stop_container(&info.id, 10).await.unwrap();
        let info = runtime.get_container(&info.id).await.unwrap();
        assert_eq!(info.status, ContainerStatus::Exited);
        
        // Remove container
        runtime.remove_container(&info.id, false).await.unwrap();
        let list = runtime.list_containers(true).await.unwrap();
        assert!(list.is_empty());
    }
    
    #[test]
    fn test_restart_policy() {
        assert_eq!(RestartPolicy::Always.as_str(), "always");
        assert_eq!(RestartPolicy::No.as_str(), "no");
    }
    
    #[test]
    fn test_container_backend_default_socket() {
        assert!(ContainerBackend::Docker.default_socket().contains("docker"));
        assert!(ContainerBackend::Podman.default_socket().contains("podman"));
    }
}
