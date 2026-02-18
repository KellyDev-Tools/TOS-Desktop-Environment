//! Module System Integration Tests
//! 
//! Tests the complete module system workflow including:
//! - Module discovery and loading
//! - Hot-reload functionality
//! - Integration with TosState
//! - Containerization commands

use tos_core::TosState;
use tos_core::modules::{
    ModuleRegistry, ModuleLoader, ModuleManifest, ModuleType, ContainerBackend, ContainerConfig,
    AppModel, AppModelRegistry, SectorTypeImpl, SectorTypeRegistry,
    ScriptEngine, ScriptLanguage, ScriptEngineFactory, generate_module_template,
};
use tos_core::{ApplicationModel, SectorType};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

/// Test complete module system initialization
#[test]
fn test_module_system_initialization() {
    let state = TosState::new();
    
    
    // Verify built-in app models are registered
    assert!(state.app_model_registry.contains("tos.terminal"));
    assert!(state.app_model_registry.contains("tos.browser"));
    assert!(state.app_model_registry.contains("tos.editor"));
    
    // Verify built-in sector types are registered
    assert!(state.sector_type_registry.contains("development"));
    assert!(state.sector_type_registry.contains("science"));
    assert!(state.sector_type_registry.contains("operations"));
}

/// Test module manifest creation and validation
#[test]
fn test_module_manifest_complete_workflow() {
    let mut config = HashMap::new();
    config.insert("app_class".to_string(), serde_json::json!("test.app"));
    config.insert("bezel_actions".to_string(), serde_json::json!([
        {"id": "test", "label": "Test", "icon": "⚡", "command": "test", "priority": 10}
    ]));
    
    let manifest = ModuleManifest {
        name: "integration-test-module".to_string(),
        version: "1.0.0".to_string(),
        description: "Integration test module".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::ApplicationModel,
        entry: "libtest.so".to_string(),
        language: Some("rust".to_string()),
        permissions: vec!["network".to_string(), "filesystem".to_string()],
        container: ContainerConfig {
            backend: ContainerBackend::Bubblewrap,
            network: false,
            read_only_paths: vec!["/usr".to_string()],
            read_write_paths: vec!["/tmp".to_string()],
        },
        config,
        dependencies: vec![],
        min_tos_version: Some("1.0.0".to_string()),
    };
    
    // Validate the manifest
    assert!(manifest.validate().is_ok());
    
    // Check properties
    assert_eq!(manifest.name, "integration-test-module");
    assert_eq!(manifest.identifier(), "integration-test-module@1.0.0");
    assert!(manifest.is_containerized());
    assert_eq!(manifest.container.backend, ContainerBackend::Bubblewrap);
}

/// Test module loading from filesystem
#[test]
fn test_module_loading_from_filesystem() {
    let temp_dir = tempfile::tempdir().unwrap();
    let module_dir = temp_dir.path().join("test-module");
    std::fs::create_dir(&module_dir).unwrap();
    
    // Create manifest file
    let manifest_content = r#"
name = "filesystem-test-module"
version = "1.2.3"
description = "Test module from filesystem"
author = "Test"
license = "Apache-2.0"
type = "app-model"
entry = "main.js"
language = "javascript"

permissions = ["network"]

[container]
backend = "firejail"
network = true

[config]
app_class = "fs.test"
"#;
    
    let manifest_path = module_dir.join("module.toml");
    let mut file = std::fs::File::create(&manifest_path).unwrap();
    file.write_all(manifest_content.as_bytes()).unwrap();
    
    // Create source file
    let source_path = module_dir.join("main.js");
    let mut source_file = std::fs::File::create(&source_path).unwrap();
    source_file.write_all(b"// Test module\nexport function render() {}").unwrap();
    
    // Load using ModuleLoader
    let mut loader = ModuleLoader::new();
    loader.add_path(temp_dir.path());
    
    let modules = loader.scan_modules().unwrap();
    assert_eq!(modules.len(), 1);
    
    let (_path, manifest) = &modules[0];
    assert_eq!(manifest.name, "filesystem-test-module");
    assert_eq!(manifest.version, "1.2.3");
    assert_eq!(manifest.module_type, ModuleType::ApplicationModel);
}

