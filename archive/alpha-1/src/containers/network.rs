//! Container Networking Implementation
//!
//! Manages container networks including virtual networks for sectors,
//! port mappings, and inter-container communication.

use super::{ContainerResult, ContainerError, ContainerId, ContainerRuntime};
use std::collections::HashMap;
use std::net::IpAddr;
use serde::{Deserialize, Serialize};

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network name
    pub name: String,
    /// Network driver (bridge, overlay, macvlan, etc.)
    pub driver: NetworkDriver,
    /// Subnet CIDR
    pub subnet: String,
    /// Gateway IP
    pub gateway: Option<IpAddr>,
    /// IP range
    pub ip_range: Option<String>,
    /// Enable IPv6
    pub enable_ipv6: bool,
    /// Internal network (no external access)
    pub internal: bool,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Network options
    pub options: HashMap<String, String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            driver: NetworkDriver::Bridge,
            subnet: "172.20.0.0/16".to_string(),
            gateway: None,
            ip_range: None,
            enable_ipv6: false,
            internal: false,
            labels: HashMap::new(),
            options: HashMap::new(),
        }
    }
}

/// Network driver types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkDriver {
    Bridge,
    Overlay,
    MacVlan,
    Host,
    None,
    Custom(String),
}

impl NetworkDriver {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bridge => "bridge",
            Self::Overlay => "overlay",
            Self::MacVlan => "macvlan",
            Self::Host => "host",
            Self::None => "none",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// Port mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Host port
    pub host_port: u16,
    /// Container port
    pub container_port: u16,
    /// Protocol (tcp/udp/sctp)
    pub protocol: Protocol,
    /// Host IP to bind (None = all interfaces)
    pub host_ip: Option<IpAddr>,
}

/// Protocol for port mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Sctp,
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
            Self::Sctp => "sctp",
        }
    }
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Network ID
    pub id: String,
    /// Network name
    pub name: String,
    /// Driver
    pub driver: NetworkDriver,
    /// Scope (local, global, swarm)
    pub scope: String,
    /// Creation time
    pub created: chrono::DateTime<chrono::Utc>,
    /// Subnet
    pub subnet: String,
    /// Gateway
    pub gateway: Option<IpAddr>,
    /// Connected containers
    pub containers: Vec<ConnectedContainer>,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Connected container in a network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedContainer {
    /// Container ID
    pub container_id: ContainerId,
    /// Container name
    pub name: String,
    /// IP address in network
    pub ip_address: IpAddr,
    /// MAC address
    pub mac_address: String,
    /// IPv6 address
    pub ipv6_address: Option<IpAddr>,
}

/// Container network manager
#[derive(Debug)]
pub struct ContainerNetwork {
    runtime: Box<dyn ContainerRuntime>,
    networks: std::sync::Mutex<HashMap<String, NetworkInfo>>,
}

