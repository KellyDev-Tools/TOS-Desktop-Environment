//! Module Loader
//! 
//! Handles loading modules from the filesystem, including:
//! - Scanning module directories
//! - Parsing manifest files
//! - Loading Rust dylibs
//! - Setting up containerization

use super::manifest::{ModuleManifest, ManifestError, ContainerBackend};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Module loader that handles filesystem operations
#[derive(Debug)]
pub struct ModuleLoader {
    /// Search paths for modules
    search_paths: Vec<PathBuf>,
    /// Loaded module manifests
    manifests: HashMap<String, (PathBuf, ModuleManifest)>,
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            manifests: HashMap::new(),
        }
    }
    
    /// Add a search path
    pub fn add_path(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref().to_path_buf();
        if path.exists() && !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }
    
    /// Get all search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
    
    /// Scan all search paths for available modules
    pub fn scan_modules(&mut self) -> Result<Vec<(PathBuf, ModuleManifest)>, ManifestError> {
        let mut found = Vec::new();
        
        for path in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let module_dir = entry.path();
                    if module_dir.is_dir() {
                        // Try to load manifest
                        if let Some(manifest) = self.try_load_manifest(&module_dir) {
                            let name = manifest.name.clone();
                            self.manifests.insert(name.clone(), (module_dir.clone(), manifest.clone()));
                            found.push((module_dir, manifest));
                        }
                    }
                }
            }
        }
        
        Ok(found)
    }
    
    /// Try to load a manifest from a module directory
    fn try_load_manifest(&self, module_dir: &Path) -> Option<ModuleManifest> {
        // Try TOML first
        let toml_path = module_dir.join("module.toml");
        if toml_path.exists() {
            if let Ok(manifest) = ModuleManifest::from_toml_file(&toml_path) {
                return Some(manifest);
            }
        }
        
        // Try JSON
        let json_path = module_dir.join("module.json");
        if json_path.exists() {
            if let Ok(manifest) = ModuleManifest::from_json_file(&json_path) {
                return Some(manifest);
            }
        }
        
        None
    }
    
    /// Get a loaded manifest by name
    pub fn get_manifest(&self, name: &str) -> Option<&(PathBuf, ModuleManifest)> {
        self.manifests.get(name)
    }
    
    /// Check if a module is available
    pub fn is_available(&self, name: &str) -> bool {
        self.manifests.contains_key(name)
    }
    
    /// Get all available module names
    pub fn available_modules(&self) -> Vec<String> {
        self.manifests.keys().cloned().collect()
    }
    
    /// Build container command for a module
    pub fn build_container_command(&self, manifest: &ModuleManifest, entry_path: &Path) -> Option<Vec<String>> {
        if !manifest.is_containerized() {
            return None;
        }
        
        match manifest.container.backend {
            ContainerBackend::Bubblewrap => {
                Some(self.build_bubblewrap_command(manifest, entry_path))
            }
            ContainerBackend::Firejail => {
                Some(self.build_firejail_command(manifest, entry_path))
            }
            ContainerBackend::Docker => {
                Some(self.build_docker_command(manifest, entry_path))
            }
            ContainerBackend::Podman => {
                Some(self.build_podman_command(manifest, entry_path))
            }
            ContainerBackend::None => None,
        }
    }
    
    /// Build bubblewrap command
    fn build_bubblewrap_command(&self, manifest: &ModuleManifest, entry_path: &Path) -> Vec<String> {
        let mut cmd = vec!["bwrap".to_string()];
        
        // Base system
        cmd.push("--ro-bind".to_string());
        cmd.push("/usr".to_string());
        cmd.push("/usr".to_string());
        
        cmd.push("--ro-bind".to_string());
        cmd.push("/lib".to_string());
        cmd.push("/lib".to_string());
        
        cmd.push("--ro-bind".to_string());
        cmd.push("/lib64".to_string());
        cmd.push("/lib64".to_string());
        
        // Proc and tmp
        cmd.push("--proc".to_string());
        cmd.push("/proc".to_string());
        
        cmd.push("--tmpfs".to_string());
        cmd.push("/tmp".to_string());
        
        // Read-only paths
        for path in &manifest.container.read_only_paths {
            cmd.push("--ro-bind".to_string());
            cmd.push(path.clone());
            cmd.push(path.clone());
        }
        
        // Read-write paths
        for path in &manifest.container.read_write_paths {
            cmd.push("--bind".to_string());
            cmd.push(path.clone());
            cmd.push(path.clone());
        }
        
        // Network
        if !manifest.container.network {
            cmd.push("--unshare-net".to_string());
        }
        
        // Module directory
        let module_dir = entry_path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        cmd.push("--bind".to_string());
        cmd.push(module_dir.clone());
        cmd.push("/app".to_string());
        
        // Working directory
        cmd.push("--chdir".to_string());
        cmd.push("/app".to_string());
        
        // Execute
        cmd.push(entry_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "module".to_string()));
        
        cmd
    }
    
    /// Build firejail command
    fn build_firejail_command(&self, manifest: &ModuleManifest, entry_path: &Path) -> Vec<String> {
        let mut cmd = vec!["firejail".to_string()];
        
        // Network
        if !manifest.container.network {
            cmd.push("--net=none".to_string());
        }
        
        // Read-only paths
        for path in &manifest.container.read_only_paths {
            cmd.push(format!("--read-only={}", path));
        }
        
        // Read-write paths
        for path in &manifest.container.read_write_paths {
            cmd.push(format!("--read-write={}", path));
        }
        
        // Quiet mode
        cmd.push("--quiet".to_string());
        
        // Execute
        cmd.push(entry_path.to_string_lossy().to_string());
        
        cmd
    }
    
    /// Build Docker command
    fn build_docker_command(&self, manifest: &ModuleManifest, entry_path: &Path) -> Vec<String> {
        let mut cmd = vec!["docker".to_string(), "run".to_string()];
        
        // Remove container after exit
        cmd.push("--rm".to_string());
        
        // Network
        if !manifest.container.network {
            cmd.push("--network=none".to_string());
        }
        
        // Read-only paths as volumes
        for path in &manifest.container.read_only_paths {
            cmd.push("-v".to_string());
            cmd.push(format!("{}:{}:ro", path, path));
        }
        
        // Read-write paths as volumes
        for path in &manifest.container.read_write_paths {
            cmd.push("-v".to_string());
            cmd.push(format!("{}:{}", path, path));
        }
        
        // Module directory
        let module_dir = entry_path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        cmd.push("-v".to_string());
        cmd.push(format!("{}:/app", module_dir));
        
        cmd.push("-w".to_string());
        cmd.push("/app".to_string());
        
        // Image name (use module name as image)
        cmd.push(format!("tos-{}", manifest.name));
        
        // Entry point
        cmd.push(entry_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "module".to_string()));
        
        cmd
    }
    
    /// Build Podman command
    fn build_podman_command(&self, manifest: &ModuleManifest, entry_path: &Path) -> Vec<String> {
        let mut cmd = vec!["podman".to_string(), "run".to_string()];
        
        // Remove container after exit
        cmd.push("--rm".to_string());
        
        // Network
        if !manifest.container.network {
            cmd.push("--network=none".to_string());
        }
        
        // Read-only paths as volumes
        for path in &manifest.container.read_only_paths {
            cmd.push("-v".to_string());
            cmd.push(format!("{}:{}:ro", path, path));
        }
        
        // Read-write paths as volumes
        for path in &manifest.container.read_write_paths {
            cmd.push("-v".to_string());
            cmd.push(format!("{}:{}", path, path));
        }
        
        // Module directory
        let module_dir = entry_path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        cmd.push("-v".to_string());
        cmd.push(format!("{}:/app", module_dir));
        
        cmd.push("-w".to_string());
        cmd.push("/app".to_string());
        
        // Image name
        cmd.push(format!("tos-{}", manifest.name));
        
        // Entry point
        cmd.push(entry_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "module".to_string()));
        
        cmd
    }
    
    /// Clear all loaded manifests
    pub fn clear(&mut self) {
        self.manifests.clear();
    }
    
    /// Get number of loaded manifests
    pub fn len(&self) -> usize {
        self.manifests.len()
    }
    
    /// Check if loader is empty
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::manifest::{ModuleType, ContainerConfig};
    use std::io::Write;
    
    fn create_test_manifest(name: &str) -> ModuleManifest {
        ModuleManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            module_type: ModuleType::ApplicationModel,
            entry: "test.so".to_string(),
            language: None,
            permissions: vec![],
            container: ContainerConfig::default(),
            config: HashMap::new(),
            dependencies: vec![],
            min_tos_version: None,
        }
    }
    
    #[test]
    fn test_loader_new() {
        let loader = ModuleLoader::new();
        assert!(loader.is_empty());
        assert!(loader.search_paths().is_empty());
    }
    
    #[test]
    fn test_add_path() {
        let mut loader = ModuleLoader::new();
        loader.add_path("/tmp");
        
        // Path should be added if it exists
        if Path::new("/tmp").exists() {
            assert!(!loader.search_paths().is_empty());
        }
    }
    
    #[test]
    fn test_build_bubblewrap_command() {
        let loader = ModuleLoader::new();
        let mut manifest = create_test_manifest("test");
        manifest.container.backend = ContainerBackend::Bubblewrap;
        manifest.container.network = false;
        
        let entry = PathBuf::from("/tmp/test/module.so");
        let cmd = loader.build_container_command(&manifest, &entry);
        
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd[0], "bwrap");
        assert!(cmd.contains(&"--unshare-net".to_string()));
    }
    
    #[test]
    fn test_build_firejail_command() {
        let loader = ModuleLoader::new();
        let mut manifest = create_test_manifest("test");
        manifest.container.backend = ContainerBackend::Firejail;
        manifest.container.network = true;
        
        let entry = PathBuf::from("/tmp/test/module.so");
        let cmd = loader.build_container_command(&manifest, &entry);
        
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd[0], "firejail");
        assert!(!cmd.contains(&"--net=none".to_string()));
    }
    
    #[test]
    fn test_build_docker_command() {
        let loader = ModuleLoader::new();
        let mut manifest = create_test_manifest("test");
        manifest.container.backend = ContainerBackend::Docker;
        
        let entry = PathBuf::from("/tmp/test/module.so");
        let cmd = loader.build_container_command(&manifest, &entry);
        
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd[0], "docker");
        assert!(cmd.contains(&"--rm".to_string()));
    }
    
    #[test]
    fn test_no_container() {
        let loader = ModuleLoader::new();
        let manifest = create_test_manifest("test");
        
        let entry = PathBuf::from("/tmp/test/module.so");
        let cmd = loader.build_container_command(&manifest, &entry);
        
        assert!(cmd.is_none());
    }
    
    #[test]
    fn test_scan_modules() {
        let temp_dir = tempfile::tempdir().unwrap();
        let module_dir = temp_dir.path().join("test-module");
        std::fs::create_dir(&module_dir).unwrap();
        
        // Create a manifest file
        let manifest_content = r#"
name = "test-module"
version = "1.0.0"
description = "Test module"
author = "Test"
license = "MIT"
type = "app-model"
entry = "test.so"
"#;
        let manifest_path = module_dir.join("module.toml");
        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest_content.as_bytes()).unwrap();
        
        let mut loader = ModuleLoader::new();
        loader.add_path(temp_dir.path());
        
        let modules = loader.scan_modules().unwrap();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].1.name, "test-module");
    }
}
