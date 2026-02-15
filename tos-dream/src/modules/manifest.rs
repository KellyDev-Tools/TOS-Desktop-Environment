//! Module Manifest System
//! 
//! Defines the manifest format for TOS modules, including:
//! - Module metadata (name, version, description, author, license)
//! - Module type (ApplicationModel, SectorType, Hybrid)
//! - Permissions required by the module
//! - Containerization configuration
//! - Module-specific configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Module type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    #[serde(rename = "app-model")]
    ApplicationModel,
    #[serde(rename = "sector-type")]
    SectorType,
    #[serde(rename = "hybrid")]
    Hybrid,
}

impl Default for ModuleType {
    fn default() -> Self {
        ModuleType::ApplicationModel
    }
}

/// Container backend options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerBackend {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "bubblewrap")]
    Bubblewrap,
    #[serde(rename = "firejail")]
    Firejail,
    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "podman")]
    Podman,
}

impl Default for ContainerBackend {
    fn default() -> Self {
        ContainerBackend::None
    }
}

/// Container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub backend: ContainerBackend,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub read_only_paths: Vec<String>,
    #[serde(default)]
    pub read_write_paths: Vec<String>,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            backend: ContainerBackend::None,
            network: false,
            read_only_paths: Vec::new(),
            read_write_paths: Vec::new(),
        }
    }
}

/// Module manifest - the core metadata for any TOS module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Module name (unique identifier)
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author information
    pub author: String,
    /// License (SPDX identifier)
    pub license: String,
    /// Module type
    #[serde(rename = "type")]
    pub module_type: ModuleType,
    /// Entry point file (relative to module directory)
    pub entry: String,
    /// Programming language (for script-based modules)
    pub language: Option<String>,
    /// Required permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Containerization configuration
    #[serde(default)]
    pub container: ContainerConfig,
    /// Module-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    /// Module dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Minimum TOS version required
    pub min_tos_version: Option<String>,
}

impl ModuleManifest {
    /// Load manifest from TOML file
    pub fn from_toml_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::Io(e))?;
        Self::from_toml_str(&content)
    }
    
    /// Load manifest from JSON file
    pub fn from_json_file(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ManifestError::Io(e))?;
        Self::from_json_str(&content)
    }
    
    /// Parse manifest from TOML string
    pub fn from_toml_str(content: &str) -> Result<Self, ManifestError> {
        toml::from_str(content)
            .map_err(|e| ManifestError::Parse(format!("TOML parse error: {}", e)))
    }
    
    /// Parse manifest from JSON string
    pub fn from_json_str(content: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(content)
            .map_err(|e| ManifestError::Parse(format!("JSON parse error: {}", e)))
    }
    
    /// Validate the manifest
    pub fn validate(&self) -> Result<(), ManifestError> {
        // Check required fields
        if self.name.is_empty() {
            return Err(ManifestError::Validation("Module name cannot be empty".to_string()));
        }
        
        if self.version.is_empty() {
            return Err(ManifestError::Validation("Module version cannot be empty".to_string()));
        }
        
        if self.entry.is_empty() {
            return Err(ManifestError::Validation("Module entry point cannot be empty".to_string()));
        }
        
        // Validate version format (basic semver check)
        if !self.version.contains('.') {
            return Err(ManifestError::Validation(
                format!("Invalid version format: {}", self.version)
            ));
        }
        
        // Validate permissions
        let valid_permissions = [
            "network", "filesystem", "process", "clipboard",
            "notifications", "audio", "camera", "microphone",
            "display", "input"
        ];
        
        for perm in &self.permissions {
            if !valid_permissions.contains(&perm.as_str()) {
                tracing::warn!("Unknown permission requested: {}", perm);
            }
        }
        
        Ok(())
    }
    
    /// Get the full entry path relative to module directory
    pub fn entry_path(&self, module_dir: &Path) -> std::path::PathBuf {
        module_dir.join(&self.entry)
    }
    
    /// Check if module requires containerization
    pub fn is_containerized(&self) -> bool {
        self.container.backend != ContainerBackend::None
    }
    
    /// Get module identifier (name@version)
    pub fn identifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// Errors that can occur when working with manifests
#[derive(Debug)]
pub enum ManifestError {
    Io(std::io::Error),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "IO error: {}", e),
            ManifestError::Parse(e) => write!(f, "Parse error: {}", e),
            ManifestError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ManifestError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ManifestError {
    fn from(e: std::io::Error) -> Self {
        ManifestError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manifest_from_toml() {
        let toml = r#"
name = "test-module"
version = "1.0.0"
description = "A test module"
author = "Test Author"
license = "MIT"
type = "app-model"
entry = "libtest.so"
language = "rust"

permissions = ["network", "filesystem"]

[container]
backend = "bubblewrap"
network = false

[config]
app_class = "test.app"
"#;
        
        let manifest = ModuleManifest::from_toml_str(toml).unwrap();
        assert_eq!(manifest.name, "test-module");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.module_type, ModuleType::ApplicationModel);
        assert_eq!(manifest.container.backend, ContainerBackend::Bubblewrap);
        assert!(manifest.permissions.contains(&"network".to_string()));
    }
    
    #[test]
    fn test_manifest_from_json() {
        let json = r#"{
            "name": "json-module",
            "version": "2.0.0",
            "description": "JSON test",
            "author": "Tester",
            "license": "Apache-2.0",
            "type": "sector-type",
            "entry": "main.js",
            "language": "javascript",
            "permissions": ["process"]
        }"#;
        
        let manifest = ModuleManifest::from_json_str(json).unwrap();
        assert_eq!(manifest.name, "json-module");
        assert_eq!(manifest.module_type, ModuleType::SectorType);
        assert_eq!(manifest.language, Some("javascript".to_string()));
    }
    
    #[test]
    fn test_manifest_validation() {
        let valid = ModuleManifest {
            name: "valid".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            module_type: ModuleType::ApplicationModel,
            entry: "lib.so".to_string(),
            language: None,
            permissions: vec![],
            container: ContainerConfig::default(),
            config: HashMap::new(),
            dependencies: vec![],
            min_tos_version: None,
        };
        
        assert!(valid.validate().is_ok());
        
        let invalid = ModuleManifest {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            module_type: ModuleType::ApplicationModel,
            entry: "lib.so".to_string(),
            language: None,
            permissions: vec![],
            container: ContainerConfig::default(),
            config: HashMap::new(),
            dependencies: vec![],
            min_tos_version: None,
        };
        
        assert!(invalid.validate().is_err());
    }
    
    #[test]
    fn test_container_backend_default() {
        let default = ContainerBackend::default();
        assert_eq!(default, ContainerBackend::None);
    }
    
    #[test]
    fn test_container_config_default() {
        let config = ContainerConfig::default();
        assert_eq!(config.backend, ContainerBackend::None);
        assert!(!config.network);
        assert!(config.read_only_paths.is_empty());
        assert!(config.read_write_paths.is_empty());
    }
    
    #[test]
    fn test_module_type_default() {
        let default = ModuleType::default();
        assert_eq!(default, ModuleType::ApplicationModel);
    }
}
