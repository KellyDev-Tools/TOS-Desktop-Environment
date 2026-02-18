use tos_core::*;
use std::collections::HashMap;

fn create_test_state_with_real_pid() -> TosState {
    let mut state = TosState::new();
    let pid = std::process::id();
    
    // Add app with real PID
    let app = Application {
        id: uuid::Uuid::new_v4(),
        title: "Test Process".to_string(),
        app_class: "Test".to_string(),
        is_minimized: false,
        pid: Some(pid),
        icon: None,
        is_dummy: false,
        settings: HashMap::new(),
        thumbnail: None,
        decoration_policy: DecorationPolicy::Native,
        bezel_actions: Vec::new(),
    };
    
    // Set level to DetailInspector
    state.current_level = HierarchyLevel::DetailInspector;
    state.viewports[0].current_level = HierarchyLevel::DetailInspector;
    state.viewports[0].active_app_index = Some(0);
    state.sectors[0].hubs[0].applications.clear();
    state.sectors[0].hubs[0].applications.push(app);
    
    state
}

#[test]
fn test_detail_inspector_dynamic_data() {
    let state = create_test_state_with_real_pid();
    let html = state.render_current_view();
    
    assert!(html.contains("UPTIME:"), "Should show uptime label");
    // Verify it's not the hardcoded old value
    assert!(!html.contains("02:14:05"), "Should not show old hardcoded uptime");
    // Verify it fetched something (UID is present)
    assert!(html.contains("PERMS:"), "Should show perms label");
    assert!(html.contains("UID:"), "Should show UID");
}

#[test]
fn test_buffer_inspector_dynamic_data() {
    let mut state = create_test_state_with_real_pid();
    state.current_level = HierarchyLevel::BufferInspector;
    state.viewports[0].current_level = HierarchyLevel::BufferInspector;
    
    let html = state.render_current_view();
    
    assert!(html.contains("BUFFER HEX DUMP"), "Should show header");
    // Verify it's not the hardcoded old value and contains dynamic content
    assert!(!html.contains("LCARS DREAM COMPLETE"), "Should not show hardcoded hex dump");
    
    // The hex dump format: "0000: xx xx ...  ascii"
    // We can check for the offset "0000:"
    assert!(html.contains("0000:"), "Should contain start offset");
    // Should contain some hex (e.g., spaces between bytes)
    assert!(html.contains("  "), "Should contain aligned columns");
}
