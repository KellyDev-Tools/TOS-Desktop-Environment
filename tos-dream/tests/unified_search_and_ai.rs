use tos_core::{TosState, CommandHubMode, HierarchyLevel};
use tos_core::system::input::SemanticEvent;
use uuid::Uuid;

#[test]
fn test_ai_manager_query() {
    let mut state = TosState::new();
    state.zoom_in(); // Enter CommandHub level
    
    // Switch to AI mode
    state.handle_semantic_event(SemanticEvent::ModeAi);
    
    let v_idx = state.active_viewport_index;
    let s_idx = state.viewports[v_idx].sector_index;
    let h_idx = state.viewports[v_idx].hub_index;
    
    let mode = state.sectors[s_idx].hubs[h_idx].mode;
    assert_eq!(mode, CommandHubMode::Ai);
    
    // Stage a query
    state.sectors[s_idx].hubs[h_idx].prompt = "Explain quantum computing".to_string();
    
    // Submit AI query
    state.handle_semantic_event(SemanticEvent::AiSubmit);
    
    assert_eq!(state.ai_manager.history.len(), 1);
    assert_eq!(state.ai_manager.history[0].role, "user");
    assert_eq!(state.ai_manager.history[0].content, "Explain quantum computing");
    assert!(state.ai_manager.is_generating);
    
    // Prompt should be cleared
    assert_eq!(state.sectors[s_idx].hubs[h_idx].prompt, "");
    
    // Stop generation
    state.handle_semantic_event(SemanticEvent::AiStop);
    assert!(!state.ai_manager.is_generating);
}

#[test]
fn test_search_manager_execution() {
    let mut state = TosState::new();
    state.zoom_in(); // Enter CommandHub level
    
    // Switch to Search mode
    state.handle_semantic_event(SemanticEvent::ModeSearch);
    
    let v_idx = state.active_viewport_index;
    let s_idx = state.viewports[v_idx].sector_index;
    let h_idx = state.viewports[v_idx].hub_index;
    
    let mode = state.sectors[s_idx].hubs[h_idx].mode;
    assert_eq!(mode, CommandHubMode::Search);
    
    // Execute search
    state.search_manager.execute_search("tos-config");
    
    assert!(state.search_manager.is_searching);
    assert!(!state.search_manager.results.is_empty());
    assert_eq!(state.search_manager.current_query, "tos-config");
    
    // Clear search
    state.handle_semantic_event(SemanticEvent::StopOperation);
    assert!(!state.search_manager.is_searching);
    assert!(state.search_manager.results.is_empty());
}

#[test]
fn test_mode_cycling_with_addendum_modes() {
    let mut state = TosState::new();
    state.zoom_in(); // Enter CommandHub level
    
    // Start at Command
    let s_idx = state.viewports[0].sector_index;
    let h_idx = state.viewports[0].hub_index;
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Command);
    
    // Cycle: Command -> Directory
    state.handle_semantic_event(SemanticEvent::CycleMode);
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Directory);
    
    // Cycle: Directory -> Activity
    state.handle_semantic_event(SemanticEvent::CycleMode);
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Activity);
    
    // Cycle: Activity -> Search (New)
    state.handle_semantic_event(SemanticEvent::CycleMode);
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Search);
    
    // Cycle: Activity -> Search -> AI (New)
    state.handle_semantic_event(SemanticEvent::CycleMode);
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Ai);
    
    // Cycle: AI -> Command
    state.handle_semantic_event(SemanticEvent::CycleMode);
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Command);
}

#[test]
fn test_ai_mode_toggle() {
    let mut state = TosState::new();
    state.zoom_in();
    
    // Toggle ON
    state.handle_semantic_event(SemanticEvent::AiModeToggle);
    let v_idx = state.active_viewport_index;
    let s_idx = state.viewports[v_idx].sector_index;
    let h_idx = state.viewports[v_idx].hub_index;
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Ai);
    
    // Toggle OFF (should return to Command)
    state.handle_semantic_event(SemanticEvent::AiModeToggle);
    assert_eq!(state.sectors[s_idx].hubs[h_idx].mode, CommandHubMode::Command);
}
