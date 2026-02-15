//! TOS Marketplace and Templates System
//! 
//! Phase 9 Implementation: Provides package management, repository handling,
//! template export/import, and digital signature verification for TOS modules.
//! 
//! ## Package Types
//! 
//! - **Sector Template** (`.tos-template`): Configuration-only export from any sector
//! - **Sector Type** (`.tos-sector`): Module package containing code and metadata
//! - **Application Model** (`.tos-appmodel`): Application model module package
//! 
//! ## Security
//! 
//! - SHA256 checksum verification
//! - Minisign signature verification
//! - Permission prompts for installation
//! - Optional containerization for code packages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod repository;
pub mod package;
pub mod template;
pub mod signature;
pub mod dependency;

pub use repository::{Repository, RepositoryIndex, RepositoryManager};
pub use package::{Package, PackageType, PackageManager, PackageInstaller};
pub use template::{Template, TemplateMetadata, TemplateExporter, TemplateImporter};
pub use signature::{SignatureVerifier, SignatureError};
pub use dependency::{DependencyResolver, DependencyGraph};

/// Marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// List of configured repositories
    pub repositories: Vec<RepositoryConfig>,
    /// Default repository for searches
    pub default_repository: Option<String>,
    /// Cache directory for downloaded packages
    pub cache_dir: PathBuf,
    /// Whether to verify signatures by default
    pub verify_signatures: bool,
    /// Whether to auto-install dependencies
    pub auto_install_dependencies: bool,
    /// Trusted public keys for signature verification
    pub trusted_keys: Vec<String>,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Self {
            repositories: Vec::new(),
            default_repository: None,
            cache_dir: PathBuf::from(format!("{}/.cache/tos/marketplace", home)),
            verify_signatures: true,
            auto_install_dependencies: true,
            trusted_keys: Vec::new(),
        }
    }
}

/// Repository configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Repository name (unique identifier)
    pub name: String,
    /// Repository URL (HTTPS endpoint)
    pub url: String,
    /// Whether this repository is enabled
    pub enabled: bool,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Optional authentication token
    pub auth_token: Option<String>,
}

/// Package metadata from repository index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author information
    pub author: String,
    /// License (SPDX identifier)
    pub license: String,
    /// Package type
    pub package_type: PackageType,
    /// Download URL
    pub download_url: String,
    /// SHA256 checksum
    pub sha256: String,
    /// Signature (optional)
    pub signature: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Minimum TOS version required
    pub min_tos_version: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Size in bytes
    pub size: u64,
    /// Creation timestamp
    pub created_at: String,
}

/// Installation request
#[derive(Debug, Clone)]
pub struct InstallRequest {
    /// Package name
    pub package_name: String,
    /// Version constraint (e.g., "1.0.0", ">=1.0.0", "latest")
    pub version_constraint: String,
    /// Target repository (None = search all)
    pub repository: Option<String>,
    /// Whether to auto-accept permissions
    pub auto_accept: bool,
    /// Whether to skip signature verification
    pub skip_signature_check: bool,
}

/// Installation result
#[derive(Debug, Clone)]
pub struct InstallResult {
    /// Installed package metadata
    pub package: PackageMetadata,
    /// Installation path
    pub install_path: PathBuf,
    /// Installed dependencies
    pub installed_dependencies: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

/// Export request for sector templates
#[derive(Debug, Clone)]
pub struct ExportRequest {
    /// Sector identifier to export
    pub sector_id: String,
    /// Template name
    pub name: String,
    /// Template version
    pub version: String,
    /// Output path
    pub output_path: PathBuf,
    /// Description
    pub description: String,
    /// Author
    pub author: String,
    /// License
    pub license: String,
    /// Whether to include application state
    pub include_state: bool,
    /// Tags
    pub tags: Vec<String>,
}

/// Export result
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Path to exported template
    pub template_path: PathBuf,
    /// Template size in bytes
    pub size: u64,
    /// SHA256 checksum
    pub sha256: String,
}

/// Marketplace error types
#[derive(Debug)]
pub enum MarketplaceError {
    /// Network error
    Network(String),
    /// Parse error
    Parse(String),
    /// Validation error
    Validation(String),
    /// Installation error
    Installation(String),
    /// Signature verification error
    Signature(String),
    /// Dependency resolution error
    Dependency(String),
    /// IO error
    Io(std::io::Error),
    /// Package not found
    NotFound(String),
    /// Permission denied
    PermissionDenied(String),
}

