//! Script Engine Support
//! 
//! Provides embedded scripting capabilities for modules using
//! JavaScript (via quickjs) and Lua (via mlua).
//! 
//! This allows modules to be written in scripting languages
//! rather than compiled Rust.

use super::manifest::ModuleManifest;
use crate::{TosModule, TosState, HierarchyLevel, ApplicationModel, SectorType, CommandHubMode};
use std::collections::HashMap;

/// Supported scripting languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    JavaScript,
    Lua,
    Python,
}

/// A script-based module
#[derive(Debug)]
pub struct ScriptEngine {
    /// Script language
    language: ScriptLanguage,
    /// Script source code
    source: String,
    /// Module manifest
    manifest: ModuleManifest,
    /// Script state (variables, functions)
    state: HashMap<String, serde_json::Value>,
}

impl ScriptEngine {
    /// Create a new script engine
    pub fn new(language: ScriptLanguage, source: String, manifest: ModuleManifest) -> Self {
        Self {
            language,
            source,
            manifest,
            state: HashMap::new(),
        }
    }
    
    /// Create a JavaScript engine
    pub fn javascript(source: String, manifest: ModuleManifest) -> Self {
        Self::new(ScriptLanguage::JavaScript, source, manifest)
    }
    
    /// Create a Lua engine
    pub fn lua(source: String, manifest: ModuleManifest) -> Self {
        Self::new(ScriptLanguage::Lua, source, manifest)
    }
    
    /// Initialize the script environment
    pub fn initialize(&mut self) -> Result<(), ScriptError> {
        match self.language {
            ScriptLanguage::JavaScript => self.init_javascript(),
            ScriptLanguage::Lua => self.init_lua(),
            ScriptLanguage::Python => self.init_python(),
        }
    }
    
    /// Initialize JavaScript environment
    fn init_javascript(&mut self) -> Result<(), ScriptError> {
        // In a full implementation, this would use quickjs or deno_core
        // to create a JS runtime and execute the script
        
        tracing::info!("Initializing JavaScript module: {}", self.manifest.name);
        
        // Parse the script to extract exports
        self.extract_js_exports()?;
        
        Ok(())
    }
    
    /// Initialize Lua environment
    fn init_lua(&mut self) -> Result<(), ScriptError> {
        // In a full implementation, this would use mlua
        // to create a Lua runtime and execute the script
        
        tracing::info!("Initializing Lua module: {}", self.manifest.name);
        
        // Parse the script to extract exports
        self.extract_lua_exports()?;
        
        Ok(())
    }
    
    /// Initialize Python environment
    fn init_python(&mut self) -> Result<(), ScriptError> {
        tracing::info!("Initializing Python module: {}", self.manifest.name);
        
        // Python support would require embedding a Python interpreter
        // or using a subprocess-based approach
        
        Err(ScriptError::UnsupportedLanguage("Python".to_string()))
    }
    
    /// Extract JavaScript exports
    fn extract_js_exports(&mut self) -> Result<(), ScriptError> {
        // Simple regex-based extraction for demonstration
        // In production, use a proper JS parser
        
        // Look for export statements
        for line in self.source.lines() {
            if line.contains("export") || line.contains("module.exports") {
                tracing::debug!("Found JS export: {}", line.trim());
            }
        }
        
        Ok(())
    }
    
    /// Extract Lua exports
    fn extract_lua_exports(&mut self) -> Result<(), ScriptError> {
        // Look for return statements or module definitions
        for line in self.source.lines() {
            if line.contains("return") || line.contains("M.") {
                tracing::debug!("Found Lua export: {}", line.trim());
            }
        }
        
        Ok(())
    }
    
    /// Execute a script function
    pub fn execute(&mut self, function: &str, args: &[serde_json::Value]) -> Result<serde_json::Value, ScriptError> {
        match self.language {
            ScriptLanguage::JavaScript => self.execute_js(function, args),
            ScriptLanguage::Lua => self.execute_lua(function, args),
            ScriptLanguage::Python => Err(ScriptError::UnsupportedLanguage("Python".to_string())),
        }
    }
    
    /// Execute JavaScript function
    fn execute_js(&mut self, function: &str, _args: &[serde_json::Value]) -> Result<serde_json::Value, ScriptError> {
        // Placeholder implementation
        tracing::debug!("Executing JS function: {}", function);
        
        // Return a mock result
        Ok(serde_json::json!({
            "result": "success",
            "function": function
        }))
    }
    