impl ContainerNetwork {
    /// Create a new network manager
    pub fn new(runtime: Box<dyn ContainerRuntime>) -> Self {
        Self {
            runtime,
            networks: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    /// Create a new network
    pub async fn create_network(&self, config: NetworkConfig) -> ContainerResult<NetworkInfo> {
        tracing::info!("Creating network: {} ({})", config.name, config.driver.as_str());
        
        // In real implementation, call runtime to create network
        
        let info = NetworkInfo {
            id: format!("net-{}", uuid::Uuid::new_v4()),
            name: config.name.clone(),
            driver: config.driver,
            scope: "local".to_string(),
            created: chrono::Utc::now(),
            subnet: config.subnet.clone(),
            gateway: config.gateway,
            containers: Vec::new(),
            labels: config.labels,
        };
        
        // Store network
        self.networks.lock().unwrap().insert(config.name, info.clone());
        
        tracing::info!("Created network: {} ({})", info.name, info.id);
        Ok(info)
    }
    
    /// Remove a network
    pub async fn remove_network(&self, name: &str) -> ContainerResult<()> {
        tracing::info!("Removing network: {}", name);
        
        let mut networks = self.networks.lock().unwrap();
        
        // Check if network exists
        let info = networks.get(name)
            .ok_or_else(|| ContainerError::Runtime(format!("Network {} not found", name)))?;
        
        // Check if containers are connected
        if !info.containers.is_empty() {
            return Err(ContainerError::Runtime(
                format!("Network {} has connected containers", name)
            ));
        }
        
        networks.remove(name);
        tracing::info!("Removed network: {}", name);
        Ok(())
    }
    
    /// Get network info
    pub async fn get_network(&self, name: &str) -> ContainerResult<NetworkInfo> {
        self.networks.lock().unwrap()
            .get(name)
            .cloned()
            .ok_or_else(|| ContainerError::Runtime(format!("Network {} not found", name)))
    }
    
    /// List all networks
    pub async fn list_networks(&self) -> ContainerResult<Vec<NetworkInfo>> {
        let networks = self.networks.lock().unwrap();
        Ok(networks.values().cloned().collect())
    }
    
    /// Connect container to network
    pub async fn connect_container(
        &self,
        network_name: &str,
        container_id: &ContainerId,
        ip_address: Option<IpAddr>,
    ) -> ContainerResult<()> {
        tracing::info!("Connecting container {} to network {}", container_id, network_name);
        
        let mut networks = self.networks.lock().unwrap();
        let network = networks.get_mut(network_name)
            .ok_or_else(|| ContainerError::Runtime(format!("Network {} not found", network_name)))?;
        
        // Check if already connected
        if network.containers.iter().any(|c| &c.container_id == container_id) {
            return Err(ContainerError::Runtime(
                format!("Container {} already connected to network {}", container_id, network_name)
            ));
        }
        
        // Get container info from runtime
        let container_info = self.runtime.get_container(container_id).await?;
        
        // Assign IP address
        let assigned_ip = ip_address.unwrap_or_else(|| {
            // Simple IP assignment logic (in real impl, use proper IPAM)
            let base: Vec<u8> = network.subnet.split('/').next()
                .unwrap()
                .split('.')
                .map(|s| s.parse().unwrap_or(0))
                .collect();
            
            let offset = network.containers.len() + 1;
            IpAddr::from([base[0], base[1], base[2], base[3] + offset as u8])
        });
        
        let connected = ConnectedContainer {
            container_id: container_id.clone(),
            name: container_info.name,
            ip_address: assigned_ip,
            mac_address: generate_mac_address(),
            ipv6_address: None,
        };
        
        network.containers.push(connected);
        
        tracing::info!("Connected container {} to network {} with IP {}", 
            container_id, network_name, assigned_ip);
        
        Ok(())
    }
    
    /// Disconnect container from network
    pub async fn disconnect_container(
        &self,
        network_name: &str,
        container_id: &ContainerId,
        _force: bool,
    ) -> ContainerResult<()> {
        tracing::info!("Disconnecting container {} from network {}", container_id, network_name);
        
        let mut networks = self.networks.lock().unwrap();
        let network = networks.get_mut(network_name)
            .ok_or_else(|| ContainerError::Runtime(format!("Network {} not found", network_name)))?;
        
        let pos = network.containers.iter()
            .position(|c| &c.container_id == container_id)
            .ok_or_else(|| ContainerError::Runtime(
                format!("Container {} not connected to network {}", container_id, network_name)
            ))?;
        
        network.containers.remove(pos);
        
        tracing::info!("Disconnected container {} from network {}", container_id, network_name);
        Ok(())
    }
    
    /// Inspect network
    pub async fn inspect_network(&self, name: &str) -> ContainerResult<NetworkInfo> {
        self.get_network(name).await
    }
    
    /// Prune unused networks
    pub async fn prune_networks(&self) -> ContainerResult<PruneResult> {
        tracing::info!("Pruning unused networks");
        
        let mut networks = self.networks.lock().unwrap();
        let before_count = networks.len();
        
        // Remove networks with no containers
        networks.retain(|_, info| !info.containers.is_empty());
        
        let after_count = networks.len();
        let removed = before_count - after_count;
        
        tracing::info!("Pruned {} unused networks", removed);
        
        Ok(PruneResult {
            networks_deleted: removed,
        })
    }
    
    /// Get container IP in network
    pub async fn get_container_ip(
        &self,
        network_name: &str,
        container_id: &ContainerId,
    ) -> ContainerResult<IpAddr> {
        let networks = self.networks.lock().unwrap();
        let network = networks.get(network_name)
            .ok_or_else(|| ContainerError::Runtime(format!("Network {} not found", network_name)))?;
        
        network.containers.iter()
            .find(|c| &c.container_id == container_id)
            .map(|c| c.ip_address)
            .ok_or_else(|| ContainerError::Runtime(
                format!("Container {} not found in network {}", container_id, network_name)
            ))
    }
    
    /// Create a sector network (isolated network for a sector)
    pub async fn create_sector_network(&self, sector_id: &str) -> ContainerResult<NetworkInfo> {
        let config = NetworkConfig {
            name: format!("tos-sector-{}", sector_id),
            driver: NetworkDriver::Bridge,
            subnet: generate_subnet(),
            internal: false,
            labels: {
                let mut labels = HashMap::new();
                labels.insert("tos.sector.id".to_string(), sector_id.to_string());
                labels.insert("tos.network.type".to_string(), "sector".to_string());
                labels
            },
            ..Default::default()
        };
        
        self.create_network(config).await
    }
    
    /// Create an overlay network for multi-host sectors
    pub async fn create_overlay_network(&self, name: &str) -> ContainerResult<NetworkInfo> {
        let config = NetworkConfig {
            name: name.to_string(),
            driver: NetworkDriver::Overlay,
            subnet: "10.0.0.0/16".to_string(),
            labels: {
                let mut labels = HashMap::new();
                labels.insert("tos.network.type".to_string(), "overlay".to_string());
                labels
            },
            ..Default::default()
        };
        
        self.create_network(config).await
    }
}

/// Prune result for networks
#[derive(Debug, Clone, Copy)]
pub struct PruneResult {
    pub networks_deleted: usize,
}

/// Generate a random MAC address
fn generate_mac_address() -> String {
    let mut mac = vec![0x02]; // Locally administered
    for _ in 0..5 {
        mac.push(rand::random::<u8>());
    }
    mac.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// Generate a unique subnet
fn generate_subnet() -> String {
    // Generate random /24 subnet in 172.20-30.x.0/24 range
    let third_octet = rand::random::<u8>() % 100 + 20;
    format!("172.20.{}.0/24", third_octet)
}

/// Network builder for fluent API
#[derive(Debug)]
pub struct NetworkBuilder {
    config: NetworkConfig,
}

impl NetworkBuilder {
    /// Create a new network builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: NetworkConfig {
                name: name.into(),
                ..Default::default()
            },
        }
    }
    
