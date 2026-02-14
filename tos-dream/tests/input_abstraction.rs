use tos_core::*;
use tos_core::system::input::SemanticEvent;

#[test]
fn test_keyboard_to_semantic_flow() {
    let mut state = TosState::new();
    state.handle_semantic_event(SemanticEvent::ZoomIn); // Level 2
    
    // Simulate F1 (ModeCommand)
    state.handle_semantic_event(SemanticEvent::ModeCommand);
    {
        let viewport = &state.viewports[0];
        assert_eq!(state.sectors[viewport.sector_index].hubs[viewport.hub_index].mode, CommandHubMode::Command);
    }
    
    // Simulate F2 (ModeDirectory)
    state.handle_semantic_event(SemanticEvent::ModeDirectory);
    {
        let viewport = &state.viewports[0];
        assert_eq!(state.sectors[viewport.sector_index].hubs[viewport.hub_index].mode, CommandHubMode::Directory);
    }
    
    // Simulate F3 (ModeActivity)
    state.handle_semantic_event(SemanticEvent::ModeActivity);
    {
        let viewport = &state.viewports[0];
        assert_eq!(state.sectors[viewport.sector_index].hubs[viewport.hub_index].mode, CommandHubMode::Activity);
    }
}

#[test]
fn test_zoom_semantic_events() {
    let mut state = TosState::new();
    
    // Initial Level 1
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    
    // PageUp -> Zoom In (Level 2)
    state.handle_semantic_event(SemanticEvent::ZoomIn);
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
    
    // PageUp -> Zoom In (Level 3)
    state.handle_semantic_event(SemanticEvent::ZoomIn);
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    
    // PageDown -> Zoom Out (Level 2)
    state.handle_semantic_event(SemanticEvent::ZoomOut);
    assert_eq!(state.current_level, HierarchyLevel::CommandHub);
}

#[test]
fn test_system_wide_semantic_events() {
    let mut state = TosState::new();
    state.handle_semantic_event(SemanticEvent::ZoomIn);
    state.handle_semantic_event(SemanticEvent::ZoomIn);
    assert_eq!(state.current_level, HierarchyLevel::ApplicationFocus);
    
    // Home -> Global Overview
    state.handle_semantic_event(SemanticEvent::OpenGlobalOverview);
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
    assert_eq!(state.viewports[0].current_level, HierarchyLevel::GlobalOverview);
    
    // Zoom back in
    state.handle_semantic_event(SemanticEvent::ZoomIn);
    
    // End -> Tactical Reset
    state.handle_semantic_event(SemanticEvent::TacticalReset);
    assert_eq!(state.current_level, HierarchyLevel::GlobalOverview);
}