    /// Execute Lua function
    fn execute_lua(&mut self, function: &str, _args: &[serde_json::Value]) -> Result<serde_json::Value, ScriptError> {
        // Placeholder implementation
        tracing::debug!("Executing Lua function: {}", function);
        
        // Return a mock result
        Ok(serde_json::json!({
            "result": "success",
            "function": function
        }))
    }
    
    /// Get the language
    pub fn language(&self) -> ScriptLanguage {
        self.language
    }
    
    /// Get the manifest
    pub fn manifest(&self) -> &ModuleManifest {
        &self.manifest
    }
    
    /// Create a TosModule wrapper for this script
    pub fn as_tos_module(&self) -> ScriptModuleWrapper<'_> {
        ScriptModuleWrapper {
            engine: self,
        }
    }

    pub fn to_owned_wrapper(self) -> OwnedScriptModule {
        OwnedScriptModule {
            engine: self,
        }
    }
}

/// Owned version of script module wrapper
#[derive(Debug)]
pub struct OwnedScriptModule {
    pub engine: ScriptEngine,
}

impl TosModule for OwnedScriptModule {
    fn name(&self) -> String { self.engine.as_tos_module().name() }
    fn version(&self) -> String { self.engine.as_tos_module().version() }
    fn on_load(&mut self, state: &mut TosState) { self.engine.as_tos_module_mut().on_load(state) }
    fn on_unload(&mut self, state: &mut TosState) { self.engine.as_tos_module_mut().on_unload(state) }
    fn render_override(&self, level: HierarchyLevel) -> Option<String> { self.engine.as_tos_module().render_override(level) }
}

impl ScriptEngine {
    pub fn as_tos_module_mut(&mut self) -> ScriptModuleWrapperMut<'_> {
        ScriptModuleWrapperMut {
            engine: self,
        }
    }
}

pub struct ScriptModuleWrapperMut<'a> {
    engine: &'a mut ScriptEngine,
}

impl<'a> ScriptModuleWrapperMut<'a> {
    pub fn on_load(&mut self, _state: &mut TosState) {
        tracing::info!("Script module loaded: {}", self.engine.manifest.name);
    }
    pub fn on_unload(&mut self, _state: &mut TosState) {
        tracing::info!("Script module unloaded: {}", self.engine.manifest.name);
    }
}

/// Wrapper to expose ScriptEngine as TosModule
#[derive(Debug)]
pub struct ScriptModuleWrapper<'a> {
    engine: &'a ScriptEngine,
}

impl<'a> TosModule for ScriptModuleWrapper<'a> {
    fn name(&self) -> String {
        format!("{} [{}]", self.engine.manifest.name, 
            match self.engine.language {
                ScriptLanguage::JavaScript => "JS",
                ScriptLanguage::Lua => "Lua",
                ScriptLanguage::Python => "Py",
            }
        )
    }
    
    fn version(&self) -> String {
        self.engine.manifest.version.clone()
    }
    
    fn on_load(&mut self, _state: &mut TosState) {
        tracing::info!("Script module loaded: {}", self.engine.manifest.name);
    }
    
    fn on_unload(&mut self, _state: &mut TosState) {
        tracing::info!("Script module unloaded: {}", self.engine.manifest.name);
    }
    
    fn render_override(&self, _level: HierarchyLevel) -> Option<String> {
        // Try to call a render function in the script
        // This is a placeholder - real implementation would
        // actually execute the script function
        
        if self.engine.source.contains("render") {
            Some(format!(
                r#"<div class="script-overlay {}-overlay">
                    <div class="script-badge">{}</div>
                </div>"#,
                self.engine.manifest.name,
                match self.engine.language {
                    ScriptLanguage::JavaScript => "JS",
                    ScriptLanguage::Lua => "LUA",
                    ScriptLanguage::Python => "PY",
                }
            ))
        } else {
            None
        }
    }
}

/// Script-based Application Model
#[derive(Debug)]
pub struct ScriptAppModel {
    engine: ScriptEngine,
}

impl ScriptAppModel {
    /// Create a new script-based app model
    pub fn new(engine: ScriptEngine) -> Self {
        Self { engine }
    }
    
    /// Get the underlying engine
    pub fn engine(&self) -> &ScriptEngine {
        &self.engine
    }
}

impl ApplicationModel for ScriptAppModel {
    fn title(&self) -> String {
        self.engine.manifest.name.clone()
    }
    
    fn app_class(&self) -> String {
        self.engine.manifest.config.get("app_class")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("script.{}", self.engine.manifest.name))
    }
    
    fn bezel_actions(&self) -> Vec<String> {
        // Extract bezel actions from script or config
        self.engine.manifest.config.get("bezel_actions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
            .unwrap_or_default()
    }
    
    fn handle_command(&self, cmd: &str) -> Option<String> {
        // Try to execute a command handler in the script
        if self.engine.source.contains(&format!("function {}(", cmd)) ||
           self.engine.source.contains(&format!("{} = function", cmd)) {
            Some(format!("Script handler for: {}", cmd))
        } else {
            None
        }
    }
}

