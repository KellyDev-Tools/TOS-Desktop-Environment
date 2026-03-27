use tos_core::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

fn create_test_state_with_apps() -> TosState {
    let mut state = TosState::new();
    // Explicitly reset to ensure clean state
    state.current_level = HierarchyLevel::GlobalOverview;
    state.viewports[0].current_level = HierarchyLevel::CommandHub;
    state.viewports[0].sector_index = 0;
    state.viewports[0].hub_index = 0;
    state.active_viewport_index = 0;
    
    // Set to Activity Mode
    state.sectors[0].hubs[0].mode = CommandHubMode::Activity;
    state.sectors[0].hubs[0].applications.clear();
    
    // Add dummy apps with safe PIDs (unlikely to exist or don't matter since signal is ignored)
    let app1 = Application {
        id: Uuid::new_v4(),
        title: "App 1".to_string(),
        app_class: "Utils".to_string(),
        is_minimized: false,
        pid: Some(999901),
        icon: None,
        is_dummy: true,
        settings: HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: vec![],
    };
    let app2 = Application {
        id: Uuid::new_v4(),
        title: "App 2".to_string(),
        app_class: "Network".to_string(),
        is_minimized: false,
        pid: Some(999902),
        icon: None,
        is_dummy: true,
        settings: HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: vec![],
    };
     state.sectors[0].hubs[0].applications.push(app1);
     state.sectors[0].hubs[0].applications.push(app2);
     
     state
}

#[test]
fn test_app_toggle_select() {
    let state = create_test_state_with_apps();
    let app1_id = state.sectors[0].hubs[0].applications[0].id.to_string();
    let app2_id = state.sectors[0].hubs[0].applications[1].id.to_string();
    
    let state_arc = Arc::new(Mutex::new(state));
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    let dispatcher = system::ipc::IpcDispatcher::new(state_arc.clone(), ptys);
    
    // 1. Select App 1
    dispatcher.handle_request(&format!("app_toggle_select:{}", app1_id));
    {
        let state = state_arc.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.selected_files.contains(&app1_id));
        assert!(!hub.selected_files.contains(&app2_id));
        assert!(hub.prompt.contains("999901"));
        assert!(!hub.prompt.contains("999902"));
    }
    
    // 2. Select App 2
    dispatcher.handle_request(&format!("app_toggle_select:{}", app2_id));
    {
        let state = state_arc.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.selected_files.contains(&app1_id));
        assert!(hub.selected_files.contains(&app2_id));
        assert!(hub.prompt.contains("999901"));
        assert!(hub.prompt.contains("999902"));
    }
    
    // 3. Deselect App 1
    dispatcher.handle_request(&format!("app_toggle_select:{}", app1_id));
    {
        let state = state_arc.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(!hub.selected_files.contains(&app1_id));
        assert!(hub.selected_files.contains(&app2_id));
        assert!(!hub.prompt.contains("999901"));
        assert!(hub.prompt.contains("999902"));
    }
}

#[test]
fn test_app_batch_kill_clears_selection() {
    let state = create_test_state_with_apps();
    let app1_id = state.sectors[0].hubs[0].applications[0].id.to_string();
    
    let state_arc = Arc::new(Mutex::new(state));
    let ptys = Arc::new(Mutex::new(HashMap::new()));
    let dispatcher = system::ipc::IpcDispatcher::new(state_arc.clone(), ptys);
    
    // Select App 1
    dispatcher.handle_request(&format!("app_toggle_select:{}", app1_id));
    
    // Call batch kill
    dispatcher.handle_request("app_batch_kill");
    
    // Verify selection is cleared
    {
        let state = state_arc.lock().unwrap();
        let hub = &state.sectors[0].hubs[0];
        assert!(hub.selected_files.is_empty());
        assert!(hub.prompt.is_empty());
        
        // Verify app was removed (since handle_kill_app removes it)
        assert_eq!(hub.applications.len(), 1); 
        assert_ne!(hub.applications[0].id.to_string(), app1_id);
    }
}

#[test]
fn test_mode_switch_clears_selection() {
    let mut state = create_test_state_with_apps();
    let app1_id = state.sectors[0].hubs[0].applications[0].id.to_string();
    
    state.sectors[0].hubs[0].selected_files.insert(app1_id);
    state.sectors[0].hubs[0].prompt = "manage 999901".to_string();
    
    // Toggle to Directory Mode
    state.toggle_mode(CommandHubMode::Directory);
    
    assert_eq!(state.sectors[0].hubs[0].mode, CommandHubMode::Directory);
    assert!(state.sectors[0].hubs[0].selected_files.is_empty());
    assert!(state.sectors[0].hubs[0].prompt.is_empty());
    
    // Toggle back to Activity - should still be empty
    state.toggle_mode(CommandHubMode::Activity);
    assert!(state.sectors[0].hubs[0].selected_files.is_empty());
}

#[test]
fn test_render_activity_toolbar() {
    let mut state = create_test_state_with_apps();
    let app1_id = state.sectors[0].hubs[0].applications[0].id.to_string();
    
    let html = state.render_current_view();
    assert!(!html.contains("class=\"action-toolbar\""));
    
    // Select app
    state.sectors[0].hubs[0].selected_files.insert(app1_id);
    
    let html = state.render_current_view();
    assert!(html.contains("class=\"action-toolbar\""));
    assert!(html.contains("APPS SELECTED"));
    assert!(html.contains("KILL"));
    assert!(html.contains("SIGINT"));
}
