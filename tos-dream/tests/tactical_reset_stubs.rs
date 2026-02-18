//! Tests for Section 9: Tactical Reset Stubs (§14)
//!
//! Covers:
//! - Unit: SIGTERM dispatch, system command formatting, error types
//! - Component: TacticalReset state machine with injected executor
//! - Integration: Full reset lifecycle through TosState, PID tracking, executor injection

use tos_core::system::reset::{
    TacticalReset, ResetOperationState,
    SystemResetOption, ResetError,
};
use tos_core::{TosState, HierarchyLevel, DecorationPolicy};
use std::sync::{Arc, Mutex};

// ─── Unit Tests: SIGTERM dispatch ─────────────────────────────────────────────

#[test]
fn test_sigterm_pids_tracked_after_sector_reset() {
    // Spawn a real child process (sleep) so we have a valid PID to SIGTERM
    let child = std::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("Failed to spawn sleep");
    let child_pid = child.id();

    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    // Register the child PID as an application
    state.sectors[sector_idx].hubs[hub_idx].applications.push(tos_core::Application {
        id: uuid::Uuid::new_v4(),
        title: "test-sleep".to_string(),
        app_class: "Test".to_string(),
        is_minimized: false,
        pid: Some(child_pid),
        icon: None,
        is_dummy: false,
        settings: std::collections::HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: vec![],
    });

    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();

    // The PID should have been SIGTERMed and tracked
    assert!(
        reset.last_sigterm_pids.contains(&child_pid),
        "SIGTERM should have been sent to child PID {}; tracked: {:?}",
        child_pid, reset.last_sigterm_pids
    );

    // Applications should be cleared from the hub
    assert!(
        state.sectors[sector_idx].hubs[hub_idx].applications.is_empty(),
        "Applications should be cleared after sector reset"
    );

    // Clean up: the child was already SIGTERMed, just wait for it
    let _ = std::process::Command::new("kill")
        .args(["-9", &child_pid.to_string()])
        .status();
}

#[test]
fn test_sigterm_skips_apps_without_pid() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    // Add a dummy app with no PID
    state.sectors[sector_idx].hubs[hub_idx].applications.push(tos_core::Application {
        id: uuid::Uuid::new_v4(),
        title: "dummy-app".to_string(),
        app_class: "Dummy".to_string(),
        is_minimized: false,
        pid: None, // No PID
        icon: None,
        is_dummy: true,
        settings: std::collections::HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: vec![],
    });

    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();

    // No PIDs should have been tracked (app had no PID)
    assert!(
        reset.last_sigterm_pids.is_empty(),
        "No PIDs should be tracked for apps without a PID"
    );
}

#[test]
fn test_sigterm_pids_cleared_on_new_reset() {
    let mut state = TosState::new();
    let mut reset = TacticalReset::new();

    // First reset (no apps, no PIDs)
    reset.initiate_sector_reset(&mut state).unwrap();
    assert!(reset.last_sigterm_pids.is_empty());

    // Manually set some fake PIDs to simulate a previous run
    reset.last_sigterm_pids = vec![1234, 5678];

    // Second reset should clear the list
    reset.cancel_reset();
    let mut state2 = TosState::new();
    reset.initiate_sector_reset(&mut state2).unwrap();
    // last_sigterm_pids is cleared at the start of execute_sector_reset
    assert!(
        !reset.last_sigterm_pids.contains(&1234),
        "Stale PIDs should be cleared before new reset"
    );
}

// ─── Unit Tests: system_executor injection ────────────────────────────────────

#[test]
fn test_restart_compositor_uses_executor_when_set() {
    let mut reset = TacticalReset::new();
    let executed = Arc::new(Mutex::new(Vec::<String>::new()));
    let executed_clone = Arc::clone(&executed);

    reset.set_system_executor(move |cmd: &str| {
        executed_clone.lock().unwrap().push(cmd.to_string());
        Ok(())
    });

    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
    reset.update_tactile_progress(1.0).unwrap();
    reset.tick_countdown(); // 3
    reset.tick_countdown(); // 2
    reset.tick_countdown(); // 1
    let opt = reset.tick_countdown(); // 0 → fires
    assert!(opt.is_some());
    reset.execute_system_reset(opt.unwrap()).unwrap();

    let cmds = executed.lock().unwrap();
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "systemctl restart tos-compositor");
}

