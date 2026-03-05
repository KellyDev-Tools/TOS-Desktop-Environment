//! Headless Brain integration tests — exercises IPC commands against
//! a Brain without requiring a display layer.
//!
//! These tests verify that state transitions triggered through the IPC
//! handler produce the expected state deltas, validating the Brain's
//! core logic independent of any display layer.

use tos_alpha2::brain::ipc_handler::IpcHandler;
use tos_alpha2::common::{TosState, HierarchyLevel, CommandHubMode};
use tos_alpha2::services::ServiceManager;
use std::sync::{Arc, Mutex};

/// Build a headless IPC handler with a real TosState but the shell uses
/// whatever is available (/bin/bash or /bin/sh).
fn headless_ipc() -> (Arc<IpcHandler>, Arc<Mutex<TosState>>) {
    let state = Arc::new(Mutex::new(TosState::default()));
    let services = Arc::new(ServiceManager::new());

    let modules = Arc::new(tos_alpha2::brain::module_manager::ModuleManager::new(
        std::path::PathBuf::from("./modules"),
    ));

    let sid = state.lock().unwrap().sectors[0].id;
    let hid = state.lock().unwrap().sectors[0].hubs[0].id;

    let shell = tos_alpha2::brain::shell::ShellApi::new(
        state.clone(), modules.clone(), sid, hid,
    ).expect("Headless tests require at least /bin/sh");

    let shell = Arc::new(Mutex::new(shell));
    let ipc = Arc::new(IpcHandler::new(state.clone(), shell, services));
    (ipc, state)
}

// ---------------------------------------------------------------------------
// Hierarchy Navigation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn zoom_in_transitions_from_global_to_hub() {
    let (ipc, state) = headless_ipc();
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::GlobalOverview);

    let result = ipc.handle_request("zoom_in:");
    assert!(result.contains("ZOOMED_IN"));
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::CommandHub);
}

#[tokio::test]
async fn zoom_out_transitions_from_hub_to_global() {
    let (ipc, state) = headless_ipc();

    ipc.handle_request("zoom_in:");
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::CommandHub);

    let result = ipc.handle_request("zoom_out:");
    assert!(result.contains("ZOOMED_OUT"));
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::GlobalOverview);
}

#[tokio::test]
async fn zoom_to_jumps_directly() {
    let (ipc, state) = headless_ipc();

    let result = ipc.handle_request("zoom_to:ApplicationFocus");
    assert!(result.contains("ZOOMED_TO"));
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::ApplicationFocus);
}

// ---------------------------------------------------------------------------
// Mode Switching
// ---------------------------------------------------------------------------

#[tokio::test]
async fn set_mode_changes_hub_mode() {
    let (ipc, state) = headless_ipc();

    let result = ipc.handle_request("set_mode:directory");
    assert!(result.contains("MODE_SET"));
    let lock = state.lock().unwrap();
    let hub = &lock.sectors[0].hubs[0];
    assert_eq!(hub.mode, CommandHubMode::Directory);
}

#[tokio::test]
async fn set_mode_rejects_unknown() {
    let (ipc, _) = headless_ipc();
    let result = ipc.handle_request("set_mode:invalid_mode");
    assert!(result.contains("ERROR"));
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

#[tokio::test]
async fn set_setting_persists_in_state() {
    let (ipc, state) = headless_ipc();

    let result = ipc.handle_request("set_setting:tos.ai.disabled;true");
    assert!(result.contains("SETTING_UPDATE"));

    let lock = state.lock().unwrap();
    assert_eq!(lock.settings.global.get("tos.ai.disabled"), Some(&"true".to_string()));
}

#[tokio::test]
async fn set_sector_setting_creates_sector_scope() {
    let (ipc, state) = headless_ipc();

    let result = ipc.handle_request("set_sector_setting:dev;theme;amber");
    assert!(result.contains("SECTOR_SETTING_UPDATE"));

    let lock = state.lock().unwrap();
    assert_eq!(
        lock.settings.sectors.get("dev").and_then(|m| m.get("theme")),
        Some(&"amber".to_string())
    );
}

// ---------------------------------------------------------------------------
// Sector Lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn sector_create_adds_new_sector() {
    let (ipc, state) = headless_ipc();

    assert_eq!(state.lock().unwrap().sectors.len(), 1);
    let result = ipc.handle_request("sector_create:Scratch");
    assert!(result.contains("SECTOR_CREATED"));
    assert_eq!(state.lock().unwrap().sectors.len(), 2);
    assert_eq!(state.lock().unwrap().sectors[1].name, "Scratch");
}

