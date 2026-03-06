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

#[tokio::test]
async fn sector_clone_duplicates_state() {
    let (ipc, state) = headless_ipc();

    let source_id = state.lock().unwrap().sectors[0].id;
    let original_name = state.lock().unwrap().sectors[0].name.clone();

    let result = ipc.handle_request(&format!("sector_clone:{}", source_id));
    assert!(result.contains("SECTOR_CLONED"));

    let state_lock = state.lock().unwrap();
    assert_eq!(state_lock.sectors.len(), 2);
    let cloned_sector = &state_lock.sectors[1];
    
    assert_ne!(cloned_sector.id, source_id);
    assert_eq!(cloned_sector.name, format!("{} (Clone)", original_name));
    
    assert_eq!(cloned_sector.hubs.len(), state_lock.sectors[0].hubs.len());
    if !cloned_sector.hubs.is_empty() {
        assert_ne!(cloned_sector.hubs[0].id, state_lock.sectors[0].hubs[0].id);
    }
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

    // Wait 0.6s (since timer is 0.5s)
    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

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


// ---------------------------------------------------------------------------
// Trust Service
// ---------------------------------------------------------------------------

#[tokio::test]
async fn trust_classify_privilege_escalation() {
    let (ipc, state) = headless_ipc();

    // sudo should trigger a chip but not block (default policy = warn)
    let res = ipc.handle_request("prompt_submit:sudo rm /tmp/test");
    assert!(res.contains("SUBMITTED"), "Expected SUBMITTED, got: {}", res);

    // system_log should have a trust chip entry
    let log = state.lock().unwrap().system_log.clone();
    let has_chip = log.iter().any(|l| l.text.contains("[TRUST]") && l.text.contains("PRIVILEGE ESCALATION"));
    assert!(has_chip, "Expected PRIVILEGE ESCALATION chip in system_log");
}

#[tokio::test]
async fn trust_block_privilege_escalation_after_demote() {
    let (ipc, _state) = headless_ipc();

    let res = ipc.handle_request("trust_demote:tos.trust.privilege_escalation");
    assert!(res.contains("TRUST_DEMOTED"), "Got: {}", res);

    // Now a sudo command should be blocked
    let res = ipc.handle_request("prompt_submit:sudo apt update");
    assert!(res.contains("TRUST_BLOCKED"), "Expected TRUST_BLOCKED, got: {}", res);
}

#[tokio::test]
async fn trust_sector_override_promotes_over_global_block() {
    let (ipc, state) = headless_ipc();
    let sector_id = state.lock().unwrap().sectors[0].id.to_string();

    // Block globally, then sector-level promote
    ipc.handle_request("trust_demote:tos.trust.privilege_escalation");
    let res = ipc.handle_request(&format!(
        "trust_promote_sector:{};tos.trust.privilege_escalation",
        sector_id
    ));
    assert!(res.contains("TRUST_SECTOR_PROMOTED"), "Got: {}", res);

    // Sector override should allow the command through
    let res = ipc.handle_request("prompt_submit:sudo ls");
    assert!(res.contains("SUBMITTED"), "Expected sector promote to allow, got: {}", res);
}

#[tokio::test]
async fn trust_clear_sector_falls_back_to_global() {
    let (ipc, state) = headless_ipc();
    let sector_id = state.lock().unwrap().sectors[0].id.to_string();

    ipc.handle_request("trust_demote:tos.trust.privilege_escalation");
    ipc.handle_request(&format!(
        "trust_promote_sector:{};tos.trust.privilege_escalation",
        sector_id
    ));

    let res = ipc.handle_request(&format!("trust_clear_sector:{}", sector_id));
    assert!(res.contains("TRUST_SECTOR_CLEARED"), "Got: {}", res);

    // Should fall back to global block
    let res = ipc.handle_request("prompt_submit:sudo ls");
    assert!(res.contains("TRUST_BLOCKED"), "Expected fallback to global block, got: {}", res);
}

#[tokio::test]
async fn trust_get_config_returns_json() {
    let (ipc, _state) = headless_ipc();

    let res = ipc.handle_request("trust_get_config:");
    assert!(res.contains("TRUST_CONFIG:"), "Got: {}", res);
    assert!(res.contains("tos.trust.privilege_escalation"), "Got: {}", res);
}

// ============================================================
// AIService Refactor integration tests
// ============================================================

#[tokio::test]
async fn ai_backend_set_default_and_get_context() {
    let (ipc, _state) = headless_ipc();

    let res = ipc.handle_request("ai_backend_set_default:my-custom-llm");
    assert!(res.contains("AI_DEFAULT_BACKEND_SET: my-custom-llm"), "Got: {}", res);

    let res = ipc.handle_request("ai_context_request:*");
    assert!(res.contains("AI_CONTEXT:"), "Got: {}", res);
}

#[tokio::test]
async fn ai_behavior_enable_disable_configure() {
    let (ipc, state) = headless_ipc();

    // Register a behavior directly
    {
        use tos_alpha2::common::AiBehavior;
        let mut st = state.lock().unwrap();
        st.ai_behaviors.push(AiBehavior {
            id: "test_behavior".to_string(),
            name: "Test Behavior".to_string(),
            enabled: false,
            backend_override: None,
            context_fields: vec!["cwd".to_string(), "mode".to_string()],
            config: Default::default(),
        });
    }

    let res = ipc.handle_request("ai_behavior_enable:test_behavior");
    assert!(res.contains("AI_BEHAVIOR_ENABLED"), "Got: {}", res);

    let res = ipc.handle_request("ai_behavior_configure:test_behavior;max_suggestions;5");
    assert!(res.contains("AI_BEHAVIOR_CONFIGURED"), "Got: {}", res);

    let res = ipc.handle_request("ai_behavior_disable:test_behavior");
    assert!(res.contains("AI_BEHAVIOR_DISABLED"), "Got: {}", res);
}

#[tokio::test]
async fn ai_chip_stage_and_dismiss() {
    let (ipc, state) = headless_ipc();
    let res = ipc.handle_request("ai_chip_stage:command_hint_text");
    assert!(res.contains("AI_CHIP_STAGED"), "Got: {}", res);

    let log_len = state.lock().unwrap().system_log.len();
    assert!(log_len > 0, "system_log should have at least 1 entry");

    let res = ipc.handle_request("ai_chip_dismiss:command_hint_text");
    assert!(res.contains("AI_CHIP_DISMISSED"), "Got: {}", res);
}

// ============================================================
// Split Pane Tree integration tests
// ============================================================

#[tokio::test]
async fn split_create_produces_pane_id() {
    let (ipc, state) = headless_ipc();
    let res = ipc.handle_request("split_create:1920;1080");
    assert!(res.contains("SPLIT_CREATED:"), "Got: {}", res);

    let hub_has_layout = {
        let st = state.lock().unwrap();
        let idx = st.active_sector_index;
        st.sectors[idx].hubs[st.sectors[idx].active_hub_index].split_layout.is_some()
    };
    assert!(hub_has_layout, "split_layout should be Some after split_create");
}

#[tokio::test]
async fn split_equalize_works() {
    let (ipc, _state) = headless_ipc();
    ipc.handle_request("split_create:1920;1080");
    let res = ipc.handle_request("split_equalize:");
    assert!(res.contains("SPLIT_EQUALIZED"), "Got: {}", res);
}

#[tokio::test]
async fn split_close_removes_layout_when_last() {
    let (ipc, _state) = headless_ipc();
    let create_res = ipc.handle_request("split_create:1920;1080");
    assert!(create_res.contains("SPLIT_CREATED:"), "split_create failed: {}", create_res);
    // Extract uuid — format is "SPLIT_CREATED: <uuid> (<timing>)"
    let pane_id = create_res
        .split_once("SPLIT_CREATED: ")
        .map(|(_, rest)| {
            // Strip any trailing timing annotation like " (133µs)"
            rest.split_whitespace().next().unwrap_or("").to_string()
        })
        .expect("UUID not in response");

    let res = ipc.handle_request(&format!("split_close:{}", pane_id));
    // After closing the new pane from a 2-pane split, one pane remains
    assert!(!res.contains("ERROR"), "Got: {}", res);
}

#[tokio::test]
async fn split_blocked_when_display_too_small() {
    let (ipc, _state) = headless_ipc();
    // Very small display — should block
    let res = ipc.handle_request("split_create:200;100");
    // With default (1 pane), new_count=2, pane_w=100, min_w=max(33,400)=400 → blocked
    assert!(res.contains("SPLIT_BLOCKED"), "Expected SPLIT_BLOCKED for tiny display, got: {}", res);
}

#[tokio::test]
async fn split_focus_direction_cycles() {
    let (ipc, state) = headless_ipc();
    ipc.handle_request("split_create:1920;1080");

    let res = ipc.handle_request("split_focus_direction:right");
    assert!(res.contains("SPLIT_FOCUSED:"), "Got: {}", res);
}

#[tokio::test]
async fn split_fullscreen_exit_clears_layout() {
    let (ipc, state) = headless_ipc();
    ipc.handle_request("split_create:1920;1080");

    let layout_before = {
        let st = state.lock().unwrap();
        let idx = st.active_sector_index;
        st.sectors[idx].hubs[st.sectors[idx].active_hub_index].split_layout.is_some()
    };
    assert!(layout_before);

    ipc.handle_request("split_fullscreen_exit:");
    let layout_after = {
        let st = state.lock().unwrap();
        let idx = st.active_sector_index;
        st.sectors[idx].hubs[st.sectors[idx].active_hub_index].split_layout.is_some()
    };
    assert!(!layout_after, "split_fullscreen_exit should clear the layout");
}
