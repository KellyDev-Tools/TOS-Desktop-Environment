//! Package Management
//! 
//! Handles package download, verification, installation, and archive extraction.
//! Supports .tos-template, .tos-sector, and .tos-appmodel package formats.

use super::{MarketplaceError, PackageMetadata};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Package type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    /// Sector template (configuration only)
    #[serde(rename = "template")]
    Template,
    /// Sector type module
    #[serde(rename = "sector-type")]
    SectorType,
    /// Application model module
    #[serde(rename = "app-model")]
    ApplicationModel,
}

impl std::fmt::Display for PackageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageType::Template => write!(f, "template"),
            PackageType::SectorType => write!(f, "sector-type"),
            PackageType::ApplicationModel => write!(f, "app-model"),
        }
    }
}

/// Package file extension mapping
impl PackageType {
    /// Get the file extension for this package type
    pub fn extension(&self) -> &'static str {
        match self {
            PackageType::Template => ".tos-template",
            PackageType::SectorType => ".tos-sector",
            PackageType::ApplicationModel => ".tos-appmodel",
        }
    }
    
    /// Parse package type from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            ".tos-template" => Some(PackageType::Template),
            ".tos-sector" => Some(PackageType::SectorType),
            ".tos-appmodel" => Some(PackageType::ApplicationModel),
            _ => None,
        }
    }
    
    /// Get the installation directory name
    pub fn install_dir(&self) -> &'static str {
        match self {
            PackageType::Template => "templates",
            PackageType::SectorType => "sector-types",
            PackageType::ApplicationModel => "app-models",
        }
    }
}

/// Package archive structure
#[derive(Debug)]
pub struct Package {
    /// Package metadata
    pub metadata: PackageMetadata,
    /// Package type
    pub package_type: PackageType,
    /// Archive contents
    pub contents: Vec<PackageFile>,
}

/// File entry in a package
#[derive(Debug, Clone)]
pub struct PackageFile {
    /// File path within archive
    pub path: String,
    /// File contents
    pub data: Vec<u8>,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File permissions (Unix mode)
    pub permissions: Option<u32>,
}

