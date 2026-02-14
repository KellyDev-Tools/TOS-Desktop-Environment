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
    assert!(html.contains("Sensor Array"));
    
    // 4. Focus App
    state.zoom_in();
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    let html = state.render_current_view();
    assert!(html.contains("APPLICATION DATA FEED: Sensor Array"));
    
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
