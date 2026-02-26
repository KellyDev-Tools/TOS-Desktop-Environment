//! Application Model Module System
//! 
//! Provides concrete implementations of the ApplicationModel trait
//! that can be customized via module manifests.

use super::manifest::ModuleManifest;
use crate::{ApplicationModel, TosState};
use std::collections::HashMap;

/// A concrete application model implementation
pub struct AppModel {
    /// Application title
    title: String,
    /// Application class identifier
    app_class: String,
    /// Bezel actions available for this app
    bezel_actions: Vec<BezelAction>,
    /// Decoration policy
    decoration_policy: DecorationPolicy,
    /// Custom CSS styles
    custom_css: Option<String>,
    /// Command handlers
    command_handlers: HashMap<String, Box<dyn Fn(&str, &mut TosState) -> Option<String> + Send + Sync>>,
    /// Module manifest reference
    manifest: Option<ModuleManifest>,
}

impl std::fmt::Debug for AppModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppModel")
            .field("title", &self.title)
            .field("app_class", &self.app_class)
            .field("bezel_actions", &self.bezel_actions)
            .field("decoration_policy", &self.decoration_policy)
            .field("custom_css", &self.custom_css)
            .field("command_handlers", &self.command_handlers.len())
            .field("manifest", &self.manifest)
            .finish()
    }
}

impl Clone for AppModel {
    fn clone(&self) -> Self {
        Self {
            title: self.title.clone(),
            app_class: self.app_class.clone(),
            bezel_actions: self.bezel_actions.clone(),
            decoration_policy: self.decoration_policy,
            custom_css: self.custom_css.clone(),
            command_handlers: HashMap::new(), // Cannot clone closures
            manifest: self.manifest.clone(),
        }
    }
}

/// Bezel action definition
#[derive(Debug, Clone)]
pub struct BezelAction {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub command: String,
    pub priority: u8,
}

/// Decoration policy for application rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorationPolicy {
    /// Suppress all decorations
    Suppress,
    /// Overlay custom decorations on top
    Overlay,
    /// Use native decorations
    Native,
}

impl Default for DecorationPolicy {
    fn default() -> Self {
        DecorationPolicy::Native
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            app_class: "unknown".to_string(),
            bezel_actions: Vec::new(),
            decoration_policy: DecorationPolicy::default(),
            custom_css: None,
            command_handlers: HashMap::new(),
            manifest: None,
        }
    }
}

impl AppModel {
    /// Create a new app model with basic properties
    pub fn new(title: String, app_class: String) -> Self {
        Self {
            title,
            app_class,
            ..Default::default()
        }
    }
    
    /// Create from a module manifest
    pub fn from_manifest(manifest: &ModuleManifest) -> Self {
        let title = manifest.name.clone();
        let app_class = manifest.config.get("app_class")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("module.{}", manifest.name));
        
        let mut model = Self::new(title, app_class);
        model.manifest = Some(manifest.clone());
        
        // Parse bezel actions from config
        if let Some(actions) = manifest.config.get("bezel_actions") {
            if let Some(arr) = actions.as_array() {
                for action in arr {
                    if let Some(obj) = action.as_object() {
                        let id = obj.get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("action")
                            .to_string();
                        let label = obj.get("label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Action")
                            .to_string();
                        let icon = obj.get("icon")
                            .and_then(|v| v.as_str())
                            .unwrap_or("⚡")
                            .to_string();
                        let command = obj.get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let priority = obj.get("priority")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(50) as u8;
                        
                        model.add_bezel_action(BezelAction {
                            id, label, icon, command, priority
                        });
                    }
                }
            }
        }
        
        // Parse decoration policy
        if let Some(policy) = manifest.config.get("decoration_policy") {
            if let Some(s) = policy.as_str() {
                model.decoration_policy = match s {
                    "suppress" => DecorationPolicy::Suppress,
                    "overlay" => DecorationPolicy::Overlay,
                    "native" | _ => DecorationPolicy::Native,
                };
            }
        }
        