/// Script-based Sector Type
#[derive(Debug)]
pub struct ScriptSectorType {
    engine: ScriptEngine,
}

impl ScriptSectorType {
    /// Create a new script-based sector type
    pub fn new(engine: ScriptEngine) -> Self {
        Self { engine }
    }
    
    /// Get the underlying engine
    pub fn engine(&self) -> &ScriptEngine {
        &self.engine
    }
}

impl SectorType for ScriptSectorType {
    fn name(&self) -> String {
        self.engine.manifest.name.clone()
    }
    
    fn command_favourites(&self) -> Vec<String> {
        self.engine.manifest.config.get("command_favourites")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect())
            .unwrap_or_default()
    }
    
    fn default_hub_mode(&self) -> CommandHubMode {
        self.engine.manifest.config.get("default_hub_mode")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "command" => CommandHubMode::Command,
                "directory" => CommandHubMode::Directory,
                "activity" => CommandHubMode::Activity,
                _ => CommandHubMode::Command,
            })
            .unwrap_or(CommandHubMode::Command)
    }
}

/// Factory for creating script engines
pub struct ScriptEngineFactory;

impl ScriptEngineFactory {
    /// Create a script engine from a manifest and source file
    pub fn from_file(manifest: &ModuleManifest, source_path: &std::path::Path) -> Result<ScriptEngine, ScriptError> {
        let source = std::fs::read_to_string(source_path)
            .map_err(|e| ScriptError::Io(e))?;
        
        let language = match manifest.language.as_deref() {
            Some("javascript") | Some("js") => ScriptLanguage::JavaScript,
            Some("lua") => ScriptLanguage::Lua,
            Some("python") | Some("py") => ScriptLanguage::Python,
            _ => {
                // Try to detect from file extension
                if let Some(ext) = source_path.extension() {
                    match ext.to_str() {
                        Some("js") => ScriptLanguage::JavaScript,
                        Some("lua") => ScriptLanguage::Lua,
                        Some("py") => ScriptLanguage::Python,
                        _ => return Err(ScriptError::UnsupportedLanguage("unknown".to_string())),
                    }
                } else {
                    return Err(ScriptError::UnsupportedLanguage("unknown".to_string()));
                }
            }
        };
        
        let mut engine = ScriptEngine::new(language, source, manifest.clone());
        engine.initialize()?;
        
        Ok(engine)
    }
    
    /// Create a script engine from source string
    pub fn from_source(language: ScriptLanguage, source: String, manifest: ModuleManifest) -> Result<ScriptEngine, ScriptError> {
        let mut engine = ScriptEngine::new(language, source, manifest);
        engine.initialize()?;
        Ok(engine)
    }
}

/// Errors that can occur in script execution
#[derive(Debug)]
pub enum ScriptError {
    Io(std::io::Error),
    Parse(String),
    Execution(String),
    UnsupportedLanguage(String),
    Runtime(String),
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptError::Io(e) => write!(f, "IO error: {}", e),
            ScriptError::Parse(e) => write!(f, "Parse error: {}", e),
            ScriptError::Execution(e) => write!(f, "Execution error: {}", e),
            ScriptError::UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
            ScriptError::Runtime(e) => write!(f, "Runtime error: {}", e),
        }
    }
}

impl std::error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScriptError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ScriptError {
    fn from(e: std::io::Error) -> Self {
        ScriptError::Io(e)
    }
}

/// Example JavaScript module template
pub const JS_MODULE_TEMPLATE: &str = r#"
// TOS JavaScript Module Template
// Module: {{name}}
// Version: {{version}}

const TOS = {
    name: "{{name}}",
    version: "{{version}}",
    
    // Called when module is loaded
    onLoad: function(state) {
        console.log(`Module ${this.name} loaded`);
    },
    
    // Called when module is unloaded
    onUnload: function(state) {
        console.log(`Module ${this.name} unloaded`);
    },
    
    // Render override for specific hierarchy levels
    render: function(level) {
        if (level === "ApplicationFocus") {
            return `<div class="custom-overlay">${this.name}</div>`;
        }
        return null;
    },
    
    // Handle bezel actions
    bezelActions: function() {
        return ["custom-action-1", "custom-action-2"];
    },
    
    // Handle commands
    handleCommand: function(cmd) {
        if (cmd === "custom-action-1") {
            return "Executed custom action 1";
        }
        return null;
    }
};

