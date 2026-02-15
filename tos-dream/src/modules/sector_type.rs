//! Sector Type Module System
//! 
//! Provides concrete implementations of the SectorType trait
//! that can be customized via module manifests.

use super::manifest::ModuleManifest;
use crate::{SectorType, CommandHubMode};
use std::collections::HashMap;

/// A command favorite with metadata
#[derive(Debug, Clone)]
pub struct CommandFavorite {
    pub command: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub icon: String,
}

/// An interesting directory pattern
#[derive(Debug, Clone)]
pub struct InterestingDirectory {
    pub pattern: String,
    pub pattern_type: DirectoryPatternType,
    pub description: String,
    pub suggested_actions: Vec<String>,
}

/// Type of directory pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryPatternType {
    Exact,
    Glob,
    Regex,
}

impl Default for DirectoryPatternType {
    fn default() -> Self {
        DirectoryPatternType::Exact
    }
}

/// A concrete sector type implementation
#[derive(Debug, Clone)]
pub struct SectorTypeImpl {
    /// Type name
    name: String,
    /// Command favorites
    command_favourites: Vec<CommandFavorite>,
    /// Default hub mode
    default_hub_mode: CommandHubMode,
    /// Environment variables to set
    environment: HashMap<String, String>,
    /// Interesting directory patterns
    interesting_directories: Vec<InterestingDirectory>,
    /// Associated application models
    associated_app_models: Vec<String>,
    /// Module manifest reference
    manifest: Option<ModuleManifest>,
}

impl Default for SectorTypeImpl {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            command_favourites: Vec::new(),
            default_hub_mode: CommandHubMode::Command,
            environment: HashMap::new(),
            interesting_directories: Vec::new(),
            associated_app_models: Vec::new(),
            manifest: None,
        }
    }
}

impl SectorTypeImpl {
    /// Create a new sector type with a name
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
    