        model
    }
    
    /// Add a bezel action
    pub fn add_bezel_action(&mut self, action: BezelAction) {
        self.bezel_actions.push(action);
        // Sort by priority
        self.bezel_actions.sort_by_key(|a| a.priority);
    }
    
    /// Set decoration policy
    pub fn set_decoration_policy(&mut self, policy: DecorationPolicy) {
        self.decoration_policy = policy;
    }
    
    /// Add a command handler
    pub fn add_command_handler<F>(&mut self, command: &str, handler: F)
    where
        F: Fn(&str, &mut TosState) -> Option<String> + Send + Sync + 'static,
    {
        self.command_handlers.insert(command.to_string(), Box::new(handler));
    }
    
    /// Get decoration policy
    pub fn decoration_policy(&self) -> DecorationPolicy {
        self.decoration_policy
    }
    
    /// Get custom CSS
    pub fn custom_css(&self) -> Option<&str> {
        self.custom_css.as_deref()
    }
    
    /// Set custom CSS
    pub fn set_custom_css(&mut self, css: String) {
        self.custom_css = Some(css);
    }
    
    /// Create a terminal application model
    pub fn terminal_model() -> Self {
        let mut model = Self::new(
            "Terminal".to_string(),
            "tos.terminal".to_string()
        );
        
        model.add_bezel_action(BezelAction {
            id: "new-tab".to_string(),
            label: "New Tab".to_string(),
            icon: "+".to_string(),
            command: "new_tab".to_string(),
            priority: 10,
        });
        
        model.add_bezel_action(BezelAction {
            id: "split".to_string(),
            label: "Split".to_string(),
            icon: "◫".to_string(),
            command: "split_pane".to_string(),
            priority: 20,
        });
        
        model.add_bezel_action(BezelAction {
            id: "clear".to_string(),
            label: "Clear".to_string(),
            icon: "⌧".to_string(),
            command: "clear".to_string(),
            priority: 30,
        });
        
        model
    }
    
    /// Create a browser application model
    pub fn browser_model() -> Self {
        let mut model = Self::new(
            "Browser".to_string(),
            "tos.browser".to_string()
        );
        
        model.add_bezel_action(BezelAction {
            id: "back".to_string(),
            label: "Back".to_string(),
            icon: "←".to_string(),
            command: "navigate_back".to_string(),
            priority: 10,
        });
        
        model.add_bezel_action(BezelAction {
            id: "forward".to_string(),
            label: "Forward".to_string(),
            icon: "→".to_string(),
            command: "navigate_forward".to_string(),
            priority: 20,
        });
        
        model.add_bezel_action(BezelAction {
            id: "refresh".to_string(),
            label: "Refresh".to_string(),
            icon: "↻".to_string(),
            command: "refresh".to_string(),
            priority: 30,
        });
        
        model
    }
    
    /// Create an editor application model
    pub fn editor_model() -> Self {
        let mut model = Self::new(
            "Editor".to_string(),
            "tos.editor".to_string()
        );
        
        model.add_bezel_action(BezelAction {
            id: "save".to_string(),
            label: "Save".to_string(),
            icon: "💾".to_string(),
            command: "save".to_string(),
            priority: 10,
        });
        
        model.add_bezel_action(BezelAction {
            id: "find".to_string(),
            label: "Find".to_string(),
            icon: "🔍".to_string(),
            command: "find".to_string(),
            priority: 20,
        });
        
        model
    }
}

impl ApplicationModel for AppModel {
    fn title(&self) -> String {
        self.title.clone()
    }
    
    fn app_class(&self) -> String {
        self.app_class.clone()
    }
    
    fn bezel_actions(&self) -> Vec<String> {
        self.bezel_actions.iter()
            .map(|a| format!("{}:{}:{}", a.id, a.label, a.icon))
            .collect()
    }
    
    fn handle_command(&self, cmd: &str) -> Option<String> {
        // Check if we have a custom handler
        if let Some(_handler) = self.command_handlers.get(cmd) {
            // Note: This would need a mutable state reference in practice
            // For now, return a placeholder
            Some(format!("Handled by custom handler: {}", cmd))
        } else {
            // Default command handling
            match cmd {
                "new_tab" => Some("Opening new tab...".to_string()),
                "split_pane" => Some("Splitting pane...".to_string()),
                "clear" => Some("Clearing screen...".to_string()),
                "navigate_back" => Some("Going back...".to_string()),
                "navigate_forward" => Some("Going forward...".to_string()),
                "refresh" => Some("Refreshing...".to_string()),
                "save" => Some("Saving...".to_string()),
                "find" => Some("Opening find dialog...".to_string()),
                _ => None,
            }
        }
    }
}

/// Registry for application models
#[derive(Debug)]
pub struct AppModelRegistry {
    models: HashMap<String, AppModel>,
}