    /// Set driver
    pub fn driver(mut self, driver: NetworkDriver) -> Self {
        self.config.driver = driver;
        self
    }
    
    /// Set subnet
    pub fn subnet(mut self, subnet: impl Into<String>) -> Self {
        self.config.subnet = subnet.into();
        self
    }
    
    /// Set gateway
    pub fn gateway(mut self, gateway: IpAddr) -> Self {
        self.config.gateway = Some(gateway);
        self
    }
    
    /// Enable IPv6
    pub fn enable_ipv6(mut self) -> Self {
        self.config.enable_ipv6 = true;
        self
    }
    
    /// Set internal
    pub fn internal(mut self) -> Self {
        self.config.internal = true;
        self
    }
    
    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.labels.insert(key.into(), value.into());
        self
    }
    
    /// Add option
    pub fn option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.options.insert(key.into(), value.into());
        self
    }
    
    /// Build the network
    pub fn build(self) -> NetworkConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::MockRuntime;
    
    #[tokio::test]
    async fn test_network_creation() {
        let runtime = Box::new(MockRuntime::new());
        let network = ContainerNetwork::new(runtime);
        
        let config = NetworkBuilder::new("test-net")
            .driver(NetworkDriver::Bridge)
            .subnet("172.20.0.0/16")
            .label("test", "true")
            .build();
        
        let info = network.create_network(config).await.unwrap();
        assert_eq!(info.name, "test-net");
        assert_eq!(info.driver, NetworkDriver::Bridge);
        
        // List networks
        let networks = network.list_networks().await.unwrap();
        assert_eq!(networks.len(), 1);
    }
    
    #[tokio::test]
    async fn test_container_connect() {
        use super::super::{ContainerConfig, ContainerRuntime};
        
        let runtime = Box::new(MockRuntime::new());
        let network = ContainerNetwork::new(runtime.clone());
        
        // Create network
        let config = NetworkBuilder::new("sector-net").build();
        let net_info = network.create_network(config).await.unwrap();
        
        // Create container directly using runtime (so it's in the same runtime as network)
        let container_config = ContainerConfig {
            name: "test-container".to_string(),
            image: "alpine:latest".to_string(),
            ..Default::default()
        };
        let container = runtime.create_container(container_config).await.unwrap();
        
        // Connect container to network
        network.connect_container(&net_info.name, &container.id, None).await.unwrap();
        
        // Verify connection
        let net_info = network.get_network(&net_info.name).await.unwrap();
        assert_eq!(net_info.containers.len(), 1);
        assert_eq!(net_info.containers[0].container_id, container.id);
        
        // Get container IP
        let ip = network.get_container_ip(&net_info.name, &container.id).await.unwrap();
        assert!(ip.to_string().starts_with("172.20."));
    }
    
    #[test]
    fn test_network_builder() {
        let config = NetworkBuilder::new("my-network")
            .driver(NetworkDriver::Overlay)
            .subnet("10.0.0.0/24")
            .gateway("10.0.0.1".parse().unwrap())
            .enable_ipv6()
            .internal()
            .label("env", "prod")
            .option("com.docker.network.bridge.name", "br0")
            .build();
        
        assert_eq!(config.name, "my-network");
        assert_eq!(config.driver, NetworkDriver::Overlay);
        assert_eq!(config.subnet, "10.0.0.0/24");
        assert!(config.enable_ipv6);
        assert!(config.internal);
    }
    
    #[test]
    fn test_port_mapping() {
        let mapping = PortMapping {
            host_port: 8080,
            container_port: 80,
            protocol: Protocol::Tcp,
            host_ip: Some("127.0.0.1".parse().unwrap()),
        };
        
        assert_eq!(mapping.host_port, 8080);
        assert_eq!(mapping.container_port, 80);
        assert_eq!(mapping.protocol, Protocol::Tcp);
    }
    
    #[test]
    fn test_generate_mac() {
        let mac = generate_mac_address();
        assert_eq!(mac.len(), 17); // xx:xx:xx:xx:xx:xx
        assert!(mac.starts_with("02:")); // Locally administered
    }
}
