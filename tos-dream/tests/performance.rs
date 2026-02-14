use tos_core::*;

#[test]
fn test_performance_alert_state() {
    let mut state = TosState::new();
    
    // Default state: no alert
    assert!(!state.performance_alert);
    assert_eq!(state.fps, 60.0);
    assert!(!state.render_current_view().contains("PERFORMANCE CRITICAL"));
    
    // Simulate performance drop
    state.fps = 15.0;
    state.performance_alert = true;
    
    let html = state.render_current_view();
    assert!(html.contains("PERFORMANCE CRITICAL"));
    assert!(html.contains("CURRENT FPS: 15.0"));
}

#[test]
fn test_depth_based_rendering_simulation() {
    // In a full implementation, we might skip rendering background viewports.
    // Let's verify that the performance overlay is appended to the view.
    let mut state = TosState::new();
    state.performance_alert = true;
    
    let html = state.render_current_view();
    assert!(html.contains("perflo-alert"));
}
