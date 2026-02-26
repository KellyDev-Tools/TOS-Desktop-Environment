use tos_core::TosState;
use tos_core::system::ipc::IpcDispatcher;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[test]
fn test_settings_modal_render() {
    let state = TosState::new_fresh();
    
    // Default state: settings overlay should not be visible
    assert!(!state.settings_open);
    let html = state.render_current_view();
    assert!(!html.contains("settings-modal-overlay"));
    
    // Use IPC to open settings
    let state_arc = Arc::new(Mutex::new(state));
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    let dispatcher = IpcDispatcher::new(state_arc.clone(), ptys);
    dispatcher.handle_request("open_settings");
    
    // Check that state is updated and overlay renders
    let state = state_arc.lock().unwrap();
    assert!(state.settings_open);
    let html = state.render_current_view();
    assert!(html.contains("settings-modal-overlay"));
    assert!(html.contains("TARGET FPS"));
    assert!(html.contains("Master Volume"));
}

#[test]
fn test_settings_ipc_mutations() {
    let state = TosState::new_fresh();
    let state_arc = Arc::new(Mutex::new(state));
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    let dispatcher = IpcDispatcher::new(state_arc.clone(), ptys);
    
    // Enable Deep Inspection
    {
        let mut state = state_arc.lock().unwrap();
        state.security.config.confirm_all_destructive = false;
    }
    
    dispatcher.handle_request("enable-deep-inspection");
    {
        let state = state_arc.lock().unwrap();
        assert!(state.security.config.confirm_all_destructive);
    }
    
    // Test FPS override
    {
        let state = state_arc.lock().unwrap();
        assert_eq!(state.fps, 60.0);
    }
    dispatcher.handle_request("set_fps:144");
    {
        let state = state_arc.lock().unwrap();
        assert_eq!(state.fps, 144.0);
    }
    
    // Test Volume override
    {
        let state = state_arc.lock().unwrap();
        assert_eq!(state.earcon_player.master_volume(), 1.0);
    }
    dispatcher.handle_request("set_master_volume:50");
    {
        let state = state_arc.lock().unwrap();
        assert_eq!(state.earcon_player.master_volume(), 0.5);
    }
    
    // Close settings
    {
        let mut state = state_arc.lock().unwrap();
        state.settings_open = true;
    }
    dispatcher.handle_request("close_settings");
    {
        let state = state_arc.lock().unwrap();
        assert!(!state.settings_open);
    }
}