impl std::fmt::Display for MarketplaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplaceError::Network(e) => write!(f, "Network error: {}", e),
            MarketplaceError::Parse(e) => write!(f, "Parse error: {}", e),
            MarketplaceError::Validation(e) => write!(f, "Validation error: {}", e),
            MarketplaceError::Installation(e) => write!(f, "Installation error: {}", e),
            MarketplaceError::Signature(e) => write!(f, "Signature error: {}", e),
            MarketplaceError::Dependency(e) => write!(f, "Dependency error: {}", e),
            MarketplaceError::Io(e) => write!(f, "IO error: {}", e),
            MarketplaceError::NotFound(e) => write!(f, "Not found: {}", e),
            MarketplaceError::PermissionDenied(e) => write!(f, "Permission denied: {}", e),
        }
    }
}

impl std::error::Error for MarketplaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MarketplaceError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MarketplaceError {
    fn from(e: std::io::Error) -> Self {
        MarketplaceError::Io(e)
    }
}

impl From<reqwest::Error> for MarketplaceError {
    fn from(e: reqwest::Error) -> Self {
        MarketplaceError::Network(e.to_string())
    }
}

impl From<serde_json::Error> for MarketplaceError {
    fn from(e: serde_json::Error) -> Self {
        MarketplaceError::Parse(e.to_string())
    }
}

impl From<zip::result::ZipError> for MarketplaceError {
    fn from(e: zip::result::ZipError) -> Self {
        MarketplaceError::Installation(e.to_string())
    }
}

/// Main marketplace manager
pub struct Marketplace {
    /// Configuration
    pub config: MarketplaceConfig,
    /// Repository manager
    pub repository_manager: RepositoryManager,
    /// Package manager
    pub package_manager: PackageManager,
    /// Template exporter/importer
    pub template_handler: template::TemplateHandler,
    /// Signature verifier
    pub signature_verifier: SignatureVerifier,
    /// Dependency resolver
    pub dependency_resolver: DependencyResolver,
}

impl Marketplace {
    /// Create a new marketplace instance with default configuration
    pub fn new() -> Self {
        let config = MarketplaceConfig::default();
        Self::with_config(config)
    }
    
    /// Create a marketplace with custom configuration
    pub fn with_config(config: MarketplaceConfig) -> Self {
        let repository_manager = RepositoryManager::new(config.repositories.clone());
        let package_manager = PackageManager::new(config.cache_dir.clone());
        let template_handler = template::TemplateHandler::new();
        let signature_verifier = SignatureVerifier::new(config.trusted_keys.clone());
        let dependency_resolver = DependencyResolver::new();
        
        Self {
            config,
            repository_manager,
            package_manager,
            template_handler,
            signature_verifier,
            dependency_resolver,
        }
    }
    
    /// Initialize the marketplace (create cache directory, etc.)
    pub fn initialize(&self) -> Result<(), MarketplaceError> {
        std::fs::create_dir_all(&self.config.cache_dir)?;
        Ok(())
    }
    
    /// Search for packages across all enabled repositories
    pub async fn search(&self, query: &str) -> Result<Vec<PackageMetadata>, MarketplaceError> {
        let mut results = Vec::new();
        
        for repo in self.repository_manager.enabled_repositories() {
            match repo.search(query).await {
                Ok(packages) => results.extend(packages),
                Err(e) => tracing::warn!("Repository {} search failed: {}", repo.name(), e),
            }
        }
        
        // Remove duplicates (same name/version from multiple repos)
        results.sort_by(|a, b| {
            a.name.cmp(&b.name)
                .then_with(|| a.version.cmp(&b.version))
        });
        results.dedup_by(|a, b| a.name == b.name && a.version == b.version);
        
        Ok(results)
    }
    
    /// Install a package
    pub async fn install(&self, request: InstallRequest) -> Result<InstallResult, MarketplaceError> {
        self.install_internal(request, 0).await
    }
    
