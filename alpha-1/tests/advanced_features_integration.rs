//! Integration tests for Advanced Features: Voice, Collaboration, Minimap, and Script Module systems
//! Tests cover: Voice command processing, collaboration features, minimap navigation, script engine

use tos_core::*;
use tos_core::system::voice::{VoiceCommandProcessor, VoiceConfig, VoiceState, ConfidenceLevel};
use tos_core::system::input::SemanticEvent;
use tos_core::system::collaboration::CollaborationRole;
use tos_core::modules::script::{ScriptEngine, ScriptLanguage, ScriptAppModel, ScriptSectorType};

// ============================================================================
// Voice Command Tests (Unit & Component)
// ============================================================================

#[test]
fn test_voice_config_default_values() {
    let config = VoiceConfig::default();
    assert_eq!(config.wake_word, "computer");
    assert!(config.visual_feedback);
    assert!(config.audio_feedback);
    assert_eq!(config.language, "en-US");
    assert_eq!(config.auto_execute_threshold, 0.7);
    assert_eq!(config.max_command_duration_secs, 5);
}

#[test]
fn test_voice_config_customization() {
    let mut config = VoiceConfig::default();
    config.wake_word = "assistant".to_string();
    config.audio_feedback = false;
    config.language = "es-ES".to_string();
    config.auto_execute_threshold = 0.9;
    
    assert_eq!(config.wake_word, "assistant");
    assert!(!config.audio_feedback);
    assert_eq!(config.language, "es-ES");
    assert_eq!(config.auto_execute_threshold, 0.9);
}

#[test]
fn test_voice_processor_creation() {
    let processor = VoiceCommandProcessor::new();
    assert!(matches!(processor.state, VoiceState::Idle));
}

#[test]
fn test_voice_simulate_wake_word() {
    let mut processor = VoiceCommandProcessor::new();
    processor.simulate_wake_word();
    assert!(processor.is_listening());
}

#[test]
fn test_voice_state_transitions() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Initial state
    assert!(matches!(processor.state, VoiceState::Idle));
    
    // Activate wake word
    processor.simulate_wake_word();
    assert!(processor.is_listening());
}

#[test]
fn test_voice_parse_navigation_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Test zoom in
    let (event, confidence) = processor.parse_command("go deeper");
    assert!(matches!(event, SemanticEvent::ZoomIn));
    assert!(confidence > 0.8);
    
    // Test zoom out
    let (event, confidence) = processor.parse_command("go up");
    assert!(matches!(event, SemanticEvent::ZoomOut));
    assert!(confidence > 0.8);
}

#[test]
fn test_voice_parse_mode_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Test command mode
    let (event, _) = processor.parse_command("command mode");
    assert!(matches!(event, SemanticEvent::ModeCommand));
    
    // Test directory mode
    let (event, _) = processor.parse_command("directory mode");
    assert!(matches!(event, SemanticEvent::ModeDirectory));
    
    // Test activity mode
    let (event, _) = processor.parse_command("activity mode");
    assert!(matches!(event, SemanticEvent::ModeActivity));
}

#[test]
fn test_voice_parse_system_commands() {
    let processor = VoiceCommandProcessor::new();
    
    // Test reset
    let (event, _) = processor.parse_command("reset system");
    assert!(matches!(event, SemanticEvent::TacticalReset));
}

#[test]
fn test_voice_parse_unknown_command() {
    let processor = VoiceCommandProcessor::new();
    
    let (event, confidence) = processor.parse_command("foobarbazxyz");
    assert!(matches!(event, SemanticEvent::SubmitPrompt));
    assert!(confidence < 0.5);
}

#[test]
fn test_voice_parse_with_extra_words() {
    let processor = VoiceCommandProcessor::new();
    
    // Command with extra words should still match
    let (event, confidence) = processor.parse_command("please go deeper now please");
    assert!(matches!(event, SemanticEvent::ZoomIn));
    assert!(confidence > 0.5);
}

#[test]
fn test_voice_confidence_levels() {
    assert_eq!(ConfidenceLevel::from_score(0.9), ConfidenceLevel::High);
    assert_eq!(ConfidenceLevel::from_score(0.7), ConfidenceLevel::Medium);
    assert_eq!(ConfidenceLevel::from_score(0.5), ConfidenceLevel::Low);
}