#[tokio::test]
async fn sector_close_removes_sector() {
    let (ipc, state) = headless_ipc();

    ipc.handle_request("sector_create:Ephemeral");
    let sector_id = state.lock().unwrap().sectors[1].id.to_string();
    assert_eq!(state.lock().unwrap().sectors.len(), 2);

    let result = ipc.handle_request(&format!("sector_close:{}", sector_id));
    assert!(result.contains("SECTOR_CLOSED"));
    assert_eq!(state.lock().unwrap().sectors.len(), 1);
}

#[tokio::test]
async fn sector_freeze_toggle() {
    let (ipc, state) = headless_ipc();

    let sector_id = state.lock().unwrap().sectors[0].id.to_string();
    assert!(!state.lock().unwrap().sectors[0].frozen);

    ipc.handle_request(&format!("sector_freeze:{}", sector_id));
    assert!(state.lock().unwrap().sectors[0].frozen);

    ipc.handle_request(&format!("sector_freeze:{}", sector_id));
    assert!(!state.lock().unwrap().sectors[0].frozen);
}

// ---------------------------------------------------------------------------
// Service Registry (via IPC)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tos_ports_returns_anchor() {
    let (ipc, _) = headless_ipc();
    let result = ipc.handle_request("tos_ports:");
    assert!(result.contains("tos-brain (anchor)"));
    assert!(result.contains("7000"));
    // Should be valid JSON
    assert!(result.contains("[{"));
}

#[tokio::test]
async fn service_register_and_list() {
    let (ipc, _) = headless_ipc();

    let result = ipc.handle_request("service_register:tos-settingsd;7002");
    assert!(result.contains("SERVICE_REGISTERED"));

    let ports = ipc.handle_request("tos_ports:");
    assert!(ports.contains("tos-settingsd"));
    assert!(ports.contains("7002"));
}

#[tokio::test]
async fn service_deregister_removes_from_list() {
    let (ipc, _) = headless_ipc();

    ipc.handle_request("service_register:tos-loggerd;9999");
    ipc.handle_request("service_deregister:tos-loggerd");

    let ports = ipc.handle_request("tos_ports:");
    assert!(!ports.contains("tos-loggerd"));
}

// ---------------------------------------------------------------------------
// State Delta Sync
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_state_delta_returns_no_change_for_current_version() {
    let (ipc, state) = headless_ipc();

    let version = state.lock().unwrap().version;
    let result = ipc.handle_request(&format!("get_state_delta:{}", version));
    assert!(result.contains("NO_CHANGE"));
}

#[tokio::test]
async fn get_state_delta_returns_full_state_for_stale_version() {
    let (ipc, state) = headless_ipc();

    // Simulate a version bump (normally done by the heartbeat thread)
    {
        let mut lock = state.lock().unwrap();
        lock.version += 1;
    }

    let result = ipc.handle_request("get_state_delta:0");
    // Should return the full serialized state, not NO_CHANGE
    assert!(!result.contains("NO_CHANGE"));
    assert!(result.contains("current_level"));
}

// ---------------------------------------------------------------------------
// AI Staged Command
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ai_stage_and_accept() {
    let (ipc, state) = headless_ipc();
    ipc.handle_request("zoom_in:"); // Must be at CommandHub to have an active hub

    let stage_result = ipc.handle_request(
        r#"ai_stage_command:{"command":"git status","explanation":"Shows working tree status"}"#
    );
    assert!(stage_result.contains("AI_COMMAND_STAGED"));

    {
        let lock = state.lock().unwrap();
        let hub = &lock.sectors[0].hubs[0];
        assert_eq!(hub.staged_command.as_deref(), Some("git status"));
        assert_eq!(hub.ai_explanation.as_deref(), Some("Shows working tree status"));
    }

    let accept_result = ipc.handle_request("ai_suggestion_accept:");
    assert!(accept_result.contains("AI_SUGGESTION_ACCEPTED"));

    {
        let lock = state.lock().unwrap();
        let hub = &lock.sectors[0].hubs[0];
        assert_eq!(hub.prompt, "git status");
        assert!(hub.staged_command.is_none());
    }
}