// Export for TOS module system
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TOS;
}
"#;

/// Example Lua module template
pub const LUA_MODULE_TEMPLATE: &str = r#"
-- TOS Lua Module Template
-- Module: {{name}}
-- Version: {{version}}

local M = {}

M.name = "{{name}}"
M.version = "{{version}}"

-- Called when module is loaded
function M.on_load(state)
    print(string.format("Module %s loaded", M.name))
end

-- Called when module is unloaded
function M.on_unload(state)
    print(string.format("Module %s unloaded", M.name))
end

-- Render override for specific hierarchy levels
function M.render(level)
    if level == "ApplicationFocus" then
        return string.format('<div class="custom-overlay">%s</div>', M.name)
    end
    return nil
end

-- Handle bezel actions
function M.bezel_actions()
    return {"custom-action-1", "custom-action-2"}
end

-- Handle commands
function M.handle_command(cmd)
    if cmd == "custom-action-1" then
        return "Executed custom action 1"
    end
    return nil
end

return M
"#;

/// Generate a module template
pub fn generate_module_template(language: ScriptLanguage, name: &str, version: &str) -> String {
    let template = match language {
        ScriptLanguage::JavaScript => JS_MODULE_TEMPLATE,
        ScriptLanguage::Lua => LUA_MODULE_TEMPLATE,
        ScriptLanguage::Python => "# Python module template not yet implemented",
    };
    
    template
        .replace("{{name}}", name)
        .replace("{{version}}", version)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_manifest(name: &str, language: &str) -> ModuleManifest {
        ModuleManifest {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            license: "MIT".to_string(),
            module_type: super::super::manifest::ModuleType::ApplicationModel,
            entry: format!("test.{}", language),
            language: Some(language.to_string()),
            permissions: vec![],
            container: Default::default(),
            config: HashMap::new(),
            dependencies: vec![],
            min_tos_version: None,
        }
    }
    
    #[test]
    fn test_script_engine_new() {
        let manifest = create_test_manifest("test", "javascript");
        let engine = ScriptEngine::new(ScriptLanguage::JavaScript, "console.log('test');".to_string(), manifest);
        
        assert_eq!(engine.language(), ScriptLanguage::JavaScript);
        assert_eq!(engine.manifest().name, "test");
    }
    
    #[test]
    fn test_script_module_wrapper() {
        let manifest = create_test_manifest("test", "javascript");
        let engine = ScriptEngine::new(ScriptLanguage::JavaScript, "function render() {}".to_string(), manifest);
        let wrapper = engine.as_tos_module();
        
        assert!(wrapper.name().contains("test"));
        assert!(wrapper.name().contains("JS"));
        assert!(wrapper.render_override(HierarchyLevel::ApplicationFocus).is_some());
    }
    
    #[test]
    fn test_generate_js_template() {
        let template = generate_module_template(ScriptLanguage::JavaScript, "my-module", "1.0.0");
        
        assert!(template.contains("my-module"));
        assert!(template.contains("1.0.0"));
        assert!(template.contains("onLoad"));
        assert!(template.contains("render"));
    }
    
    #[test]
    fn test_generate_lua_template() {
        let template = generate_module_template(ScriptLanguage::Lua, "my-module", "1.0.0");
        
        assert!(template.contains("my-module"));
        assert!(template.contains("1.0.0"));
        assert!(template.contains("on_load"));
        assert!(template.contains("render"));
    }
    
    #[test]
    fn test_script_app_model() {
        let manifest = create_test_manifest("test-app", "javascript");
        let engine = ScriptEngine::new(ScriptLanguage::JavaScript, "function test() {}".to_string(), manifest);
        let app_model = ScriptAppModel::new(engine);
        
        assert_eq!(app_model.title(), "test-app");
        assert!(app_model.app_class().starts_with("script."));
    }
    
    #[test]
    fn test_script_sector_type() {
        let mut config = HashMap::new();
        config.insert("default_hub_mode".to_string(), serde_json::json!("directory"));
        
        let mut manifest = create_test_manifest("test-sector", "lua");
        manifest.module_type = super::super::manifest::ModuleType::SectorType;
        manifest.config = config;
        
        let engine = ScriptEngine::new(ScriptLanguage::Lua, "-- test".to_string(), manifest);
        let sector_type = ScriptSectorType::new(engine);
        
        assert_eq!(sector_type.name(), "test-sector");
        assert_eq!(sector_type.default_hub_mode(), CommandHubMode::Directory);
    }
}