/// Package manager for downloads and caching
#[derive(Clone)]
pub struct PackageManager {
    /// Cache directory for downloaded packages
    cache_dir: PathBuf,
    /// HTTP client
    client: reqwest::Client,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            client: reqwest::Client::new(),
        }
    }
    
    /// Download a package to cache
    pub async fn download(&self, metadata: &PackageMetadata) -> Result<PathBuf, MarketplaceError> {
        // Create cache directory if needed
        std::fs::create_dir_all(&self.cache_dir)?;
        
        // Generate cache filename
        let filename = format!(
            "{}-{}-{}{}",
            metadata.name,
            metadata.version,
            &metadata.sha256[..8], // Use first 8 chars of hash for uniqueness
            metadata.package_type.extension()
        );
        let cache_path = self.cache_dir.join(&filename);
        
        // Check if already cached
        if cache_path.exists() {
            // Verify checksum
            if self.verify_checksum(&cache_path, &metadata.sha256)? {
                tracing::info!("Using cached package: {}", filename);
                return Ok(cache_path);
            } else {
                tracing::warn!("Cached package checksum mismatch, re-downloading");
            }
        }
        
        // Download the package
        tracing::info!("Downloading package: {} from {}", metadata.name, metadata.download_url);
        
        let response = self.client.get(&metadata.download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(MarketplaceError::Network(
                format!("HTTP {} downloading {}", response.status(), metadata.download_url)
            ));
        }
        
        let bytes = response.bytes().await?;
        
        // Verify checksum before saving
        let computed_hash = self.compute_hash(&bytes);
        if computed_hash != metadata.sha256 {
            return Err(MarketplaceError::Validation(
                format!("Checksum mismatch: expected {}, got {}", metadata.sha256, computed_hash)
            ));
        }
        
        // Save to cache
        let mut file = std::fs::File::create(&cache_path)?;
        file.write_all(&bytes)?;
        
        tracing::info!("Package downloaded and cached: {}", filename);
        Ok(cache_path)
    }
    
    /// Install a package from cache to the modules directory
    pub async fn install(
        &self,
        package_path: &Path,
        metadata: &PackageMetadata,
    ) -> Result<PathBuf, MarketplaceError> {
        // Determine installation directory
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let install_base = PathBuf::from(format!("{}/.local/share/tos", home));
        let install_dir = install_base.join(metadata.package_type.install_dir());
        
        // Create install directory
        std::fs::create_dir_all(&install_dir)?;
        
        // Create module directory
        let module_dir = install_dir.join(&metadata.name);
        
        // Check if already installed
        if module_dir.exists() {
            // Backup existing installation
            let backup_dir = install_dir.join(format!("{}.backup-{}", metadata.name, 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()));
            std::fs::rename(&module_dir, &backup_dir)?;
            tracing::info!("Backed up existing installation to {}", backup_dir.display());
        }
        
        // Extract package
        self.extract_package(package_path, &module_dir, metadata.package_type)?;
        
        tracing::info!("Package installed: {} to {}", metadata.name, module_dir.display());
        Ok(module_dir)
    }
    
    /// Uninstall a package
    pub fn uninstall(&self, metadata: &PackageMetadata) -> Result<(), MarketplaceError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let install_dir = PathBuf::from(format!("{}/.local/share/tos/{}", 
            home, metadata.package_type.install_dir()));
        let module_dir = install_dir.join(&metadata.name);
        
        if module_dir.exists() {
            std::fs::remove_dir_all(&module_dir)?;
            tracing::info!("Package uninstalled: {}", metadata.name);
        } else {
            return Err(MarketplaceError::NotFound(
                format!("Package {} not found at {}", metadata.name, module_dir.display())
            ));
        }
        
        Ok(())
    }
    
    /// Extract a package archive
    fn extract_package(
        &self,
        package_path: &Path,
        target_dir: &Path,
        package_type: PackageType,
    ) -> Result<(), MarketplaceError> {
        match package_type {
            PackageType::Template => {
                // Templates are simple ZIP archives
                self.extract_zip(package_path, target_dir)
            }
            PackageType::SectorType | PackageType::ApplicationModel => {
                // Module packages are also ZIP archives with manifest
                self.extract_zip(package_path, target_dir)?;
                
                // Verify manifest exists
                let manifest_path = target_dir.join("module.toml");
                if !manifest_path.exists() {
                    let json_manifest = target_dir.join("module.json");
                    if !json_manifest.exists() {
                        return Err(MarketplaceError::Validation(
                            "Package missing module.toml or module.json manifest".to_string()
                        ));
                    }
                }
                
                Ok(())
            }
        }
    }
    
    /// Extract ZIP archive
    fn extract_zip(&self, zip_path: &Path, target_dir: &Path) -> Result<(), MarketplaceError> {
        let file = std::fs::File::open(zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        std::fs::create_dir_all(target_dir)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = target_dir.join(file.name());
            
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
                
                // Set permissions if available
                #[cfg(unix)]
                if let Some(permissions) = file.unix_mode() {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(permissions))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Create a ZIP package from directory
    pub fn create_package(
        &self,
        source_dir: &Path,
        output_path: &Path,
        package_type: PackageType,
    ) -> Result<(), MarketplaceError> {
        let file = std::fs::File::create(output_path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        self.add_directory_to_zip(&mut zip, source_dir, source_dir, options)?;
        
        zip.finish()?;
        
        tracing::info!("Package created: {} ({})", output_path.display(), package_type);
        Ok(())
    }
    
    /// Recursively add directory to ZIP
    fn add_directory_to_zip(
        &self,
        zip: &mut zip::ZipWriter<std::fs::File>,
        base_path: &Path,
        current_dir: &Path,
        options: zip::write::FileOptions,
    ) -> Result<(), MarketplaceError> {
        for entry in std::fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.strip_prefix(base_path)
                .map_err(|e| MarketplaceError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData, e.to_string())))?
                .to_string_lossy();
            
            if path.is_file() {
                zip.start_file(name.to_string(), options)?;
                let mut file = std::fs::File::open(&path)?;
                std::io::copy(&mut file, zip)?;
            } else if path.is_dir() {
                zip.add_directory(name.to_string() + "/", options)?;
                self.add_directory_to_zip(zip, base_path, &path, options)?;
            }
        }
        
        Ok(())
    }
    
    /// Verify SHA256 checksum of a file
    fn verify_checksum(&self, path: &Path, expected_hash: &str) -> Result<bool, MarketplaceError> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let result = hasher.finalize();
        let computed_hash = hex::encode(result);
        
        Ok(computed_hash == expected_hash)
    }
    
    /// Compute SHA256 hash of bytes
    fn compute_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
    
    /// Get cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
    
    /// Clear package cache
    pub fn clear_cache(&self) -> Result<(), MarketplaceError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
    
    /// List cached packages
    pub fn list_cache(&self) -> Result<Vec<(String, u64)>, MarketplaceError> {
        let mut packages = Vec::new();
        
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let size = entry.metadata()?.len();
                    packages.push((name, size));
                }
            }
        }
        
        Ok(packages)
    }
}