// ---------------------------------------------------------------------------
// Error Handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn malformed_request_returns_error() {
    let (ipc, _) = headless_ipc();
    let result = ipc.handle_request("no_colon_here");
    assert!(result.contains("ERROR"));
}

#[tokio::test]
async fn unknown_prefix_returns_error() {
    let (ipc, _) = headless_ipc();
    let result = ipc.handle_request("nonexistent_command:payload");
    assert!(result.contains("ERROR"));
}

#[tokio::test]
async fn test_ipc_semicolon_parsing() {
    let (ipc, state) = headless_ipc();
    
    // 1. set_setting:theme;lcars-dark
    let result = ipc.handle_request("set_setting:theme;lcars-dark");
    assert!(result.contains("SETTING_UPDATE"));
    assert_eq!(state.lock().unwrap().settings.global.get("theme"), Some(&"lcars-dark".to_string()));

    // 2. signal_app:00000000-0000-0000-0000-000000000123;SIGKILL
    let result = ipc.handle_request("signal_app:00000000-0000-0000-0000-000000000123;SIGKILL");
    // We expect it to be parsed correctly and trigger an internal signal event
    assert!(result.contains("APP_SIGNALED"));
    // Further assertions on state if signal_app mutates the state
}

#[tokio::test]
async fn test_terminal_buffer_wrap() {
    let (ipc, state) = headless_ipc();
    
    // Setup limit to 5
    ipc.handle_request("set_setting:terminal_buffer_limit;5");

    // Push 6 lines
    for i in 1..=6 {
        ipc.handle_request(&format!("system_log_append:1;Line {}", i));
    }

    // Expected: Buffer contains lines 2-6
    let lock = state.lock().unwrap();
    assert_eq!(lock.system_log.len(), 5);
    assert_eq!(lock.system_log[0].text, "Line 2");
    assert_eq!(lock.system_log[4].text, "Line 6");
}

#[tokio::test]
async fn test_remote_disconnect_timer() {
    let (ipc, state) = headless_ipc();
    
    // Create remote sector
    ipc.handle_request("sector_create:Remote Desktop");
    let sector_id = state.lock().unwrap().sectors[1].id;
    state.lock().unwrap().sectors[1].is_remote = true;

    // Trigger disconnect
    let res = ipc.handle_request(&format!("remote_disconnect:{}", sector_id));
    assert!(res.contains("REMOTE_DISCONNECTED"));
    
    assert!(state.lock().unwrap().sectors[1].disconnected);
    assert_eq!(state.lock().unwrap().sectors.len(), 2);

    // Wait 5.2s (since timer is 5.1s)
    tokio::time::sleep(tokio::time::Duration::from_millis(5200)).await;

    // Verify it's removed
    assert_eq!(state.lock().unwrap().sectors.len(), 1);
}

#[tokio::test]
async fn test_bezel_label_rejection() {
    let (ipc, state) = headless_ipc();
    
    // Call click:ZOOM OUT which should fail
    let res = ipc.handle_request("click:ZOOM OUT");
    assert!(res.contains("ERROR"));

    // Expected: Log warning (verified manually/implicitly via trace output); no state change
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::GlobalOverview);

    // Call click:zoom_in (to move to CommandHub)
    let res = ipc.handle_request("click:zoom_in");
    assert!(res.contains("ZOOMED_IN"));
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::CommandHub);

    // Call click:zoom_out (Identifier)
    let res = ipc.handle_request("click:zoom_out");
    assert!(res.contains("ZOOMED_OUT"));
    assert_eq!(state.lock().unwrap().current_level, HierarchyLevel::GlobalOverview);
}