    /// Internal install with depth tracking to avoid recursion issues
    async fn install_internal(&self, request: InstallRequest, depth: u32) -> Result<InstallResult, MarketplaceError> {
        const MAX_DEPTH: u32 = 50;
        if depth > MAX_DEPTH {
            return Err(MarketplaceError::Dependency(
                "Maximum dependency depth exceeded".to_string()
            ));
        }
        
        // Find the package
        let package = self.find_package(&request).await?;
        
        // Resolve dependencies
        let dependencies = if self.config.auto_install_dependencies {
            self.dependency_resolver.resolve(&package, &self.repository_manager).await?
        } else {
            Vec::new()
        };
        
        // Download the package
        let package_path = self.package_manager.download(&package).await?;
        
        // Verify signature if required
        if self.config.verify_signatures && !request.skip_signature_check {
            if let Some(ref sig) = package.signature {
                self.signature_verifier.verify(&package_path, sig)?;
            } else {
                return Err(MarketplaceError::Signature(
                    "Package has no signature but verification is required".to_string()
                ));
            }
        }
        
        // Install dependencies first (iteratively to avoid async recursion)
        let mut installed_deps = Vec::new();
        for dep in dependencies {
            let dep_request = InstallRequest {
                package_name: dep.name.clone(),
                version_constraint: dep.version.clone(),
                repository: request.repository.clone(),
                auto_accept: request.auto_accept,
                skip_signature_check: request.skip_signature_check,
            };
            // Use Box::pin to handle the recursive async call
            let dep_result = Box::pin(self.install_internal(dep_request, depth + 1)).await?;
            installed_deps.push(dep_result.package.name);
        }
        
        // Install the main package
        let install_path = self.package_manager.install(&package_path, &package).await?;
        
        Ok(InstallResult {
            package,
            install_path,
            installed_dependencies: installed_deps,
            warnings: Vec::new(),
        })
    }
    
    /// Export a sector as a template
    pub fn export_sector(&self, request: ExportRequest) -> Result<ExportResult, MarketplaceError> {
        self.template_handler.export_sector(request)
    }
    
    /// Import a template
    pub fn import_template(&self, path: &Path) -> Result<Template, MarketplaceError> {
        self.template_handler.import_template(path)
    }
    
    /// Add a repository
    pub fn add_repository(&mut self, config: RepositoryConfig) {
        self.repository_manager.add_repository(config.clone());
        self.config.repositories.push(config);
    }
    
    /// Remove a repository
    pub fn remove_repository(&mut self, name: &str) {
        self.repository_manager.remove_repository(name);
        self.config.repositories.retain(|r| r.name != name);
    }
    
    /// Find a package matching the install request
    async fn find_package(&self, request: &InstallRequest) -> Result<PackageMetadata, MarketplaceError> {
        if let Some(ref repo_name) = request.repository {
            // Search specific repository
            let repo = self.repository_manager.get_repository(repo_name)
                .ok_or_else(|| MarketplaceError::NotFound(
                    format!("Repository {} not found", repo_name)
                ))?;
            repo.find_package(&request.package_name, &request.version_constraint).await
        } else {
            // Search all repositories
            let repos = self.repository_manager.enabled_repositories();
            for repo in repos {
                if let Ok(package) = repo.find_package(&request.package_name, &request.version_constraint).await {
                    return Ok(package);
                }
            }
            Err(MarketplaceError::NotFound(
                format!("Package {}@{} not found in any repository", 
                    request.package_name, request.version_constraint)
            ))
        }
    }
}

impl Default for Marketplace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_marketplace_config_default() {
        let config = MarketplaceConfig::default();
        assert!(config.verify_signatures);
        assert!(config.auto_install_dependencies);
        assert!(config.repositories.is_empty());
    }
    
    #[test]
    fn test_repository_config() {
        let repo = RepositoryConfig {
            name: "test-repo".to_string(),
            url: "https://example.com/repo".to_string(),
            enabled: true,
            priority: 1,
            auth_token: None,
        };
        
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.priority, 1);
    }
    
    #[test]
    fn test_package_metadata() {
        let metadata = PackageMetadata {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "Test package".to_string(),
            author: "Test Author".to_string(),
            license: "MIT".to_string(),
            package_type: PackageType::SectorType,
            download_url: "https://example.com/pkg.tos-sector".to_string(),
            sha256: "abc123".to_string(),
            signature: None,
            dependencies: vec![],
            min_tos_version: None,
            tags: vec!["test".to_string()],
            size: 1024,
            created_at: "2024-01-01".to_string(),
        };
        
        assert_eq!(metadata.name, "test-package");
        assert_eq!(metadata.package_type, PackageType::SectorType);
    }
    
    #[test]
    fn test_marketplace_error_display() {
        let err = MarketplaceError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Not found: test");
        
        let err = MarketplaceError::Network("timeout".to_string());
        assert_eq!(err.to_string(), "Network error: timeout");
    }
}