/// Package installer with permission handling
pub struct PackageInstaller {
    /// Package manager reference
    package_manager: PackageManager,
    /// Whether to auto-accept permissions
    auto_accept: bool,
    /// Permission callback (if not auto-accept)
    permission_callback: Option<Box<dyn Fn(&str, &[String]) -> bool>>,
}

impl PackageInstaller {
    /// Create a new package installer
    pub fn new(package_manager: PackageManager) -> Self {
        Self {
            package_manager,
            auto_accept: false,
            permission_callback: None,
        }
    }
    
    /// Set auto-accept for permissions
    pub fn with_auto_accept(mut self, auto_accept: bool) -> Self {
        self.auto_accept = auto_accept;
        self
    }
    
    /// Set permission callback
    pub fn with_permission_callback<F>(mut self, callback: F) -> Self 
    where
        F: Fn(&str, &[String]) -> bool + 'static,
    {
        self.permission_callback = Some(Box::new(callback));
        self
    }
    
    /// Install with permission check
    pub async fn install_with_permissions(
        &self,
        package_path: &Path,
        metadata: &PackageMetadata,
    ) -> Result<PathBuf, MarketplaceError> {
        // Check if we need permission confirmation
        if !metadata.dependencies.is_empty() && !self.auto_accept {
            if let Some(ref callback) = self.permission_callback {
                let dep_names: Vec<String> = metadata.dependencies.clone();
                if !callback(&metadata.name, &dep_names) {
                return Err(MarketplaceError::PermissionDenied(
                    "User declined dependency installation".to_string()
                ));

                }
            }
        }
        
        self.package_manager.install(package_path, metadata).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_package_type_extension() {
        assert_eq!(PackageType::Template.extension(), ".tos-template");
        assert_eq!(PackageType::SectorType.extension(), ".tos-sector");
        assert_eq!(PackageType::ApplicationModel.extension(), ".tos-appmodel");
    }
    
    #[test]
    fn test_package_type_from_extension() {
        assert_eq!(PackageType::from_extension(".tos-template"), Some(PackageType::Template));
        assert_eq!(PackageType::from_extension(".tos-sector"), Some(PackageType::SectorType));
        assert_eq!(PackageType::from_extension(".unknown"), None);
    }
    
    #[test]
    fn test_package_type_display() {
        assert_eq!(PackageType::Template.to_string(), "template");
        assert_eq!(PackageType::SectorType.to_string(), "sector-type");
    }
    
    #[test]
    fn test_compute_hash() {
        let manager = PackageManager::new(PathBuf::from("/tmp"));
        let data = b"hello world";
        let hash = manager.compute_hash(data);
        assert_eq!(hash.len(), 64); // SHA256 hex string length
    }
}
