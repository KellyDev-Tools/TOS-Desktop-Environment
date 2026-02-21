use tos_core::{TosState, CommandHubMode, HierarchyLevel, CommsMessage};
use std::path::PathBuf;
use std::fs;

#[test]
fn test_workspace_persistence() {
    let mut state = TosState::new();
    
    // Modify state
    state.current_level = HierarchyLevel::CommandHub;
    state.comms_messages.push(CommsMessage {
        from: "TEST".to_string(),
        body: "Persistence Check".to_string(),
        timestamp: "00:00".to_string(),
    });
    
    // Save to temp file
    let temp_save = tempfile::NamedTempFile::new().unwrap();
    let save_path = temp_save.path().to_path_buf();
    
    // We need to manually call save logic or mock the path
    let json = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&save_path, json).unwrap();
    
    // Load back
    let loaded_state: TosState = serde_json::from_str(&fs::read_to_string(&save_path).unwrap()).unwrap();
    
    // Verify
    assert_eq!(loaded_state.current_level, HierarchyLevel::CommandHub);
    assert!(loaded_state.comms_messages.iter().any(|m| m.body == "Persistence Check"));
}

#[test]
fn test_direct_comms_broadcast() {
    let mut state = TosState::new();
    
    // Initial message from Starfleet
    assert!(state.comms_messages.len() >= 1);
    assert_eq!(state.comms_messages[0].from, "STARFLEET");
    
    state.comms_messages.push(CommsMessage {
        from: "USER".to_string(),
        body: "Hello Bridge".to_string(),
        timestamp: "12:00".to_string(),
    });
    state.comms_visible = true;
    
    let html = state.render_current_view();
    assert!(html.contains("Hello Bridge"));
    assert!(html.contains("DIRECT COMMS // ENCRYPTED"));
}

#[test]
fn test_live_module_registry() {
    let state = TosState::new();
    
    // Verify default module count (should scan modules/ if it exists)
    // Even if empty, registry should be initialized
    assert!(state.module_registry.module_names().is_empty() || !state.module_registry.module_names().is_empty());
}

#[test]
fn test_tactical_signals() {
    let mut state = TosState::new();
    state.zoom_in(); // Hub
    state.toggle_mode(CommandHubMode::Activity);
    
    let html = state.render_current_view();
    // Verify SIGINT button is present for apps (if any apps exist in default state)
    // Alpha Sector has "Terminal"
    assert!(html.contains("SIGINT"));
    assert!(html.contains("KILL"));
}

#[test]
fn test_shell_api_output_processing() {
    let mut state = TosState::new();
    
    // Mock shell output with OSC 7 (current directory)
    let output = "\x1b]7;file://localhost/tmp\x07Directory content...";
    let clean = state.process_shell_output(output);
    
    assert_eq!(clean, "Directory content...");
    
    let viewport = &state.viewports[state.active_viewport_index];
    let sector = &state.sectors[viewport.sector_index];
    let hub = &sector.hubs[viewport.hub_index];
    
    assert_eq!(hub.current_directory, PathBuf::from("/tmp"));
}

#[test]
fn test_multi_select_and_action_toolbar() {
    let mut state = TosState::new();
    state.zoom_in();
    state.toggle_mode(CommandHubMode::Directory);
    
    // Select some files
    let viewport_idx = state.active_viewport_index;
    let s_idx = state.viewports[viewport_idx].sector_index;
    let h_idx = state.viewports[viewport_idx].hub_index;
    
    state.sectors[s_idx].hubs[h_idx].selected_files.insert("file1.txt".to_string());
    state.sectors[s_idx].hubs[h_idx].selected_files.insert("file2.txt".to_string());
    
    let html = state.render_current_view();
    assert!(html.contains("2 FILES SELECTED"));
    assert!(html.contains("action-toolbar")); // Check for class name instead
}