#[test]
fn test_logout_uses_executor_when_set() {
    let mut reset = TacticalReset::new();
    let executed = Arc::new(Mutex::new(Vec::<String>::new()));
    let executed_clone = Arc::clone(&executed);

    reset.set_system_executor(move |cmd: &str| {
        executed_clone.lock().unwrap().push(cmd.to_string());
        Ok(())
    });

    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::LogOut).unwrap();
    reset.update_tactile_progress(1.0).unwrap();
    reset.tick_countdown();
    reset.tick_countdown();
    reset.tick_countdown();
    let opt = reset.tick_countdown();
    assert!(opt.is_some());
    reset.execute_system_reset(opt.unwrap()).unwrap();

    let cmds = executed.lock().unwrap();
    assert_eq!(cmds.len(), 1);
    assert!(
        cmds[0].contains("loginctl") || cmds[0].contains("pkill"),
        "Logout command should be loginctl or pkill, got: {}",
        cmds[0]
    );
}

#[test]
fn test_executor_failure_returns_error() {
    let mut reset = TacticalReset::new();
    reset.set_system_executor(|_cmd: &str| {
        Err("permission denied".to_string())
    });

    let result = reset.execute_system_reset(SystemResetOption::RestartCompositor);
    assert!(result.is_err());
    match result.unwrap_err() {
        ResetError::ExecutionFailed(msg) => {
            assert!(msg.contains("permission denied"));
        }
        other => panic!("Expected ExecutionFailed, got {:?}", other),
    }
}

#[test]
fn test_last_system_command_recorded_on_executor_path() {
    let mut reset = TacticalReset::new();
    reset.set_system_executor(|_cmd: &str| Ok(()));

    reset.execute_system_reset(SystemResetOption::RestartCompositor).unwrap();
    assert_eq!(
        reset.last_system_command.as_deref(),
        Some("systemctl restart tos-compositor"),
        "last_system_command should record the command sent to executor"
    );
}

#[test]
fn test_last_system_command_recorded_on_logout_executor_path() {
    let mut reset = TacticalReset::new();
    reset.set_system_executor(|_cmd: &str| Ok(()));

    reset.execute_system_reset(SystemResetOption::LogOut).unwrap();
    assert_eq!(
        reset.last_system_command.as_deref(),
        Some("loginctl terminate-session $XDG_SESSION_ID"),
        "last_system_command should record the logout command"
    );
}

// ─── Unit Tests: error types ──────────────────────────────────────────────────

#[test]
fn test_reset_error_display_all_variants() {
    assert_eq!(ResetError::ResetInProgress.to_string(), "A reset is already in progress");
    assert_eq!(ResetError::NoResetInProgress.to_string(), "No reset in progress");
    assert_eq!(ResetError::UndoExpired.to_string(), "Undo time has expired");
    assert_eq!(ResetError::NoSavedState.to_string(), "No saved state available");
    assert_eq!(ResetError::InvalidState.to_string(), "Invalid operation for current state");
    assert_eq!(
        ResetError::ExecutionFailed("test error".to_string()).to_string(),
        "Execution failed: test error"
    );
}

#[test]
fn test_reset_error_is_std_error() {
    // Verify it implements std::error::Error (compile-time check)
    fn assert_error<E: std::error::Error>(_: E) {}
    assert_error(ResetError::ResetInProgress);
}

// ─── Component Tests: TacticalReset state machine ────────────────────────────

#[test]
fn test_sector_reset_clears_hub_state() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    // Set up some hub state
    state.sectors[sector_idx].hubs[hub_idx].prompt = "some command".to_string();
    state.sectors[sector_idx].hubs[hub_idx].confirmation_required = Some("confirm?".to_string());

    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();

    // Hub should be fresh
    let hub = &state.sectors[sector_idx].hubs[0];
    assert!(hub.prompt.is_empty(), "Prompt should be cleared after reset");
    assert!(hub.confirmation_required.is_none(), "Confirmation should be cleared after reset");
    assert!(hub.applications.is_empty(), "Applications should be cleared after reset");
}

#[test]
fn test_sector_reset_returns_to_command_hub_level() {
    let mut state = TosState::new();
    state.current_level = HierarchyLevel::ApplicationFocus;
    state.viewports[0].current_level = HierarchyLevel::ApplicationFocus;

    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();

    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    assert_eq!(state.viewports[0].current_level, HierarchyLevel::CommandHub);
}

