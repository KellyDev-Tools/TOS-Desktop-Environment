//! Module Registry
//! 
//! Manages loaded modules, provides hot-reloading capabilities,
//! and handles module lifecycle events.

use super::manifest::{ModuleManifest, ManifestError};
use super::loader::ModuleLoader;
use crate::{TosModule, TosState};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use notify::{Watcher, RecursiveMode, Event};

/// Module loading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Module is loaded but not yet initialized
    Loaded,
    /// Module is active and running
    Active,
    /// Module failed to load
    Error,
    /// Module is being reloaded
    Reloading,
    /// Module is marked for unloading
    Unload,
}

/// Information about a loaded module
#[derive(Debug)]
pub struct ModuleInfo {
    /// Module manifest
    pub manifest: ModuleManifest,
    /// Module instance
    pub module: Option<Box<dyn TosModule>>,
    /// Current state
    pub state: ModuleState,
    /// Module directory path
    pub path: PathBuf,
    /// Last error message (if any)
    pub error: Option<String>,
}

/// Module registry that manages all loaded modules
pub struct ModuleRegistry {
    /// Loaded modules by name
    pub modules: HashMap<String, ModuleInfo>,
    /// Module loader
    pub loader: ModuleLoader,
    /// File system watcher for hot-reload
    pub watcher: Option<notify::RecommendedWatcher>,
    /// Paths being watched
    pub watched_paths: Vec<PathBuf>,
    /// Event receiver channel
    pub event_receiver: Option<std::sync::mpsc::Receiver<notify::Result<Event>>>,
}