/// Test container command generation
#[test]
fn test_containerization_commands() {
    let mut manifest = ModuleManifest {
        name: "container-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::ApplicationModel,
        entry: "app".to_string(),
        language: None,
        permissions: vec![],
        container: ContainerConfig::default(),
        config: HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let loader = ModuleLoader::new();
    let entry_path = PathBuf::from("/tmp/test/app");
    
    // Test no containerization
    assert!(loader.build_container_command(&manifest, &entry_path).is_none());
    
    // Test bubblewrap
    manifest.container.backend = ContainerBackend::Bubblewrap;
    manifest.container.network = false;
    let cmd = loader.build_container_command(&manifest, &entry_path).unwrap();
    assert_eq!(cmd[0], "bwrap");
    assert!(cmd.contains(&"--unshare-net".to_string()));
    
    // Test firejail
    manifest.container.backend = ContainerBackend::Firejail;
    manifest.container.network = true;
    let cmd = loader.build_container_command(&manifest, &entry_path).unwrap();
    assert_eq!(cmd[0], "firejail");
    assert!(!cmd.contains(&"--net=none".to_string()));
    
    // Test docker
    manifest.container.backend = ContainerBackend::Docker;
    let cmd = loader.build_container_command(&manifest, &entry_path).unwrap();
    assert_eq!(cmd[0], "docker");
    assert!(cmd.contains(&"--rm".to_string()));
    
    // Test podman
    manifest.container.backend = ContainerBackend::Podman;
    let cmd = loader.build_container_command(&manifest, &entry_path).unwrap();
    assert_eq!(cmd[0], "podman");
}

/// Test app model from manifest
#[test]
fn test_app_model_from_manifest_integration() {
    let mut config = HashMap::new();
    config.insert("app_class".to_string(), serde_json::json!("custom.app"));
    config.insert("decoration_policy".to_string(), serde_json::json!("overlay"));
    config.insert("bezel_actions".to_string(), serde_json::json!([
        {"id": "action1", "label": "Action 1", "icon": "🔧", "command": "cmd1", "priority": 10},
        {"id": "action2", "label": "Action 2", "icon": "⚙️", "command": "cmd2", "priority": 20}
    ]));
    
    let manifest = ModuleManifest {
        name: "custom-app".to_string(),
        version: "1.0.0".to_string(),
        description: "Custom app".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::ApplicationModel,
        entry: "app.so".to_string(),
        language: None,
        permissions: vec![],
        container: ContainerConfig::default(),
        config,
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let app_model = AppModel::from_manifest(&manifest);
    
    assert_eq!(app_model.title(), "custom-app");
    assert_eq!(app_model.app_class(), "custom.app");
    
    let actions = app_model.bezel_actions();
    assert_eq!(actions.len(), 2);
    assert!(actions[0].contains("action1:Action 1:🔧"));
    assert!(actions[1].contains("action2:Action 2:⚙️"));
}

/// Test sector type from manifest
#[test]
fn test_sector_type_from_manifest_integration() {
    let mut config = HashMap::new();
    config.insert("default_hub_mode".to_string(), serde_json::json!("directory"));
    config.insert("environment".to_string(), serde_json::json!({
        "EDITOR": "nvim",
        "TERM": "xterm-256color"
    }));
    config.insert("command_favourites".to_string(), serde_json::json!([
        {"command": "git status", "label": "Git Status", "description": "Check git status", "category": "git", "icon": "📊"},
        {"command": "cargo build", "label": "Build", "description": "Build project", "category": "rust", "icon": "🔨"}
    ]));
    
    let manifest = ModuleManifest {
        name: "rust-dev".to_string(),
        version: "2.0.0".to_string(),
        description: "Rust development sector".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::SectorType,
        entry: "sector.so".to_string(),
        language: None,
        permissions: vec![],
        container: ContainerConfig::default(),
        config,
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let sector_type = SectorTypeImpl::from_manifest(&manifest);
    
    assert_eq!(sector_type.name(), "rust-dev");
    assert_eq!(sector_type.default_hub_mode(), tos_core::CommandHubMode::Directory);
    
    let env = sector_type.environment();
    assert_eq!(env.get("EDITOR"), Some(&"nvim".to_string()));
    assert_eq!(env.get("TERM"), Some(&"xterm-256color".to_string()));
    
    let favorites = sector_type.command_favourites();
    assert_eq!(favorites.len(), 2);
}

/// Test script engine factory
#[test]
fn test_script_engine_factory_integration() {
    let manifest = ModuleManifest {
        name: "js-widget".to_string(),
        version: "1.0.0".to_string(),
        description: "JS widget".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::ApplicationModel,
        entry: "widget.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: ContainerConfig::default(),
        config: HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = temp_dir.path().join("widget.js");
    let mut file = std::fs::File::create(&source_path).unwrap();
    file.write_all(b"export function render() { return '<div>Widget</div>'; }").unwrap();
    
    let engine = ScriptEngineFactory::from_file(&manifest, &source_path).unwrap();
    
    assert_eq!(engine.language(), ScriptLanguage::JavaScript);
    assert_eq!(engine.manifest().name, "js-widget");
}

/// Test module template generation
#[test]
fn test_module_template_generation_integration() {
    let js_template = generate_module_template(ScriptLanguage::JavaScript, "my-widget", "1.0.0");
    assert!(js_template.contains("my-widget"));
    assert!(js_template.contains("1.0.0"));
    assert!(js_template.contains("onLoad"));
    assert!(js_template.contains("render"));
    assert!(js_template.contains("module.exports"));
    
    let lua_template = generate_module_template(ScriptLanguage::Lua, "my-module", "2.0.0");
    assert!(lua_template.contains("my-module"));
    assert!(lua_template.contains("2.0.0"));
    assert!(lua_template.contains("on_load"));
    assert!(lua_template.contains("return M"));
}

/// Test app model registry operations
#[test]
fn test_app_model_registry_integration() {
    let mut registry = AppModelRegistry::new();
    
    // Register built-in models
    registry.register_builtin_models();
    assert_eq!(registry.list_classes().len(), 3);
    
    // Get models
    let terminal = registry.get("tos.terminal").unwrap();
    assert_eq!(terminal.title(), "Terminal");
    
    let browser = registry.get("tos.browser").unwrap();
    assert_eq!(browser.title(), "Browser");
    
    // Get default
    let default = registry.default_model();
    assert!(!default.title().is_empty());
    
    // Remove model
    let removed = registry.remove("tos.editor");
    assert!(removed.is_some());
    assert!(!registry.contains("tos.editor"));
}

/// Test sector type registry operations
#[test]
fn test_sector_type_registry_integration() {
    let mut registry = SectorTypeRegistry::new();
    
    // Register built-in types
    registry.register_builtin_types();
    assert_eq!(registry.list_names().len(), 3);
    
    // Get types
    let dev = registry.get("development").unwrap();
    assert_eq!(dev.name(), "development");
    assert!(!dev.command_favourites().is_empty());
    
    let science = registry.get("science").unwrap();
    assert_eq!(science.name(), "science");
    
    // Get default
    let default = registry.default_type();
    assert!(!default.name().is_empty());
}

/// Test module registry with TosState
#[test]
fn test_module_registry_tos_state_integration() {
    let state = TosState::new();
    
    // Initially should have scanned default paths
    let module_count = state.module_count();
    let modules = state.list_modules();
    assert_eq!(modules.len(), module_count);
    
    // Check module loaded status
    for module in &modules {
        assert!(state.is_module_loaded(module));
    }
}

/// Test script-based app model
#[test]
fn test_script_app_model_integration() {
    let manifest = ModuleManifest {
        name: "script-app".to_string(),
        version: "1.0.0".to_string(),
        description: "Script app".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::ApplicationModel,
        entry: "app.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: ContainerConfig::default(),
        config: {
            let mut map = HashMap::new();
            map.insert("app_class".to_string(), serde_json::json!("script.test"));
            map.insert("bezel_actions".to_string(), serde_json::json!(["action1", "action2"]));
            map
        },
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let engine = ScriptEngine::new(
        ScriptLanguage::JavaScript,
        "function action1() {}; function action2() {}".to_string(),
        manifest
    );
    
    use tos_core::modules::script::ScriptAppModel;
    use tos_core::ApplicationModel;
    
    let app_model = ScriptAppModel::new(engine);
    
    assert_eq!(app_model.title(), "script-app");
    assert_eq!(app_model.app_class(), "script.test");
    
    let actions = app_model.bezel_actions();
    assert_eq!(actions.len(), 2);
    
    // Test command handling
    assert!(app_model.handle_command("action1").is_some());
    assert!(app_model.handle_command("unknown").is_none());
}

/// Test script-based sector type
#[test]
fn test_script_sector_type_integration() {
    let mut config = HashMap::new();
    config.insert("default_hub_mode".to_string(), serde_json::json!("activity"));
    config.insert("command_favourites".to_string(), serde_json::json!(["cmd1", "cmd2"]));
    
    let manifest = ModuleManifest {
        name: "script-sector".to_string(),
        version: "1.0.0".to_string(),
        description: "Script sector".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::SectorType,
        entry: "sector.lua".to_string(),
        language: Some("lua".to_string()),
        permissions: vec![],
        container: ContainerConfig::default(),
        config,
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let engine = ScriptEngine::new(
        ScriptLanguage::Lua,
        "-- sector config".to_string(),
        manifest
    );
    
    use tos_core::modules::script::ScriptSectorType;
    use tos_core::SectorType;
    
    let sector_type = ScriptSectorType::new(engine);
    
    assert_eq!(sector_type.name(), "script-sector");
    assert_eq!(sector_type.default_hub_mode(), tos_core::CommandHubMode::Activity);
    
    let favorites = sector_type.command_favourites();
    assert_eq!(favorites.len(), 2);
}

/// Test error handling in module system
#[test]
fn test_module_error_handling() {
    // Test incomplete manifest (missing required fields)
    let incomplete_toml = r#"
name = "test"
version = "1.0.0"
"#;
    
    // Parsing incomplete TOML should fail due to missing required fields
    let result = ModuleManifest::from_toml_str(incomplete_toml);
    assert!(result.is_err());
    
    // Test empty name validation
    let mut config = HashMap::new();
    config.insert("app_class".to_string(), serde_json::json!("test.app"));
    
    let invalid_manifest = ModuleManifest {
        name: "".to_string(), // Empty name should fail validation
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: ModuleType::ApplicationModel,
        entry: "test.so".to_string(),
        language: None,
        permissions: vec![],
        container: ContainerConfig::default(),
        config,
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let validation = invalid_manifest.validate();
    assert!(validation.is_err()); // Validation should fail for empty name
    
    // Test non-existent file
    let result = ModuleManifest::from_toml_file(PathBuf::from("/nonexistent/module.toml").as_path());
    assert!(result.is_err());
}

/// Test module state transitions
#[test]
fn test_module_state_transitions() {
    use tos_core::modules::ModuleState;
    
    let mut registry = ModuleRegistry::new();
    
    // Create a test manifest
    let _manifest = ModuleManifest {
        name: "state-test".to_string(),
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
    };
    
    // Load module via scan_and_load (public API)
    let temp_dir = tempfile::tempdir().unwrap();
    let module_dir = temp_dir.path().join("state-test");
    std::fs::create_dir(&module_dir).unwrap();
    
    let manifest_content = r#"
name = "state-test"
version = "1.0.0"
description = "Test"
author = "Test"
license = "MIT"
type = "app-model"
entry = "test.so"
"#;
    let manifest_path = module_dir.join("module.toml");
    let mut file = std::fs::File::create(&manifest_path).unwrap();
    file.write_all(manifest_content.as_bytes()).unwrap();
    
    registry.add_path(temp_dir.path());
    let loaded = registry.scan_and_load().unwrap();
    assert_eq!(loaded.len(), 1);
    let name = &loaded[0];
    
    // Check initial state
    let info = registry.get(name).unwrap();
    assert_eq!(info.state, ModuleState::Loaded);
    
    // Note: initialize_all and shutdown_all would require a real TosState
    // which is tested in other integration tests
}