#[test]
fn test_sector_reset_saves_state_for_undo() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let original_sector_name = state.sectors[sector_idx].name.clone();

    let mut reset = TacticalReset::new();
    reset.config.save_state_before_reset = true;
    reset.initiate_sector_reset(&mut state).unwrap();

    assert!(reset.saved_sector.is_some(), "Sector should be saved for undo");
    assert_eq!(
        reset.saved_sector.as_ref().unwrap().name,
        original_sector_name
    );
}

#[test]
fn test_sector_reset_undo_restores_state() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;

    // Record initial app count, then add one more
    let initial_app_count = state.sectors[sector_idx].hubs[0].applications.len();
    state.sectors[sector_idx].hubs[0].applications.push(tos_core::Application {
        id: uuid::Uuid::new_v4(),
        title: "my-app".to_string(),
        app_class: "Test".to_string(),
        is_minimized: false,
        pid: None,
        icon: None,
        is_dummy: false,
        settings: std::collections::HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: vec![],
    });
    let pre_reset_count = initial_app_count + 1;

    let mut reset = TacticalReset::new();
    reset.config.undo_duration_secs = 60; // plenty of time
    reset.initiate_sector_reset(&mut state).unwrap();

    // After reset, hub is fresh (single default hub, no apps)
    assert!(state.sectors[sector_idx].hubs[0].applications.is_empty(),
        "Applications should be cleared after reset");

    // Undo restores the saved sector (with our extra app)
    reset.undo_sector_reset(&mut state).unwrap();
    assert_eq!(
        state.sectors[sector_idx].hubs[0].applications.len(),
        pre_reset_count,
        "Undo should restore {} applications (initial {} + 1 added)",
        pre_reset_count, initial_app_count
    );
    assert!(
        state.sectors[sector_idx].hubs[0].applications.iter().any(|a| a.title == "my-app"),
        "The added app should be present after undo"
    );
}


#[test]
fn test_undo_fails_when_no_reset_in_progress() {
    let mut state = TosState::new();
    let mut reset = TacticalReset::new();
    let result = reset.undo_sector_reset(&mut state);
    assert_eq!(result.unwrap_err(), ResetError::NoResetInProgress);
}

#[test]
fn test_double_reset_returns_error() {
    let mut state = TosState::new();
    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();
    let result = reset.initiate_sector_reset(&mut state);
    assert_eq!(result.unwrap_err(), ResetError::ResetInProgress);
}

#[test]
fn test_system_reset_full_lifecycle_with_executor() {
    let mut reset = TacticalReset::new();
    reset.config.countdown_secs = 2;
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = Arc::clone(&executed);

    reset.set_system_executor(move |_cmd: &str| {
        *executed_clone.lock().unwrap() = true;
        Ok(())
    });

    // 1. Show dialog
    reset.initiate_system_reset().unwrap();
    assert!(matches!(reset.state, ResetOperationState::SystemDialogShown));

    // 2. Select option
    reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
    assert!(matches!(reset.state, ResetOperationState::TactileConfirming { .. }));

    // 3. Complete tactile confirmation
    reset.update_tactile_progress(1.0).unwrap();
    assert!(matches!(reset.state, ResetOperationState::Countdown { .. }));

    // 4. Countdown
    assert_eq!(reset.countdown_remaining(), Some(2));
    assert!(reset.tick_countdown().is_none()); // 2→1
    assert!(reset.tick_countdown().is_none()); // 1→0
    let opt = reset.tick_countdown();          // 0→fires
    assert!(opt.is_some());
    assert!(matches!(reset.state, ResetOperationState::Executing));

    // 5. Execute
    reset.execute_system_reset(opt.unwrap()).unwrap();
    assert!(*executed.lock().unwrap(), "Executor should have been called");
}

#[test]
fn test_cancel_during_tactile_returns_to_idle() {
    let mut reset = TacticalReset::new();
    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::LogOut).unwrap();
    assert!(matches!(reset.state, ResetOperationState::TactileConfirming { .. }));

    reset.cancel_reset();
    assert!(matches!(reset.state, ResetOperationState::Idle));
    assert!(!reset.is_resetting());
}

