//! Phase 16: Container Image Management
//!
//! Manages container images including building, pulling, caching,
//! and registry integration for TOS containerized components.

use super::{ContainerResult, ContainerError, ContainerRuntime};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image ID (digest)
    pub id: String,
    /// Repository name
    pub repository: String,
    /// Tag
    pub tag: String,
    /// Full image reference (repo:tag)
    pub reference: String,
    /// Image size in bytes
    pub size: u64,
    /// Creation timestamp
    pub created: chrono::DateTime<chrono::Utc>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Parent image ID
    pub parent: Option<String>,
    /// Architecture
    pub architecture: String,
    /// OS
    pub os: String,
    /// Layer count
    pub layers: usize,
    /// Config (exposed ports, env vars, etc.)
    pub config: ImageConfig,
}

/// Image configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageConfig {
    /// Exposed ports
    pub exposed_ports: Vec<u16>,
    /// Environment variables
    pub env: Vec<String>,
    /// Default command
    pub cmd: Vec<String>,
    /// Entrypoint
    pub entrypoint: Vec<String>,
    /// Working directory
    pub working_dir: String,
    /// User
    pub user: String,
    /// Volumes
    pub volumes: Vec<String>,
    /// Labels
    pub labels: HashMap<String, String>,
}

/// Image manager handles image operations
#[derive(Debug)]
pub struct ImageManager {
    _runtime: Box<dyn ContainerRuntime>,
    cache: std::sync::Mutex<HashMap<String, ImageInfo>>,
}

