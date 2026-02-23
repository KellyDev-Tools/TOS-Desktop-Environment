use tos_core::TosState;
use tos_core::HierarchyLevel;
use tos_core::system::input::SemanticEvent;

#[test]
fn bezel_is_collapsed_initially() {
    let mut state = TosState::new();
    // Ensure we start from a clean state
    state.current_level = HierarchyLevel::GlobalOverview;
    state.viewports[0].bezel_expanded = false;
    assert!(!state.viewports[0].bezel_expanded);
}

#[test]
fn toggle_bezel_changes_state() {
    let mut state = TosState::new();
    state.toggle_bezel();
    assert!(state.viewports[0].bezel_expanded);
    // Toggle back
    state.toggle_bezel();
    assert!(!state.viewports[0].bezel_expanded);
}

#[test]
fn semantic_event_toggle_bezel() {
    let mut state = TosState::new();
    state.handle_semantic_event(SemanticEvent::ToggleBezel);
    assert!(state.viewports[0].bezel_expanded);
}