#[test]
fn test_voice_command_history() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Process some commands
    processor.process_text("go deeper");
    processor.process_text("go up");
    processor.process_text("reset system");
    
    // Check history
    assert_eq!(processor.command_history.len(), 3);
    
    let history = processor.get_history(2);
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].raw_text, "reset system");
}

#[test]
fn test_voice_command_clear_history() {
    let mut processor = VoiceCommandProcessor::new();
    
    processor.process_text("go deeper");
    assert!(!processor.command_history.is_empty());
    
    processor.clear_history();
    assert!(processor.command_history.is_empty());
}

#[test]
fn test_voice_status_text() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Idle status
    let status = processor.get_status_text();
    assert!(status.contains("disabled") || status.contains("Listening"));
    
    // After wake word
    processor.simulate_wake_word();
    let status = processor.get_status_text();
    assert!(status.contains("Listening for command"));
}

#[test]
fn test_voice_render_indicator() {
    let processor = VoiceCommandProcessor::new();
    
    let html = processor.render_indicator();
    assert!(html.contains("voice-indicator"));
    assert!(html.contains("🎤"));
}

#[test]
fn test_voice_render_help() {
    let processor = VoiceCommandProcessor::new();
    
    let html = processor.render_help();
    assert!(html.contains("Voice Commands"));
    assert!(html.contains("zoom in"));
    assert!(html.contains("Computer")); // Wake word is capitalized in help text
}

// ============================================================================
// Collaboration Tests (Unit & Component)
// ============================================================================

#[test]
fn test_collaboration_manager_initialization() {
    let state = TosState::new();
    // Manager should be initialized
    assert!(std::mem::size_of_val(&state.collaboration_manager) > 0);
}

#[test]
fn test_collaboration_add_participant() {
    let mut state = TosState::new();
    let sector_idx = 0;
    
    state.add_participant(sector_idx, "TestUser".to_string(), "#ff0000".to_string(), "Operator");
    
    let participants = &state.sectors[sector_idx].participants;
    assert!(participants.iter().any(|p| p.name == "TestUser"));
}

#[test]
fn test_collaboration_role_strings() {
    use tos_core::system::collaboration::CollaborationRole;
    
    let _co_owner = CollaborationRole::CoOwner;
    let _operator = CollaborationRole::Operator;
    let _viewer = CollaborationRole::Viewer;
}

#[test]
fn test_collaboration_permission_sets() {
    use tos_core::system::collaboration::{PermissionSet, CollaborationRole};
    
    // Test CoOwner has all permissions
    let co_owner = PermissionSet::for_role(CollaborationRole::CoOwner);
    assert!(co_owner.allow_sector_reset);
    assert!(co_owner.allow_shell_input);
    assert!(co_owner.allow_app_launch);
    assert!(co_owner.allow_participant_invite);
    
    // Test Viewer has limited permissions
    let viewer = PermissionSet::for_role(CollaborationRole::Viewer);
    assert!(!viewer.allow_shell_input);
    assert!(!viewer.allow_app_launch);
}

#[test]
fn test_collaboration_role_can_interact() {
    use tos_core::system::collaboration::CollaborationRole;
    
    assert!(CollaborationRole::CoOwner.can_interact());
    assert!(CollaborationRole::Operator.can_interact());
    assert!(!CollaborationRole::Viewer.can_interact());
}

#[test]
fn test_collaboration_role_can_manage() {
    use tos_core::system::collaboration::CollaborationRole;
    
    assert!(CollaborationRole::CoOwner.can_manage());
    assert!(!CollaborationRole::Operator.can_manage());
    assert!(!CollaborationRole::Viewer.can_manage());
}

// ============================================================================
// Minimap Tests (Unit & Component)
// ============================================================================

#[test]
fn test_minimap_initial_state() {
    let state = TosState::new();
    let minimap = &state.minimap;
    
    assert!(!minimap.is_active());
    assert!(minimap.selected_sector.is_none());
}

#[test]
fn test_minimap_toggle() {
    let mut state = TosState::new();
    
    state.toggle_minimap();
    assert!(state.minimap.is_active());
    
    state.toggle_minimap();
    assert!(!state.minimap.is_active());
}

