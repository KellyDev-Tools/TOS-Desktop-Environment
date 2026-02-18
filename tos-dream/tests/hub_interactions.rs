use tos_core::*;

#[test]
fn test_hub_mode_toggling() {
    let mut state = TosState::new();
    state.zoom_in(); // Hub level
    
    // Initial mode is Command
    assert!(state.render_current_view().contains("COMMAND"));
    
    // Toggle to Directory
    state.toggle_mode(CommandHubMode::Directory);
    assert!(state.render_current_view().contains("DIRECTORY"));
    assert!(state.render_current_view().contains("FILES")); // Check for hidden toggle button

    
    // Toggle to Activity
    state.toggle_mode(CommandHubMode::Activity);
    assert!(state.render_current_view().contains("ACTIVITY"));
    assert!(state.render_current_view().contains("TERMINAL"));
}

#[test]
fn test_command_staging() {
    let mut state = TosState::new();
    state.zoom_in(); // Hub level
    
    // Stage a command
    state.stage_command("rm -rf /".to_string());
    
    let html = state.render_current_view();
    assert!(html.contains("value=\"rm -rf /\""));
}

#[test]
fn test_dangerous_command_flow() {
    // This is more complex because main.rs handles the detection.
    // However, we can test the state transition in a unit test in lib.rs if we add the logic there.
    // For now, let's test state transitions for multi-viewport.
}

#[test]
fn test_multi_viewport_switching() {
    let mut state = TosState::new();
    
    // Split viewport
    // (Simulating split logic from main.rs)
    let sector_idx = state.viewports[0].sector_index;
    let new_viewport = Viewport {
        id: uuid::Uuid::new_v4(),
        sector_index: sector_idx,
        hub_index: 0,
        current_level: HierarchyLevel::GlobalOverview,
        active_app_index: None,
        bezel_expanded: false,
    };
    state.viewports.push(new_viewport);
    
    assert_eq!(state.viewports.len(), 2);
    
    // Switch active viewport
    state.active_viewport_index = 1;
    state.zoom_in(); // Zoom into hub in viewport 1
    
    assert_eq!(state.viewports[1].current_level, HierarchyLevel::CommandHub);
    assert_eq!(state.viewports[0].current_level, HierarchyLevel::GlobalOverview);
}