impl ImageManager {
    /// Create a new image manager
    pub fn new(runtime: Box<dyn ContainerRuntime>) -> Self {
        Self {
            _runtime: runtime,
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    /// Check if an image exists locally
    pub async fn image_exists(&self, reference: &str) -> ContainerResult<bool> {
        // Check cache first
        if self.cache.lock().unwrap().contains_key(reference) {
            return Ok(true);
        }
        
        // Query runtime
        match self.get_image(reference).await {
            Ok(_) => Ok(true),
            Err(ContainerError::ImageNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    /// Get image information
    pub async fn get_image(&self, reference: &str) -> ContainerResult<ImageInfo> {
        // Check cache
        if let Some(info) = self.cache.lock().unwrap().get(reference) {
            return Ok(info.clone());
        }
        
        // Query runtime (mock implementation)
        // In real implementation, this would call the container runtime API
        Err(ContainerError::ImageNotFound(reference.to_string()))
    }
    
    /// Pull an image from registry
    pub async fn pull_image(&self, reference: &str) -> ContainerResult<ImageInfo> {
        tracing::info!("Pulling image: {}", reference);
        
        // Parse reference
        let (repository, tag) = parse_image_reference(reference);
        
        // In real implementation, this would:
        // 1. Authenticate with registry if needed
        // 2. Pull image layers
        // 3. Verify checksums
        // 4. Store locally
        
        // Mock implementation
        let info = ImageInfo {
            id: format!("sha256:{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            repository: repository.clone(),
            tag: tag.clone(),
            reference: reference.to_string(),
            size: 100_000_000, // 100MB mock
            created: chrono::Utc::now(),
            labels: HashMap::new(),
            parent: None,
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            layers: 5,
            config: ImageConfig::default(),
        };
        
        // Cache the image
        self.cache.lock().unwrap().insert(reference.to_string(), info.clone());
        
        tracing::info!("Pulled image: {} ({} layers, {} bytes)", 
            reference, info.layers, info.size);
        
        Ok(info)
    }
    
    /// Pull an image with authentication
    pub async fn pull_image_with_auth(
        &self,
        reference: &str,
        _username: &str,
        _password: &str,
    ) -> ContainerResult<ImageInfo> {
        tracing::info!("Pulling image with auth: {}", reference);
        
        // In real implementation, this would use registry credentials
        // For now, delegate to regular pull
        self.pull_image(reference).await
    }
    
    /// Push an image to registry
    pub async fn push_image(
        &self,
        reference: &str,
        _username: &str,
        _password: &str,
    ) -> ContainerResult<()> {
        tracing::info!("Pushing image: {}", reference);
        
        // Verify image exists locally
        if !self.image_exists(reference).await? {
            return Err(ContainerError::ImageNotFound(reference.to_string()));
        }
        
        // In real implementation, this would:
        // 1. Authenticate with registry
        // 2. Push image layers
        // 3. Update manifest
        
        tracing::info!("Pushed image: {}", reference);
        Ok(())
    }
    
    /// Remove an image
    pub async fn remove_image(&self, reference: &str, force: bool) -> ContainerResult<()> {
        tracing::info!("Removing image: {}", reference);
        
        // Check if image is in use
        if !force {
            // In real implementation, check if any containers use this image
        }
        
        // Remove from cache
        self.cache.lock().unwrap().remove(reference);
        
        tracing::info!("Removed image: {}", reference);
        Ok(())
    }
    
    /// List all images
    pub async fn list_images(&self) -> ContainerResult<Vec<ImageInfo>> {
        let cache = self.cache.lock().unwrap();
        let images: Vec<_> = cache.values().cloned().collect();
        Ok(images)
    }
    
    /// Search for images in registry
    pub async fn search_images(&self, term: &str) -> ContainerResult<Vec<ImageSearchResult>> {
        tracing::info!("Searching for images: {}", term);
        
        // In real implementation, this would query Docker Hub or other registries
        
        // Mock results
        let results = vec![
            ImageSearchResult {
                name: format!("tos/{}", term),
                description: "Official TOS image".to_string(),
                stars: 1000,
                official: true,
                automated: false,
            },
            ImageSearchResult {
                name: format!("tos/{}-minimal", term),
                description: "Minimal TOS image".to_string(),
                stars: 500,
                official: true,
                automated: false,
            },
        ];
        
        Ok(results)
    }
    
    /// Get image history (layers)
    pub async fn get_image_history(&self, _reference: &str) -> ContainerResult<Vec<ImageLayer>> {
        // In real implementation, query runtime for layer history
        Ok(vec![
            ImageLayer {
                id: "layer1".to_string(),
                created: chrono::Utc::now(),
                created_by: "ADD file:abc123".to_string(),
                size: 50_000_000,
                comment: "Base layer".to_string(),
            },
        ])
    }
    
    /// Tag an image
    pub async fn tag_image(&self, source: &str, target: &str) -> ContainerResult<()> {
        tracing::info!("Tagging image: {} -> {}", source, target);
        
        // Get source image
        let mut info = self.get_image(source).await?;
        
        // Update reference
        info.reference = target.to_string();
        let (repo, tag) = parse_image_reference(target);
        info.repository = repo;
        info.tag = tag;
        
        // Store under new reference
        self.cache.lock().unwrap().insert(target.to_string(), info);
        
        Ok(())
    }
    
    /// Prune unused images
    pub async fn prune_images(&self) -> ContainerResult<PruneResult> {
        tracing::info!("Pruning unused images");
        
        // In real implementation, remove dangling images
        
        Ok(PruneResult {
            images_deleted: 0,
            space_reclaimed: 0,
        })
    }
    
    /// Get image size
    pub async fn get_image_size(&self, reference: &str) -> ContainerResult<u64> {
        let info = self.get_image(reference).await?;
        Ok(info.size)
    }
    
    /// Export image to tar archive
    pub async fn export_image(&self, reference: &str, path: &Path) -> ContainerResult<()> {
        tracing::info!("Exporting image {} to {}", reference, path.display());
        
        // In real implementation, export image layers to tar
        Ok(())
    }
    
    /// Import image from tar archive
    pub async fn import_image(&self, path: &Path, reference: &str) -> ContainerResult<ImageInfo> {
        tracing::info!("Importing image from {} as {}", path.display(), reference);
        
        // In real implementation, import from tar
        self.pull_image(reference).await
    }
}

/// Image search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSearchResult {
    pub name: String,
    pub description: String,
    pub stars: u32,
    pub official: bool,
    pub automated: bool,
}

/// Image layer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLayer {
    pub id: String,
    pub created: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub size: u64,
    pub comment: String,
}

/// Prune operation result
#[derive(Debug, Clone, Copy)]
pub struct PruneResult {
    pub images_deleted: usize,
    pub space_reclaimed: u64,
}

/// Image builder for creating custom TOS images
#[derive(Debug)]
pub struct ImageBuilder {
    context_path: PathBuf,
    dockerfile: String,
    build_args: HashMap<String, String>,
    labels: HashMap<String, String>,
    tags: Vec<String>,
    no_cache: bool,
    pull: bool,
    platform: String,
}

impl ImageBuilder {
    /// Create a new image builder
    pub fn new(context_path: impl AsRef<Path>) -> Self {
        Self {
            context_path: context_path.as_ref().to_path_buf(),
            dockerfile: "Dockerfile".to_string(),
            build_args: HashMap::new(),
            labels: HashMap::new(),
            tags: Vec::new(),
            no_cache: false,
            pull: false,
            platform: "linux/amd64".to_string(),
        }
    }
    
    /// Set Dockerfile name
    pub fn dockerfile(mut self, name: impl Into<String>) -> Self {
        self.dockerfile = name.into();
        self
    }
    
    /// Add build argument
    pub fn build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_args.insert(key.into(), value.into());
        self
    }
    
    /// Add label
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
    
    /// Add tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Disable cache
    pub fn no_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }
    
    /// Always pull base images
    pub fn pull(mut self) -> Self {
        self.pull = true;
        self
    }
    
    /// Set target platform
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = platform.into();
        self
    }
    
    /// Build the image
    pub async fn build(self) -> ContainerResult<ImageInfo> {
        tracing::info!("Building image from {}", self.context_path.display());
        
        // In real implementation:
        // 1. Create build context (tar archive)
        // 2. Send to Docker API
        // 3. Stream build output
        // 4. Tag result
        
        let primary_reference = self.tags.first()
            .cloned()
            .unwrap_or_else(|| "tos/custom:latest".to_string());
        
        let (repository, tag) = parse_image_reference(&primary_reference);
        
        let info = ImageInfo {
            id: format!("sha256:{}", uuid::Uuid::new_v4().to_string().replace("-", "")),
            repository: repository.clone(),
            tag: tag.clone(),
            reference: format!("{}:{}", repository, tag),
            size: 200_000_000, // 200MB mock
            created: chrono::Utc::now(),
            labels: self.labels,
            parent: None,
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            layers: 10,
            config: ImageConfig::default(),
        };
        
        tracing::info!("Built image: {} ({} bytes)", info.reference, info.size);
        
        Ok(info)
    }
}

/// Parse image reference into (repository, tag)
fn parse_image_reference(reference: &str) -> (String, String) {
    if let Some(pos) = reference.rfind(':') {
        // Check if it's a port (e.g., registry:5000/image) or tag
        let after_colon = &reference[pos+1..];
        if after_colon.contains('/') || after_colon.contains(':') {
            // It's part of the registry URL, no tag
            (reference.to_string(), "latest".to_string())
        } else {
            // It's a tag
            let repo = &reference[..pos];
            let tag = after_colon;
            (repo.to_string(), tag.to_string())
        }
    } else {
        // No tag specified, use latest
        (reference.to_string(), "latest".to_string())
    }
}

/// Registry authentication
#[derive(Debug, Clone)]
pub struct RegistryAuth {
    pub username: String,
    pub password: String,
    pub server_address: String,
    pub identity_token: Option<String>,
    pub registry_token: Option<String>,
}

impl RegistryAuth {
    /// Create basic auth
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            server_address: "https://index.docker.io/v1/".to_string(),
            identity_token: None,
            registry_token: None,
        }
    }
    
    /// Encode as base64 for Docker API
    pub fn encode(&self) -> String {
        let auth = serde_json::json!({
            "username": self.username,
            "password": self.password,
            "serveraddress": self.server_address,
        });
        base64_helper::encode(auth.to_string())
    }
}

