//! Phase 16: Local Sandbox Security Manager
//!
//! Orchestrates various isolation vectors (Containers, Namespaces, 
//! Filtered IPC, Display Isolation) to provide a unified secure 
//! environment for untrusted local applications.

use super::{ContainerManager, ContainerConfig, ContainerResult, ContainerId};
use super::security::{SecurityPolicy, DisplayIsolation, AudioIsolation, NetworkIsolation};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// Sandbox profile defines the security level for a workspace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxLevel {
    /// No isolation (for core system components)
    None,
    /// Light isolation (cgroups, shared display/audio)
    Standard,
    /// High isolation (private network, virtualized display/audio)
    Restricted,
    /// Maximum isolation (no network, no display, ephemeral storage)
    Paranoid,
}

/// Sandbox metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: String,
    pub level: SandboxLevel,
    pub container_id: Option<ContainerId>,
    pub active: bool,
}

/// Sandbox manager handles local isolation policies
#[derive(Debug)]
pub struct SandboxManager {
    container_manager: Arc<ContainerManager>,
    active_sandboxes: Arc<Mutex<Vec<SandboxInfo>>>,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new(container_manager: Arc<ContainerManager>) -> Self {
        Self {
            container_manager,
            active_sandboxes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new sandbox based on a level
    pub async fn create_sandbox(&self, id: &str, level: SandboxLevel) -> ContainerResult<SandboxInfo> {
        let policy = match level {
            SandboxLevel::None => SecurityPolicy::minimal(),
            SandboxLevel::Standard => SecurityPolicy::default(),
            SandboxLevel::Restricted | SandboxLevel::Paranoid => SecurityPolicy::restricted(),
        };

        // If level is paranoid, we override even restricted settings
        let mut policy = policy;
        if level == SandboxLevel::Paranoid {
            policy.network_isolation = NetworkIsolation::None;
            policy.display_isolation = DisplayIsolation::None;
            policy.audio_isolation = AudioIsolation::None;
        }

        // Translate SandboxLevel to ContainerConfig
        let config = ContainerConfig {
            name: format!("tos-sandbox-{}", id),
            image: "tos/sandbox-base:latest".to_string(),
            labels: {
                let mut l = std::collections::HashMap::new();
                l.insert("tos.sandbox.level".to_string(), format!("{:?}", level));
                l
            },
            // Apply policy to config
            ..Default::default()
        };

        // In a real implementation, we would apply all policy fields (DisplayIsolation etc.)
        // to the container runtime options (like X11 socket mounting or Wayland proxy setup)
        
        let container_info = self.container_manager.create_container(config).await?;
        
        let sandbox = SandboxInfo {
            id: id.to_string(),
            level,
            container_id: Some(container_info.id),
            active: true,
        };

        self.active_sandboxes.lock().unwrap().push(sandbox.clone());
        Ok(sandbox)
    }

    /// List active sandboxes
    pub fn list_sandboxes(&self) -> Vec<SandboxInfo> {
        self.active_sandboxes.lock().unwrap().clone()
    }

    /// Terminate a sandbox
    pub async fn terminate_sandbox(&self, id: &str) -> ContainerResult<()> {
        let mut sandboxes = self.active_sandboxes.lock().unwrap();
        if let Some(pos) = sandboxes.iter().position(|s| s.id == id) {
            let sandbox = sandboxes.remove(pos);
            if let Some(cid) = sandbox.container_id {
                self.container_manager.stop_container(&cid, 10).await?;
                self.container_manager.remove_container(&cid, true).await?;
            }
        }
        Ok(())
    }
}