impl Default for AppModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AppModelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }
    
    /// Register a built-in model
    pub fn register(&mut self, model: AppModel) {
        self.models.insert(model.app_class.clone(), model);
    }
    
    /// Get a model by app class
    pub fn get(&self, app_class: &str) -> Option<&AppModel> {
        self.models.get(app_class)
    }
    
    /// Get a mutable reference to a model
    pub fn get_mut(&mut self, app_class: &str) -> Option<&mut AppModel> {
        self.models.get_mut(app_class)
    }
    
    /// Check if a model exists
    pub fn contains(&self, app_class: &str) -> bool {
        self.models.contains_key(app_class)
    }
    
    /// Remove a model
    pub fn remove(&mut self, app_class: &str) -> Option<AppModel> {
        self.models.remove(app_class)
    }
    
    /// List all registered app classes
    pub fn list_classes(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }
    
    /// Register built-in application models
    pub fn register_builtin_models(&mut self) {
        self.register(AppModel::terminal_model());
        self.register(AppModel::browser_model());
        self.register(AppModel::editor_model());
    }
    
    /// Get default model for unknown app classes
    pub fn default_model(&self) -> &AppModel {
        // Return terminal as default, or first available
        self.get("tos.terminal")
            .or_else(|| self.models.values().next())
            .expect("No models registered")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_model_new() {
        let model = AppModel::new("Test".to_string(), "test.app".to_string());
        assert_eq!(model.title(), "Test");
        assert_eq!(model.app_class(), "test.app");
    }
    
    #[test]
    fn test_app_model_bezel_actions() {
        let mut model = AppModel::new("Test".to_string(), "test.app".to_string());
        model.add_bezel_action(BezelAction {
            id: "test".to_string(),
            label: "Test Action".to_string(),
            icon: "⚡".to_string(),
            command: "test_cmd".to_string(),
            priority: 50,
        });
        
        let actions = model.bezel_actions();
        assert_eq!(actions.len(), 1);
        assert!(actions[0].contains("test:Test Action:⚡"));
    }
    
    #[test]
    fn test_app_model_from_manifest() {
        let manifest = ModuleManifest {
            name: "test-module".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            module_type: super::super::manifest::ModuleType::ApplicationModel,
            entry: "test.so".to_string(),
            language: None,
            permissions: vec![],
            container: Default::default(),
            config: {
                let mut map = HashMap::new();
                map.insert("app_class".to_string(), serde_json::json!("custom.app"));
                map
            },
            dependencies: vec![],
            min_tos_version: None,
        };
        
        let model = AppModel::from_manifest(&manifest);
        assert_eq!(model.app_class(), "custom.app");
    }
    
    #[test]
    fn test_app_model_handle_command() {
        let model = AppModel::terminal_model();
        
        assert!(model.handle_command("new_tab").is_some());
        assert!(model.handle_command("unknown").is_none());
    }
    
    #[test]
    fn test_decoration_policy() {
        let mut model = AppModel::new("Test".to_string(), "test.app".to_string());
        assert_eq!(model.decoration_policy(), DecorationPolicy::Native);
        
        model.set_decoration_policy(DecorationPolicy::Overlay);
        assert_eq!(model.decoration_policy(), DecorationPolicy::Overlay);
    }
    
    #[test]
    fn test_app_model_registry() {
        let mut registry = AppModelRegistry::new();
        registry.register_builtin_models();
        
        assert!(registry.contains("tos.terminal"));
        assert!(registry.contains("tos.browser"));
        assert!(registry.contains("tos.editor"));
        
        let classes = registry.list_classes();
        assert_eq!(classes.len(), 3);
    }
    
    #[test]
    fn test_terminal_model() {
        let model = AppModel::terminal_model();
        assert_eq!(model.title(), "Terminal");
        assert_eq!(model.app_class(), "tos.terminal");
        
        let actions = model.bezel_actions();
        assert!(!actions.is_empty());
    }
    
    #[test]
    fn test_browser_model() {
        let model = AppModel::browser_model();
        assert_eq!(model.title(), "Browser");
        assert_eq!(model.app_class(), "tos.browser");
    }
    
    #[test]
    fn test_editor_model() {
        let model = AppModel::editor_model();
        assert_eq!(model.title(), "Editor");
        assert_eq!(model.app_class(), "tos.editor");
    }
}