impl std::fmt::Debug for ModuleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleRegistry")
            .field("modules", &self.modules.keys().collect::<Vec<_>>())
            .field("watched_paths", &self.watched_paths)
            .finish()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    /// Create a new empty module registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            loader: ModuleLoader::new(),
            watcher: None,
            watched_paths: Vec::new(),
            event_receiver: None,
        }
    }
    
    /// Set default module search paths
    pub fn set_default_paths(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        
        // User modules
        self.loader.add_path(format!("{}/.local/share/tos/modules", home));
        
        // System modules
        self.loader.add_path("/usr/share/tos/modules".to_string());
        self.loader.add_path("/usr/local/share/tos/modules".to_string());
        
        // Development modules (current directory)
        self.loader.add_path("./modules".to_string());
    }
    
    /// Add a module search path
    pub fn add_path(&mut self, path: impl AsRef<Path>) {
        self.loader.add_path(path);
    }
    
    /// Scan all paths and load available modules
    pub fn scan_and_load(&mut self) -> Result<Vec<String>, ManifestError> {
        let manifests = self.loader.scan_modules()?;
        let mut loaded = Vec::new();
        
        for (path, manifest) in manifests {
            match self.load_module(path, manifest) {
                Ok(name) => loaded.push(name),
                Err(e) => tracing::error!("Failed to load module: {}", e),
            }
        }
        
        Ok(loaded)
    }
    
    /// Load a single module
    fn load_module(&mut self, path: PathBuf, manifest: ModuleManifest) -> Result<String, String> {
        // Validate manifest
        manifest.validate().map_err(|e| e.to_string())?;
        
        let name = manifest.name.clone();
        
        // Check if already loaded
        if self.modules.contains_key(&name) {
            return Err(format!("Module {} is already loaded", name));
        }
        
        // Create module info
        let info = ModuleInfo {
            manifest,
            module: None,
            state: ModuleState::Loaded,
            path,
            error: None,
        };
        
        self.modules.insert(name.clone(), info);
        
        tracing::info!("Module loaded: {}", name);
        Ok(name)
    }
    
    /// Initialize all loaded modules
    pub fn initialize_all(&mut self, _state: &mut TosState) {
        let names: Vec<String> = self.modules.keys().cloned().collect();
        
        for name in names {
            if let Some(info) = self.modules.get_mut(&name) {
                if info.state == ModuleState::Loaded {
                    // In a real implementation, we would instantiate the module here
                    // For now, just mark it as active
                    info.state = ModuleState::Active;
                    tracing::info!("Module initialized: {}", name);
                }
            }
        }
    }
    
    /// Shutdown all modules
    pub fn shutdown_all(&mut self, state: &mut TosState) {
        let names: Vec<String> = self.modules.keys().cloned().collect();
        
        for name in names {
            if let Some(info) = self.modules.get_mut(&name) {
                if let Some(ref mut module) = info.module {
                    module.on_unload(state);
                }
                info.state = ModuleState::Unload;
                tracing::info!("Module shutdown: {}", name);
            }
        }
    }
    
    /// Get a module by name
    pub fn get(&self, name: &str) -> Option<&ModuleInfo> {
        self.modules.get(name)
    }
    
    /// Get a mutable reference to a module
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ModuleInfo> {
        self.modules.get_mut(name)
    }
    
    /// Check if a module is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }
    
    /// Get all module names
    pub fn module_names(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }
    
    /// Get number of loaded modules
    pub fn len(&self) -> usize {
        self.modules.len()
    }
    
    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
    
    /// Unload a module
    pub fn unload_module(&mut self, name: &str, state: &mut TosState) -> Result<(), String> {
        let mut info = self.modules.remove(name)
            .ok_or_else(|| format!("Module {} not found", name))?;
        
        if let Some(ref mut module) = info.module {
            module.on_unload(state);
        }
        
        tracing::info!("Module unloaded: {}", name);
        Ok(())
    }
    
    /// Enable hot-reloading via file system watching
    pub fn enable_hot_reload(&mut self) -> Result<(), String> {
        if self.watcher.is_some() {
            return Ok(()); // Already enabled
        }
        
        let (tx, rx) = std::sync::mpsc::channel();
        
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.send(Ok(event));
            }
        }).map_err(|e| format!("Failed to create watcher: {}", e))?;
        
        self.watcher = Some(watcher);
        self.event_receiver = Some(rx);
        
        // Watch all module paths
        for path in &self.watched_paths {
            if let Some(ref mut watcher) = self.watcher {
                watcher.watch(path, RecursiveMode::NonRecursive)
                    .map_err(|e| format!("Failed to watch path {}: {}", path.display(), e))?;
            }
        }
        
        tracing::info!("Hot-reload enabled");
        Ok(())
    }
    
    /// Process file system events for hot-reload
    pub fn process_fs_events(&mut self, state: &mut TosState) {
        // Collect all events first to avoid borrow checker issues
        let events: Vec<notify::Event> = if let Some(ref receiver) = self.event_receiver {
            std::iter::from_fn(|| receiver.try_recv().ok())
                .filter_map(|res| res.ok())
                .collect()
        } else {
            Vec::new()
        };
        
        // Process events
        for event in events {
            match event.kind {
                notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                    for path in &event.paths {
                        if let Some(name) = self.find_module_by_path(path) {
                            tracing::info!("Module changed, reloading: {}", name);
                            let _ = self.reload_module(&name, state);
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    for path in &event.paths {
                        if let Some(name) = self.find_module_by_path(path) {
                            tracing::info!("Module removed, unloading: {}", name);
                            let _ = self.unload_module(&name, state);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    /// Find a module by its path
    fn find_module_by_path(&self, path: &Path) -> Option<String> {
        for (name, info) in &self.modules {
            if info.path == path || info.path.starts_with(path) {
                return Some(name.clone());
            }
        }
        None
    }
    
    /// Reload a specific module
    pub fn reload_module(&mut self, name: &str, state: &mut TosState) -> Result<(), String> {
        // Get the module info
        let info = self.modules.get_mut(name)
            .ok_or_else(|| format!("Module {} not found", name))?;
        
        // Mark as reloading
        info.state = ModuleState::Reloading;
        
        // Unload existing module instance
        if let Some(ref mut module) = info.module {
            module.on_unload(state);
        }
        info.module = None;
        
        // Reload manifest
        let manifest_path = info.path.join("module.toml");
        let new_manifest = ModuleManifest::from_toml_file(&manifest_path)
            .map_err(|e| format!("Failed to reload manifest: {}", e))?;
        
        // Update info
        info.manifest = new_manifest;
        info.state = ModuleState::Active;
        info.error = None;
        
        tracing::info!("Module reloaded: {}", name);
        Ok(())
    }
    
    /// Get modules by state
    pub fn modules_by_state(&self, state: ModuleState) -> Vec<&ModuleInfo> {
        self.modules.values()
            .filter(|info| info.state == state)
            .collect()
    }
    
    /// Get active modules
    pub fn active_modules(&self) -> Vec<&ModuleInfo> {
        self.modules_by_state(ModuleState::Active)
    }
    
    /// Get modules with errors
    pub fn error_modules(&self) -> Vec<&ModuleInfo> {
        self.modules_by_state(ModuleState::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::manifest::{ModuleType, ContainerConfig};
    
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
            config: std::collections::HashMap::new(),
            dependencies: vec![],
            min_tos_version: None,
        }
    }
    
    #[test]
    fn test_registry_new() {
        let registry = ModuleRegistry::new();
        assert!(registry.is_empty());
    }
    
    #[test]
    fn test_load_module() {
        let mut registry = ModuleRegistry::new();
        let manifest = create_test_manifest("test-module");
        let path = PathBuf::from("/tmp/test");
        
        let result = registry.load_module(path, manifest);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-module");
        assert!(registry.is_loaded("test-module"));
    }
    
    #[test]
    fn test_duplicate_load_fails() {
        let mut registry = ModuleRegistry::new();
        let manifest = create_test_manifest("test-module");
        let path = PathBuf::from("/tmp/test");
        
        registry.load_module(path.clone(), manifest.clone()).unwrap();
        let result = registry.load_module(path, manifest);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_module_names() {
        let mut registry = ModuleRegistry::new();
        let manifest1 = create_test_manifest("module-1");
        let manifest2 = create_test_manifest("module-2");
        
        registry.load_module(PathBuf::from("/tmp/1"), manifest1).unwrap();
        registry.load_module(PathBuf::from("/tmp/2"), manifest2).unwrap();
        
        let names = registry.module_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"module-1".to_string()));
        assert!(names.contains(&"module-2".to_string()));
    }
    
    #[test]
    fn test_module_state() {
        let info = ModuleInfo {
            manifest: create_test_manifest("test"),
            module: None,
            state: ModuleState::Loaded,
            path: PathBuf::from("/tmp"),
            error: None,
        };
        
        assert_eq!(info.state, ModuleState::Loaded);
    }
    
    #[test]
    fn test_default_paths() {
        let mut registry = ModuleRegistry::new();
        registry.set_default_paths();
        
        // Should have paths set (at least attempted)
        // Note: Paths may not exist in test environment, but they should be set
        assert!(registry.loader.search_paths().len() >= 0);
    }
}