/// Base64 encoding helper
mod base64_helper {
    pub fn encode(input: impl AsRef<[u8]>) -> String {
        use std::io::Write;
        let mut encoder = base64::write::EncoderStringWriter::new(
            &base64::engine::general_purpose::STANDARD
        );
        encoder.write_all(input.as_ref()).unwrap();
        encoder.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_image_reference() {
        let (repo, tag) = parse_image_reference("alpine:latest");
        assert_eq!(repo, "alpine");
        assert_eq!(tag, "latest");
        
        let (repo, tag) = parse_image_reference("myregistry:5000/tos/sector");
        assert_eq!(repo, "myregistry:5000/tos/sector");
        assert_eq!(tag, "latest");
        
        let (repo, tag) = parse_image_reference("ubuntu:20.04");
        assert_eq!(repo, "ubuntu");
        assert_eq!(tag, "20.04");
    }
    
    #[tokio::test]
    async fn test_image_manager() {
        use crate::containers::MockRuntime;
        
        let runtime = Box::new(MockRuntime::new());
        let manager = ImageManager::new(runtime);
        
        // Pull image
        let info = manager.pull_image("alpine:latest").await.unwrap();
        assert_eq!(info.repository, "alpine");
        assert_eq!(info.tag, "latest");
        
        // Check exists
        assert!(manager.image_exists("alpine:latest").await.unwrap());
        
        // List images
        let images = manager.list_images().await.unwrap();
        assert_eq!(images.len(), 1);
        
        // Remove image
        manager.remove_image("alpine:latest", false).await.unwrap();
        assert!(!manager.image_exists("alpine:latest").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_image_builder() {
        let builder = ImageBuilder::new("/tmp/build")
            .tag("tos/test:1.0")
            .label("version", "1.0")
            .build_arg("TOS_VERSION", "0.1.0");
        
        let info = builder.build().await.unwrap();
        assert_eq!(info.tag, "1.0");
        assert!(info.labels.contains_key("version"));
    }
}
