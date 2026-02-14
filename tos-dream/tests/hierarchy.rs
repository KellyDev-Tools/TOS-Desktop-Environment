use tos_core::{TosState, HierarchyLevel, CommandHubMode};

#[test]
fn test_complex_navigation_flow() {
    let mut state = TosState::new();
    
    // 1. Initial State
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    
    // 2. Select Sector (This logic is usually in main.rs IPC, but we can call it)
    state.active_viewport_index = 0;
    state.viewports[0].sector_index = 1; // Science Labs
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    
    // 3. Change Mode
    state.toggle_mode(CommandHubMode::Activity);
    let html = state.render_current_view();
    assert!(html.contains("ACTIVITY"));
    assert!(html.contains("SENSOR ARRAY"));
    
    // 4. Focus App
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    let html = state.render_current_view();
    assert!(html.contains("APPLICATION DATA FEED: SENSOR ARRAY"));
    
    // 5. Tactical Reset (Zoom all the way out)
    state.zoom_out(); // Focus -> Hub
    state.zoom_out(); // Hub -> Global
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
}

#[test]
fn test_viewport_independence() {
    let mut state = TosState::new();
    
    // Create a split view manually
    state.zoom_in(); // Hub 1
    
    let second_viewport = tos_core::Viewport {
        id: uuid::Uuid::new_v4(),
        sector_index: 1,
        hub_index: 0,
        current_level: HierarchyLevel::GlobalOverview,
        active_app_index: None,
        bezel_expanded: false,
    };
    state.viewports.push(second_viewport);
    state.current_level = HierarchyLevel::SplitView;
    
    // Viewport 0 is at CommandHub, Viewport 1 is at GlobalOverview
    assert_eq!(state.viewports[0].current_level, HierarchyLevel::CommandHub);
    assert_eq!(state.viewports[1].current_level, HierarchyLevel::GlobalOverview);
    
    // Render check
    let html = state.render_current_view();
    assert!(html.contains("split-viewport-grid"));
    assert!(html.contains("viewport-cell"));
}

#[test]
fn test_deep_inspection() {
    let mut state = TosState::new();
    state.zoom_in(); // Hub
    state.zoom_in(); // Focus
    
    // Test Level 4
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::DetailInspector);
    let html = state.render_current_view();
    assert!(html.contains("NODE INSPECTOR"));
    
    // Test Level 5
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::BufferInspector);
    let html = state.render_current_view();
    assert!(html.contains("BUFFER HEX DUMP"));
    
    // Zoom out back to Hub
    state.zoom_out(); // Buffer -> Detail
    state.zoom_out(); // Detail -> Focus
    state.zoom_out(); // Focus -> Hub
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
}

#[test]
fn test_command_staging_flow() {
    let mut state = TosState::new();
    state.zoom_in(); // Hub
    
    // Simulator staging a command
    state.stage_command("focus Stellar Cartography".to_string());
    
    let sector = &state.sectors[state.viewports[0].sector_index];
    let hub = &sector.hubs[state.viewports[0].hub_index];
    assert_eq!(hub.prompt, "focus Stellar Cartography");
    
    // Verify it renders in the prompt
    let html = state.render_current_view();
    assert!(html.contains("focus Stellar Cartography"));
}