    /// Create from a module manifest
    pub fn from_manifest(manifest: &ModuleManifest) -> Self {
        let name = manifest.name.clone();
        let mut sector_type = Self::new(name);
        sector_type.manifest = Some(manifest.clone());
        
        // Parse default hub mode
        if let Some(mode) = manifest.config.get("default_hub_mode") {
            if let Some(s) = mode.as_str() {
                sector_type.default_hub_mode = match s {
                    "command" => CommandHubMode::Command,
                    "directory" => CommandHubMode::Directory,
                    "activity" => CommandHubMode::Activity,
                    _ => CommandHubMode::Command,
                };
            }
        }
        
        // Parse environment variables
        if let Some(env) = manifest.config.get("environment") {
            if let Some(obj) = env.as_object() {
                for (key, value) in obj {
                    if let Some(val_str) = value.as_str() {
                        sector_type.environment.insert(key.clone(), val_str.to_string());
                    }
                }
            }
        }
        
        // Parse command favorites
        if let Some(favs) = manifest.config.get("command_favourites") {
            if let Some(arr) = favs.as_array() {
                for fav in arr {
                    if let Some(obj) = fav.as_object() {
                        let command = obj.get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let label = obj.get("label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let description = obj.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let category = obj.get("category")
                            .and_then(|v| v.as_str())
                            .unwrap_or("general")
                            .to_string();
                        let icon = obj.get("icon")
                            .and_then(|v| v.as_str())
                            .unwrap_or("⚡")
                            .to_string();
                        
                        sector_type.add_command_favorite(CommandFavorite {
                            command, label, description, category, icon
                        });
                    }
                }
            }
        }
        
        // Parse interesting directories
        if let Some(dirs) = manifest.config.get("interesting_directories") {
            if let Some(arr) = dirs.as_array() {
                for dir in arr {
                    if let Some(obj) = dir.as_object() {
                        let pattern = obj.get("pattern")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let pattern_type = obj.get("type")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "exact" => DirectoryPatternType::Exact,
                                "glob" => DirectoryPatternType::Glob,
                                "regex" => DirectoryPatternType::Regex,
                                _ => DirectoryPatternType::Exact,
                            })
                            .unwrap_or_default();
                        let description = obj.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let suggested_actions = obj.get("suggested_actions")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect())
                            .unwrap_or_default();
                        
                        sector_type.add_interesting_directory(InterestingDirectory {
                            pattern, pattern_type, description, suggested_actions
                        });
                    }
                }
            }
        }
        
        // Parse associated app models
        if let Some(models) = manifest.config.get("associated_app_models") {
            if let Some(obj) = models.as_object() {
                if let Some(arr) = obj.get("models") {
                    if let Some(models_arr) = arr.as_array() {
                        for model in models_arr {
                            if let Some(s) = model.as_str() {
                                sector_type.associated_app_models.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        sector_type
    }
    
    /// Add a command favorite
    pub fn add_command_favorite(&mut self, favorite: CommandFavorite) {
        self.command_favourites.push(favorite);
    }
    
    /// Add an interesting directory pattern
    pub fn add_interesting_directory(&mut self, dir: InterestingDirectory) {
        self.interesting_directories.push(dir);
    }
    
    /// Set an environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }
    
    /// Get environment variables
    pub fn environment(&self) -> &HashMap<String, String> {
        &self.environment
    }
    
    /// Get interesting directories
    pub fn interesting_directories(&self) -> &[InterestingDirectory] {
        &self.interesting_directories
    }
    
    /// Get associated app models
    pub fn associated_app_models(&self) -> &[String] {
        &self.associated_app_models
    }
    
    /// Create a development sector type
    pub fn development_type() -> Self {
        let mut sector = Self::new("development".to_string());
        sector.default_hub_mode = CommandHubMode::Command;
        
        sector.add_command_favorite(CommandFavorite {
            command: "git status".to_string(),
            label: "Git Status".to_string(),
            description: "Show git repository status".to_string(),
            category: "git".to_string(),
            icon: "📊".to_string(),
        });
        
        sector.add_command_favorite(CommandFavorite {
            command: "cargo build".to_string(),
            label: "Build".to_string(),
            description: "Build the project".to_string(),
            category: "rust".to_string(),
            icon: "🔨".to_string(),
        });
        
        sector.add_command_favorite(CommandFavorite {
            command: "cargo test".to_string(),
            label: "Test".to_string(),
            description: "Run tests".to_string(),
            category: "rust".to_string(),
            icon: "🧪".to_string(),
        });
        
        sector.set_env("EDITOR".to_string(), "nvim".to_string());
        sector.set_env("RUST_BACKTRACE".to_string(), "1".to_string());
        
        sector
    }
    
    /// Create a science/research sector type
    pub fn science_type() -> Self {
        let mut sector = Self::new("science".to_string());
        sector.default_hub_mode = CommandHubMode::Activity;
        
        sector.add_command_favorite(CommandFavorite {
            command: "jupyter notebook".to_string(),
            label: "Jupyter".to_string(),
            description: "Start Jupyter notebook".to_string(),
            category: "python".to_string(),
            icon: "📓".to_string(),
        });
        
        sector.add_command_favorite(CommandFavorite {
            command: "python3 -m http.server".to_string(),
            label: "HTTP Server".to_string(),
            description: "Start HTTP server".to_string(),
            category: "python".to_string(),
            icon: "🌐".to_string(),
        });
        
        sector.set_env("PYTHONPATH".to_string(), ".".to_string());
        
        sector
    }
    
    /// Create an operations/monitoring sector type
    pub fn operations_type() -> Self {
        let mut sector = Self::new("operations".to_string());
        sector.default_hub_mode = CommandHubMode::Activity;
        
        sector.add_command_favorite(CommandFavorite {
            command: "htop".to_string(),
            label: "System Monitor".to_string(),
            description: "Monitor system resources".to_string(),
            category: "system".to_string(),
            icon: "📈".to_string(),
        });
        
        sector.add_command_favorite(CommandFavorite {
            command: "docker ps".to_string(),
            label: "Containers".to_string(),
            description: "List Docker containers".to_string(),
            category: "docker".to_string(),
            icon: "🐳".to_string(),
        });
        
        sector
    }
}

impl SectorType for SectorTypeImpl {
    fn name(&self) -> String {
        self.name.clone()
    }
    
    fn command_favourites(&self) -> Vec<String> {
        self.command_favourites.iter()
            .map(|f| format!("{}:{}", f.label, f.command))
            .collect()
    }
    
    fn default_hub_mode(&self) -> CommandHubMode {
        self.default_hub_mode
    }
}

/// Registry for sector types
#[derive(Debug)]
pub struct SectorTypeRegistry {
    types: HashMap<String, SectorTypeImpl>,
}

impl Default for SectorTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SectorTypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }
    
    /// Register a sector type
    pub fn register(&mut self, sector_type: SectorTypeImpl) {
        self.types.insert(sector_type.name.clone(), sector_type);
    }
    
    /// Get a sector type by name
    pub fn get(&self, name: &str) -> Option<&SectorTypeImpl> {
        self.types.get(name)
    }
    
    /// Get a mutable reference to a sector type
    pub fn get_mut(&mut self, name: &str) -> Option<&mut SectorTypeImpl> {
        self.types.get_mut(name)
    }
    
    /// Check if a sector type exists
    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }
    
    /// Remove a sector type
    pub fn remove(&mut self, name: &str) -> Option<SectorTypeImpl> {
        self.types.remove(name)
    }
    
    /// List all registered sector type names
    pub fn list_names(&self) -> Vec<String> {
        self.types.keys().cloned().collect()
    }
    
    /// Register built-in sector types
    pub fn register_builtin_types(&mut self) {
        self.register(SectorTypeImpl::development_type());
        self.register(SectorTypeImpl::science_type());
        self.register(SectorTypeImpl::operations_type());
    }
    
    /// Get default sector type
    pub fn default_type(&self) -> &SectorTypeImpl {
        self.get("development")
            .or_else(|| self.types.values().next())
            .expect("No sector types registered")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sector_type_new() {
        let sector = SectorTypeImpl::new("test".to_string());
        assert_eq!(sector.name(), "test");
        assert_eq!(sector.default_hub_mode(), CommandHubMode::Command);
    }
    
    #[test]
    fn test_sector_type_default() {
        let sector = SectorTypeImpl::default();
        assert_eq!(sector.name(), "default");
    }
    
    #[test]
    fn test_sector_type_command_favourites() {
        let mut sector = SectorTypeImpl::new("test".to_string());
        sector.add_command_favorite(CommandFavorite {
            command: "test".to_string(),
            label: "Test".to_string(),
            description: "Test command".to_string(),
            category: "test".to_string(),
            icon: "⚡".to_string(),
        });
        
        let favs = sector.command_favourites();
        assert_eq!(favs.len(), 1);
        assert!(favs[0].contains("Test:test"));
    }
    
    #[test]
    fn test_sector_type_environment() {
        let mut sector = SectorTypeImpl::new("test".to_string());
        sector.set_env("TEST_VAR".to_string(), "test_value".to_string());
        
        assert_eq!(sector.environment().get("TEST_VAR"), Some(&"test_value".to_string()));
    }
    
    #[test]
    fn test_development_type() {
        let sector = SectorTypeImpl::development_type();
        assert_eq!(sector.name(), "development");
        assert!(!sector.command_favourites().is_empty());
        assert_eq!(sector.environment().get("EDITOR"), Some(&"nvim".to_string()));
    }
    
    #[test]
    fn test_science_type() {
        let sector = SectorTypeImpl::science_type();
        assert_eq!(sector.name(), "science");
        assert_eq!(sector.default_hub_mode(), CommandHubMode::Activity);
    }
    
    #[test]
    fn test_operations_type() {
        let sector = SectorTypeImpl::operations_type();
        assert_eq!(sector.name(), "operations");
    }
    
    #[test]
    fn test_sector_type_registry() {
        let mut registry = SectorTypeRegistry::new();
        registry.register_builtin_types();
        
        assert!(registry.contains("development"));
        assert!(registry.contains("science"));
        assert!(registry.contains("operations"));
        
        let names = registry.list_names();
        assert_eq!(names.len(), 3);
    }
    
    #[test]
    fn test_directory_pattern_type_default() {
        let default = DirectoryPatternType::default();
        assert_eq!(default, DirectoryPatternType::Exact);
    }
}