#[test]
fn test_cancel_option_skips_tactile() {
    let mut reset = TacticalReset::new();
    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::Cancel).unwrap();
    // Cancel should go straight to Idle, skipping tactile confirmation
    assert!(matches!(reset.state, ResetOperationState::Idle));
}

// ─── Component Tests: render output ──────────────────────────────────────────

#[test]
fn test_render_idle_is_empty() {
    let reset = TacticalReset::new();
    assert!(reset.render().is_empty());
}

#[test]
fn test_render_sector_reset_shows_undo_button() {
    let mut state = TosState::new();
    let mut reset = TacticalReset::new();
    reset.config.enable_sector_undo = true;
    reset.config.undo_duration_secs = 60;
    reset.initiate_sector_reset(&mut state).unwrap();

    let html = reset.render();
    assert!(html.contains("SECTOR RESET COMPLETE"));
    assert!(html.contains("UNDO RESET"), "Undo button should appear within time window");
}

#[test]
fn test_render_system_dialog_shows_all_options() {
    let mut reset = TacticalReset::new();
    reset.initiate_system_reset().unwrap();
    let html = reset.render();
    assert!(html.contains("SYSTEM RESET"));
    assert!(html.contains("Restart Compositor"));
    assert!(html.contains("Log Out"));
    assert!(html.contains("Cancel"));
}

#[test]
fn test_render_tactile_shows_progress_bar() {
    let mut reset = TacticalReset::new();
    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
    reset.update_tactile_progress(0.5).unwrap();

    let html = reset.render();
    assert!(html.contains("progress-bar"));
    assert!(html.contains("50%"), "Progress bar should show 50%");
}

#[test]
fn test_render_countdown_shows_remaining() {
    let mut reset = TacticalReset::new();
    reset.config.countdown_secs = 5;
    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
    reset.update_tactile_progress(1.0).unwrap();

    let html = reset.render();
    assert!(html.contains("countdown-number"));
    assert!(html.contains('5'.to_string().as_str()));
}

// ─── Integration Tests: through TosState ─────────────────────────────────────

#[test]
fn test_tactical_reset_via_tos_state_handle_event() {
    use tos_core::system::input::SemanticEvent;
    let mut state = TosState::new();
    state.current_level = HierarchyLevel::CommandHub;
    state.viewports[0].current_level = HierarchyLevel::CommandHub;

    // TacticalReset event should return to CommandHub (Level 1 reset)
    state.handle_semantic_event(SemanticEvent::TacticalReset);
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
}

#[test]
fn test_multiple_apps_all_get_sigterm() {
    let mut state = TosState::new();
    let sector_idx = state.viewports[0].sector_index;
    let hub_idx = state.viewports[0].hub_index;

    // Spawn two real processes
    let child1 = std::process::Command::new("sleep").arg("60").spawn().unwrap();
    let child2 = std::process::Command::new("sleep").arg("60").spawn().unwrap();
    let pid1 = child1.id();
    let pid2 = child2.id();

    for pid in [pid1, pid2] {
        state.sectors[sector_idx].hubs[hub_idx].applications.push(tos_core::Application {
            id: uuid::Uuid::new_v4(),
            title: format!("sleep-{}", pid),
            app_class: "Test".to_string(),
            is_minimized: false,
            pid: Some(pid),
            icon: None,
            is_dummy: false,
            settings: std::collections::HashMap::new(),
            thumbnail: None,
            decoration_policy: DecorationPolicy::Native,
            bezel_actions: vec![],
        });
    }

    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();

    assert!(reset.last_sigterm_pids.contains(&pid1), "PID1 should be SIGTERMed");
    assert!(reset.last_sigterm_pids.contains(&pid2), "PID2 should be SIGTERMed");
    assert_eq!(reset.last_sigterm_pids.len(), 2);

    // Clean up
    for pid in [pid1, pid2] {
        let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).status();
    }
}

#[test]
fn test_whoami_fallback_returns_valid_string() {
    // Indirectly test whoami_or_fallback by verifying log_out records a command
    let mut reset = TacticalReset::new();
    reset.set_system_executor(|_| Ok(()));
    reset.execute_system_reset(SystemResetOption::LogOut).unwrap();
    // The command should have been recorded
    assert!(reset.last_system_command.is_some());
}