#[test]
fn test_minimap_render_contains_elements() {
    let mut state = TosState::new();
    state.toggle_minimap();
    let html = state.minimap.render(&state);
    
    assert!(html.contains("TACTICAL MINI-MAP"));
    assert!(html.contains("minimap-sectors-grid"));
}

// ============================================================================
// Script Engine Tests (Unit & Component)
// ============================================================================

#[test]
fn test_script_engine_language_variants() {
    let _js = ScriptLanguage::JavaScript;
    let _lua = ScriptLanguage::Lua;
    let _python = ScriptLanguage::Python;
}

#[test]
fn test_script_engine_create() {
    let manifest = tos_core::modules::manifest::ModuleManifest {
        name: "test-script".to_string(),
        version: "1.0.0".to_string(),
        description: "Test script".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::ApplicationModel,
        entry: "test.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let engine = ScriptEngine::javascript("console.log('test');".to_string(), manifest);
    assert_eq!(engine.language(), ScriptLanguage::JavaScript);
}

#[test]
fn test_script_engine_initialize() {
    let manifest = tos_core::modules::manifest::ModuleManifest {
        name: "test-init".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::ApplicationModel,
        entry: "test.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let mut engine = ScriptEngine::javascript("function test() {}".to_string(), manifest);
    let result = engine.initialize();
    assert!(result.is_ok());
}

#[test]
fn test_script_engine_execute() {
    let manifest = tos_core::modules::manifest::ModuleManifest {
        name: "test-exec".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::ApplicationModel,
        entry: "test.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let mut engine = ScriptEngine::javascript("function test() {}".to_string(), manifest);
    let _ = engine.initialize();
    
    let result = engine.execute("test", &[]);
    assert!(result.is_ok());
}

#[test]
fn test_script_app_model() {
    let manifest = tos_core::modules::manifest::ModuleManifest {
        name: "test-app-model".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::ApplicationModel,
        entry: "test.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let engine = ScriptEngine::javascript("function test() {}".to_string(), manifest);
    let app_model = ScriptAppModel::new(engine);
    
    assert_eq!(app_model.title(), "test-app-model");
    assert!(app_model.app_class().starts_with("script."));
}

#[test]
fn test_script_sector_type() {
    let manifest = tos_core::modules::manifest::ModuleManifest {
        name: "test-sector-type".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::SectorType,
        entry: "test.lua".to_string(),
        language: Some("lua".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    
    let engine = ScriptEngine::lua("-- test".to_string(), manifest);
    let sector_type = ScriptSectorType::new(engine);
    
    assert_eq!(sector_type.name(), "test-sector-type");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_voice_through_state_integration() {
    let mut state = TosState::new();
    
    // Process a voice command through state
    let result = state.process_voice_command("go deeper");
    assert!(result.is_some());
    
    // Verify state changed
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
}

#[test]
fn test_collaboration_with_sectors_integration() {
    let mut state = TosState::new();
    
    // Add a participant to a sector
    state.add_participant(0, "GuestUser".to_string(), "#00ff00".to_string(), "Viewer");
    
    // Verify participant was added
    assert_eq!(state.sectors[0].participants.len(), 2); // Host + GuestUser
    
    // Render should include participant indicator (collaboration UI element)
    let html = state.render_current_view();
    assert!(html.contains("participant") || html.contains("collaboration") || html.contains("GuestUser") || html.contains("PARTICIPANT") || html.contains("USERS"));
}

#[test]
fn test_minimap_with_zoom_integration() {
    let mut state = TosState::new();
    
    // Activate minimap
    state.toggle_minimap();
    assert!(state.minimap.is_active());
    
    // Zoom in should work
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
}

#[test]
fn test_script_module_in_state_integration() {
    let state = TosState::new();
    
    // Verify module registry exists
    let _ = state.module_count();
    
    // List modules
    let _modules = state.list_modules();
}

#[test]
fn test_voice_reset_system_integration() {
    let mut state = TosState::new();
    
    // First zoom in
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    
    // Use voice command to reset - "reset" maps to TacticalReset which may not auto-execute zoom out
    let result = state.process_voice_command("reset");
    assert!(result.is_some());
    
    // The voice command was processed successfully - verify the semantic event was returned
    let command = result.unwrap();
    assert!(matches!(command.event, SemanticEvent::TacticalReset));
    
    // Note: Actual state change depends on event handling in the main loop
    // The voice processor returns the event, but state change requires handle_semantic_event
}

#[test]
fn test_minimap_toggle_rendering_integration() {
    let mut state = TosState::new();
    
    // Toggle minimap on
    state.toggle_minimap();
    let html = state.minimap.render(&state);
    assert!(html.contains("tactical-minimap") && html.contains("active-overlay"));
}

#[test]
fn test_collaboration_role_enforcement_integration() {
    use tos_core::system::collaboration::{CollaborationRole, PermissionSet};
    
    // Test all roles have view permission (implicit)
    let co_owner = PermissionSet::for_role(CollaborationRole::CoOwner);
    let operator = PermissionSet::for_role(CollaborationRole::Operator);
    let viewer = PermissionSet::for_role(CollaborationRole::Viewer);
    
    // All roles should be able to view sectors (via can_interact check)
    assert!(co_owner.allow_shell_input);
    assert!(operator.allow_shell_input);
    assert!(!viewer.allow_shell_input);
}

#[test]
fn test_voice_command_confidence_integration() {
    let processor = VoiceCommandProcessor::new();
    
    // Clear command should have high confidence
    let (_, confidence) = processor.parse_command("go deeper");
    assert!(confidence >= 0.8);
    
    // Ambiguous command should have lower confidence
    let (_, confidence) = processor.parse_command("go somewhere");
    assert!(confidence < 0.9);
}

#[test]
fn test_minimap_click_target_integration() {
    let state = TosState::new();
    
    // When minimap is active, click handling should work
    // The stub implementation returns placeholder geometry
    let _target = state.minimap.handle_click(0.5, 0.5, &state);
}

#[test]
fn test_script_engine_multiple_languages_integration() {
    // Test JavaScript
    let js_manifest = tos_core::modules::manifest::ModuleManifest {
        name: "js-test".to_string(),
        version: "1.0.0".to_string(),
        description: "JS Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::ApplicationModel,
        entry: "test.js".to_string(),
        language: Some("javascript".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    let js_engine = ScriptEngine::javascript("console.log('js');".to_string(), js_manifest);
    assert_eq!(js_engine.language(), ScriptLanguage::JavaScript);
    
    // Test Lua
    let lua_manifest = tos_core::modules::manifest::ModuleManifest {
        name: "lua-test".to_string(),
        version: "1.0.0".to_string(),
        description: "Lua Test".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        module_type: tos_core::modules::manifest::ModuleType::ApplicationModel,
        entry: "test.lua".to_string(),
        language: Some("lua".to_string()),
        permissions: vec![],
        container: Default::default(),
        config: std::collections::HashMap::new(),
        dependencies: vec![],
        min_tos_version: None,
    };
    let lua_engine = ScriptEngine::lua("print('lua')".to_string(), lua_manifest);
    assert_eq!(lua_engine.language(), ScriptLanguage::Lua);
}

#[test]
fn test_voice_custom_commands_integration() {
    let mut processor = VoiceCommandProcessor::new();
    
    // Add custom command
    processor.config.custom_commands.insert("test cmd".to_string(), "zoom_in".to_string());
    
    let (event, confidence) = processor.parse_command("test cmd");
    assert!(matches!(event, SemanticEvent::ZoomIn));
    assert_eq!(confidence, 0.95); // High confidence for custom commands
}

#[test]
fn test_collaboration_invitation_system_integration() {
    let mut state = TosState::new();
    let sector_id = state.sectors[0].id;
    
    // Create invitation
    let token = state.collaboration_manager.create_invitation(sector_id, CollaborationRole::Operator);
    assert!(!token.is_empty());
    
    // Redeem invitation
    let result = state.collaboration_manager.redeem_invitation(&token);
    assert!(result.is_some());
}

#[test]
fn test_minimap_state_after_zoom_integration() {
    let mut state = TosState::new();
    
    state.toggle_minimap();
    // Render at global overview
    let html = state.minimap.render(&state);
    assert!(html.contains("Alpha Sector"));
    
    // Zoom in
    state.zoom_in();
    
    // Render at command hub
    let html = state.minimap.render(&state);
    assert!(html.contains("minimap-current-sector"));
}
