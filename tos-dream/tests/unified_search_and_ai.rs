use tos_core::{TosState, CommandHubMode};
use tos_core::system::input::SemanticEvent;

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
    
    // Check default backend registration
    assert!(state.ai_manager.backends.contains_key("ollama"));
    assert_eq!(state.ai_manager.active_backend, Some("ollama".to_string()));
    
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
    state.perform_search("tos-config");
    
    assert!(state.search_manager.is_searching);
    assert!(!state.search_manager.results.is_empty());
    assert_eq!(state.search_manager.current_query, "tos-config");
    
    // Clear search
    state.handle_semantic_event(SemanticEvent::StopOperation);
    assert!(!state.search_manager.is_searching);
    assert!(state.search_manager.results.is_empty());
}

#[test]
fn test_search_logs_integration() {
    use tos_core::system::log::LogType;
    use tos_core::system::search::SearchDomain;
    
    let mut state = TosState::new();
    state.zoom_in();
    state.handle_semantic_event(SemanticEvent::ModeSearch);

    // Add some logs
    state.log_manager.log(LogType::System, "Kernel", "System started", None);
    state.log_manager.log(LogType::Security, "Auth", "User logged in", None);

    // Search for "logged"
    state.perform_search("logged");
    
    // Check results
    let results = &state.search_manager.results;
    assert!(!results.is_empty());
    
    let log_result = results.iter().find(|r| r.domain == SearchDomain::Logs);
    assert!(log_result.is_some());
    assert!(log_result.unwrap().title.contains("User logged in"));
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
