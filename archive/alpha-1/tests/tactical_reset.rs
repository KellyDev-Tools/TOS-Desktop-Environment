//! Tests for Section 14: Tactical Reset (Dream complete.md)
//!
//! Two-level emergency recovery system:
//! - **Level 1 – Sector Reset**: Super+Backspace / `tos sector reset`; SIGTERM sector processes,
//!   close viewports, return to Level 2 Command Hub; optional 5s undo.
//! - **Level 2 – System Reset**: Super+Alt+Backspace / `tos system reset`; dialog with Restart
//!   Compositor, Log Out, Cancel; tactile confirmation and countdown with cancel.

use tos_core::TosState;
use tos_core::HierarchyLevel;
use tos_core::system::reset::{
    TacticalReset, ResetConfig, ResetOperationState, SystemResetOption,
};

#[test]
fn test_section_14_level1_sector_reset_lifecycle() {
    let mut state = TosState::new();
    let mut reset = TacticalReset::new();

    assert!(!reset.is_resetting());
    let _ = reset.initiate_sector_reset(&mut state);
    assert!(reset.is_resetting());
    assert!(matches!(reset.state, ResetOperationState::SectorResetting { .. }));
    assert!(reset.can_undo());
    assert!(reset.undo_remaining_secs().is_some());

    // Optional undo within window (Section 14.1)
    let _ = reset.undo_sector_reset(&mut state);
    assert!(!reset.is_resetting());
    assert!(matches!(reset.state, ResetOperationState::Idle));
}

#[test]
fn test_section_14_level1_sector_reset_returns_to_hub() {
    let mut state = TosState::new();
    state.current_level = HierarchyLevel::DetailInspector;
    for v in &mut state.viewports {
        v.current_level = HierarchyLevel::DetailInspector;
    }

    state.tactical_reset();

    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    assert!(state.viewports.iter().all(|v| v.current_level == HierarchyLevel::GlobalOverview));
    assert_eq!(state.escape_count, 0);
}

#[test]
fn test_section_14_level2_system_reset_dialog_options() {
    let mut reset = TacticalReset::new();

    let _ = reset.initiate_system_reset();
    assert!(matches!(reset.state, ResetOperationState::SystemDialogShown));

    let html = reset.render();
    assert!(html.contains("SYSTEM RESET"));
    assert!(html.contains("Restart Compositor"));
    assert!(html.contains("Log Out"));
    assert!(html.contains("Cancel"));
    assert!(html.contains("tactical-reset-overlay"));
    assert!(html.contains("system-dialog"));

    reset.select_system_option(SystemResetOption::RestartCompositor).unwrap();
    assert!(matches!(reset.state, ResetOperationState::TactileConfirming { .. }));

    reset.cancel_reset();
    assert!(matches!(reset.state, ResetOperationState::Idle));
}

#[test]
fn test_section_14_level2_tactile_confirmation_and_countdown() {
    let mut reset = TacticalReset::new();
    reset.config.countdown_secs = 2;

    reset.initiate_system_reset().unwrap();
    reset.select_system_option(SystemResetOption::LogOut).unwrap();
    assert!(matches!(reset.state, ResetOperationState::TactileConfirming { .. }));

    let _ = reset.update_tactile_progress(1.0);
    assert!(matches!(reset.state, ResetOperationState::Countdown { .. }));
    assert_eq!(reset.countdown_remaining(), Some(2));

    assert!(reset.tick_countdown().is_none());
    assert_eq!(reset.countdown_remaining(), Some(1));
    assert!(reset.tick_countdown().is_none());
    assert_eq!(reset.countdown_remaining(), Some(0));
    assert!(reset.tick_countdown().is_some());
}

#[test]
fn test_section_14_config_undo_duration() {
    let config = ResetConfig::default();
    assert!(config.enable_sector_undo);
    assert_eq!(config.undo_duration_secs, 5);
    assert_eq!(config.countdown_secs, 3);
    assert!(!config.confirm_sector_reset);
    assert!(config.save_state_before_reset);
}

#[test]
fn test_section_14_render_idle_empty() {
    let reset = TacticalReset::new();
    assert!(reset.render().is_empty());
}

#[test]
fn test_section_14_render_sector_reset_overlay() {
    let mut state = TosState::new();
    let mut reset = TacticalReset::new();
    reset.initiate_sector_reset(&mut state).unwrap();

    let html = reset.render();
    assert!(html.contains("tactical-reset-overlay"));
    assert!(html.contains("sector-reset"));
    assert!(html.contains("SECTOR RESET COMPLETE"));
}

#[test]
fn test_section_14_ipc_system_reset_shows_dialog() {
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use tos_core::system::ipc::IpcDispatcher;
    use tos_core::system::pty::PtyHandle;

    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    dispatcher.handle_request("semantic_event:SystemReset");

    {
        let s = state.lock().unwrap();
        let reset_html = s.tactical_reset.render();
        assert!(reset_html.contains("SYSTEM RESET"), "System reset should show dialog");
        assert!(reset_html.contains("Restart Compositor"));
        assert!(reset_html.contains("Log Out"));
    }
}

#[test]
fn test_section_14_ipc_tactical_reset_returns_to_hub() {
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use tos_core::system::ipc::IpcDispatcher;
    use tos_core::system::pty::PtyHandle;

    let state = Arc::new(Mutex::new(TosState::new()));
    let ptys = Arc::new(Mutex::new(HashMap::<uuid::Uuid, PtyHandle>::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));

    dispatcher.handle_request("select_sector:0");
    dispatcher.handle_request("semantic_event:ZoomIn");
    dispatcher.handle_request("semantic_event:ZoomIn");
    {
        let s = state.lock().unwrap();
        assert_eq!(s.current_level, HierarchyLevel::DetailInspector);
    }

    dispatcher.handle_request("tactical_reset");

    {
        let s = state.lock().unwrap();
        assert_eq!(s.current_level, HierarchyLevel::CommandHub);
        let html = s.render_current_view();
        assert!(html.contains("command-hub"), "reset should show Command Hub");
    }
}
